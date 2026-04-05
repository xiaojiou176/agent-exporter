use std::fs;
use std::path::{Path, PathBuf};

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

fn conversations_dir(workspace_root: &Path) -> PathBuf {
    workspace_root.join(".agents").join("Conversations")
}

fn exported_markdown_paths(workspace_root: &Path) -> Vec<PathBuf> {
    let mut paths = fs::read_dir(conversations_dir(workspace_root))
        .expect("conversations dir should exist")
        .map(|entry| entry.expect("dir entry").path())
        .collect::<Vec<_>>();
    paths.sort();
    paths
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

#[test]
fn claude_code_requires_session_path() {
    let workspace = tempdir().expect("workspace");
    let mut command = Command::cargo_bin("agent-exporter").expect("binary should build");
    command
        .arg("export")
        .arg("claude-code")
        .arg("--destination")
        .arg("workspace-conversations")
        .arg("--workspace-root")
        .arg(workspace.path());

    command
        .assert()
        .failure()
        .stderr(predicate::str::contains("--session-path <SESSION_PATH>"));
}

#[test]
fn claude_code_export_writes_degraded_markdown_with_shared_skeleton() {
    let workspace = tempdir().expect("workspace");
    build_claude_command(
        &fixture_path("claude_session_minimal.jsonl"),
        workspace.path(),
    )
    .assert()
    .success()
    .stdout(predicate::str::contains("Connector    : claude-code"))
    .stdout(predicate::str::contains("Selection    : session-path"))
    .stdout(predicate::str::contains("Completeness : degraded"))
    .stdout(predicate::str::contains(
        "Source       : claude-session-path",
    ));

    let paths = exported_markdown_paths(workspace.path());
    assert_eq!(paths.len(), 1);

    let content = fs::read_to_string(&paths[0]).expect("markdown content");
    assert!(content.contains("完整性: `degraded`"));
    assert!(content.contains("来源: `claude-session-path`"));
    assert!(content.contains("# 第1轮"));
    assert!(content.contains("## 用户"));
    assert!(content.contains("## 助手"));
    assert!(content.contains("### 工具"));
    assert_eq!(
        content.matches("# 第").count(),
        2,
        "queue/progress should not create extra rounds"
    );
    assert!(content.contains("#### File changes (completed)"));
    assert!(content.contains("- `/tmp/claude-demo/hello.py` [write]"));
    assert!(content.contains("#### Command: python -m pytest tests/ (completed)"));
    assert!(content.contains("```text\n2 passed\n```"));
    assert!(!content.contains("Dynamic tool: Write"));
    assert!(!content.contains("Dynamic tool: Bash"));
    assert!(!content.contains("queue-operation"));
    assert!(!content.contains("progress"));
}

#[test]
fn claude_code_export_supports_wrapped_json_loglines() {
    let workspace = tempdir().expect("workspace");
    build_claude_command(
        &fixture_path("claude_session_wrapped.json"),
        workspace.path(),
    )
    .assert()
    .success()
    .stdout(predicate::str::contains("Connector    : claude-code"))
    .stdout(predicate::str::contains("Completeness : degraded"));

    let paths = exported_markdown_paths(workspace.path());
    assert_eq!(paths.len(), 1);

    let content = fs::read_to_string(&paths[0]).expect("markdown content");
    assert!(content.contains("Summarize the repository state"));
    assert!(content.contains("Glob"));
}

#[test]
fn claude_code_summary_is_used_as_preview_fallback_without_leaking_progress_noise() {
    let workspace = tempdir().expect("workspace");
    build_claude_command(
        &fixture_path("claude_session_summary_only.jsonl"),
        workspace.path(),
    )
    .assert()
    .success()
    .stdout(predicate::str::contains("Connector    : claude-code"))
    .stdout(predicate::str::contains("Completeness : degraded"));

    let paths = exported_markdown_paths(workspace.path());
    assert_eq!(paths.len(), 1);

    let content = fs::read_to_string(&paths[0]).expect("markdown content");
    assert!(content.contains("Continuation summary: user was refining calculator behaviors."));
    assert!(content.contains("I picked up the previous context and I'm ready to continue."));
    assert!(!content.contains("queue-operation"));
    assert!(!content.contains("progress"));
}
