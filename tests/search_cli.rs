use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

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
        .stdout(predicate::str::contains("--model-dir <MODEL_DIR>"));
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
        .stdout(predicate::str::contains("--model-dir <MODEL_DIR>"));
}
