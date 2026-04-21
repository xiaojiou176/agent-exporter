mod app_server;
mod local;
pub mod state_index;

use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use serde_json::{Map, Value};

use crate::core::archive::{
    ArchiveCompleteness, ArchiveRound, ArchiveThreadStatus, ArchiveToolCall, ArchiveTranscript,
    ArchiveTurnError, ArchiveTurnItem, ArchiveTurnStatus, CommandExecutionRecord,
    ConnectorSourceKind, DynamicToolCallRecord, ExportRequest, ExportSelector, ExportSource,
    FileChangeRecord, McpToolCallRecord,
};
use crate::model::{ConnectorDefinition, ConnectorKind, SupportStage};

pub use self::app_server::{AppServerClient, AppServerResponseError};

pub const DEFINITION: ConnectorDefinition = ConnectorDefinition {
    kind: ConnectorKind::Codex,
    stage: SupportStage::Current,
    summary: "Canonical Codex app-server export path for the shared archive transcript contract.",
    source_of_truth: "CodexMonitor export contract + official Codex thread/read pipeline",
};

pub fn load_transcript(request: &ExportRequest) -> Result<ArchiveTranscript> {
    match request.source {
        ExportSource::AppServer => load_app_server_transcript(request),
        ExportSource::Local => local::load_transcript(request),
        ExportSource::SessionPath => {
            bail!("codex connector does not support `session-path`; use `app-server` or `local`")
        }
    }
}

fn load_app_server_transcript(request: &ExportRequest) -> Result<ArchiveTranscript> {
    let thread_id = match &request.selector {
        ExportSelector::ThreadId(thread_id) => thread_id.clone(),
        ExportSelector::RolloutPath(_) => {
            bail!("app-server source does not support `--rollout-path`; use `--thread-id`")
        }
        ExportSelector::SessionPath(_) => {
            bail!("app-server source does not support `--session-path`; use `--thread-id`")
        }
    };

    let mut client = AppServerClient::spawn(&request.app_server)
        .with_context(|| "failed to launch the Codex app-server client")?;
    client.initialize()?;

    match client.read_thread(&thread_id, true) {
        Ok(result) => map_thread_result(
            result,
            ArchiveCompleteness::Complete,
            ConnectorSourceKind::AppServerThreadRead,
        ),
        Err(error) if should_fallback_to_resume(&error) => {
            let result = client.resume_thread(&thread_id).with_context(|| {
                format!(
                    "thread/read fallback triggered for `{thread_id}`, but thread/resume failed"
                )
            })?;
            let transcript = map_thread_result(
                result,
                ArchiveCompleteness::Incomplete,
                ConnectorSourceKind::AppServerResumeFallback,
            )?;
            if transcript.item_count() == 0 {
                bail!(
                    "Current thread could not provide exportable history via thread/read or live resume fallback."
                );
            }
            Ok(transcript)
        }
        Err(error) => {
            Err(error).with_context(|| format!("failed to read Codex thread `{thread_id}`"))
        }
    }
}

fn should_fallback_to_resume(error: &AppServerResponseError) -> bool {
    error
        .message
        .contains("includeTurns is unavailable before first user message")
        || error
            .message
            .contains("ephemeral threads do not support includeTurns")
}

fn map_thread_result(
    result: Value,
    completeness: ArchiveCompleteness,
    source_kind: ConnectorSourceKind,
) -> Result<ArchiveTranscript> {
    let thread = result
        .get("thread")
        .and_then(Value::as_object)
        .context("Codex app-server result did not include a `thread` object")?;

    let rounds = thread
        .get("turns")
        .and_then(Value::as_array)
        .map(|turns| turns.iter().filter_map(map_round).collect::<Vec<_>>())
        .unwrap_or_default();

    Ok(ArchiveTranscript {
        connector: ConnectorKind::Codex,
        thread_id: get_string(thread, "id"),
        thread_name: optional_string(thread, "name"),
        preview: optional_string(thread, "preview"),
        completeness,
        source_kind,
        thread_status: map_thread_status(thread.get("status")),
        cwd: optional_path(thread, "cwd"),
        path: optional_path(thread, "path"),
        model_provider: optional_string(thread, "modelProvider"),
        created_at: optional_i64(thread, "createdAt"),
        updated_at: optional_i64(thread, "updatedAt"),
        rounds,
    })
}

fn map_round(value: &Value) -> Option<ArchiveRound> {
    let turn = value.as_object()?;
    let items = turn
        .get("items")
        .and_then(Value::as_array)
        .map(|items| items.iter().filter_map(map_item).collect::<Vec<_>>())
        .unwrap_or_default();

    Some(ArchiveRound {
        turn_id: get_string(turn, "id"),
        status: map_turn_status(turn.get("status")),
        error: turn
            .get("error")
            .and_then(Value::as_object)
            .and_then(map_turn_error),
        items,
    })
}

fn map_turn_error(record: &Map<String, Value>) -> Option<ArchiveTurnError> {
    let message = get_string(record, "message");
    if message.is_empty() {
        None
    } else {
        Some(ArchiveTurnError { message })
    }
}

fn map_thread_status(status: Option<&Value>) -> ArchiveThreadStatus {
    let status_type = status
        .and_then(Value::as_object)
        .and_then(|record| record.get("type"))
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    match status_type {
        "notLoaded" => ArchiveThreadStatus::NotLoaded,
        "idle" => ArchiveThreadStatus::Idle,
        "systemError" => ArchiveThreadStatus::SystemError,
        "active" => ArchiveThreadStatus::Active,
        other => ArchiveThreadStatus::Unknown(other.to_string()),
    }
}

fn map_turn_status(status: Option<&Value>) -> ArchiveTurnStatus {
    match status.and_then(Value::as_str).unwrap_or("unknown") {
        "completed" => ArchiveTurnStatus::Completed,
        "interrupted" => ArchiveTurnStatus::Interrupted,
        "failed" => ArchiveTurnStatus::Failed,
        "inProgress" => ArchiveTurnStatus::InProgress,
        other => ArchiveTurnStatus::Unknown(other.to_string()),
    }
}

fn map_item(value: &Value) -> Option<ArchiveTurnItem> {
    let record = value.as_object()?;
    let item_type = get_string(record, "type");
    match item_type.as_str() {
        "userMessage" => Some(map_user_message(record)),
        "hookPrompt" => Some(ArchiveTurnItem::HookPrompt {
            id: get_string(record, "id"),
            fragments: record
                .get("fragments")
                .and_then(Value::as_array)
                .map(|fragments| {
                    fragments
                        .iter()
                        .filter_map(Value::as_object)
                        .filter_map(|fragment| optional_string(fragment, "text"))
                        .filter(|text| !text.trim().is_empty())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default(),
        }),
        "agentMessage" => Some(ArchiveTurnItem::AssistantMessage {
            id: get_string(record, "id"),
            text: get_string(record, "text"),
            phase: optional_string(record, "phase"),
        }),
        "plan" => Some(ArchiveTurnItem::Plan {
            id: get_string(record, "id"),
            text: get_string(record, "text"),
        }),
        "reasoning" => Some(ArchiveTurnItem::Reasoning {
            id: get_string(record, "id"),
            summary: string_array(record.get("summary")),
            content: string_array(record.get("content")),
        }),
        "commandExecution" => Some(ArchiveTurnItem::ToolCall(
            ArchiveToolCall::CommandExecution(CommandExecutionRecord {
                id: get_string(record, "id"),
                command: parse_command(record.get("command")),
                cwd: optional_path(record, "cwd"),
                status: optional_string(record, "status"),
                aggregated_output: optional_string(record, "aggregatedOutput"),
                exit_code: optional_i32(record, "exitCode"),
            }),
        )),
        "fileChange" => Some(ArchiveTurnItem::ToolCall(ArchiveToolCall::FileChange {
            id: get_string(record, "id"),
            status: optional_string(record, "status"),
            changes: record
                .get("changes")
                .and_then(Value::as_array)
                .map(|changes| {
                    changes
                        .iter()
                        .filter_map(Value::as_object)
                        .map(|change| FileChangeRecord {
                            path: get_string(change, "path"),
                            kind: optional_string(change, "kind"),
                            diff: optional_string(change, "diff"),
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default(),
        })),
        "mcpToolCall" => Some(ArchiveTurnItem::ToolCall(ArchiveToolCall::McpToolCall(
            McpToolCallRecord {
                id: get_string(record, "id"),
                server: get_string(record, "server"),
                tool: get_string(record, "tool"),
                status: optional_string(record, "status"),
                result: record
                    .get("result")
                    .map(stringify_json)
                    .filter(|value| !value.trim().is_empty()),
                error: record
                    .get("error")
                    .map(stringify_json)
                    .filter(|value| !value.trim().is_empty()),
            },
        ))),
        "dynamicToolCall" => Some(ArchiveTurnItem::ToolCall(ArchiveToolCall::DynamicToolCall(
            DynamicToolCallRecord {
                id: get_string(record, "id"),
                tool: get_string(record, "tool"),
                status: optional_string(record, "status"),
                content_items: record
                    .get("contentItems")
                    .and_then(Value::as_array)
                    .map(|items| {
                        items
                            .iter()
                            .filter_map(Value::as_object)
                            .filter_map(|item| {
                                let item_type = get_string(item, "type");
                                match item_type.as_str() {
                                    "inputText" => optional_string(item, "text"),
                                    "inputImage" => optional_string(item, "imageUrl"),
                                    _ => None,
                                }
                            })
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default(),
                success: record.get("success").and_then(Value::as_bool),
            },
        ))),
        "collabAgentToolCall" => Some(ArchiveTurnItem::ToolCall(
            ArchiveToolCall::CollabAgentToolCall {
                id: get_string(record, "id"),
                tool: get_string(record, "tool"),
                status: optional_string(record, "status"),
                prompt: optional_string(record, "prompt"),
                receiver_thread_ids: record
                    .get("receiverThreadIds")
                    .and_then(Value::as_array)
                    .map(|ids| {
                        ids.iter()
                            .filter_map(Value::as_str)
                            .map(str::to_string)
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default(),
            },
        )),
        "webSearch" => Some(ArchiveTurnItem::ToolCall(ArchiveToolCall::WebSearch {
            id: get_string(record, "id"),
            query: get_string(record, "query"),
            action: record.get("action").map(stringify_json),
        })),
        "imageView" => Some(ArchiveTurnItem::ToolCall(ArchiveToolCall::ImageView {
            id: get_string(record, "id"),
            path: get_string(record, "path"),
        })),
        "contextCompaction" => Some(ArchiveTurnItem::ToolCall(ArchiveToolCall::LifecycleNote {
            id: get_string(record, "id"),
            label: "Context compaction".to_string(),
        })),
        "enteredReviewMode" => Some(ArchiveTurnItem::ToolCall(ArchiveToolCall::LifecycleNote {
            id: get_string(record, "id"),
            label: "Review mode entered".to_string(),
        })),
        "exitedReviewMode" => Some(ArchiveTurnItem::ToolCall(ArchiveToolCall::LifecycleNote {
            id: get_string(record, "id"),
            label: "Review mode exited".to_string(),
        })),
        other => Some(ArchiveTurnItem::ToolCall(ArchiveToolCall::Unsupported {
            id: get_string(record, "id"),
            kind: other.to_string(),
            payload: value.clone(),
        })),
    }
}

fn map_user_message(record: &Map<String, Value>) -> ArchiveTurnItem {
    let mut text_parts = Vec::new();
    let mut images = Vec::new();
    if let Some(content) = record.get("content").and_then(Value::as_array) {
        for input in content.iter().filter_map(Value::as_object) {
            match get_string(input, "type").as_str() {
                "text" => {
                    if let Some(text) = optional_string(input, "text") {
                        text_parts.push(text);
                    }
                }
                "skill" => {
                    if let Some(name) = optional_string(input, "name") {
                        text_parts.push(format!("${name}"));
                    }
                }
                "image" => {
                    if let Some(url) = optional_string(input, "url") {
                        images.push(url);
                    }
                }
                "localImage" => {
                    if let Some(path) = optional_string(input, "path") {
                        images.push(path);
                    }
                }
                _ => {}
            }
        }
    }
    ArchiveTurnItem::UserMessage {
        id: get_string(record, "id"),
        text: text_parts.join(" ").trim().to_string(),
        images,
    }
}

fn parse_command(value: Option<&Value>) -> String {
    match value {
        Some(Value::String(command)) => command.clone(),
        Some(Value::Array(parts)) => parts
            .iter()
            .filter_map(Value::as_str)
            .collect::<Vec<_>>()
            .join(" "),
        Some(other) => stringify_json(other),
        None => String::new(),
    }
}

fn string_array(value: Option<&Value>) -> Vec<String> {
    value
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
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

fn optional_i64(record: &Map<String, Value>, field: &str) -> Option<i64> {
    record.get(field).and_then(Value::as_i64)
}

fn optional_i32(record: &Map<String, Value>, field: &str) -> Option<i32> {
    record
        .get(field)
        .and_then(Value::as_i64)
        .and_then(|value| i32::try_from(value).ok())
}

fn stringify_json(value: &Value) -> String {
    if let Some(text) = value.as_str() {
        text.to_string()
    } else {
        serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::app_server::AppServerResponseError;
    use super::{map_thread_result, should_fallback_to_resume};
    use crate::core::archive::{ArchiveCompleteness, ConnectorSourceKind};

    #[test]
    fn fallback_only_matches_known_include_turns_errors() {
        assert!(should_fallback_to_resume(&AppServerResponseError {
            code: -32000,
            message: "ephemeral threads do not support includeTurns".into(),
            data: None,
        }));
        assert!(should_fallback_to_resume(&AppServerResponseError {
            code: -32000,
            message:
                "thread abc is not materialized yet; includeTurns is unavailable before first user message"
                    .into(),
            data: None,
        }));
        assert!(!should_fallback_to_resume(&AppServerResponseError {
            code: -32000,
            message: "thread/read failed: other".into(),
            data: None,
        }));
    }

    #[test]
    fn maps_thread_payload_into_typed_rounds() {
        let transcript = map_thread_result(
            json!({
                "thread": {
                    "id": "thr_123",
                    "preview": "hello from preview",
                    "status": { "type": "notLoaded" },
                    "cwd": "/tmp/workspace",
                    "turns": [
                        {
                            "id": "turn-1",
                            "status": "completed",
                            "items": [
                                {
                                    "type": "userMessage",
                                    "id": "item-1",
                                    "content": [{ "type": "text", "text": "hello" }]
                                },
                                {
                                    "type": "agentMessage",
                                    "id": "item-2",
                                    "text": "hi there"
                                },
                                {
                                    "type": "commandExecution",
                                    "id": "item-3",
                                    "command": "pwd",
                                    "cwd": "/tmp/workspace",
                                    "status": "completed",
                                    "aggregatedOutput": "/tmp/workspace\n",
                                    "exitCode": 0
                                }
                            ]
                        }
                    ]
                }
            }),
            ArchiveCompleteness::Complete,
            ConnectorSourceKind::AppServerThreadRead,
        )
        .expect("typed transcript");

        assert_eq!(transcript.thread_id, "thr_123");
        assert_eq!(transcript.rounds.len(), 1);
        assert_eq!(transcript.item_count(), 3);
        assert_eq!(transcript.rounds[0].turn_id, "turn-1");
    }
}
