use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use assert_cmd::Command;
use predicates::prelude::*;
use rusqlite::{Connection, params};
use tempfile::tempdir;

fn python_command() -> String {
    std::env::var("PYTHON").unwrap_or_else(|_| "python3".to_string())
}

fn app_server_fixture_path() -> PathBuf {
    fixture_path("mock_codex_app_server.py")
}

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
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
        .filter(|path| {
            path.file_name()
                .and_then(|value| value.to_str())
                .is_none_or(|value| {
                    value != "index.html"
                        && value != "index.json"
                        && value != "fleet-view.html"
                        && value != "fleet-view.json"
                        && value != "memory-lane.html"
                        && value != "memory-lane.json"
                })
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

fn build_claude_command(session_path: &Path, workspace_root: &Path) -> Command {
    let mut command = Command::cargo_bin("agent-exporter").expect("binary should build");
    command
        .arg("export")
        .arg("claude-code")
        .arg("--session-path")
        .arg(session_path)
        .arg("--destination")
        .arg("workspace-conversations")
        .arg("--workspace-root")
        .arg(workspace_root);
    command
}

fn install_fake_codex(fake_bin_dir: &Path, log_path: &Path) -> PathBuf {
    let script_path = fake_bin_dir.join("codex");
    fs::create_dir_all(fake_bin_dir).expect("fake codex dir");
    fs::write(
        &script_path,
        r##"#!/usr/bin/env python3
import json
import os
import pathlib
import sys
import time

args = sys.argv[1:]
prompt = sys.stdin.read()
delay_seconds = float(os.environ.get("AGENT_EXPORTER_FAKE_CODEX_DELAY_SECONDS", "0"))
stdout_bytes = int(os.environ.get("AGENT_EXPORTER_FAKE_CODEX_STDOUT_BYTES", "0"))
output_path = None
for index, value in enumerate(args):
    if value == "-o" and index + 1 < len(args):
        output_path = pathlib.Path(args[index + 1])
        break

if output_path is None:
    print("missing -o output path", file=sys.stderr)
    sys.exit(2)

pathlib.Path(os.environ["AGENT_EXPORTER_FAKE_CODEX_LOG"]).write_text(
    json.dumps({"args": args, "prompt": prompt}),
    encoding="utf-8",
)
output_path.write_text("# AI 梳理\n\nfake summary\n", encoding="utf-8")
if delay_seconds:
    time.sleep(delay_seconds)
if stdout_bytes:
    sys.stdout.write("x" * stdout_bytes)
    sys.stdout.flush()
"##,
    )
    .expect("fake codex script");
    #[cfg(unix)]
    {
        let mut permissions = fs::metadata(&script_path)
            .expect("fake codex metadata")
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&script_path, permissions).expect("fake codex chmod");
    }
    assert!(
        log_path.is_absolute(),
        "fake codex log path should be absolute for subprocess access"
    );
    script_path
}

fn prepend_path(fake_bin_dir: &Path) -> String {
    let current_path = std::env::var_os("PATH").unwrap_or_default();
    let mut paths = vec![fake_bin_dir.to_path_buf()];
    paths.extend(std::env::split_paths(&current_path));
    std::env::join_paths(paths)
        .expect("join PATH")
        .to_string_lossy()
        .to_string()
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
                thread_id,
                rollout_rel,
                "/tmp/mock-workspace",
                "openai",
                1_712_198_400_i64,
                1_712_198_460_i64,
                "cli",
                "0.1.0",
                "Renamed local thread",
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
    assert!(content.contains("Open archive shell"));
    assert!(content.contains("Open retrieval reports"));
    assert!(content.contains("agent-exporter:workspace-shell-href"));
    assert!(content.contains("agent-exporter:workspace-reports-shell-href"));
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
                title TEXT,
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
                "missing-rollout",
                "sessions/does-not-exist.jsonl",
                "/tmp/workspace",
                "openai",
                1_712_198_400_i64,
                1_712_198_460_i64,
                "cli",
                "0.1.0",
                "Missing rollout",
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

#[test]
fn codex_export_ai_summary_accepts_profile_model_provider_controls() {
    let workspace = tempdir().expect("workspace");
    let codex_home = tempdir().expect("codex home");
    let fake_bin = tempdir().expect("fake bin");
    let log_path = workspace.path().join("fake-codex-log.json");
    create_local_fixture(
        codex_home.path(),
        "ai-summary-controls-thread",
        "sessions/ai-summary-controls-thread.jsonl",
    );
    install_fake_codex(fake_bin.path(), &log_path);

    build_local_command(workspace.path())
        .env("PATH", prepend_path(fake_bin.path()))
        .env("AGENT_EXPORTER_FAKE_CODEX_LOG", &log_path)
        .arg("--codex-home")
        .arg(codex_home.path())
        .arg("--thread-id")
        .arg("ai-summary-controls-thread")
        .arg("--ai-summary")
        .arg("--ai-summary-profile")
        .arg("summary-fast")
        .arg("--ai-summary-preset")
        .arg("handoff")
        .arg("--ai-summary-model")
        .arg("o3")
        .arg("--ai-summary-provider")
        .arg("cliproxyapi")
        .assert()
        .success()
        .stdout(predicate::str::contains("- AI Summary"));

    let payload: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&log_path).expect("fake codex log"))
            .expect("fake codex json");
    let args = payload["args"]
        .as_array()
        .expect("args array")
        .iter()
        .filter_map(|value| value.as_str())
        .collect::<Vec<_>>();
    assert!(
        args.windows(2)
            .any(|pair| pair == ["--profile", "summary-fast"])
    );
    assert!(args.windows(2).any(|pair| pair == ["--model", "o3"]));
    assert!(
        args.windows(2)
            .any(|pair| pair == ["-c", "model_provider=\"cliproxyapi\""])
    );

    let json_paths = exported_paths_with_extension(workspace.path(), "json");
    let summary_json = json_paths
        .iter()
        .find(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.contains("-ai-summary-rounds-"))
        })
        .expect("structured summary json");
    let summary_document: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(summary_json).expect("summary json"))
            .expect("summary json value");
    assert_eq!(summary_document["profileId"], "handoff");
    assert_eq!(summary_document["runtimeProfile"], "summary-fast");
    assert_eq!(summary_document["threadId"], "ai-summary-controls-thread");
    assert_eq!(
        summary_document["outputFiles"]["json"],
        summary_json
            .file_name()
            .and_then(|name| name.to_str())
            .expect("summary json file name")
    );
}

#[test]
fn codex_export_ai_summary_handles_noisy_child_output_without_timing_out() {
    let workspace = tempdir().expect("workspace");
    let codex_home = tempdir().expect("codex home");
    let fake_bin = tempdir().expect("fake bin");
    let log_path = workspace.path().join("fake-codex-log.json");
    create_local_fixture(
        codex_home.path(),
        "ai-summary-noisy-thread",
        "sessions/ai-summary-noisy-thread.jsonl",
    );
    install_fake_codex(fake_bin.path(), &log_path);

    build_local_command(workspace.path())
        .env("PATH", prepend_path(fake_bin.path()))
        .env("AGENT_EXPORTER_FAKE_CODEX_LOG", &log_path)
        .env("AGENT_EXPORTER_FAKE_CODEX_STDOUT_BYTES", "1048576")
        .arg("--codex-home")
        .arg(codex_home.path())
        .arg("--thread-id")
        .arg("ai-summary-noisy-thread")
        .arg("--ai-summary")
        .arg("--ai-summary-timeout-seconds")
        .arg("2")
        .assert()
        .success()
        .stdout(predicate::str::contains("- AI Summary"));
}

#[test]
fn claude_export_ai_summary_accepts_profile_model_provider_controls() {
    let workspace = tempdir().expect("workspace");
    let fake_bin = tempdir().expect("fake bin");
    let log_path = workspace.path().join("fake-codex-log.json");
    install_fake_codex(fake_bin.path(), &log_path);

    build_claude_command(
        &fixture_path("claude_session_minimal.jsonl"),
        workspace.path(),
    )
    .env("PATH", prepend_path(fake_bin.path()))
    .env("AGENT_EXPORTER_FAKE_CODEX_LOG", &log_path)
    .arg("--ai-summary")
    .arg("--ai-summary-profile")
    .arg("claude-summary")
    .arg("--ai-summary-model")
    .arg("gpt-5")
    .arg("--ai-summary-provider")
    .arg("openai")
    .assert()
    .success()
    .stdout(predicate::str::contains("Connector    : claude-code"))
    .stdout(predicate::str::contains("- AI Summary"));

    let payload: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&log_path).expect("fake codex log"))
            .expect("fake codex json");
    let args = payload["args"]
        .as_array()
        .expect("args array")
        .iter()
        .filter_map(|value| value.as_str())
        .collect::<Vec<_>>();
    assert!(
        args.windows(2)
            .any(|pair| pair == ["--profile", "claude-summary"])
    );
    assert!(args.windows(2).any(|pair| pair == ["--model", "gpt-5"]));
    assert!(
        args.windows(2)
            .any(|pair| pair == ["-c", "model_provider=\"openai\""])
    );
}
