use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use chrono::DateTime;
use rusqlite::{Connection, OpenFlags, OptionalExtension, params};
use serde_json::{Map, Value};

use crate::core::archive::{
    ArchiveCompleteness, ArchiveRound, ArchiveThreadStatus, ArchiveToolCall, ArchiveTranscript,
    ArchiveTurnError, ArchiveTurnItem, ArchiveTurnStatus, CommandExecutionRecord,
    ConnectorSourceKind, ExportRequest, ExportSelector, FileChangeRecord,
};

#[derive(Debug)]
struct LocalThreadMetadata {
    thread_id: String,
    rollout_path: PathBuf,
    cwd: Option<PathBuf>,
    model_provider: Option<String>,
    created_at: Option<i64>,
    updated_at: Option<i64>,
    first_user_message: Option<String>,
}

pub fn load_transcript(request: &ExportRequest) -> Result<ArchiveTranscript> {
    match &request.selector {
        ExportSelector::ThreadId(thread_id) => {
            let codex_home = resolve_codex_home(request.codex_home.as_ref())?;
            let metadata = lookup_thread_metadata(&codex_home, thread_id)?;
            let rollout_path = resolve_rollout_path(&metadata.rollout_path, Some(&codex_home))?;
            load_rollout_transcript(
                &rollout_path,
                Some(&metadata),
                ConnectorSourceKind::LocalThreadId,
            )
        }
        ExportSelector::RolloutPath(rollout_path) => {
            let rollout_path = resolve_rollout_path(rollout_path, request.codex_home.as_deref())?;
            load_rollout_transcript(&rollout_path, None, ConnectorSourceKind::LocalRolloutPath)
        }
    }
}

fn resolve_codex_home(explicit: Option<&PathBuf>) -> Result<PathBuf> {
    if let Some(path) = explicit {
        return Ok(path.clone());
    }
    if let Ok(path) = env::var("CODEX_HOME") {
        let trimmed = path.trim();
        if !trimmed.is_empty() {
            return Ok(PathBuf::from(trimmed));
        }
    }
    let home_dir = dirs::home_dir().context("Failed to resolve the current home directory.")?;
    Ok(home_dir.join(".codex"))
}

fn state_db_path(codex_home: &Path) -> PathBuf {
    codex_home.join("state_5.sqlite")
}

fn lookup_thread_metadata(codex_home: &Path, thread_id: &str) -> Result<LocalThreadMetadata> {
    let state_db = state_db_path(codex_home);
    if !state_db.exists() {
        bail!(
            "local source could not find `{}`; pass `--codex-home <PATH>` or ensure the default Codex home exists",
            state_db.display()
        );
    }

    let connection = Connection::open_with_flags(&state_db, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .with_context(|| format!("failed to open local state db `{}`", state_db.display()))?;
    let mut statement = connection.prepare(
        "SELECT id, rollout_path, cwd, model_provider, created_at, updated_at, source, cli_version, first_user_message
         FROM threads
         WHERE id = ?1
         LIMIT 1",
    )?;

    let row = statement
        .query_row(params![thread_id], |row| {
            let rollout_path = row.get::<_, Option<String>>(1)?;
            Ok(LocalThreadMetadata {
                thread_id: row.get(0)?,
                rollout_path: PathBuf::from(rollout_path.unwrap_or_default()),
                cwd: row.get::<_, Option<String>>(2)?.map(PathBuf::from),
                model_provider: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
                first_user_message: row.get(8)?,
            })
        })
        .optional()?;

    let Some(metadata) = row else {
        bail!(
            "local source could not find thread `{thread_id}` in `{}`",
            state_db.display()
        );
    };

    if metadata.rollout_path.as_os_str().is_empty() {
        bail!(
            "thread `{thread_id}` exists in `{}`, but sqlite does not contain a rollout_path",
            state_db.display()
        );
    }

    Ok(metadata)
}

fn resolve_rollout_path(path: &Path, codex_home: Option<&Path>) -> Result<PathBuf> {
    let rollout_path = if path.is_absolute() {
        path.to_path_buf()
    } else if let Some(codex_home) = codex_home {
        codex_home.join(path)
    } else {
        env::current_dir()
            .context("failed to resolve current working directory for relative rollout path")?
            .join(path)
    };

    if !rollout_path.exists() {
        bail!("rollout file does not exist: {}", rollout_path.display());
    }
    Ok(rollout_path)
}

fn load_rollout_transcript(
    rollout_path: &Path,
    metadata: Option<&LocalThreadMetadata>,
    source_kind: ConnectorSourceKind,
) -> Result<ArchiveTranscript> {
    let file = File::open(rollout_path)
        .with_context(|| format!("failed to open rollout `{}`", rollout_path.display()))?;
    let reader = BufReader::new(file);
    let mut replay = LocalReplay::new();

    for line in reader.lines() {
        let line = line.with_context(|| {
            format!(
                "failed to read rollout line from `{}`",
                rollout_path.display()
            )
        })?;
        if line.trim().is_empty() {
            continue;
        }
        let entry: Value = serde_json::from_str(&line).with_context(|| {
            format!(
                "failed to parse rollout JSONL line in `{}`",
                rollout_path.display()
            )
        })?;
        replay.handle_entry(&entry);
    }

    let replay_result = replay.finish();
    let thread_id = metadata
        .map(|metadata| metadata.thread_id.clone())
        .or(replay_result.thread_id)
        .context("local source could not determine thread id from sqlite metadata or rollout session_meta")?;

    let preview = metadata
        .and_then(|metadata| metadata.first_user_message.clone())
        .or(replay_result.first_user_message);
    let cwd = metadata
        .and_then(|metadata| metadata.cwd.clone())
        .or(replay_result.cwd);
    let model_provider = metadata
        .and_then(|metadata| metadata.model_provider.clone())
        .or(replay_result.model_provider);
    let created_at = metadata
        .and_then(|metadata| metadata.created_at)
        .or(replay_result.created_at);
    let updated_at = metadata.and_then(|metadata| metadata.updated_at);

    Ok(ArchiveTranscript {
        connector: crate::model::ConnectorKind::Codex,
        thread_id,
        thread_name: None,
        preview,
        completeness: ArchiveCompleteness::Degraded,
        source_kind,
        thread_status: ArchiveThreadStatus::Unknown("archival-only".to_string()),
        cwd,
        path: Some(rollout_path.to_path_buf()),
        model_provider,
        created_at,
        updated_at,
        rounds: replay_result.rounds,
    })
}

#[derive(Default)]
struct LocalReplayResult {
    thread_id: Option<String>,
    first_user_message: Option<String>,
    cwd: Option<PathBuf>,
    model_provider: Option<String>,
    created_at: Option<i64>,
    rounds: Vec<ArchiveRound>,
}

#[derive(Default)]
struct LocalReplay {
    rounds: Vec<ArchiveRound>,
    current_round: Option<ArchiveRound>,
    next_item_index: usize,
    thread_id: Option<String>,
    first_user_message: Option<String>,
    cwd: Option<PathBuf>,
    model_provider: Option<String>,
    created_at: Option<i64>,
}

impl LocalReplay {
    fn new() -> Self {
        Self {
            rounds: Vec::new(),
            current_round: None,
            next_item_index: 1,
            thread_id: None,
            first_user_message: None,
            cwd: None,
            model_provider: None,
            created_at: None,
        }
    }

    fn handle_entry(&mut self, entry: &Value) {
        let Some(record) = entry.as_object() else {
            return;
        };
        match string_field(record, "type").as_deref() {
            Some("session_meta") => self.handle_session_meta(record.get("payload")),
            Some("turn_context") => self.handle_turn_context(record.get("payload")),
            Some("event_msg") => self.handle_event(record.get("payload")),
            Some("response_item") => self.handle_response_item(record.get("payload")),
            _ => {}
        }
    }

    fn finish(mut self) -> LocalReplayResult {
        self.finish_current_round();
        LocalReplayResult {
            thread_id: self.thread_id,
            first_user_message: self.first_user_message,
            cwd: self.cwd,
            model_provider: self.model_provider,
            created_at: self.created_at,
            rounds: self.rounds,
        }
    }

    fn handle_session_meta(&mut self, payload: Option<&Value>) {
        let Some(payload) = payload.and_then(Value::as_object) else {
            return;
        };
        self.thread_id = string_field(payload, "id");
        self.cwd = path_field(payload, "cwd");
        self.model_provider = string_field(payload, "model_provider");
        self.created_at = string_field(payload, "timestamp").and_then(|timestamp| {
            DateTime::parse_from_rfc3339(&timestamp)
                .ok()
                .map(|parsed| parsed.timestamp())
        });
    }

    fn handle_turn_context(&mut self, payload: Option<&Value>) {
        let turn_id = payload
            .and_then(Value::as_object)
            .and_then(|payload| string_field(payload, "turn_id"))
            .unwrap_or_else(|| format!("local-turn-{}", self.rounds.len() + 1));
        self.finish_current_round();
        self.current_round = Some(ArchiveRound {
            turn_id,
            status: ArchiveTurnStatus::InProgress,
            error: None,
            items: Vec::new(),
        });
    }

    fn handle_event(&mut self, payload: Option<&Value>) {
        let Some(payload) = payload.and_then(Value::as_object) else {
            return;
        };
        match string_field(payload, "type").as_deref() {
            Some("user_message") => {
                let text = string_field(payload, "message").unwrap_or_default();
                if self.first_user_message.is_none() && !text.trim().is_empty() {
                    self.first_user_message = Some(text.clone());
                }
                let item_id = self.next_item_id();
                self.push_item(ArchiveTurnItem::UserMessage {
                    id: item_id,
                    text,
                    images: Vec::new(),
                });
            }
            Some("agent_message") => {
                let item_id = self.next_item_id();
                self.push_item(ArchiveTurnItem::AssistantMessage {
                    id: item_id,
                    text: string_field(payload, "message").unwrap_or_default(),
                    phase: string_field(payload, "phase"),
                });
            }
            Some("turn_started") => {
                if self.current_round.is_none() {
                    let turn_id = string_field(payload, "turn_id")
                        .unwrap_or_else(|| format!("local-turn-{}", self.rounds.len() + 1));
                    self.current_round = Some(ArchiveRound {
                        turn_id,
                        status: ArchiveTurnStatus::InProgress,
                        error: None,
                        items: Vec::new(),
                    });
                }
            }
            Some("turn_complete") => {
                if let Some(round) = self.current_round.as_mut() {
                    round.status = ArchiveTurnStatus::Completed;
                }
            }
            Some("turn_aborted") => {
                if let Some(round) = self.current_round.as_mut() {
                    round.status = ArchiveTurnStatus::Interrupted;
                }
            }
            Some("error") => {
                if let Some(round) = self.current_round.as_mut() {
                    round.status = ArchiveTurnStatus::Failed;
                    round.error = Some(ArchiveTurnError {
                        message: string_field(payload, "message")
                            .unwrap_or_else(|| "unknown local replay error".to_string()),
                    });
                }
            }
            Some("exec_command_end") => {
                let record = CommandExecutionRecord {
                    id: string_field(payload, "call_id").unwrap_or_else(|| self.next_item_id()),
                    command: command_field(payload.get("command")),
                    cwd: path_field(payload, "cwd"),
                    status: string_field(payload, "status"),
                    aggregated_output: string_field(payload, "aggregated_output").or_else(|| {
                        combine_output(
                            string_field(payload, "stdout"),
                            string_field(payload, "stderr"),
                        )
                    }),
                    exit_code: i32_field(payload, "exit_code"),
                };
                self.push_item(ArchiveTurnItem::ToolCall(
                    ArchiveToolCall::CommandExecution(record),
                ));
            }
            Some("patch_apply_begin") | Some("apply_patch_approval_request") => {
                let status = string_field(payload, "type").map(|kind| match kind.as_str() {
                    "patch_apply_begin" => "inProgress".to_string(),
                    _ => "requested".to_string(),
                });
                let item_id =
                    string_field(payload, "call_id").unwrap_or_else(|| self.next_item_id());
                self.push_item(ArchiveTurnItem::ToolCall(ArchiveToolCall::FileChange {
                    id: item_id,
                    status,
                    changes: parse_file_changes(payload.get("changes")),
                }));
            }
            _ => {}
        }
    }

    fn handle_response_item(&mut self, payload: Option<&Value>) {
        let Some(payload) = payload.and_then(Value::as_object) else {
            return;
        };
        match string_field(payload, "type").as_deref() {
            Some("function_call") => {
                let name = string_field(payload, "name").unwrap_or_else(|| "tool".to_string());
                if matches!(name.as_str(), "exec_command" | "apply_patch") {
                    return;
                }
                let id = string_field(payload, "call_id").unwrap_or_else(|| self.next_item_id());
                self.push_item(ArchiveTurnItem::ToolCall(ArchiveToolCall::Unsupported {
                    id,
                    kind: format!("function_call:{name}"),
                    payload: Value::Object(payload.clone()),
                }));
            }
            Some("function_call_output") => {}
            _ => {}
        }
    }

    fn push_item(&mut self, item: ArchiveTurnItem) {
        if self.current_round.is_none() {
            self.current_round = Some(ArchiveRound {
                turn_id: format!("local-turn-{}", self.rounds.len() + 1),
                status: ArchiveTurnStatus::InProgress,
                error: None,
                items: Vec::new(),
            });
        }
        if let Some(round) = self.current_round.as_mut() {
            round.items.push(item);
        }
    }

    fn finish_current_round(&mut self) {
        if let Some(mut round) = self.current_round.take() {
            if matches!(round.status, ArchiveTurnStatus::InProgress) {
                round.status = ArchiveTurnStatus::Interrupted;
            }
            self.rounds.push(round);
        }
    }

    fn next_item_id(&mut self) -> String {
        let id = format!("local-item-{}", self.next_item_index);
        self.next_item_index += 1;
        id
    }
}

fn parse_file_changes(value: Option<&Value>) -> Vec<FileChangeRecord> {
    match value {
        Some(Value::Object(record)) => record
            .iter()
            .map(|(path, change)| FileChangeRecord {
                path: path.clone(),
                kind: change
                    .as_object()
                    .and_then(|change| string_field(change, "type")),
                diff: Some(change.to_string()),
            })
            .collect(),
        Some(Value::Array(entries)) => entries
            .iter()
            .filter_map(Value::as_object)
            .map(|change| FileChangeRecord {
                path: string_field(change, "path").unwrap_or_default(),
                kind: string_field(change, "kind"),
                diff: change
                    .get("diff")
                    .map(|value| value.to_string())
                    .or_else(|| Some(Value::Object(change.clone()).to_string())),
            })
            .collect(),
        _ => Vec::new(),
    }
}

fn command_field(value: Option<&Value>) -> String {
    match value {
        Some(Value::Array(parts)) => parts
            .iter()
            .filter_map(Value::as_str)
            .collect::<Vec<_>>()
            .join(" "),
        Some(Value::String(command)) => command.clone(),
        Some(other) => other.to_string(),
        None => String::new(),
    }
}

fn combine_output(stdout: Option<String>, stderr: Option<String>) -> Option<String> {
    let mut pieces = Vec::new();
    if let Some(stdout) = stdout.filter(|value| !value.trim().is_empty()) {
        pieces.push(stdout);
    }
    if let Some(stderr) = stderr.filter(|value| !value.trim().is_empty()) {
        pieces.push(stderr);
    }
    if pieces.is_empty() {
        None
    } else {
        Some(pieces.join("\n"))
    }
}

fn string_field(record: &Map<String, Value>, field: &str) -> Option<String> {
    record
        .get(field)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn path_field(record: &Map<String, Value>, field: &str) -> Option<PathBuf> {
    string_field(record, field).map(PathBuf::from)
}

fn i32_field(record: &Map<String, Value>, field: &str) -> Option<i32> {
    record
        .get(field)
        .and_then(Value::as_i64)
        .and_then(|value| i32::try_from(value).ok())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};

    use rusqlite::{Connection, params};
    use tempfile::tempdir;

    use super::{lookup_thread_metadata, resolve_codex_home, resolve_rollout_path};

    #[test]
    fn resolve_codex_home_prefers_explicit_then_env_then_default() {
        let explicit_dir = tempdir().expect("explicit");
        let explicit = explicit_dir.path().to_path_buf();
        assert_eq!(
            resolve_codex_home(Some(&explicit)).expect("explicit codex home"),
            explicit
        );
    }

    #[test]
    fn lookup_thread_metadata_reads_rollout_path_from_state_db() {
        let codex_home = tempdir().expect("codex home");
        let db_path = codex_home.path().join("state_5.sqlite");
        let connection = Connection::open(&db_path).expect("sqlite db");
        connection
            .execute_batch(
                "CREATE TABLE threads (
                    id TEXT PRIMARY KEY,
                    rollout_path TEXT,
                    cwd TEXT,
                    model_provider TEXT,
                    created_at INTEGER,
                    updated_at INTEGER,
                    source TEXT,
                    cli_version TEXT,
                    first_user_message TEXT
                );",
            )
            .expect("schema");
        connection
            .execute(
                "INSERT INTO threads (id, rollout_path, cwd, model_provider, created_at, updated_at, source, cli_version, first_user_message)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    "thread-1",
                    "sessions/rollout-thread-1.jsonl",
                    "/tmp/workspace",
                    "openai",
                    1000_i64,
                    1001_i64,
                    "cli",
                    "0.1.0",
                    "preview"
                ],
            )
            .expect("insert");

        let metadata = lookup_thread_metadata(codex_home.path(), "thread-1").expect("metadata");
        assert_eq!(metadata.thread_id, "thread-1");
        assert_eq!(
            metadata.rollout_path,
            PathBuf::from("sessions/rollout-thread-1.jsonl")
        );
        assert_eq!(metadata.first_user_message.as_deref(), Some("preview"));
    }

    #[test]
    fn resolve_rollout_path_joins_relative_paths_under_codex_home() {
        let codex_home = tempdir().expect("codex home");
        let rollout = codex_home.path().join("sessions").join("thread.jsonl");
        fs::create_dir_all(rollout.parent().expect("parent")).expect("mkdirs");
        fs::write(&rollout, "{}\n").expect("rollout");

        let resolved =
            resolve_rollout_path(Path::new("sessions/thread.jsonl"), Some(codex_home.path()))
                .expect("resolved rollout");
        assert_eq!(resolved, rollout);
    }
}
