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

fn transcript_html_paths(workspace_root: &Path) -> Vec<PathBuf> {
    exported_paths_with_extension(workspace_root, "html")
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

    let paths = exported_paths_with_extension(workspace.path(), "md");
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

    let paths = exported_paths_with_extension(workspace.path(), "md");
    assert_eq!(paths.len(), 1);
    let content = fs::read_to_string(&paths[0]).expect("markdown content");
    assert!(content.contains("完整性: `incomplete`"));
    assert!(content.contains("Preview recovered through resume fallback"));
    assert!(content.contains("Recovered from resume fallback"));
}

#[test]
fn export_codex_writes_workspace_conversations_json() {
    let workspace = tempdir().expect("temp workspace");
    build_export_command("complete-thread", workspace.path())
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("Format       : json"))
        .stdout(predicate::str::contains("Completeness : complete"))
        .stdout(predicate::str::contains(
            "Source       : app-server-thread-read",
        ));

    let paths = exported_paths_with_extension(workspace.path(), "json");
    assert_eq!(paths.len(), 1);

    let content = fs::read_to_string(&paths[0]).expect("json content");
    let document: serde_json::Value = serde_json::from_str(&content).expect("valid json");
    assert_eq!(document["schema_version"], 1);
    assert_eq!(document["format"], "json");
    assert_eq!(document["transcript"]["connector"], "codex");
    assert_eq!(document["transcript"]["completeness"], "complete");
    assert_eq!(
        document["transcript"]["source_kind"],
        "app-server-thread-read"
    );
    assert_eq!(
        document["transcript"]["rounds"][0]["items"][0]["kind"],
        "user_message"
    );
    assert!(
        document["transcript"]["round_count"]
            .as_u64()
            .is_some_and(|value| value >= 1)
    );
}

#[test]
fn export_codex_writes_workspace_conversations_html() {
    let workspace = tempdir().expect("temp workspace");
    build_export_command("complete-thread", workspace.path())
        .arg("--format")
        .arg("html")
        .assert()
        .success()
        .stdout(predicate::str::contains("Format       : html"))
        .stdout(predicate::str::contains("Completeness : complete"))
        .stdout(predicate::str::contains(
            "Source       : app-server-thread-read",
        ));

    let paths = transcript_html_paths(workspace.path());
    assert_eq!(paths.len(), 1);
    let content = fs::read_to_string(&paths[0]).expect("html content");
    assert!(content.contains("<!DOCTYPE html>"));
    assert!(content.contains("第1轮"));
    assert!(content.contains("app-server-thread-read"));
    assert!(content.contains("pwd"));
    assert!(content.contains("Open archive shell"));
    assert!(content.contains("agent-exporter:workspace-shell-href"));
    assert!(content.contains("Open retrieval reports"));
    assert!(content.contains("agent-exporter:workspace-reports-shell-href"));
}

#[test]
fn export_codex_html_marks_incomplete_when_resume_fallback_is_used() {
    let workspace = tempdir().expect("temp workspace");
    build_export_command("fallback-thread", workspace.path())
        .arg("--format")
        .arg("html")
        .assert()
        .success()
        .stdout(predicate::str::contains("Format       : html"))
        .stdout(predicate::str::contains("Completeness : incomplete"))
        .stdout(predicate::str::contains(
            "Source       : app-server-resume-fallback",
        ));

    let paths = transcript_html_paths(workspace.path());
    assert_eq!(paths.len(), 1);
    let content = fs::read_to_string(&paths[0]).expect("html content");
    assert!(content.contains("Preview recovered through resume fallback"));
    assert!(content.contains("app-server-resume-fallback"));
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
