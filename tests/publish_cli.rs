use std::fs;
use std::path::{Path, PathBuf};

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::{Value, json};
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

fn archive_index_json_path(workspace_root: &Path) -> PathBuf {
    conversations_dir(workspace_root).join("index.json")
}

fn refresh_manifest_path(workspace_root: &Path) -> PathBuf {
    conversations_dir(workspace_root).join("refresh-manifest.json")
}

fn fleet_view_path(workspace_root: &Path) -> PathBuf {
    conversations_dir(workspace_root).join("fleet-view.html")
}

fn fleet_view_json_path(workspace_root: &Path) -> PathBuf {
    conversations_dir(workspace_root).join("fleet-view.json")
}

fn action_packs_dir(workspace_root: &Path) -> PathBuf {
    conversations_dir(workspace_root).join("action-packs")
}

fn action_packs_index_path(workspace_root: &Path) -> PathBuf {
    action_packs_dir(workspace_root).join("index.html")
}

fn action_packs_index_json_path(workspace_root: &Path) -> PathBuf {
    action_packs_dir(workspace_root).join("index.json")
}

fn memory_lane_path(workspace_root: &Path) -> PathBuf {
    conversations_dir(workspace_root).join("memory-lane.html")
}

fn memory_lane_json_path(workspace_root: &Path) -> PathBuf {
    conversations_dir(workspace_root).join("memory-lane.json")
}

fn share_safe_packet_path(workspace_root: &Path) -> PathBuf {
    conversations_dir(workspace_root).join("share-safe-packet.md")
}

fn share_safe_packet_variant_path(workspace_root: &Path, variant: &str) -> PathBuf {
    conversations_dir(workspace_root).join(format!("share-safe-packet.{variant}.md"))
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

fn build_pin_answer_command(workspace_root: &Path, artifact: &Path, label: &str) -> Command {
    build_pin_answer_command_with_options(workspace_root, artifact, label, None, None)
}

fn build_pin_answer_command_with_options(
    workspace_root: &Path,
    artifact: &Path,
    label: &str,
    note: Option<&str>,
    supersedes: Option<&str>,
) -> Command {
    let mut command = Command::cargo_bin("agent-exporter").expect("binary should build");
    command
        .arg("publish")
        .arg("pin-answer")
        .arg("--workspace-root")
        .arg(workspace_root)
        .arg("--artifact")
        .arg(artifact)
        .arg("--label")
        .arg(label);
    if let Some(note) = note {
        command.arg("--note").arg(note);
    }
    if let Some(label) = supersedes {
        command.arg("--supersedes").arg(label);
    }
    command
}

fn build_unpin_answer_command(workspace_root: &Path, label: &str) -> Command {
    let mut command = Command::cargo_bin("agent-exporter").expect("binary should build");
    command
        .arg("publish")
        .arg("unpin-answer")
        .arg("--workspace-root")
        .arg(workspace_root)
        .arg("--label")
        .arg(label);
    command
}

fn build_resolve_answer_command(workspace_root: &Path, label: &str, note: Option<&str>) -> Command {
    let mut command = Command::cargo_bin("agent-exporter").expect("binary should build");
    command
        .arg("publish")
        .arg("resolve-answer")
        .arg("--workspace-root")
        .arg(workspace_root)
        .arg("--label")
        .arg(label);
    if let Some(note) = note {
        command.arg("--note").arg(note);
    }
    command
}

struct SummaryFixtureSpec<'a> {
    file_stem: &'a str,
    thread_id: &'a str,
    generated_at: &'a str,
    title: &'a str,
    overview: &'a str,
    profile_id: &'a str,
    files_touched: &'a [&'a str],
    tests_run: &'a [&'a str],
    risks: &'a [&'a str],
    blockers: &'a [&'a str],
    next_steps: &'a [&'a str],
    citations: &'a [&'a str],
}

fn write_structured_summary_fixture(
    workspace_root: &Path,
    spec: SummaryFixtureSpec<'_>,
) -> PathBuf {
    let dir = conversations_dir(workspace_root);
    fs::create_dir_all(&dir).expect("create conversations dir");
    let markdown = dir.join(format!("{}.md", spec.file_stem));
    let html = dir.join(format!("{}.html", spec.file_stem));
    let json_path = dir.join(format!("{}.json", spec.file_stem));

    fs::write(
        &markdown,
        format!("# {}\n\n{}\n", spec.title, spec.overview),
    )
    .expect("write summary markdown");
    fs::write(
        &html,
        format!(
            "<!DOCTYPE html><html><head><title>{}</title></head><body>{}</body></html>",
            spec.title, spec.overview
        ),
    )
    .expect("write summary html");
    fs::write(
        &json_path,
        serde_json::to_string_pretty(&json!({
            "schemaVersion": 1,
            "threadId": spec.thread_id,
            "connector": "codex",
            "sourceKind": "app-server-thread-read",
            "completeness": "complete",
            "generatedAt": spec.generated_at,
            "profileId": spec.profile_id,
            "runtimeProfile": null,
            "runtimeModel": null,
            "runtimeProvider": null,
            "familyKey": format!("thread:{}", spec.thread_id),
            "title": spec.title,
            "overview": spec.overview,
            "shareSafeSummary": spec.overview,
            "goals": ["make the workbench genuinely day-to-day usable"],
            "filesTouched": spec.files_touched,
            "testsRun": spec.tests_run,
            "risks": spec.risks,
            "blockers": spec.blockers,
            "nextSteps": spec.next_steps,
            "citations": spec.citations,
            "extractionMode": "json-block",
            "outputFiles": {
                "markdown": markdown.file_name().expect("markdown file").to_string_lossy(),
                "html": html.file_name().expect("html file").to_string_lossy(),
                "json": json_path.file_name().expect("json file").to_string_lossy(),
            }
        }))
        .expect("render summary json"),
    )
    .expect("write summary json");

    json_path
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
                            && name.starts_with("integration-report-")
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
                            && name.starts_with("integration-report-")
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
    assert!(content.contains("Browse transcript evidence first."));
    assert!(content.contains("Open reports shell"));
    assert!(content.contains("Open retrieval lane"));
    assert!(content.contains("data-filter-group=\"connector\""));
    assert!(content.contains(".html"));
    assert!(!content.contains(".json"));
}

#[test]
fn workspace_export_auto_refreshes_workbench_and_writes_machine_readable_projection() {
    let workspace = tempdir().expect("workspace");

    build_codex_export_command("complete-thread", workspace.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Workbench"));

    let archive_index = conversations_dir(workspace.path()).join("index.html");
    let archive_index_json = archive_index_json_path(workspace.path());
    let share_safe_packet = share_safe_packet_path(workspace.path());
    let reports_index = search_reports_dir(workspace.path()).join("index.html");
    let integration_index = integration_reports_dir(workspace.path()).join("index.html");

    assert!(archive_index.exists(), "archive index should auto-refresh");
    assert!(
        archive_index_json.exists(),
        "archive index json projection should exist"
    );
    assert!(reports_index.exists(), "reports shell should exist");
    assert!(integration_index.exists(), "integration shell should exist");
    assert!(share_safe_packet.exists(), "share-safe packet should exist");

    let projection: Value =
        serde_json::from_str(&fs::read_to_string(&archive_index_json).expect("archive index json"))
            .expect("valid archive index json");
    assert!(
        projection["timeline"]
            .as_array()
            .is_some_and(|items| !items.is_empty()),
        "timeline should contain transcript activity"
    );
    assert!(
        projection["families"]
            .as_array()
            .is_some_and(|items| !items.is_empty()),
        "family projection should exist"
    );

    let share_safe = fs::read_to_string(share_safe_packet).expect("share-safe packet");
    assert!(share_safe.contains("Share-safe workbench packet"));
    assert!(
        !share_safe.contains(&workspace.path().display().to_string()),
        "share-safe packet should avoid absolute workspace paths"
    );
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
    assert!(
        reports_index
            .contains("Revisit saved retrieval work without leaving the archive workbench.")
    );
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
                && path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.starts_with("integration-report-"))
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
    assert!(content.contains("Governance snapshot"));
    assert!(content.contains("Official baseline"));
    assert!(content.contains("Candidate"));
    assert!(content.contains("Remediation bundle"));
    assert!(content.contains("Official baseline / active policy / promotion"));
    assert!(content.contains("baseline name:"));
    assert!(content.contains("active policy:"));
    assert!(content.contains("promotion status:"));
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
    assert!(content.contains("Governance snapshot"));
    assert!(content.contains("insufficient"));
    assert!(content.contains("No artifact selected"));
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
    assert!(content.contains("Governance snapshot"));
    assert!(content.contains("Official baseline"));
    assert!(content.contains("baseline name: <code>codex-main</code>"));
    assert!(content.contains("active policy: <code>codex</code> <code>1.0.0</code>"));
    assert!(content.contains("promotion status: <code>"));
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
    assert!(content.contains("Governance snapshot"));
    assert!(content.contains("Official baseline"));
    assert!(content.contains("baseline name: <code>codex-main</code>"));
    assert!(content.contains("active policy: <code>codex</code> <code>1.0.0</code>"));
    assert!(content.contains("promotion status: <code>"));
    assert!(content.contains("codex-main"));
}

#[test]
fn publish_archive_index_renders_workspace_intelligence_and_pinned_answers_with_stale_status() {
    let workspace = tempdir().expect("workspace");
    let target = tempdir().expect("target dir");

    build_codex_export_command("complete-thread", workspace.path())
        .assert()
        .success();

    build_onboard_report_command(target.path(), workspace.path())
        .assert()
        .success();

    let ready_report = collect_integration_report_jsons(workspace.path())
        .into_iter()
        .find(|path| report_readiness(path) == "ready")
        .expect("ready report");

    build_pin_answer_command(
        workspace.path(),
        &ready_report,
        "Current official readiness",
    )
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

    build_publish_command(workspace.path()).assert().success();

    let content = fs::read_to_string(conversations_dir(workspace.path()).join("index.html"))
        .expect("archive index html");
    assert!(content.contains("Workspace timeline"));
    assert!(content.contains("Thread families"));
    assert!(content.contains("Official answers"));
    assert!(content.contains("Current official readiness"));
    assert!(content.contains("stale"));

    let projection: Value = serde_json::from_str(
        &fs::read_to_string(archive_index_json_path(workspace.path())).expect("archive index json"),
    )
    .expect("valid archive index json");
    assert!(
        projection["officialAnswers"]
            .as_array()
            .is_some_and(|pins| !pins.is_empty()),
        "pinned answers should be projected"
    );
    assert!(
        projection["officialAnswers"][0]["stale"].as_bool() == Some(true),
        "pin should be marked stale after a newer related report exists"
    );
}

#[test]
fn official_answer_workflow_supports_supersede_resolve_and_unpin_commands() {
    let workspace = tempdir().expect("workspace");
    build_codex_export_command("complete-thread", workspace.path())
        .assert()
        .success();

    let initial_summary = write_structured_summary_fixture(
        workspace.path(),
        SummaryFixtureSpec {
            file_stem: "release-case-a1b2c3d4-ai-summary-rounds-1-2-2026-04-21_12-00-00",
            thread_id: "complete-thread",
            generated_at: "2026-04-21T12:00:00Z",
            title: "Release answer v1",
            overview: "Initial official answer before final review.",
            profile_id: "decision",
            files_touched: &["src/workbench.rs"],
            tests_run: &["cargo test"],
            risks: &["copy drift"],
            blockers: &[],
            next_steps: &["rerun publish"],
            citations: &["summary-v1"],
        },
    );
    let final_summary = write_structured_summary_fixture(
        workspace.path(),
        SummaryFixtureSpec {
            file_stem: "release-case-a1b2c3d4-ai-summary-rounds-3-4-2026-04-21_13-00-00",
            thread_id: "complete-thread",
            generated_at: "2026-04-21T13:00:00Z",
            title: "Release answer v2",
            overview: "Final official answer after final review.",
            profile_id: "release-note",
            files_touched: &["src/workbench.rs", "src/output/archive_index.rs"],
            tests_run: &["cargo test", "cargo run -- publish archive-index"],
            risks: &["release wording drift"],
            blockers: &[],
            next_steps: &["ship v0.1.6"],
            citations: &["summary-v2"],
        },
    );

    build_pin_answer_command_with_options(
        workspace.path(),
        &initial_summary,
        "Release verdict v1",
        Some("initial call"),
        None,
    )
    .assert()
    .success();
    build_pin_answer_command_with_options(
        workspace.path(),
        &final_summary,
        "Release verdict v2",
        Some("final call"),
        Some("Release verdict v1"),
    )
    .assert()
    .success();
    build_resolve_answer_command(
        workspace.path(),
        "Release verdict v2",
        Some("accepted and shipped"),
    )
    .assert()
    .success();
    build_publish_command(workspace.path()).assert().success();

    let projection: Value = serde_json::from_str(
        &fs::read_to_string(archive_index_json_path(workspace.path())).expect("archive index json"),
    )
    .expect("valid archive index json");
    let pins = projection["officialAnswers"]
        .as_array()
        .expect("official answers array");
    assert_eq!(pins.len(), 2);
    assert_eq!(pins[0]["label"], "Release verdict v2");
    assert_eq!(pins[0]["status"], "resolved");
    assert_eq!(pins[1]["label"], "Release verdict v1");
    assert_eq!(pins[1]["status"], "superseded");
    assert_eq!(pins[1]["supersededBy"], "Release verdict v2");
    assert!(
        pins[0]["note"]
            .as_str()
            .is_some_and(|note| note.contains("accepted and shipped")),
        "resolved answer should preserve the resolution note"
    );

    let content = fs::read_to_string(conversations_dir(workspace.path()).join("index.html"))
        .expect("archive index");
    assert!(content.contains("Resolve via CLI"));
    assert!(content.contains("Unpin via CLI"));
    assert!(content.contains("resolved"));
    assert!(content.contains("superseded"));
    assert!(content.contains("Pin as official answer"));

    build_unpin_answer_command(workspace.path(), "Release verdict v1")
        .assert()
        .success();
    build_publish_command(workspace.path()).assert().success();

    let refreshed: Value = serde_json::from_str(
        &fs::read_to_string(archive_index_json_path(workspace.path())).expect("archive index json"),
    )
    .expect("valid archive index json");
    assert_eq!(
        refreshed["officialAnswers"]
            .as_array()
            .expect("official answers after unpin")
            .len(),
        1
    );
}

#[test]
fn publish_archive_index_projects_case_view_and_share_safe_packet_tiers() {
    let workspace = tempdir().expect("workspace");
    build_codex_export_command("complete-thread", workspace.path())
        .assert()
        .success();

    let earlier_summary = write_structured_summary_fixture(
        workspace.path(),
        SummaryFixtureSpec {
            file_stem: "case-thread-a1b2c3d4-ai-summary-rounds-1-2-2026-04-21_12-00-00",
            thread_id: "complete-thread",
            generated_at: "2026-04-21T12:00:00Z",
            title: "Bug RCA snapshot",
            overview: "Older pinned case answer.",
            profile_id: "bug-rca",
            files_touched: &["src/workbench.rs"],
            tests_run: &["cargo test"],
            risks: &["copy drift"],
            blockers: &["stale answer"],
            next_steps: &["refresh the answer"],
            citations: &["citation-a"],
        },
    );
    write_structured_summary_fixture(
        workspace.path(),
        SummaryFixtureSpec {
            file_stem: "case-thread-a1b2c3d4-ai-summary-rounds-3-4-2026-04-21_14-00-00",
            thread_id: "complete-thread",
            generated_at: "2026-04-21T14:00:00Z",
            title: "Decision snapshot",
            overview: "Latest case snapshot with broader coverage.",
            profile_id: "decision",
            files_touched: &["src/workbench.rs", "src/output/archive_index.rs"],
            tests_run: &[
                "cargo test",
                "cargo run -- publish archive-index --workspace-root /tmp/repo",
            ],
            risks: &["public wording drift"],
            blockers: &["manual review pending"],
            next_steps: &["pin final answer", "cut release"],
            citations: &["citation-b"],
        },
    );

    build_pin_answer_command_with_options(
        workspace.path(),
        &earlier_summary,
        "Pinned case answer",
        Some("older official view"),
        None,
    )
    .assert()
    .success();
    build_publish_command(workspace.path()).assert().success();

    let projection: Value = serde_json::from_str(
        &fs::read_to_string(archive_index_json_path(workspace.path())).expect("archive index json"),
    )
    .expect("valid archive index json");
    let family = &projection["families"][0];
    assert_eq!(family["officialAnswer"]["label"], "Pinned case answer");
    assert_eq!(family["latestSummary"]["title"], "Decision snapshot");
    assert_eq!(family["latestVsPinned"]["status"], "ahead");
    assert!(
        family["latestVsPinned"]["newFilesTouched"]
            .as_array()
            .is_some_and(|items| items
                .iter()
                .any(|item| item == "src/output/archive_index.rs")),
        "case diff should surface files introduced after the pinned answer"
    );
    assert!(
        family["testsRun"]
            .as_array()
            .is_some_and(|items| items.iter().any(|item| item == "cargo test")),
        "family projection should aggregate tests"
    );

    let teammate_packet = share_safe_packet_variant_path(workspace.path(), "teammate");
    let reviewer_packet = share_safe_packet_variant_path(workspace.path(), "reviewer");
    let public_packet = share_safe_packet_variant_path(workspace.path(), "public");
    assert!(teammate_packet.exists(), "teammate packet should exist");
    assert!(reviewer_packet.exists(), "reviewer packet should exist");
    assert!(public_packet.exists(), "public packet should exist");

    let reviewer = fs::read_to_string(reviewer_packet).expect("reviewer packet");
    assert!(reviewer.contains("citation-a"));
    let public = fs::read_to_string(public_packet).expect("public packet");
    assert!(!public.contains("citation-a"));
    assert!(
        !public.contains(&workspace.path().display().to_string()),
        "public packet should avoid absolute workspace paths"
    );

    let content = fs::read_to_string(conversations_dir(workspace.path()).join("index.html"))
        .expect("archive index html");
    assert!(content.contains("Case view"));
    assert!(content.contains("Latest vs pinned answer"));
    assert!(content.contains("share-safe-packet.public.md"));
    assert!(content.contains("Pin as official answer"));
}

#[test]
fn publish_archive_index_writes_refresh_manifest_and_reuses_when_inputs_are_unchanged() {
    let workspace = tempdir().expect("workspace");
    build_codex_export_command("complete-thread", workspace.path())
        .assert()
        .success();

    build_publish_command(workspace.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Refresh Manifest:"));
    build_publish_command(workspace.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Refresh Mode : reused"))
        .stdout(predicate::str::contains("Changed Families: 0"));

    let manifest: Value = serde_json::from_str(
        &fs::read_to_string(refresh_manifest_path(workspace.path())).expect("refresh manifest"),
    )
    .expect("valid refresh manifest json");
    assert_eq!(manifest["refreshMode"], "reused");
    assert_eq!(manifest["changedFamilies"], json!([]));
}

#[test]
fn publish_archive_index_generates_dedicated_fleet_view_and_action_bridge_outputs() {
    let workspace = tempdir().expect("workspace");
    let target = tempdir().expect("target dir");

    build_codex_export_command("complete-thread", workspace.path())
        .assert()
        .success();

    let earlier_summary = write_structured_summary_fixture(
        workspace.path(),
        SummaryFixtureSpec {
            file_stem: "bridge-thread-a1b2c3d4-ai-summary-rounds-1-2-2026-04-21_12-00-00",
            thread_id: "complete-thread",
            generated_at: "2026-04-21T12:00:00Z",
            title: "Decision snapshot v1",
            overview: "Older pinned case answer.",
            profile_id: "decision",
            files_touched: &["src/workbench.rs"],
            tests_run: &["cargo test"],
            risks: &["copy drift"],
            blockers: &["manual review pending"],
            next_steps: &["refresh the answer"],
            citations: &["citation-a"],
        },
    );
    write_structured_summary_fixture(
        workspace.path(),
        SummaryFixtureSpec {
            file_stem: "bridge-thread-a1b2c3d4-ai-summary-rounds-3-4-2026-04-21_14-00-00",
            thread_id: "complete-thread",
            generated_at: "2026-04-21T14:00:00Z",
            title: "Decision snapshot v2",
            overview: "Latest case snapshot with broader coverage.",
            profile_id: "handoff",
            files_touched: &["src/workbench.rs", "src/output/archive_index.rs"],
            tests_run: &[
                "cargo test",
                "cargo clippy --all-targets --all-features -- -D warnings",
            ],
            risks: &["public wording drift"],
            blockers: &["manual review pending"],
            next_steps: &["pin final answer", "cut release"],
            citations: &["citation-b"],
        },
    );
    build_pin_answer_command_with_options(
        workspace.path(),
        &earlier_summary,
        "Bridge verdict",
        Some("older official view"),
        None,
    )
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

    build_publish_command(workspace.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Fleet View  :"))
        .stdout(predicate::str::contains("Action Packs:"));

    let fleet_html = fs::read_to_string(fleet_view_path(workspace.path())).expect("fleet page");
    assert!(fleet_html.contains("Fleet readiness board"));
    assert!(fleet_html.contains("Readiness drift timeline"));
    let fleet_json: Value = serde_json::from_str(
        &fs::read_to_string(fleet_view_json_path(workspace.path())).expect("fleet json"),
    )
    .expect("valid fleet json");
    assert!(
        fleet_json["entries"]
            .as_array()
            .is_some_and(|entries| !entries.is_empty()),
        "fleet json should include at least one relation"
    );

    let packs_index =
        fs::read_to_string(action_packs_index_path(workspace.path())).expect("action packs index");
    assert!(packs_index.contains("Report-to-action bridge"));
    assert!(packs_index.contains("re-review pack"));
    assert!(packs_index.contains("what changed note"));
    let packs_json: Value = serde_json::from_str(
        &fs::read_to_string(action_packs_index_json_path(workspace.path()))
            .expect("action pack index json"),
    )
    .expect("valid action pack index json");
    assert!(
        packs_json["packs"]
            .as_array()
            .is_some_and(|packs| packs.len() >= 2),
        "action bridge should emit multiple generated packs"
    );
}

#[test]
fn publish_archive_index_builds_workspace_memory_lane_from_sibling_workbench_indexes() {
    let parent = tempdir().expect("parent");
    let workspace_a = parent.path().join("repo-a");
    let workspace_b = parent.path().join("repo-b");
    fs::create_dir_all(&workspace_a).expect("mkdir repo-a");
    fs::create_dir_all(&workspace_b).expect("mkdir repo-b");

    build_codex_export_command("complete-thread", &workspace_a)
        .assert()
        .success();
    build_publish_command(&workspace_a).assert().success();

    build_codex_export_command("complete-thread", &workspace_b)
        .assert()
        .success();
    build_publish_command(&workspace_b).assert().success();
    build_publish_command(&workspace_a).assert().success();

    let memory_lane = fs::read_to_string(memory_lane_path(&workspace_a)).expect("memory lane");
    assert!(memory_lane.contains("Team and org memory lane"));
    assert!(memory_lane.contains("repo-a"));
    assert!(memory_lane.contains("repo-b"));
    assert!(memory_lane.contains("maintainer"));
    assert!(memory_lane.contains("reviewer"));
    assert!(memory_lane.contains("operator"));

    let memory_lane_json: Value = serde_json::from_str(
        &fs::read_to_string(memory_lane_json_path(&workspace_a)).expect("memory lane json"),
    )
    .expect("valid memory lane json");
    assert!(
        memory_lane_json["workspaces"]
            .as_array()
            .is_some_and(|workspaces| workspaces.len() >= 2),
        "memory lane should aggregate sibling workspaces with published indexes"
    );
}
