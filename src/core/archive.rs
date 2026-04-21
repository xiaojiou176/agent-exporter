use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use chrono::{DateTime, Local};
use serde_json::Value;

use crate::model::{ConnectorKind, OutputFormat};
use crate::output::markdown::RenderedArchivePart;

const DEFAULT_EXPORT_STEM: &str = "agent-exporter-thread";
const MAX_THREAD_DISPLAY_NAME_FILENAME_CHARS: usize = 48;
// HOST_SAFETY_RULES_BEGIN
const BLOCKED_APP_SERVER_COMMANDS: &[&str] = &[
    "bash",
    "cmd",
    "cmd.exe",
    "fish",
    "kill",
    "killall",
    "open",
    "osascript",
    "pkill",
    "powershell",
    "pwsh",
    "sh",
    "zsh",
];
const BLOCKED_APP_SERVER_ARG_PATTERNS: &[&str] = &[
    "aevt,apwn",
    "force quit",
    "kaeshowapplicationwindow",
    "killall",
    "loginwindow",
    "pkill",
    "showforcequitpanel",
    "system events",
];
const BLOCKED_INLINE_EVAL_FLAGS: &[&str] = &["-c", "-e", "-m", "-r"];
// HOST_SAFETY_RULES_END

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExportSelector {
    ThreadId(String),
    RolloutPath(PathBuf),
    SessionPath(PathBuf),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExportSource {
    AppServer,
    Local,
    SessionPath,
}

impl ExportSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::AppServer => "app-server",
            Self::Local => "local",
            Self::SessionPath => "session-path",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppServerLaunchConfig {
    pub command: String,
    pub args: Vec<String>,
}

impl Default for AppServerLaunchConfig {
    fn default() -> Self {
        Self {
            command: "codex".to_string(),
            args: Vec::new(),
        }
    }
}

impl AppServerLaunchConfig {
    pub fn resolved_args(&self) -> Vec<String> {
        if self.args.is_empty() {
            vec!["app-server".to_string()]
        } else {
            self.args.clone()
        }
    }

    pub fn validate_host_safety(&self) -> Result<()> {
        let trimmed_command = self.command.trim();
        if trimmed_command.is_empty() {
            bail!("app-server command cannot be empty");
        }

        let command_name = app_server_command_name(trimmed_command);
        if BLOCKED_APP_SERVER_COMMANDS.contains(&command_name.as_str()) {
            bail!(
                "app-server override refuses to launch host-control utility `{}`; use a direct repo-owned app-server executable instead",
                trimmed_command
            );
        }

        if uses_inline_eval(&command_name, &self.args) {
            bail!(
                "app-server override refuses inline-eval launcher `{}`; pass a direct script or executable path instead",
                trimmed_command
            );
        }

        for arg in &self.args {
            let normalized = arg.trim().to_ascii_lowercase();
            if let Some(pattern) = BLOCKED_APP_SERVER_ARG_PATTERNS
                .iter()
                .find(|pattern| normalized.contains(**pattern))
            {
                bail!(
                    "app-server override rejects unsafe argument pattern `{}` in `{}`",
                    pattern,
                    arg
                );
            }
        }

        Ok(())
    }
}

fn app_server_command_name(command: &str) -> String {
    Path::new(command)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(command)
        .trim()
        .to_ascii_lowercase()
}

fn uses_inline_eval(command_name: &str, args: &[String]) -> bool {
    let Some(first_arg) = args.first().map(|arg| arg.trim()) else {
        return false;
    };
    if !BLOCKED_INLINE_EVAL_FLAGS.contains(&first_arg) {
        return false;
    }

    command_name.starts_with("python")
        || matches!(command_name, "node" | "nodejs" | "perl" | "php" | "ruby")
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OutputTarget {
    Downloads,
    WorkspaceConversations { workspace_root: PathBuf },
}

impl OutputTarget {
    pub fn resolve_output_dir(&self) -> Result<PathBuf> {
        match self {
            Self::Downloads => resolve_downloads_dir(),
            Self::WorkspaceConversations { workspace_root } => {
                if !workspace_root.exists() {
                    bail!(
                        "Current workspace root does not exist: {}. Try exporting to Downloads instead.",
                        workspace_root.display()
                    );
                }
                if !workspace_root.is_dir() {
                    bail!(
                        "Current workspace root is not a directory: {}. Try exporting to Downloads instead.",
                        workspace_root.display()
                    );
                }
                Ok(workspace_root.join(".agents").join("Conversations"))
            }
        }
    }

    pub fn workspace_display_name(&self) -> Option<String> {
        match self {
            Self::Downloads => None,
            Self::WorkspaceConversations { workspace_root } => workspace_root
                .file_name()
                .map(|name| name.to_string_lossy().to_string())
                .filter(|value| !value.trim().is_empty()),
        }
    }
}

fn resolve_downloads_dir() -> Result<PathBuf> {
    if let Some(path) = dirs::download_dir() {
        fs::create_dir_all(&path).with_context(|| {
            format!(
                "Failed to ensure Downloads folder exists at {}.",
                path.display()
            )
        })?;
        return Ok(path);
    }

    let home =
        dirs::home_dir().context("Failed to resolve Downloads folder from this environment.")?;
    let fallback = home.join("Downloads");
    fs::create_dir_all(&fallback).with_context(|| {
        format!(
            "Failed to ensure fallback Downloads folder exists at {}.",
            fallback.display()
        )
    })?;
    Ok(fallback)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExportRequest {
    pub connector: ConnectorKind,
    pub source: ExportSource,
    pub selector: ExportSelector,
    pub format: OutputFormat,
    pub output_target: OutputTarget,
    pub app_server: AppServerLaunchConfig,
    pub codex_home: Option<PathBuf>,
    pub ai_summary: AiSummaryOptions,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AiSummaryOptions {
    pub enabled: bool,
    pub instructions: Option<String>,
    pub timeout_seconds: Option<u64>,
    pub profile: Option<String>,
    pub preset: Option<String>,
    pub model: Option<String>,
    pub provider: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArchiveCompleteness {
    Complete,
    Incomplete,
    Degraded,
}

impl ArchiveCompleteness {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Complete => "complete",
            Self::Incomplete => "incomplete",
            Self::Degraded => "degraded",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConnectorSourceKind {
    AppServerThreadRead,
    AppServerResumeFallback,
    LocalThreadId,
    LocalRolloutPath,
    ClaudeSessionPath,
}

impl ConnectorSourceKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::AppServerThreadRead => "app-server-thread-read",
            Self::AppServerResumeFallback => "app-server-resume-fallback",
            Self::LocalThreadId => "local-thread-id",
            Self::LocalRolloutPath => "local-rollout-path",
            Self::ClaudeSessionPath => "claude-session-path",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ArchiveThreadStatus {
    NotLoaded,
    Idle,
    SystemError,
    Active,
    Unknown(String),
}

impl ArchiveThreadStatus {
    pub fn as_str(&self) -> &str {
        match self {
            Self::NotLoaded => "notLoaded",
            Self::Idle => "idle",
            Self::SystemError => "systemError",
            Self::Active => "active",
            Self::Unknown(value) => value.as_str(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ArchiveTranscript {
    pub connector: ConnectorKind,
    pub thread_id: String,
    pub thread_name: Option<String>,
    pub preview: Option<String>,
    pub completeness: ArchiveCompleteness,
    pub source_kind: ConnectorSourceKind,
    pub thread_status: ArchiveThreadStatus,
    pub cwd: Option<PathBuf>,
    pub path: Option<PathBuf>,
    pub model_provider: Option<String>,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
    pub rounds: Vec<ArchiveRound>,
}

impl ArchiveTranscript {
    pub fn item_count(&self) -> usize {
        self.rounds.iter().map(|round| round.items.len()).sum()
    }

    pub fn round_count(&self) -> usize {
        self.rounds.len()
    }

    pub fn archive_title(&self, output_target: &OutputTarget) -> String {
        output_target
            .workspace_display_name()
            .or_else(|| {
                self.cwd.as_ref().and_then(|cwd| {
                    cwd.file_name()
                        .map(|name| name.to_string_lossy().to_string())
                })
            })
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "agent-exporter".to_string())
    }

    pub fn thread_display_name(&self) -> Option<&str> {
        self.thread_name
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
    }

    pub fn has_user_messages(&self) -> bool {
        self.rounds.iter().any(|round| {
            round
                .items
                .iter()
                .any(|item| matches!(item, ArchiveTurnItem::UserMessage { .. }))
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ArchiveRound {
    pub turn_id: String,
    pub status: ArchiveTurnStatus,
    pub error: Option<ArchiveTurnError>,
    pub items: Vec<ArchiveTurnItem>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ArchiveTurnStatus {
    Completed,
    Interrupted,
    Failed,
    InProgress,
    Unknown(String),
}

impl ArchiveTurnStatus {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Completed => "completed",
            Self::Interrupted => "interrupted",
            Self::Failed => "failed",
            Self::InProgress => "inProgress",
            Self::Unknown(value) => value.as_str(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArchiveTurnError {
    pub message: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ArchiveTurnItem {
    UserMessage {
        id: String,
        text: String,
        images: Vec<String>,
    },
    HookPrompt {
        id: String,
        fragments: Vec<String>,
    },
    AssistantMessage {
        id: String,
        text: String,
        phase: Option<String>,
    },
    Plan {
        id: String,
        text: String,
    },
    Reasoning {
        id: String,
        summary: Vec<String>,
        content: Vec<String>,
    },
    ToolCall(ArchiveToolCall),
}

#[derive(Clone, Debug, PartialEq)]
pub enum ArchiveToolCall {
    CommandExecution(CommandExecutionRecord),
    FileChange {
        id: String,
        status: Option<String>,
        changes: Vec<FileChangeRecord>,
    },
    McpToolCall(McpToolCallRecord),
    DynamicToolCall(DynamicToolCallRecord),
    CollabAgentToolCall {
        id: String,
        tool: String,
        status: Option<String>,
        prompt: Option<String>,
        receiver_thread_ids: Vec<String>,
    },
    WebSearch {
        id: String,
        query: String,
        action: Option<String>,
    },
    ImageView {
        id: String,
        path: String,
    },
    LifecycleNote {
        id: String,
        label: String,
    },
    Unsupported {
        id: String,
        kind: String,
        payload: Value,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandExecutionRecord {
    pub id: String,
    pub command: String,
    pub cwd: Option<PathBuf>,
    pub status: Option<String>,
    pub aggregated_output: Option<String>,
    pub exit_code: Option<i32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FileChangeRecord {
    pub path: String,
    pub kind: Option<String>,
    pub diff: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct McpToolCallRecord {
    pub id: String,
    pub server: String,
    pub tool: String,
    pub status: Option<String>,
    pub result: Option<String>,
    pub error: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DynamicToolCallRecord {
    pub id: String,
    pub tool: String,
    pub status: Option<String>,
    pub content_items: Vec<String>,
    pub success: Option<bool>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExportOutcome {
    pub output_paths: Vec<PathBuf>,
    pub exported_part_count: usize,
    pub exported_item_count: usize,
    pub exported_turn_count: usize,
    pub completeness: ArchiveCompleteness,
}

pub fn write_markdown_parts(
    transcript: &ArchiveTranscript,
    output_target: &OutputTarget,
    parts: &[RenderedArchivePart],
) -> Result<ExportOutcome> {
    write_markdown_parts_at(transcript, output_target, parts, Local::now())
}

fn write_markdown_parts_at(
    transcript: &ArchiveTranscript,
    output_target: &OutputTarget,
    parts: &[RenderedArchivePart],
    now: DateTime<Local>,
) -> Result<ExportOutcome> {
    if parts.is_empty() {
        bail!("At least one Markdown export part is required.");
    }

    let output_dir = output_target.resolve_output_dir()?;
    fs::create_dir_all(&output_dir).with_context(|| {
        format!(
            "Failed to prepare export directory `{}`",
            output_dir.display()
        )
    })?;

    let filename_stem = build_thread_archive_filename_stem(
        output_target.workspace_display_name().as_deref(),
        transcript.thread_display_name(),
        &transcript.thread_id,
    );
    let timestamp = now.format("%Y-%m-%d_%H-%M-%S").to_string();

    for attempt in 0..1000usize {
        let paths = parts
            .iter()
            .map(|part| {
                output_dir.join(build_archive_part_filename(
                    &filename_stem,
                    &timestamp,
                    part,
                    attempt,
                ))
            })
            .collect::<Vec<_>>();

        if paths.iter().any(|path| path.exists()) {
            continue;
        }

        for (path, part) in paths.iter().zip(parts.iter()) {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).with_context(|| {
                    format!("Failed to prepare export directory `{}`", parent.display())
                })?;
            }
            fs::write(path, &part.content)
                .with_context(|| format!("Failed to write export file `{}`", path.display()))?;
        }

        return Ok(ExportOutcome {
            output_paths: paths,
            exported_part_count: parts.len(),
            exported_item_count: transcript.item_count(),
            exported_turn_count: transcript.round_count(),
            completeness: transcript.completeness,
        });
    }

    bail!("Failed to allocate unique archive export filenames.")
}

pub fn write_json_document(
    transcript: &ArchiveTranscript,
    output_target: &OutputTarget,
    document: &Value,
) -> Result<ExportOutcome> {
    write_json_document_at(transcript, output_target, document, Local::now())
}

fn write_json_document_at(
    transcript: &ArchiveTranscript,
    output_target: &OutputTarget,
    document: &Value,
    now: DateTime<Local>,
) -> Result<ExportOutcome> {
    let output_dir = output_target.resolve_output_dir()?;
    fs::create_dir_all(&output_dir).with_context(|| {
        format!(
            "Failed to prepare export directory `{}`",
            output_dir.display()
        )
    })?;

    let filename_stem = build_thread_archive_filename_stem(
        output_target.workspace_display_name().as_deref(),
        transcript.thread_display_name(),
        &transcript.thread_id,
    );
    let timestamp = now.format("%Y-%m-%d_%H-%M-%S").to_string();
    let rendered = format!(
        "{}\n",
        serde_json::to_string_pretty(document).context("failed to render JSON export document")?
    );
    let start_round = usize::from(transcript.round_count() > 0);
    let end_round = transcript.round_count();

    for attempt in 0..1000usize {
        let path = output_dir.join(build_archive_document_filename(
            &filename_stem,
            &timestamp,
            start_round,
            end_round,
            "json",
            attempt,
        ));

        if path.exists() {
            continue;
        }

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("Failed to prepare export directory `{}`", parent.display())
            })?;
        }
        fs::write(&path, &rendered)
            .with_context(|| format!("Failed to write export file `{}`", path.display()))?;

        return Ok(ExportOutcome {
            output_paths: vec![path],
            exported_part_count: 1,
            exported_item_count: transcript.item_count(),
            exported_turn_count: transcript.round_count(),
            completeness: transcript.completeness,
        });
    }

    bail!("Failed to allocate unique archive export filenames.")
}

pub fn write_html_document(
    transcript: &ArchiveTranscript,
    output_target: &OutputTarget,
    document: &str,
) -> Result<ExportOutcome> {
    write_html_document_at(transcript, output_target, document, Local::now())
}

fn write_html_document_at(
    transcript: &ArchiveTranscript,
    output_target: &OutputTarget,
    document: &str,
    now: DateTime<Local>,
) -> Result<ExportOutcome> {
    let output_dir = output_target.resolve_output_dir()?;
    fs::create_dir_all(&output_dir).with_context(|| {
        format!(
            "Failed to prepare export directory `{}`",
            output_dir.display()
        )
    })?;

    let filename_stem = build_thread_archive_filename_stem(
        output_target.workspace_display_name().as_deref(),
        transcript.thread_display_name(),
        &transcript.thread_id,
    );
    let timestamp = now.format("%Y-%m-%d_%H-%M-%S").to_string();
    let rendered = format!("{}\n", document.trim_end());
    let start_round = usize::from(transcript.round_count() > 0);
    let end_round = transcript.round_count();

    for attempt in 0..1000usize {
        let path = output_dir.join(build_archive_document_filename(
            &filename_stem,
            &timestamp,
            start_round,
            end_round,
            "html",
            attempt,
        ));

        if path.exists() {
            continue;
        }

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("Failed to prepare export directory `{}`", parent.display())
            })?;
        }
        fs::write(&path, &rendered)
            .with_context(|| format!("Failed to write export file `{}`", path.display()))?;

        return Ok(ExportOutcome {
            output_paths: vec![path],
            exported_part_count: 1,
            exported_item_count: transcript.item_count(),
            exported_turn_count: transcript.round_count(),
            completeness: transcript.completeness,
        });
    }

    bail!("Failed to allocate unique archive export filenames.")
}

pub fn allocate_ai_summary_document_path(
    transcript: &ArchiveTranscript,
    output_target: &OutputTarget,
) -> Result<PathBuf> {
    allocate_ai_summary_document_path_at(transcript, output_target, Local::now())
}

fn allocate_ai_summary_document_path_at(
    transcript: &ArchiveTranscript,
    output_target: &OutputTarget,
    now: DateTime<Local>,
) -> Result<PathBuf> {
    let output_dir = output_target.resolve_output_dir()?;
    fs::create_dir_all(&output_dir).with_context(|| {
        format!(
            "Failed to prepare export directory `{}`",
            output_dir.display()
        )
    })?;

    let filename_stem = build_thread_archive_filename_stem(
        output_target.workspace_display_name().as_deref(),
        transcript.thread_display_name(),
        &transcript.thread_id,
    );
    let timestamp = now.format("%Y-%m-%d_%H-%M-%S").to_string();
    let start_round = usize::from(transcript.round_count() > 0);
    let end_round = transcript.round_count();

    for attempt in 0..1000usize {
        let path = output_dir.join(build_archive_ai_summary_filename(
            &filename_stem,
            &timestamp,
            start_round,
            end_round,
            attempt,
        ));
        if path.exists() {
            continue;
        }
        return Ok(path);
    }

    bail!("Failed to allocate unique AI summary filenames.")
}

fn sanitize_filename_component(value: &str) -> String {
    let sanitized: String = value
        .chars()
        .map(|ch| match ch {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => '-',
            c if c.is_control() => ' ',
            c => c,
        })
        .collect();

    let collapsed = sanitized
        .split_whitespace()
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>()
        .join("-");

    let trimmed = collapsed.trim_matches(|ch: char| ch == '-' || ch == '.');
    if trimmed.is_empty() {
        DEFAULT_EXPORT_STEM.to_string()
    } else {
        trimmed.to_string()
    }
}

fn truncate_filename_component(value: &str, max_chars: usize) -> String {
    if max_chars == 0 {
        return String::new();
    }
    value
        .chars()
        .take(max_chars)
        .collect::<String>()
        .trim_matches(|ch: char| ch == '-' || ch == '.')
        .to_string()
}

fn normalize_optional_filename_component(value: Option<&str>) -> Option<String> {
    let raw = value?.trim();
    if raw.is_empty() {
        return None;
    }
    let sanitized = sanitize_filename_component(raw);
    if sanitized == DEFAULT_EXPORT_STEM {
        return None;
    }
    Some(sanitized)
}

pub(crate) fn build_thread_archive_filename_stem(
    workspace_name: Option<&str>,
    thread_display_name: Option<&str>,
    thread_id: &str,
) -> String {
    let safe_workspace_name =
        sanitize_filename_component(workspace_name.unwrap_or("agent-exporter"));
    let safe_thread_id =
        sanitize_filename_component(&thread_id.chars().take(8).collect::<String>());
    let safe_thread_display_name = normalize_optional_filename_component(thread_display_name)
        .map(|value| truncate_filename_component(&value, MAX_THREAD_DISPLAY_NAME_FILENAME_CHARS))
        .filter(|value| !value.is_empty());

    match safe_thread_display_name {
        Some(thread_name) => {
            format!("{safe_workspace_name}-thread-{thread_name}-{safe_thread_id}")
        }
        None => format!("{safe_workspace_name}-thread-{safe_thread_id}"),
    }
}

fn build_archive_ai_summary_filename(
    stem: &str,
    timestamp: &str,
    start_round: usize,
    end_round: usize,
    attempt: usize,
) -> String {
    let base = format!("{stem}-ai-summary-rounds-{start_round}-{end_round}-{timestamp}");
    if attempt == 0 {
        format!("{base}.md")
    } else {
        format!("{base}-{}.md", attempt + 1)
    }
}

fn build_archive_part_filename(
    stem: &str,
    timestamp: &str,
    part: &RenderedArchivePart,
    attempt: usize,
) -> String {
    let base = format!(
        "{stem}-part-{:02}-rounds-{}-{}-{timestamp}",
        part.part_index, part.start_round, part.end_round
    );
    if attempt == 0 {
        format!("{base}.md")
    } else {
        format!("{base}-{}.md", attempt + 1)
    }
}

fn build_archive_document_filename(
    stem: &str,
    timestamp: &str,
    start_round: usize,
    end_round: usize,
    extension: &str,
    attempt: usize,
) -> String {
    let base = format!("{stem}-rounds-{start_round}-{end_round}-{timestamp}");
    if attempt == 0 {
        format!("{base}.{extension}")
    } else {
        format!("{base}-{}.{}", attempt + 1, extension)
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;
    use tempfile::tempdir;

    use super::{
        AppServerLaunchConfig, ArchiveCompleteness, ArchiveRound, ArchiveThreadStatus,
        ArchiveTranscript, ArchiveTurnItem, ArchiveTurnStatus, ConnectorSourceKind, OutputTarget,
        write_html_document_at, write_json_document_at, write_markdown_parts_at,
    };
    use crate::model::ConnectorKind;
    use crate::output::markdown::RenderedArchivePart;

    fn sample_transcript() -> ArchiveTranscript {
        ArchiveTranscript {
            connector: ConnectorKind::Codex,
            thread_id: "thread-12345678".to_string(),
            thread_name: Some("demo".to_string()),
            preview: Some("hello world".to_string()),
            completeness: ArchiveCompleteness::Complete,
            source_kind: ConnectorSourceKind::AppServerThreadRead,
            thread_status: ArchiveThreadStatus::NotLoaded,
            cwd: None,
            path: None,
            model_provider: Some("openai".to_string()),
            created_at: None,
            updated_at: None,
            rounds: vec![ArchiveRound {
                turn_id: "turn-1".to_string(),
                status: ArchiveTurnStatus::Completed,
                error: None,
                items: vec![ArchiveTurnItem::UserMessage {
                    id: "item-1".to_string(),
                    text: "hello".to_string(),
                    images: Vec::new(),
                }],
            }],
        }
    }

    #[test]
    fn workspace_target_rejects_missing_root() {
        let missing = tempdir().expect("temp dir").path().join("missing-root");
        let error = OutputTarget::WorkspaceConversations {
            workspace_root: missing.clone(),
        }
        .resolve_output_dir()
        .expect_err("missing workspace root should fail");
        assert!(
            error
                .to_string()
                .contains("Try exporting to Downloads instead.")
        );
    }

    #[test]
    fn write_markdown_parts_increments_conflict_suffix() {
        let workspace = tempdir().expect("temp dir");
        let transcript = sample_transcript();
        let parts = vec![RenderedArchivePart {
            part_index: 1,
            start_round: 1,
            end_round: 1,
            content: "# demo".to_string(),
            line_count: 1,
        }];
        let target = OutputTarget::WorkspaceConversations {
            workspace_root: workspace.path().to_path_buf(),
        };
        let timestamp = chrono::Local
            .with_ymd_and_hms(2026, 4, 4, 12, 0, 0)
            .single()
            .expect("fixed timestamp");

        let first =
            write_markdown_parts_at(&transcript, &target, &parts, timestamp).expect("first export");
        let second = write_markdown_parts_at(&transcript, &target, &parts, timestamp)
            .expect("second export");

        assert_eq!(first.exported_part_count, 1);
        assert_eq!(second.exported_part_count, 1);
        assert_ne!(first.output_paths[0], second.output_paths[0]);
        assert!(
            second.output_paths[0]
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.ends_with("-2.md"))
        );
    }

    #[test]
    fn write_json_document_increments_conflict_suffix() {
        let workspace = tempdir().expect("temp dir");
        let transcript = sample_transcript();
        let document = serde_json::json!({
            "schema_version": 1,
            "transcript": {
                "thread_id": transcript.thread_id,
            }
        });
        let target = OutputTarget::WorkspaceConversations {
            workspace_root: workspace.path().to_path_buf(),
        };
        let timestamp = chrono::Local
            .with_ymd_and_hms(2026, 4, 4, 12, 0, 0)
            .single()
            .expect("fixed timestamp");

        let first = write_json_document_at(&transcript, &target, &document, timestamp)
            .expect("first export");
        let second = write_json_document_at(&transcript, &target, &document, timestamp)
            .expect("second export");

        assert_eq!(first.exported_part_count, 1);
        assert_eq!(second.exported_part_count, 1);
        assert_ne!(first.output_paths[0], second.output_paths[0]);
        assert!(
            second.output_paths[0]
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.ends_with("-2.json"))
        );
    }

    #[test]
    fn write_html_document_increments_conflict_suffix() {
        let workspace = tempdir().expect("temp dir");
        let transcript = sample_transcript();
        let document = "<!DOCTYPE html><html><body>demo</body></html>";
        let target = OutputTarget::WorkspaceConversations {
            workspace_root: workspace.path().to_path_buf(),
        };
        let timestamp = chrono::Local
            .with_ymd_and_hms(2026, 4, 4, 12, 0, 0)
            .single()
            .expect("fixed timestamp");

        let first = write_html_document_at(&transcript, &target, document, timestamp)
            .expect("first export");
        let second = write_html_document_at(&transcript, &target, document, timestamp)
            .expect("second export");

        assert_eq!(first.exported_part_count, 1);
        assert_eq!(second.exported_part_count, 1);
        assert_ne!(first.output_paths[0], second.output_paths[0]);
        assert!(
            second.output_paths[0]
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.ends_with("-2.html"))
        );
    }

    #[test]
    fn default_app_server_launch_config_uses_codex_app_server() {
        let config = AppServerLaunchConfig::default();
        assert_eq!(config.command, "codex");
        assert_eq!(config.resolved_args(), vec!["app-server"]);
        config
            .validate_host_safety()
            .expect("default codex launcher should stay allowed");
    }
}
