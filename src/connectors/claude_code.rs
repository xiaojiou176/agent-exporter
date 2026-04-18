use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use chrono::DateTime;
use serde_json::{Map, Value};

use crate::core::archive::{
    ArchiveCompleteness, ArchiveRound, ArchiveThreadStatus, ArchiveToolCall, ArchiveTranscript,
    ArchiveTurnItem, ArchiveTurnStatus, CommandExecutionRecord, ConnectorSourceKind,
    DynamicToolCallRecord, ExportRequest, ExportSelector, ExportSource, FileChangeRecord,
};
use crate::model::{ConnectorDefinition, ConnectorKind, SupportStage};

pub const DEFINITION: ConnectorDefinition = ConnectorDefinition {
    kind: ConnectorKind::ClaudeCode,
    stage: SupportStage::Current,
    summary: "Minimal Claude Code session-path import that reuses the shared archive transcript contract.",
    source_of_truth: "Claude local session artifacts (`--session-path`) mapped into the shared archive core",
};

pub fn load_transcript(request: &ExportRequest) -> Result<ArchiveTranscript> {
    if request.source != ExportSource::SessionPath {
        bail!("claude-code export only supports `--session-path <PATH>`");
    }

    let session_path = match &request.selector {
        ExportSelector::SessionPath(path) => path.clone(),
        ExportSelector::ThreadId(_) | ExportSelector::RolloutPath(_) => {
            bail!("claude-code export requires `--session-path <PATH>`")
        }
    };

    if !session_path.exists() {
        bail!(
            "Claude session file does not exist: {}",
            session_path.display()
        );
    }

    let entries = load_entries(&session_path)?;
    let mut replay = ClaudeReplay::new();
    for entry in &entries {
        replay.handle_entry(entry);
    }
    let result = replay.finish();

    let thread_id = result
        .thread_id
        .or_else(|| {
            session_path
                .file_stem()
                .map(|stem| stem.to_string_lossy().to_string())
                .filter(|value| !value.trim().is_empty())
        })
        .unwrap_or_else(|| "claude-session".to_string());

    if result.rounds.is_empty() {
        bail!(
            "Claude session `{}` did not contain exportable rounds",
            session_path.display()
        );
    }

    Ok(ArchiveTranscript {
        connector: ConnectorKind::ClaudeCode,
        thread_id,
        thread_name: None,
        preview: result.preview,
        completeness: ArchiveCompleteness::Degraded,
        source_kind: ConnectorSourceKind::ClaudeSessionPath,
        thread_status: ArchiveThreadStatus::Unknown("archival-only".to_string()),
        cwd: result.cwd,
        path: Some(session_path),
        model_provider: None,
        created_at: result.created_at,
        updated_at: result.updated_at,
        rounds: result.rounds,
    })
}

fn load_entries(session_path: &Path) -> Result<Vec<Value>> {
    let content = fs::read_to_string(session_path)
        .with_context(|| format!("failed to read Claude session `{}`", session_path.display()))?;

    if content.trim().is_empty() {
        bail!("Claude session file is empty: {}", session_path.display());
    }

    if let Ok(json_value) = serde_json::from_str::<Value>(&content)
        && let Some(entries) = json_value.get("loglines").and_then(Value::as_array)
    {
        return Ok(entries.clone());
    }

    let mut entries = Vec::new();
    for (index, line) in content.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let entry = serde_json::from_str::<Value>(line).with_context(|| {
            format!(
                "failed to parse Claude session JSONL line {} in `{}`",
                index + 1,
                session_path.display()
            )
        })?;
        entries.push(entry);
    }

    if entries.is_empty() {
        bail!(
            "Claude session `{}` did not contain any JSONL entries",
            session_path.display()
        );
    }

    Ok(entries)
}

#[derive(Default)]
struct ClaudeReplayResult {
    thread_id: Option<String>,
    preview: Option<String>,
    cwd: Option<PathBuf>,
    created_at: Option<i64>,
    updated_at: Option<i64>,
    rounds: Vec<ArchiveRound>,
}

#[derive(Default)]
struct ClaudeReplay {
    rounds: Vec<ArchiveRound>,
    current_round: Option<ArchiveRound>,
    next_round_index: usize,
    next_item_index: usize,
    thread_id: Option<String>,
    preview: Option<String>,
    preview_from_summary: bool,
    cwd: Option<PathBuf>,
    created_at: Option<i64>,
    updated_at: Option<i64>,
    pending_tool_uses: HashMap<String, PendingClaudeToolUse>,
    pending_tool_order: Vec<String>,
}

enum PendingClaudeToolUse {
    Command {
        id: String,
        command: String,
        cwd: Option<PathBuf>,
    },
    FileChange {
        id: String,
        path: String,
        kind: String,
    },
    Dynamic {
        id: String,
        tool: String,
        content_items: Vec<String>,
    },
}

impl ClaudeReplay {
    fn new() -> Self {
        Self {
            next_round_index: 1,
            next_item_index: 1,
            ..Self::default()
        }
    }

    fn handle_entry(&mut self, entry: &Value) {
        let Some(record) = entry.as_object() else {
            return;
        };

        self.capture_metadata(record);

        match get_string(record, "type").as_str() {
            "user" => self.handle_user_entry(record),
            "assistant" => self.handle_assistant_entry(record),
            "summary" => self.handle_summary_entry(record),
            "queue-operation" | "progress" => {}
            _ => {}
        }
    }

    fn finish(mut self) -> ClaudeReplayResult {
        self.flush_pending_tool_uses();
        self.finish_current_round();
        ClaudeReplayResult {
            thread_id: self.thread_id,
            preview: self.preview,
            cwd: self.cwd,
            created_at: self.created_at,
            updated_at: self.updated_at,
            rounds: self.rounds,
        }
    }

    fn capture_metadata(&mut self, record: &Map<String, Value>) {
        if self.thread_id.is_none() {
            self.thread_id = optional_string(record, "sessionId");
        }
        if self.cwd.is_none() {
            self.cwd = optional_path(record, "cwd");
        }
        if let Some(timestamp) =
            optional_string(record, "timestamp").and_then(|value| parse_timestamp(&value))
        {
            self.created_at = Some(
                self.created_at
                    .map(|current| current.min(timestamp))
                    .unwrap_or(timestamp),
            );
            self.updated_at = Some(
                self.updated_at
                    .map(|current| current.max(timestamp))
                    .unwrap_or(timestamp),
            );
        }
    }

    fn handle_summary_entry(&mut self, record: &Map<String, Value>) {
        if self.preview.is_some() {
            return;
        }
        if let Some(summary) = optional_string(record, "summary") {
            self.preview = Some(summary);
            self.preview_from_summary = true;
        }
    }

    fn handle_user_entry(&mut self, record: &Map<String, Value>) {
        let Some(message) = record.get("message").and_then(Value::as_object) else {
            return;
        };

        let content = message.get("content");
        let prompt_text = extract_user_text(content);
        let tool_results = extract_tool_results(content);

        if !prompt_text.is_empty() {
            self.flush_pending_tool_uses();
            self.start_new_round(record);
            if self.preview.is_none() || self.preview_from_summary {
                self.preview = Some(prompt_text.clone());
                self.preview_from_summary = false;
            }
            let item_id =
                optional_string(record, "uuid").unwrap_or_else(|| self.next_item_id("user"));
            self.push_item(ArchiveTurnItem::UserMessage {
                id: item_id,
                text: prompt_text,
                images: Vec::new(),
            });
        } else if !tool_results.is_empty() {
            self.ensure_round(record);
        }

        for result in tool_results {
            let tool_call = self.map_tool_result(result);
            self.push_item(ArchiveTurnItem::ToolCall(tool_call));
        }
    }

    fn handle_assistant_entry(&mut self, record: &Map<String, Value>) {
        let Some(message) = record.get("message").and_then(Value::as_object) else {
            return;
        };

        self.ensure_round(record);
        match message.get("content") {
            Some(Value::String(text)) => {
                let text = text.trim();
                if !text.is_empty() {
                    let item_id = self.next_item_id("assistant");
                    self.push_item(ArchiveTurnItem::AssistantMessage {
                        id: item_id,
                        text: text.to_string(),
                        phase: None,
                    });
                }
            }
            Some(Value::Array(items)) => {
                for item in items.iter().filter_map(Value::as_object) {
                    match get_string(item, "type").as_str() {
                        "text" => {
                            if let Some(text) = optional_string(item, "text") {
                                let item_id = self.next_item_id("assistant");
                                self.push_item(ArchiveTurnItem::AssistantMessage {
                                    id: item_id,
                                    text,
                                    phase: None,
                                });
                            }
                        }
                        "thinking" => {
                            if let Some(thinking) = optional_string(item, "thinking") {
                                let item_id = self.next_item_id("reasoning");
                                self.push_item(ArchiveTurnItem::Reasoning {
                                    id: item_id,
                                    summary: Vec::new(),
                                    content: vec![thinking],
                                });
                            }
                        }
                        "tool_use" => {
                            self.register_tool_use(item);
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    fn register_tool_use(&mut self, item: &Map<String, Value>) {
        let id = optional_string(item, "id").unwrap_or_else(|| self.next_item_id("tool-use"));
        let tool = optional_string(item, "name").unwrap_or_else(|| "tool_use".to_string());
        let pending = match tool.as_str() {
            "Bash" => PendingClaudeToolUse::Command {
                id: id.clone(),
                command: item
                    .get("input")
                    .and_then(Value::as_object)
                    .and_then(|input| optional_string(input, "command"))
                    .unwrap_or_default(),
                cwd: self.cwd.clone(),
            },
            "Write" | "Edit" | "MultiEdit" => PendingClaudeToolUse::FileChange {
                id: id.clone(),
                path: item
                    .get("input")
                    .and_then(Value::as_object)
                    .and_then(|input| optional_string(input, "file_path"))
                    .unwrap_or_default(),
                kind: tool.to_ascii_lowercase(),
            },
            _ => {
                let mut content_items = vec![format!("tool_use_id: `{id}`")];
                if let Some(input) = item.get("input") {
                    content_items.push(code_fence("json", &stringify_json(input)));
                }
                PendingClaudeToolUse::Dynamic {
                    id: id.clone(),
                    tool,
                    content_items,
                }
            }
        };

        self.pending_tool_order.push(id.clone());
        self.pending_tool_uses.insert(id, pending);
    }

    fn map_tool_result(&mut self, item: Map<String, Value>) -> ArchiveToolCall {
        let tool_use_id = optional_string(&item, "tool_use_id");
        let is_error = item
            .get("is_error")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let result_status = if is_error { "error" } else { "completed" }.to_string();
        let rendered_content = item.get("content").map(render_content_value);

        if let Some(tool_use_id) = tool_use_id.clone() {
            if let Some(index) = self
                .pending_tool_order
                .iter()
                .position(|id| id == &tool_use_id)
            {
                self.pending_tool_order.remove(index);
            }
            if let Some(pending) = self.pending_tool_uses.remove(&tool_use_id) {
                return match pending {
                    PendingClaudeToolUse::Command { id, command, cwd } => {
                        ArchiveToolCall::CommandExecution(CommandExecutionRecord {
                            id,
                            command,
                            cwd,
                            status: Some(result_status),
                            aggregated_output: rendered_content,
                            exit_code: None,
                        })
                    }
                    PendingClaudeToolUse::FileChange { id, path, kind } => {
                        ArchiveToolCall::FileChange {
                            id,
                            status: Some(result_status),
                            changes: vec![FileChangeRecord {
                                path,
                                kind: Some(kind),
                                diff: None,
                            }],
                        }
                    }
                    PendingClaudeToolUse::Dynamic {
                        id,
                        tool,
                        mut content_items,
                    } => {
                        if let Some(content) = rendered_content {
                            content_items.push(content);
                        }
                        ArchiveToolCall::DynamicToolCall(DynamicToolCallRecord {
                            id,
                            tool,
                            status: Some(result_status),
                            content_items,
                            success: Some(!is_error),
                        })
                    }
                };
            }
        }

        let id = self.next_item_id("tool-result");
        let mut content_items = Vec::new();
        if let Some(tool_use_id) = tool_use_id {
            content_items.push(format!("tool_use_id: `{tool_use_id}`"));
        }
        if let Some(content) = rendered_content {
            content_items.push(content);
        }
        ArchiveToolCall::DynamicToolCall(DynamicToolCallRecord {
            id,
            tool: "tool_result".to_string(),
            status: Some(result_status),
            content_items,
            success: Some(!is_error),
        })
    }

    fn start_new_round(&mut self, record: &Map<String, Value>) {
        self.finish_current_round();
        self.current_round = Some(ArchiveRound {
            turn_id: optional_string(record, "uuid")
                .unwrap_or_else(|| format!("claude-round-{}", self.next_round_index)),
            status: ArchiveTurnStatus::Completed,
            error: None,
            items: Vec::new(),
        });
        self.next_round_index += 1;
    }

    fn ensure_round(&mut self, record: &Map<String, Value>) {
        if self.current_round.is_none() {
            self.start_new_round(record);
        }
    }

    fn push_item(&mut self, item: ArchiveTurnItem) {
        if let Some(round) = self.current_round.as_mut() {
            round.items.push(item);
        }
    }

    fn finish_current_round(&mut self) {
        if let Some(round) = self.current_round.take()
            && !round.items.is_empty()
        {
            self.rounds.push(round);
        }
    }

    fn flush_pending_tool_uses(&mut self) {
        let pending_ids = self.pending_tool_order.drain(..).collect::<Vec<_>>();
        for pending_id in pending_ids {
            let Some(pending) = self.pending_tool_uses.remove(&pending_id) else {
                continue;
            };
            let tool_call = match pending {
                PendingClaudeToolUse::Command { id, command, cwd } => {
                    ArchiveToolCall::CommandExecution(CommandExecutionRecord {
                        id,
                        command,
                        cwd,
                        status: Some("called".to_string()),
                        aggregated_output: None,
                        exit_code: None,
                    })
                }
                PendingClaudeToolUse::FileChange { id, path, kind } => {
                    ArchiveToolCall::FileChange {
                        id,
                        status: Some("called".to_string()),
                        changes: vec![FileChangeRecord {
                            path,
                            kind: Some(kind),
                            diff: None,
                        }],
                    }
                }
                PendingClaudeToolUse::Dynamic {
                    id,
                    tool,
                    content_items,
                } => ArchiveToolCall::DynamicToolCall(DynamicToolCallRecord {
                    id,
                    tool,
                    status: Some("called".to_string()),
                    content_items,
                    success: None,
                }),
            };
            self.push_item(ArchiveTurnItem::ToolCall(tool_call));
        }
    }

    fn next_item_id(&mut self, prefix: &str) -> String {
        let id = format!("{prefix}-{}", self.next_item_index);
        self.next_item_index += 1;
        id
    }
}

fn extract_user_text(content: Option<&Value>) -> String {
    match content {
        Some(Value::String(text)) => text.trim().to_string(),
        Some(Value::Array(items)) => items
            .iter()
            .filter_map(Value::as_object)
            .filter_map(|item| match get_string(item, "type").as_str() {
                "text" => optional_string(item, "text"),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join(" ")
            .trim()
            .to_string(),
        _ => String::new(),
    }
}

fn extract_tool_results(content: Option<&Value>) -> Vec<Map<String, Value>> {
    content
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_object)
                .filter(|item| get_string(item, "type") == "tool_result")
                .cloned()
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn parse_timestamp(value: &str) -> Option<i64> {
    DateTime::parse_from_rfc3339(value)
        .ok()
        .map(|timestamp| timestamp.timestamp())
}

fn optional_path(record: &Map<String, Value>, field: &str) -> Option<PathBuf> {
    record.get(field).and_then(Value::as_str).map(PathBuf::from)
}

fn optional_string(record: &Map<String, Value>, field: &str) -> Option<String> {
    record
        .get(field)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn get_string(record: &Map<String, Value>, field: &str) -> String {
    optional_string(record, field).unwrap_or_default()
}

fn render_content_value(value: &Value) -> String {
    match value {
        Value::String(text) => code_fence("text", text),
        other => code_fence("json", &stringify_json(other)),
    }
}

fn stringify_json(value: &Value) -> String {
    if let Some(text) = value.as_str() {
        text.to_string()
    } else {
        serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string())
    }
}

fn code_fence(language: &str, content: &str) -> String {
    format!("```{language}\n{content}\n```")
}
