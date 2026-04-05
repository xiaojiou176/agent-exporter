use std::fs;
use std::path::{Path, PathBuf};

use assert_cmd::Command;
use predicates::prelude::*;
use rusqlite::{Connection, params};
use tempfile::tempdir;

fn python_command() -> String {
    std::env::var("PYTHON").unwrap_or_else(|_| "python3".to_string())
}

fn app_server_fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("mock_codex_app_server.py")
}

fn conversations_dir(workspace_root: &Path) -> PathBuf {
    workspace_root.join(".agents").join("Conversations")
}

fn exported_paths_with_extension(workspace_root: &Path, extension: &str) -> Vec<PathBuf> {
    let mut paths = fs::read_dir(conversations_dir(workspace_root))
        .expect("conversations dir should exist")
        .map(|entry| entry.expect("dir entry").path())
        .filter(|path| {
            path.extension()
                .and_then(|value| value.to_str())
                .is_some_and(|value| value == extension)
        })
        .collect::<Vec<_>>();
    paths.sort();
    paths
}

fn build_local_command(workspace_root: &Path) -> Command {
    let mut command = Command::cargo_bin("agent-exporter").expect("binary should build");
    command
        .arg("export")
        .arg("codex")
        .arg("--source")
        .arg("local")
        .arg("--destination")
        .arg("workspace-conversations")
        .arg("--workspace-root")
        .arg(workspace_root);
    command
}

fn build_app_server_command(thread_id: &str, workspace_root: &Path) -> Command {
    let mut command = Command::cargo_bin("agent-exporter").expect("binary should build");
    command
        .arg("export")
        .arg("codex")
        .arg("--thread-id")
        .arg(thread_id)
        .arg("--destination")
        .arg("workspace-conversations")
        .arg("--workspace-root")
        .arg(workspace_root)
        .arg("--app-server-command")
        .arg(python_command())
        .arg("--app-server-arg")
        .arg(app_server_fixture_path());
    command
}

fn create_local_fixture(codex_home: &Path, thread_id: &str, rollout_rel: &str) -> PathBuf {
    let rollout_path = codex_home.join(rollout_rel);
    fs::create_dir_all(rollout_path.parent().expect("rollout parent")).expect("mkdirs");
    fs::write(
        &rollout_path,
        format!(
            concat!(
                "{{\"timestamp\":\"2026-04-05T02:39:26.735Z\",\"type\":\"session_meta\",\"payload\":{{\"id\":\"{thread_id}\",\"timestamp\":\"2026-04-05T02:39:25.341Z\",\"cwd\":\"/tmp/mock-workspace\",\"cli_version\":\"0.1.0\",\"source\":\"cli\",\"model_provider\":\"openai\"}}}}\n",
                "{{\"timestamp\":\"2026-04-05T02:39:26.736Z\",\"type\":\"turn_context\",\"payload\":{{\"turn_id\":\"turn-1\",\"cwd\":\"/tmp/mock-workspace\"}}}}\n",
                "{{\"timestamp\":\"2026-04-05T02:39:26.737Z\",\"type\":\"event_msg\",\"payload\":{{\"type\":\"user_message\",\"message\":\"hello from user\"}}}}\n",
                "{{\"timestamp\":\"2026-04-05T02:39:57.601Z\",\"type\":\"response_item\",\"payload\":{{\"type\":\"function_call\",\"name\":\"spawn_agent\",\"arguments\":\"{{}}\",\"call_id\":\"call-1\"}}}}\n",
                "{{\"timestamp\":\"2026-04-05T02:39:58.601Z\",\"type\":\"response_item\",\"payload\":{{\"type\":\"function_call_output\",\"call_id\":\"call-1\",\"output\":\"done\"}}}}\n",
                "{{\"timestamp\":\"2026-04-05T02:39:57.539Z\",\"type\":\"event_msg\",\"payload\":{{\"type\":\"agent_message\",\"message\":\"hello from assistant\",\"phase\":\"commentary\"}}}}\n",
                "{{\"timestamp\":\"2026-04-05T02:40:10.000Z\",\"type\":\"event_msg\",\"payload\":{{\"type\":\"exec_command_end\",\"call_id\":\"command-1\",\"command\":[\"pwd\"],\"cwd\":\"/tmp/mock-workspace\",\"status\":\"completed\",\"aggregated_output\":\"/tmp/mock-workspace\\n\",\"exit_code\":0}}}}\n",
                "{{\"timestamp\":\"2026-04-05T02:40:11.000Z\",\"type\":\"event_msg\",\"payload\":{{\"type\":\"turn_complete\",\"turn_id\":\"turn-1\"}}}}\n"
            ),
            thread_id = thread_id,
        ),
    )
    .expect("rollout fixture");

    let state_db = codex_home.join("state_5.sqlite");
    let connection = Connection::open(&state_db).expect("sqlite db");
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
                thread_id,
                rollout_rel,
                "/tmp/mock-workspace",
                "openai",
                1_712_198_400_i64,
                1_712_198_460_i64,
                "cli",
                "0.1.0",
                "hello from user"
            ],
        )
        .expect("insert");

    rollout_path
}

#[test]
fn local_source_with_thread_id_exports_degraded_markdown() {
    let workspace = tempdir().expect("workspace");
    let codex_home = tempdir().expect("codex home");
    create_local_fixture(
        codex_home.path(),
        "local-thread",
        "sessions/local-thread.jsonl",
    );

    build_local_command(workspace.path())
        .arg("--codex-home")
        .arg(codex_home.path())
        .arg("--thread-id")
        .arg("local-thread")
        .assert()
        .success()
        .stdout(predicate::str::contains("Selection    : local"))
        .stdout(predicate::str::contains("Completeness : degraded"))
        .stdout(predicate::str::contains("Source       : local-thread-id"));

    let paths = exported_paths_with_extension(workspace.path(), "md");
    assert_eq!(paths.len(), 1);
    let content = fs::read_to_string(&paths[0]).expect("markdown content");
    assert!(content.contains("完整性: `degraded`"));
    assert!(content.contains("来源: `local-thread-id`"));
    assert!(content.contains("# 第1轮"));
    assert!(content.contains("## 用户"));
    assert!(content.contains("## 助手"));
    assert!(content.contains("### 工具"));
}

#[test]
fn local_source_with_rollout_path_exports_degraded_markdown() {
    let workspace = tempdir().expect("workspace");
    let codex_home = tempdir().expect("codex home");
    let rollout_path = create_local_fixture(
        codex_home.path(),
        "rollout-thread",
        "sessions/rollout-thread.jsonl",
    );

    build_local_command(workspace.path())
        .arg("--rollout-path")
        .arg(&rollout_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Completeness : degraded"))
        .stdout(predicate::str::contains(
            "Source       : local-rollout-path",
        ));

    let paths = exported_paths_with_extension(workspace.path(), "md");
    let content = fs::read_to_string(&paths[0]).expect("markdown content");
    assert!(content.contains("来源: `local-rollout-path`"));
    assert!(content.contains("spawn_agent"));
}

#[test]
fn local_source_with_thread_id_exports_degraded_json() {
    let workspace = tempdir().expect("workspace");
    let codex_home = tempdir().expect("codex home");
    create_local_fixture(
        codex_home.path(),
        "local-json-thread",
        "sessions/local-json-thread.jsonl",
    );

    build_local_command(workspace.path())
        .arg("--codex-home")
        .arg(codex_home.path())
        .arg("--thread-id")
        .arg("local-json-thread")
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("Format       : json"))
        .stdout(predicate::str::contains("Completeness : degraded"))
        .stdout(predicate::str::contains("Source       : local-thread-id"));

    let paths = exported_paths_with_extension(workspace.path(), "json");
    assert_eq!(paths.len(), 1);
    let content = fs::read_to_string(&paths[0]).expect("json content");
    let document: serde_json::Value = serde_json::from_str(&content).expect("valid json");
    assert_eq!(document["transcript"]["connector"], "codex");
    assert_eq!(document["transcript"]["completeness"], "degraded");
    assert_eq!(document["transcript"]["source_kind"], "local-thread-id");
    assert_eq!(
        document["transcript"]["rounds"][0]["items"][1]["kind"],
        "tool_call"
    );
}

#[test]
fn local_source_with_thread_id_exports_degraded_html() {
    let workspace = tempdir().expect("workspace");
    let codex_home = tempdir().expect("codex home");
    create_local_fixture(
        codex_home.path(),
        "local-html-thread",
        "sessions/local-html-thread.jsonl",
    );

    build_local_command(workspace.path())
        .arg("--codex-home")
        .arg(codex_home.path())
        .arg("--thread-id")
        .arg("local-html-thread")
        .arg("--format")
        .arg("html")
        .assert()
        .success()
        .stdout(predicate::str::contains("Format       : html"))
        .stdout(predicate::str::contains("Completeness : degraded"))
        .stdout(predicate::str::contains("Source       : local-thread-id"));

    let paths = exported_paths_with_extension(workspace.path(), "html");
    assert_eq!(paths.len(), 1);
    let content = fs::read_to_string(&paths[0]).expect("html content");
    assert!(content.contains("<!DOCTYPE html>"));
    assert!(content.contains("local-thread-id"));
    assert!(content.contains("第1轮"));
    assert!(content.contains("spawn_agent"));
}

#[test]
fn local_and_app_server_exports_keep_same_structure_skeleton() {
    let app_workspace = tempdir().expect("app workspace");
    let local_workspace = tempdir().expect("local workspace");
    let codex_home = tempdir().expect("codex home");
    create_local_fixture(
        codex_home.path(),
        "structure-thread",
        "sessions/structure-thread.jsonl",
    );

    build_app_server_command("complete-thread", app_workspace.path())
        .assert()
        .success();
    build_local_command(local_workspace.path())
        .arg("--codex-home")
        .arg(codex_home.path())
        .arg("--thread-id")
        .arg("structure-thread")
        .assert()
        .success();

    let app_markdown =
        fs::read_to_string(&exported_paths_with_extension(app_workspace.path(), "md")[0])
            .expect("app markdown");
    let local_markdown =
        fs::read_to_string(&exported_paths_with_extension(local_workspace.path(), "md")[0])
            .expect("local markdown");

    for marker in ["# 第1轮", "## 用户", "## 助手", "### 工具"] {
        assert_eq!(
            app_markdown.matches(marker).count(),
            1,
            "app marker {marker}"
        );
        assert_eq!(
            local_markdown.matches(marker).count(),
            1,
            "local marker {marker}"
        );
    }
}

#[test]
fn local_source_requires_selector() {
    let workspace = tempdir().expect("workspace");
    build_local_command(workspace.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "local source requires either --thread-id <THREAD_ID> or --rollout-path <PATH>",
        ));
}

#[test]
fn local_source_rejects_both_thread_id_and_rollout_path() {
    let workspace = tempdir().expect("workspace");
    build_local_command(workspace.path())
        .arg("--thread-id")
        .arg("thread-1")
        .arg("--rollout-path")
        .arg("/tmp/thread.jsonl")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "local source accepts either --thread-id or --rollout-path, not both",
        ));
}

#[test]
fn app_server_source_rejects_rollout_path() {
    let workspace = tempdir().expect("workspace");
    let mut command = Command::cargo_bin("agent-exporter").expect("binary should build");
    command
        .arg("export")
        .arg("codex")
        .arg("--rollout-path")
        .arg("/tmp/thread.jsonl")
        .arg("--destination")
        .arg("workspace-conversations")
        .arg("--workspace-root")
        .arg(workspace.path());

    command
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "`--rollout-path` is only valid with `--source local`; app-server source accepts `--thread-id` only",
        ));
}

#[test]
fn local_source_errors_when_thread_is_missing_from_state_db() {
    let workspace = tempdir().expect("workspace");
    let codex_home = tempdir().expect("codex home");
    Connection::open(codex_home.path().join("state_5.sqlite"))
        .expect("sqlite db")
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

    build_local_command(workspace.path())
        .arg("--codex-home")
        .arg(codex_home.path())
        .arg("--thread-id")
        .arg("missing-thread")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "local source could not find thread `missing-thread`",
        ));
}

#[test]
fn local_source_errors_when_rollout_file_is_missing() {
    let workspace = tempdir().expect("workspace");
    let codex_home = tempdir().expect("codex home");
    let state_db = codex_home.path().join("state_5.sqlite");
    let connection = Connection::open(&state_db).expect("sqlite db");
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
                "missing-rollout",
                "sessions/does-not-exist.jsonl",
                "/tmp/workspace",
                "openai",
                1_712_198_400_i64,
                1_712_198_460_i64,
                "cli",
                "0.1.0",
                "preview"
            ],
        )
        .expect("insert");

    build_local_command(workspace.path())
        .arg("--codex-home")
        .arg(codex_home.path())
        .arg("--thread-id")
        .arg("missing-rollout")
        .assert()
        .failure()
        .stderr(predicate::str::contains("rollout file does not exist:"));
}
