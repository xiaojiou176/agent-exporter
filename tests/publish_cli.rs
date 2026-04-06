use std::fs;
use std::path::{Path, PathBuf};

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
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

fn search_reports_dir(workspace_root: &Path) -> PathBuf {
    workspace_root
        .join(".agents")
        .join("Search")
        .join("Reports")
}

fn integration_reports_dir(workspace_root: &Path) -> PathBuf {
    workspace_root
        .join(".agents")
        .join("Integration")
        .join("Reports")
}

fn report_readiness(path: &Path) -> String {
    let document: Value =
        serde_json::from_str(&fs::read_to_string(path).expect("report json should exist"))
            .expect("valid report json");
    document["readiness"]
        .as_str()
        .expect("top-level readiness")
        .to_string()
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

fn build_onboard_report_command(target: &Path, workspace_root: &Path) -> Command {
    let mut command = Command::cargo_bin("agent-exporter").expect("binary should build");
    command
        .current_dir(workspace_root)
        .arg("onboard")
        .arg("codex")
        .arg("--target")
        .arg(target)
        .arg("--save-report");
    command
}

fn build_doctor_report_command(target: &Path, workspace_root: &Path) -> Command {
    let mut command = Command::cargo_bin("agent-exporter").expect("binary should build");
    command
        .current_dir(workspace_root)
        .arg("doctor")
        .arg("integrations")
        .arg("--platform")
        .arg("codex")
        .arg("--target")
        .arg(target)
        .arg("--save-report");
    command
}

fn collect_integration_report_jsons(workspace_root: &Path) -> Vec<PathBuf> {
    let reports_root = integration_reports_dir(workspace_root);
    let mut report_jsons = fs::read_dir(&reports_root)
        .expect("read reports root")
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .and_then(|value| value.to_str())
                .is_some_and(|value| value.eq_ignore_ascii_case("json"))
                && path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| {
                        name != "index.json"
                            && name != "baseline-registry.json"
                            && name != "decision-history.json"
                    })
        })
        .collect::<Vec<_>>();
    report_jsons.sort();
    report_jsons
}

fn build_evidence_baseline_promote_command(report: &Path, workspace_root: &Path) -> Command {
    let mut command = Command::cargo_bin("agent-exporter").expect("binary should build");
    command
        .current_dir(workspace_root)
        .arg("evidence")
        .arg("baseline")
        .arg("promote")
        .arg("--report")
        .arg(report)
        .arg("--name")
        .arg("codex-main")
        .arg("--policy")
        .arg("codex")
        .arg("--verdict")
        .arg("bootstrap");
    command
}

fn build_evidence_promote_command(candidate: &Path, workspace_root: &Path) -> Command {
    let mut command = Command::cargo_bin("agent-exporter").expect("binary should build");
    command
        .current_dir(workspace_root)
        .arg("evidence")
        .arg("promote")
        .arg("--candidate")
        .arg(candidate)
        .arg("--baseline-name")
        .arg("codex-main");
    command
}

fn integration_report_jsons(workspace_root: &Path) -> Vec<PathBuf> {
    let mut report_jsons = fs::read_dir(integration_reports_dir(workspace_root))
        .expect("read integration reports")
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .and_then(|value| value.to_str())
                .is_some_and(|value| value.eq_ignore_ascii_case("json"))
                && path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| {
                        name != "index.json"
                            && name != "baseline-registry.json"
                            && name != "decision-history.json"
                    })
        })
        .collect::<Vec<_>>();
    report_jsons.sort();
    report_jsons
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
        .stdout(predicate::str::contains("Reports      : 0"))
        .stdout(predicate::str::contains("Reports Index:"))
        .stdout(predicate::str::contains("index.html"));

    let index_path = conversations_dir(workspace.path()).join("index.html");
    assert!(index_path.exists());
    assert!(
        search_reports_dir(workspace.path())
            .join("index.html")
            .exists()
    );

    let content = fs::read_to_string(index_path).expect("index html");
    assert!(content.contains("<!DOCTYPE html>"));
    assert!(content.contains("Open transcript"));
    assert!(content.contains("Mock Complete Thread"));
    assert!(content.contains("claude-session-minimal"));
    assert!(content.contains("complete"));
    assert!(content.contains("degraded"));
    assert!(content.contains("archive-search"));
    assert!(content.contains("data-search-text"));
    assert!(content.contains("agent-exporter local archive shell"));
    assert!(content.contains("Open reports shell"));
    assert!(content.contains("search hybrid --workspace-root &lt;repo-root&gt;"));
    assert!(content.contains("data-filter-group=\"connector\""));
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

#[test]
fn publish_archive_index_links_saved_search_reports() {
    let workspace = tempdir().expect("workspace");
    build_codex_export_command("complete-thread", workspace.path())
        .assert()
        .success();

    let reports_dir = search_reports_dir(workspace.path());
    fs::create_dir_all(&reports_dir).expect("reports mkdirs");
    fs::write(
        reports_dir.join("search-report-semantic-demo.html"),
        concat!(
            "<!DOCTYPE html><html><head>",
            "<title>Semantic Retrieval Report</title>",
            "<meta name=\"agent-exporter:report-title\" content=\"Semantic Retrieval Report\">",
            "<meta name=\"agent-exporter:report-kind\" content=\"semantic\">",
            "<meta name=\"agent-exporter:search-query\" content=\"login issue\">",
            "<meta name=\"agent-exporter:generated-at\" content=\"2026-04-05T12:00:00Z\">",
            "</head><body></body></html>"
        ),
    )
    .expect("write report");

    build_publish_command(workspace.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Reports      : 1"))
        .stdout(predicate::str::contains("Reports Index:"));

    let content = fs::read_to_string(conversations_dir(workspace.path()).join("index.html"))
        .expect("index html");
    assert!(content.contains("Open report"));
    assert!(content.contains("Semantic Retrieval Report"));
    assert!(content.contains("login issue"));
    assert!(content.contains("../Search/Reports/search-report-semantic-demo.html"));

    let reports_index = fs::read_to_string(search_reports_dir(workspace.path()).join("index.html"))
        .expect("reports index");
    assert!(reports_index.contains("agent-exporter reports shell"));
    assert!(reports_index.contains("Open archive shell"));
    assert!(reports_index.contains("search-report-semantic-demo.html"));
    assert!(reports_index.contains("report-search"));
    assert!(reports_index.contains("data-report-kind"));
}

#[test]
fn publish_archive_index_renders_decision_desk_from_integration_evidence() {
    let workspace = tempdir().expect("workspace");
    let target = tempdir().expect("target dir");

    build_codex_export_command("complete-thread", workspace.path())
        .assert()
        .success();

    build_onboard_report_command(target.path(), workspace.path())
        .assert()
        .success();

    fs::write(
        target.path().join(".codex").join("config.toml"),
        "[mcp_servers.agent_exporter]\ncommand = \"python3\"\n",
    )
    .expect("break codex config");

    build_doctor_report_command(target.path(), workspace.path())
        .assert()
        .success();

    let reports_root = integration_reports_dir(workspace.path());
    let mut report_jsons = fs::read_dir(&reports_root)
        .expect("read reports")
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .and_then(|value| value.to_str())
                .is_some_and(|value| value.eq_ignore_ascii_case("json"))
                && path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name != "index.json")
        })
        .collect::<Vec<_>>();
    report_jsons.sort();
    let (baseline, candidate) = if report_readiness(&report_jsons[0]) == "ready" {
        (&report_jsons[0], &report_jsons[1])
    } else {
        (&report_jsons[1], &report_jsons[0])
    };

    Command::cargo_bin("agent-exporter")
        .expect("binary should build")
        .current_dir(workspace.path())
        .arg("evidence")
        .arg("baseline")
        .arg("promote")
        .arg("--report")
        .arg(baseline)
        .arg("--name")
        .arg("codex-main")
        .arg("--policy")
        .arg("codex")
        .assert()
        .success();

    Command::cargo_bin("agent-exporter")
        .expect("binary should build")
        .current_dir(workspace.path())
        .arg("evidence")
        .arg("promote")
        .arg("--candidate")
        .arg(candidate)
        .arg("--baseline-name")
        .arg("codex-main")
        .assert()
        .success();

    build_publish_command(workspace.path()).assert().success();

    let content = fs::read_to_string(conversations_dir(workspace.path()).join("index.html"))
        .expect("archive index");
    assert!(content.contains("Local Evidence Decision Desk"));
    assert!(content.contains("Baseline"));
    assert!(content.contains("Candidate"));
    assert!(content.contains("Remediation order"));
    assert!(content.contains("Changed checks"));
    assert!(content.contains("Official baseline / policy / promotion"));
    assert!(content.contains("baseline name:"));
    assert!(content.contains("active policy:"));
    assert!(content.contains("promotion status:"));
    assert!(content.contains("Decision history"));
    assert!(content.contains("Recent governance ledger"));
    assert!(content.contains("Open integration reports"));

    let integration_index =
        fs::read_to_string(integration_reports_dir(workspace.path()).join("index.html"))
            .expect("integration reports index");
    assert!(integration_index.contains("Open archive shell"));
    assert!(integration_index.contains("Open retrieval reports"));
}

#[test]
fn publish_archive_index_shows_insufficient_when_reports_are_not_comparable() {
    let workspace = tempdir().expect("workspace");
    let codex_target = tempdir().expect("codex target");
    let claude_target = tempdir().expect("claude target");

    build_codex_export_command("complete-thread", workspace.path())
        .assert()
        .success();

    build_onboard_report_command(codex_target.path(), workspace.path())
        .assert()
        .success();

    Command::cargo_bin("agent-exporter")
        .expect("binary should build")
        .current_dir(workspace.path())
        .arg("onboard")
        .arg("claude-code")
        .arg("--target")
        .arg(claude_target.path())
        .arg("--save-report")
        .assert()
        .success();

    build_publish_command(workspace.path()).assert().success();

    let content = fs::read_to_string(conversations_dir(workspace.path()).join("index.html"))
        .expect("archive index");
    assert!(content.contains("Local Evidence Decision Desk"));
    assert!(content.contains("insufficient"));
    assert!(content.contains("Insufficient comparison input"));
    assert!(content.contains("No artifact selected"));
}

#[test]
fn publish_archive_index_renders_phase31_governance_fields() {
    let workspace = tempdir().expect("workspace");
    let target = tempdir().expect("target dir");

    build_onboard_report_command(target.path(), workspace.path())
        .assert()
        .success();

    let ready_report = collect_integration_report_jsons(workspace.path())
        .into_iter()
        .find(|path| report_readiness(path) == "ready")
        .expect("ready report");

    Command::cargo_bin("agent-exporter")
        .expect("binary should build")
        .current_dir(workspace.path())
        .arg("evidence")
        .arg("baseline")
        .arg("promote")
        .arg("--report")
        .arg(&ready_report)
        .arg("--name")
        .arg("codex-main")
        .arg("--policy")
        .arg("codex")
        .assert()
        .success();

    fs::write(
        target.path().join(".codex").join("config.toml"),
        "[mcp_servers.agent_exporter]\ncommand = \"python3\"\n",
    )
    .expect("break codex config");

    build_doctor_report_command(target.path(), workspace.path())
        .assert()
        .success();

    let partial_report = collect_integration_report_jsons(workspace.path())
        .into_iter()
        .find(|path| report_readiness(path) == "partial")
        .expect("partial report");

    Command::cargo_bin("agent-exporter")
        .expect("binary should build")
        .current_dir(workspace.path())
        .arg("evidence")
        .arg("promote")
        .arg("--candidate")
        .arg(&partial_report)
        .arg("--baseline-name")
        .arg("codex-main")
        .arg("--policy")
        .arg("codex")
        .assert()
        .success();

    build_publish_command(workspace.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Archive index published"));

    let content = fs::read_to_string(conversations_dir(workspace.path()).join("index.html"))
        .expect("archive index html");
    assert!(content.contains("Official baseline"));
    assert!(content.contains("baseline name: <code>codex-main</code>"));
    assert!(content.contains("active policy: <code>codex</code> <code>1.0.0</code>"));
    assert!(content.contains("promotion status: <code>"));
    assert!(content.contains("Recent governance ledger"));
    assert!(content.contains("codex-main"));
}

#[test]
fn publish_archive_index_renders_governance_details_for_phase31() {
    let workspace = tempdir().expect("workspace");
    let target = tempdir().expect("target dir");

    build_codex_export_command("complete-thread", workspace.path())
        .assert()
        .success();

    build_onboard_report_command(target.path(), workspace.path())
        .assert()
        .success();

    let baseline = integration_report_jsons(workspace.path())
        .into_iter()
        .next()
        .expect("baseline report");

    build_evidence_baseline_promote_command(&baseline, workspace.path())
        .assert()
        .success();

    build_onboard_report_command(target.path(), workspace.path())
        .assert()
        .success();

    let candidate = integration_report_jsons(workspace.path())
        .into_iter()
        .find(|path| path != &baseline)
        .expect("candidate report");

    build_evidence_promote_command(&candidate, workspace.path())
        .assert()
        .success();

    build_publish_command(workspace.path()).assert().success();

    let content = fs::read_to_string(conversations_dir(workspace.path()).join("index.html"))
        .expect("archive index");
    assert!(content.contains("Official baseline"));
    assert!(content.contains("baseline name: <code>codex-main</code>"));
    assert!(content.contains("active policy: <code>codex</code> <code>1.0.0</code>"));
    assert!(content.contains("promotion status: <code>"));
    assert!(content.contains("Decision history"));
    assert!(content.contains("codex-main"));
}
