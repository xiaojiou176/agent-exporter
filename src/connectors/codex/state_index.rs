use std::env;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use rusqlite::{Connection, OpenFlags, OptionalExtension, params};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CodexThreadMetadata {
    pub thread_id: String,
    pub rollout_path: PathBuf,
    pub cwd: Option<PathBuf>,
    pub model_provider: Option<String>,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
    pub title: Option<String>,
    pub first_user_message: Option<String>,
}

pub fn resolve_codex_home(explicit: Option<&Path>) -> Result<PathBuf> {
    if let Some(path) = explicit {
        return Ok(path.to_path_buf());
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

fn non_empty_string(value: Option<String>) -> Option<String> {
    value.and_then(|text| {
        let trimmed = text.trim();
        (!trimmed.is_empty()).then(|| trimmed.to_string())
    })
}

pub fn lookup_thread_metadata(codex_home: &Path, thread_id: &str) -> Result<CodexThreadMetadata> {
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
        "SELECT id, rollout_path, cwd, model_provider, created_at, updated_at, source, cli_version, title, first_user_message
         FROM threads
         WHERE id = ?1
         LIMIT 1",
    )?;

    let row = statement
        .query_row(params![thread_id], |row| {
            let rollout_path = row.get::<_, Option<String>>(1)?;
            Ok(CodexThreadMetadata {
                thread_id: row.get(0)?,
                rollout_path: PathBuf::from(rollout_path.unwrap_or_default()),
                cwd: row.get::<_, Option<String>>(2)?.map(PathBuf::from),
                model_provider: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
                title: non_empty_string(row.get(8)?),
                first_user_message: non_empty_string(row.get(9)?),
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

pub fn list_primary_thread_metadata(codex_home: &Path) -> Result<Vec<CodexThreadMetadata>> {
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
        "SELECT id, rollout_path, cwd, model_provider, created_at, updated_at, source, cli_version, title, first_user_message
         FROM threads
         WHERE archived = 0
           AND source = 'vscode'
           AND cwd IS NOT NULL",
    )?;

    let rows = statement.query_map(params![], |row| {
        let rollout_path = row.get::<_, Option<String>>(1)?;
        Ok(CodexThreadMetadata {
            thread_id: row.get(0)?,
            rollout_path: PathBuf::from(rollout_path.unwrap_or_default()),
            cwd: row.get::<_, Option<String>>(2)?.map(PathBuf::from),
            model_provider: row.get(3)?,
            created_at: row.get(4)?,
            updated_at: row.get(5)?,
            title: non_empty_string(row.get(8)?),
            first_user_message: non_empty_string(row.get(9)?),
        })
    })?;

    let mut entries = rows.collect::<std::result::Result<Vec<_>, _>>()?;
    entries.sort_by(|left, right| {
        right
            .updated_at
            .cmp(&left.updated_at)
            .then_with(|| right.created_at.cmp(&left.created_at))
            .then_with(|| left.thread_id.cmp(&right.thread_id))
    });
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use rusqlite::{Connection, params};
    use tempfile::tempdir;

    use super::{list_primary_thread_metadata, lookup_thread_metadata, resolve_codex_home};

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
                    title TEXT,
                    first_user_message TEXT
                );",
            )
            .expect("schema");
        connection
            .execute(
                "INSERT INTO threads (id, rollout_path, cwd, model_provider, created_at, updated_at, source, cli_version, title, first_user_message)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    "thread-1",
                    "sessions/rollout-thread-1.jsonl",
                    "/tmp/workspace",
                    "openai",
                    1000_i64,
                    1001_i64,
                    "cli",
                    "0.1.0",
                    "Renamed thread",
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
        assert_eq!(metadata.title.as_deref(), Some("Renamed thread"));
        assert_eq!(metadata.first_user_message.as_deref(), Some("preview"));
    }

    #[test]
    fn list_primary_thread_metadata_keeps_only_active_vscode_threads() {
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
                    archived INTEGER NOT NULL DEFAULT 0,
                    cli_version TEXT,
                    title TEXT,
                    first_user_message TEXT
                );",
            )
            .expect("schema");

        connection
            .execute(
                "INSERT INTO threads (id, rollout_path, cwd, model_provider, created_at, updated_at, source, archived, cli_version, title, first_user_message)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                params![
                    "thread-a",
                    "sessions/a.jsonl",
                    "/tmp/ws-a",
                    "openai",
                    10_i64,
                    20_i64,
                    "vscode",
                    0_i64,
                    "0.1.0",
                    "Workspace A",
                    "hello a"
                ],
            )
            .expect("insert a");

        connection
            .execute(
                "INSERT INTO threads (id, rollout_path, cwd, model_provider, created_at, updated_at, source, archived, cli_version, title, first_user_message)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                params![
                    "thread-b",
                    "sessions/b.jsonl",
                    "/tmp/ws-a/nested/project",
                    "openai",
                    30_i64,
                    40_i64,
                    "vscode",
                    0_i64,
                    "0.1.0",
                    "Workspace B",
                    "hello b"
                ],
            )
            .expect("insert b");

        connection
            .execute(
                "INSERT INTO threads (id, rollout_path, cwd, model_provider, created_at, updated_at, source, archived, cli_version, title, first_user_message)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                params![
                    "thread-c",
                    "sessions/c.jsonl",
                    "/tmp/ws-c",
                    "openai",
                    50_i64,
                    60_i64,
                    "exec",
                    0_i64,
                    "0.1.0",
                    "Ignore exec",
                    "ignore exec"
                ],
            )
            .expect("insert c");

        connection
            .execute(
                "INSERT INTO threads (id, rollout_path, cwd, model_provider, created_at, updated_at, source, archived, cli_version, title, first_user_message)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                params![
                    "thread-d",
                    "sessions/d.jsonl",
                    "/tmp/ws-d",
                    "openai",
                    70_i64,
                    80_i64,
                    "vscode",
                    1_i64,
                    "0.1.0",
                    "Ignore archived",
                    "ignore archived"
                ],
            )
            .expect("insert d");

        let entries = list_primary_thread_metadata(codex_home.path()).expect("list");

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].thread_id, "thread-b");
        assert_eq!(entries[1].thread_id, "thread-a");
    }
}
