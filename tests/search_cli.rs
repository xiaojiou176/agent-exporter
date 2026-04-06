use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

fn write_transcript_fixture(workspace_root: &Path, file_name: &str, title: &str, body: &str) {
    let archive_dir = workspace_root.join(".agents").join("Conversations");
    fs::create_dir_all(&archive_dir).expect("archive dir");
    fs::write(
        archive_dir.join(file_name),
        format!(
            concat!(
                "<!DOCTYPE html><html><head>",
                "<meta name=\"agent-exporter:thread-display-name\" content=\"{title}\">",
                "<meta name=\"agent-exporter:connector\" content=\"codex\">",
                "<meta name=\"agent-exporter:thread-id\" content=\"thread-1\">",
                "<meta name=\"agent-exporter:completeness\" content=\"complete\">",
                "<meta name=\"agent-exporter:source-kind\" content=\"app-server-thread-read\">",
                "<meta name=\"agent-exporter:exported-at\" content=\"2026-04-05T00:00:00Z\">",
                "</head><body><p>{body}</p></body></html>"
            ),
            title = title,
            body = body,
        ),
    )
    .expect("write transcript");
}

fn report_files(workspace_root: &Path) -> Vec<PathBuf> {
    let reports_dir = workspace_root
        .join(".agents")
        .join("Search")
        .join("Reports");
    if !reports_dir.exists() {
        return Vec::new();
    }
    let mut paths = fs::read_dir(reports_dir)
        .expect("reports dir")
        .map(|entry| entry.expect("dir entry").path())
        .filter(|path| {
            path.extension()
                .and_then(|value| value.to_str())
                .is_some_and(|value| value == "html")
        })
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name != "index.html")
        })
        .collect::<Vec<_>>();
    paths.sort();
    paths
}

fn maybe_model_dir() -> Option<PathBuf> {
    let path =
        agent_exporter::core::semantic_search::FastEmbedSemanticEmbedder::default_model_dir()
            .ok()?;
    path.is_dir().then_some(path)
}

#[test]
fn semantic_search_requires_query() {
    let workspace = tempdir().expect("workspace");
    let mut command = Command::cargo_bin("agent-exporter").expect("binary should build");
    command
        .arg("search")
        .arg("semantic")
        .arg("--workspace-root")
        .arg(workspace.path());

    command
        .assert()
        .failure()
        .stderr(predicate::str::contains("--query <QUERY>"));
}

#[test]
fn hybrid_search_requires_query() {
    let workspace = tempdir().expect("workspace");
    let mut command = Command::cargo_bin("agent-exporter").expect("binary should build");
    command
        .arg("search")
        .arg("hybrid")
        .arg("--workspace-root")
        .arg(workspace.path());

    command
        .assert()
        .failure()
        .stderr(predicate::str::contains("--query <QUERY>"));
}

#[test]
fn semantic_search_errors_when_model_dir_is_missing() {
    let workspace = tempdir().expect("workspace");
    let missing_model_dir = workspace.path().join("missing-model");

    let mut command = Command::cargo_bin("agent-exporter").expect("binary should build");
    command
        .arg("search")
        .arg("semantic")
        .arg("--workspace-root")
        .arg(workspace.path())
        .arg("--query")
        .arg("auth failure")
        .arg("--model-dir")
        .arg(&missing_model_dir);

    command.assert().failure().stderr(predicate::str::contains(
        "semantic retrieval requires local embedding model files",
    ));
}

#[test]
fn hybrid_search_errors_when_model_dir_is_missing() {
    let workspace = tempdir().expect("workspace");
    let missing_model_dir = workspace.path().join("missing-model");

    let mut command = Command::cargo_bin("agent-exporter").expect("binary should build");
    command
        .arg("search")
        .arg("hybrid")
        .arg("--workspace-root")
        .arg(workspace.path())
        .arg("--query")
        .arg("auth failure")
        .arg("--model-dir")
        .arg(&missing_model_dir);

    command.assert().failure().stderr(predicate::str::contains(
        "semantic retrieval requires local embedding model files",
    ));
}

#[test]
fn semantic_search_help_describes_embedding_based_retrieval() {
    let mut command = Command::cargo_bin("agent-exporter").expect("binary should build");
    command.arg("search").arg("semantic").arg("--help");

    command
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Run embedding-based semantic retrieval over local HTML transcript exports",
        ))
        .stdout(predicate::str::contains("--model-dir <MODEL_DIR>"))
        .stdout(predicate::str::contains("--save-report"));
}

#[test]
fn hybrid_search_help_describes_blended_retrieval() {
    let mut command = Command::cargo_bin("agent-exporter").expect("binary should build");
    command.arg("search").arg("hybrid").arg("--help");

    command
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Run hybrid retrieval that blends semantic ranking with lexical metadata signals",
        ))
        .stdout(predicate::str::contains("--model-dir <MODEL_DIR>"))
        .stdout(predicate::str::contains("--save-report"));
}

#[test]
fn semantic_search_save_report_writes_html_report_when_model_assets_exist() {
    let Some(model_dir) = maybe_model_dir() else {
        return;
    };

    let workspace = tempdir().expect("workspace");
    write_transcript_fixture(
        workspace.path(),
        "auth.html",
        "Auth transcript",
        "How do I fix login issues in auth flow?",
    );

    let mut command = Command::cargo_bin("agent-exporter").expect("binary should build");
    command
        .arg("search")
        .arg("semantic")
        .arg("--workspace-root")
        .arg(workspace.path())
        .arg("--query")
        .arg("login issues")
        .arg("--model-dir")
        .arg(&model_dir)
        .arg("--save-report");

    command
        .assert()
        .success()
        .stdout(predicate::str::contains("Semantic search completed"))
        .stdout(predicate::str::contains("Report       :"));

    let reports = report_files(workspace.path());
    assert_eq!(reports.len(), 1);
    let content = fs::read_to_string(&reports[0]).expect("report content");
    assert!(content.contains("agent-exporter:report-kind"));
    assert!(content.contains("Semantic Retrieval Report"));
    assert!(content.contains("../../Conversations/index.html"));
}

#[test]
fn hybrid_search_save_report_writes_html_report_when_model_assets_exist() {
    let Some(model_dir) = maybe_model_dir() else {
        return;
    };

    let workspace = tempdir().expect("workspace");
    write_transcript_fixture(
        workspace.path(),
        "auth.html",
        "Auth transcript",
        "How do I fix login issues in auth flow?",
    );

    let mut command = Command::cargo_bin("agent-exporter").expect("binary should build");
    command
        .arg("search")
        .arg("hybrid")
        .arg("--workspace-root")
        .arg(workspace.path())
        .arg("--query")
        .arg("thread-1")
        .arg("--model-dir")
        .arg(&model_dir)
        .arg("--save-report");

    command
        .assert()
        .success()
        .stdout(predicate::str::contains("Hybrid search completed"))
        .stdout(predicate::str::contains("Report       :"));

    let reports = report_files(workspace.path());
    assert_eq!(reports.len(), 1);
    let content = fs::read_to_string(&reports[0]).expect("report content");
    assert!(content.contains("agent-exporter:report-kind"));
    assert!(content.contains("Hybrid Retrieval Report"));
    assert!(content.contains("lexical score"));
}
