use std::fs;
use std::path::{Path, PathBuf};

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("mock_codex_app_server.py")
}

fn python_command() -> String {
    std::env::var("PYTHON").unwrap_or_else(|_| "python3".to_string())
}

fn conversations_dir(workspace_root: &Path) -> PathBuf {
    workspace_root.join(".agents").join("Conversations")
}

fn build_export_command(thread_id: &str, workspace_root: &Path) -> Command {
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
        .arg(fixture_path());
    command
}

fn exported_markdown_paths(workspace_root: &Path) -> Vec<PathBuf> {
    let mut paths = fs::read_dir(conversations_dir(workspace_root))
        .expect("conversations dir should exist")
        .map(|entry| entry.expect("dir entry").path())
        .collect::<Vec<_>>();
    paths.sort();
    paths
}

#[test]
fn export_codex_writes_workspace_conversations_markdown() {
    let workspace = tempdir().expect("temp workspace");
    build_export_command("complete-thread", workspace.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Completeness : complete"))
        .stdout(predicate::str::contains(
            "Source       : app-server-thread-read",
        ));

    let paths = exported_markdown_paths(workspace.path());
    assert_eq!(paths.len(), 1);
    let content = fs::read_to_string(&paths[0]).expect("markdown content");
    assert!(content.contains("# 第1轮"));
    assert!(content.contains("## 用户"));
    assert!(content.contains("## 助手"));
    assert!(content.contains("### 工具"));
    assert!(content.contains("完整性: `complete`"));
    assert!(content.contains("pwd"));
}

#[test]
fn export_codex_marks_incomplete_when_resume_fallback_is_used() {
    let workspace = tempdir().expect("temp workspace");
    build_export_command("fallback-thread", workspace.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Completeness : incomplete"))
        .stdout(predicate::str::contains(
            "Source       : app-server-resume-fallback",
        ));

    let paths = exported_markdown_paths(workspace.path());
    assert_eq!(paths.len(), 1);
    let content = fs::read_to_string(&paths[0]).expect("markdown content");
    assert!(content.contains("完整性: `incomplete`"));
    assert!(content.contains("Preview recovered through resume fallback"));
    assert!(content.contains("Recovered from resume fallback"));
}

#[test]
fn workspace_conversations_requires_workspace_root() {
    let mut command = Command::cargo_bin("agent-exporter").expect("binary should build");
    command
        .arg("export")
        .arg("codex")
        .arg("--thread-id")
        .arg("complete-thread")
        .arg("--destination")
        .arg("workspace-conversations");

    command.assert().failure().stderr(predicate::str::contains(
        "destination `workspace-conversations` requires --workspace-root <path>",
    ));
}
