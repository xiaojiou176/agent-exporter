use std::fs;
use std::path::{Path, PathBuf};

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

fn python_command() -> String {
    std::env::var("PYTHON").unwrap_or_else(|_| "python3".to_string())
}

fn codex_fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("mock_codex_app_server.py")
}

fn claude_fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

fn conversations_dir(workspace_root: &Path) -> PathBuf {
    workspace_root.join(".agents").join("Conversations")
}

fn build_codex_export_command(thread_id: &str, workspace_root: &Path) -> Command {
    let mut command = Command::cargo_bin("agent-exporter").expect("binary should build");
    command
        .arg("export")
        .arg("codex")
        .arg("--thread-id")
        .arg(thread_id)
        .arg("--format")
        .arg("html")
        .arg("--destination")
        .arg("workspace-conversations")
        .arg("--workspace-root")
        .arg(workspace_root)
        .arg("--app-server-command")
        .arg(python_command())
        .arg("--app-server-arg")
        .arg(codex_fixture_path());
    command
}

fn build_claude_export_command(session_path: &Path, workspace_root: &Path) -> Command {
    let mut command = Command::cargo_bin("agent-exporter").expect("binary should build");
    command
        .arg("export")
        .arg("claude-code")
        .arg("--session-path")
        .arg(session_path)
        .arg("--format")
        .arg("html")
        .arg("--destination")
        .arg("workspace-conversations")
        .arg("--workspace-root")
        .arg(workspace_root);
    command
}

fn build_json_export_command(thread_id: &str, workspace_root: &Path) -> Command {
    let mut command = Command::cargo_bin("agent-exporter").expect("binary should build");
    command
        .arg("export")
        .arg("codex")
        .arg("--thread-id")
        .arg(thread_id)
        .arg("--format")
        .arg("json")
        .arg("--destination")
        .arg("workspace-conversations")
        .arg("--workspace-root")
        .arg(workspace_root)
        .arg("--app-server-command")
        .arg(python_command())
        .arg("--app-server-arg")
        .arg(codex_fixture_path());
    command
}

fn build_publish_command(workspace_root: &Path) -> Command {
    let mut command = Command::cargo_bin("agent-exporter").expect("binary should build");
    command
        .arg("publish")
        .arg("archive-index")
        .arg("--workspace-root")
        .arg(workspace_root);
    command
}

#[test]
fn publish_archive_index_generates_static_index_for_html_transcripts() {
    let workspace = tempdir().expect("workspace");
    build_codex_export_command("complete-thread", workspace.path())
        .assert()
        .success();
    build_claude_export_command(
        &claude_fixture_path("claude_session_minimal.jsonl"),
        workspace.path(),
    )
    .assert()
    .success();
    build_json_export_command("complete-thread", workspace.path())
        .assert()
        .success();

    build_publish_command(workspace.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Archive index published"))
        .stdout(predicate::str::contains("Entries      : 2"))
        .stdout(predicate::str::contains("index.html"));

    let index_path = conversations_dir(workspace.path()).join("index.html");
    assert!(index_path.exists());

    let content = fs::read_to_string(index_path).expect("index html");
    assert!(content.contains("<!DOCTYPE html>"));
    assert!(content.contains("Open transcript"));
    assert!(content.contains("Mock Complete Thread"));
    assert!(content.contains("claude-session-minimal"));
    assert!(content.contains("complete"));
    assert!(content.contains("degraded"));
    assert!(content.contains("archive-search"));
    assert!(content.contains("data-search-text"));
    assert!(content.contains(".html"));
    assert!(!content.contains(".json"));
}

#[test]
fn publish_archive_index_writes_empty_state_when_no_html_exports_exist() {
    let workspace = tempdir().expect("workspace");

    build_publish_command(workspace.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Entries      : 0"));

    let index_path = conversations_dir(workspace.path()).join("index.html");
    assert!(index_path.exists());

    let content = fs::read_to_string(index_path).expect("index html");
    assert!(content.contains("还没有 HTML transcript exports"));
}
