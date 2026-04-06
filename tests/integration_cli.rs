use std::fs;
use std::path::{Path, PathBuf};

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use tempfile::tempdir;

const MCP_PLACEHOLDER: &str = "/absolute/path/to/agent-exporter/scripts/agent_exporter_mcp.py";

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn expected_launcher_fragment() -> String {
    let debug_bin = repo_root()
        .join("target")
        .join("debug")
        .join("agent-exporter");
    if debug_bin.is_file() {
        debug_bin.display().to_string()
    } else {
        repo_root().join("Cargo.toml").display().to_string()
    }
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).expect("file should exist")
}

fn report_readiness(path: &Path) -> String {
    let document: Value = serde_json::from_str(&read(path)).expect("valid report json");
    document["readiness"]
        .as_str()
        .expect("top-level readiness")
        .to_string()
}

fn integration_reports_root(workspace_root: &Path) -> PathBuf {
    workspace_root
        .join(".agents")
        .join("Integration")
        .join("Reports")
}

fn baseline_registry_path(workspace_root: &Path) -> PathBuf {
    integration_reports_root(workspace_root).join("baseline-registry.json")
}

fn decision_history_path(workspace_root: &Path) -> PathBuf {
    integration_reports_root(workspace_root).join("decision-history.json")
}

fn collect_integration_report_jsons(workspace_root: &Path) -> Vec<PathBuf> {
    let reports_root = workspace_root
        .join(".agents")
        .join("Integration")
        .join("Reports");
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

fn collect_remediation_bundle_jsons(workspace_root: &Path) -> Vec<PathBuf> {
    let reports_root = integration_reports_root(workspace_root);
    let mut bundle_jsons = fs::read_dir(&reports_root)
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
                    .is_some_and(|name| name.starts_with("remediation-bundle-"))
        })
        .collect::<Vec<_>>();
    bundle_jsons.sort();
    bundle_jsons
}

#[test]
fn integrate_codex_materializes_target_with_resolved_paths() {
    let target = tempdir().expect("target dir");

    let mut command = Command::cargo_bin("agent-exporter").expect("binary should build");
    command
        .arg("integrate")
        .arg("codex")
        .arg("--target")
        .arg(target.path());

    command
        .assert()
        .success()
        .stdout(predicate::str::contains("Integration materialized"))
        .stdout(predicate::str::contains("Platform     : codex"));

    let agents = read(&target.path().join("AGENTS.md"));
    let skill = read(
        &target
            .path()
            .join(".agents")
            .join("skills")
            .join("export-archive")
            .join("SKILL.md"),
    );
    let config = read(&target.path().join(".codex").join("config.toml"));

    assert!(!agents.contains("agent-exporter publish archive-index"));
    assert!(agents.contains(&expected_launcher_fragment()));
    assert!(skill.contains(&expected_launcher_fragment()));
    assert!(!config.contains(MCP_PLACEHOLDER));
    assert!(
        config.contains(
            &repo_root()
                .join("scripts")
                .join("agent_exporter_mcp.py")
                .display()
                .to_string()
        )
    );
}

#[test]
fn onboard_codex_materializes_and_explains_next_steps() {
    let target = tempdir().expect("target dir");

    let mut command = Command::cargo_bin("agent-exporter").expect("binary should build");
    command
        .arg("onboard")
        .arg("codex")
        .arg("--target")
        .arg(target.path());

    command
        .assert()
        .success()
        .stdout(predicate::str::contains("Integration onboarding completed"))
        .stdout(predicate::str::contains("Readiness    : ready"))
        .stdout(predicate::str::contains("Next Steps"))
        .stdout(predicate::str::contains(
            "review `AGENTS.md`, `.agents/skills/`, and `.codex/config.toml`",
        ));

    assert!(target.path().join("AGENTS.md").is_file());
    assert!(
        target
            .path()
            .join(".agents")
            .join("skills")
            .join("export-archive")
            .join("SKILL.md")
            .is_file()
    );
    assert!(target.path().join(".codex").join("config.toml").is_file());
}

#[test]
fn onboard_codex_save_report_writes_integration_evidence() {
    let workspace = tempdir().expect("workspace");
    let target = tempdir().expect("target dir");

    let mut command = Command::cargo_bin("agent-exporter").expect("binary should build");
    command
        .current_dir(workspace.path())
        .arg("onboard")
        .arg("codex")
        .arg("--target")
        .arg(target.path())
        .arg("--save-report");

    command
        .assert()
        .success()
        .stdout(predicate::str::contains("Integration onboarding completed"))
        .stdout(predicate::str::contains("Reports Index"))
        .stdout(predicate::str::contains("Remediation  :"));

    let reports_root = workspace
        .path()
        .join(".agents")
        .join("Integration")
        .join("Reports");
    let index = reports_root.join("index.html");
    assert!(index.is_file());
    let index_json = reports_root.join("index.json");
    assert!(index_json.is_file());

    let report_files = fs::read_dir(&reports_root)
        .expect("read reports root")
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .and_then(|value| value.to_str())
                .is_some_and(|value| value.eq_ignore_ascii_case("html"))
                && path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name != "index.html")
        })
        .collect::<Vec<_>>();
    assert_eq!(report_files.len(), 1);
    let report_json_files = fs::read_dir(&reports_root)
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
                    .is_some_and(|name| name != "index.json")
                && path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.starts_with("integration-report-"))
        })
        .collect::<Vec<_>>();
    assert_eq!(report_json_files.len(), 1);

    let report = read(&report_files[0]);
    assert!(report.contains("agent-exporter:report-kind\" content=\"onboard"));
    assert!(report.contains("agent-exporter:integration-platform\" content=\"codex"));
    assert!(report.contains("Open integration reports"));
    let report_json = read(&report_json_files[0]);
    assert!(report_json.contains("\"platform\": \"codex\""));
    assert!(report_json.contains("\"kind\": \"onboard\""));
    assert!(report_json.contains("\"artifact_links\""));
    assert!(report_json.contains("\"index_json\": \"index.json\""));
}

#[test]
fn doctor_codex_save_report_writes_front_door_without_touching_transcript_corpus() {
    let workspace = tempdir().expect("workspace");
    let target = tempdir().expect("target dir");

    Command::cargo_bin("agent-exporter")
        .expect("binary should build")
        .current_dir(workspace.path())
        .arg("integrate")
        .arg("codex")
        .arg("--target")
        .arg(target.path())
        .assert()
        .success();

    let mut command = Command::cargo_bin("agent-exporter").expect("binary should build");
    command
        .current_dir(workspace.path())
        .arg("doctor")
        .arg("integrations")
        .arg("--platform")
        .arg("codex")
        .arg("--target")
        .arg(target.path())
        .arg("--save-report");

    command
        .assert()
        .success()
        .stdout(predicate::str::contains("Integration doctor completed"))
        .stdout(predicate::str::contains("Report"))
        .stdout(predicate::str::contains("Reports Root"));

    let reports_root = workspace
        .path()
        .join(".agents")
        .join("Integration")
        .join("Reports");
    let index = reports_root.join("index.html");
    assert!(index.is_file());
    let index_json = reports_root.join("index.json");
    assert!(index_json.is_file());
    let index_content = read(&index);
    assert!(index_content.contains("integration-report-search"));
    assert!(index_content.contains("No integration reports matched the current search."));
    let index_json_content = read(&index_json);
    assert!(index_json_content.contains("\"timeline\""));
    assert!(index_json_content.contains("\"report_count\": 1"));

    let conversations_dir = workspace.path().join(".agents").join("Conversations");
    assert!(!conversations_dir.exists());
}

#[test]
fn doctor_integrations_explain_prints_remediation_plan() {
    let target = tempdir().expect("target dir");

    Command::cargo_bin("agent-exporter")
        .expect("binary should build")
        .arg("integrate")
        .arg("codex")
        .arg("--target")
        .arg(target.path())
        .assert()
        .success();

    fs::write(
        target.path().join(".codex").join("config.toml"),
        "[mcp_servers.agent_exporter]\ncommand = \"python3\"\n",
    )
    .expect("break codex config");

    Command::cargo_bin("agent-exporter")
        .expect("binary should build")
        .arg("doctor")
        .arg("integrations")
        .arg("--platform")
        .arg("codex")
        .arg("--target")
        .arg(target.path())
        .arg("--explain")
        .assert()
        .success()
        .stdout(predicate::str::contains("- Remediation"))
        .stdout(predicate::str::contains(".codex/config.toml"))
        .stdout(predicate::str::contains("recheck :"));
}

#[test]
fn evidence_gate_and_explain_surfaces_verdict_and_steps() {
    let workspace = tempdir().expect("workspace");
    let target = tempdir().expect("target dir");

    Command::cargo_bin("agent-exporter")
        .expect("binary should build")
        .current_dir(workspace.path())
        .arg("onboard")
        .arg("codex")
        .arg("--target")
        .arg(target.path())
        .arg("--save-report")
        .assert()
        .success();

    fs::write(
        target.path().join(".codex").join("config.toml"),
        "[mcp_servers.agent_exporter]\ncommand = \"python3\"\n",
    )
    .expect("break codex config");

    Command::cargo_bin("agent-exporter")
        .expect("binary should build")
        .current_dir(workspace.path())
        .arg("doctor")
        .arg("integrations")
        .arg("--platform")
        .arg("codex")
        .arg("--target")
        .arg(target.path())
        .arg("--save-report")
        .assert()
        .success();

    let reports_root = workspace
        .path()
        .join(".agents")
        .join("Integration")
        .join("Reports");
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
                        name != "index.json" && name.starts_with("integration-report-")
                    })
        })
        .collect::<Vec<_>>();
    report_jsons.sort();
    assert_eq!(report_jsons.len(), 2);

    let (baseline, candidate) = if report_readiness(&report_jsons[0]) == "ready" {
        (&report_jsons[0], &report_jsons[1])
    } else {
        (&report_jsons[1], &report_jsons[0])
    };

    Command::cargo_bin("agent-exporter")
        .expect("binary should build")
        .arg("evidence")
        .arg("gate")
        .arg("--baseline")
        .arg(baseline)
        .arg("--candidate")
        .arg(candidate)
        .assert()
        .success()
        .stdout(predicate::str::contains("Integration evidence gate"))
        .stdout(predicate::str::contains("Verdict      : fail"))
        .stdout(predicate::str::contains("Blocking Changes"))
        .stdout(predicate::str::contains("codex_config_shape"));

    Command::cargo_bin("agent-exporter")
        .expect("binary should build")
        .arg("evidence")
        .arg("explain")
        .arg("--report")
        .arg(candidate)
        .assert()
        .success()
        .stdout(predicate::str::contains("Integration evidence explain"))
        .stdout(predicate::str::contains("- Remediation"))
        .stdout(predicate::str::contains(".codex/config.toml"));

    Command::cargo_bin("agent-exporter")
        .expect("binary should build")
        .arg("evidence")
        .arg("remediation")
        .arg("--report")
        .arg(candidate)
        .assert()
        .success()
        .stdout(predicate::str::contains("Integration remediation bundle"))
        .stdout(predicate::str::contains("- Bundle"))
        .stdout(predicate::str::contains(".codex/config.toml"));
}

#[test]
fn evidence_baseline_list_show_and_promote_happy_path() {
    let workspace = tempdir().expect("workspace");
    let target = tempdir().expect("target dir");

    Command::cargo_bin("agent-exporter")
        .expect("binary should build")
        .current_dir(workspace.path())
        .arg("onboard")
        .arg("codex")
        .arg("--target")
        .arg(target.path())
        .arg("--save-report")
        .assert()
        .success();

    let report_path = integration_reports_root(workspace.path())
        .join("integration-report-onboard-codex-2026-04-06t12-00-00z.json");
    let report = fs::read_dir(integration_reports_root(workspace.path()))
        .expect("read reports")
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .find(|path| {
            path.extension()
                .and_then(|value| value.to_str())
                .is_some_and(|value| value == "json")
                && path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| {
                        name != "index.json" && name.starts_with("integration-report-")
                    })
        })
        .unwrap_or(report_path);

    Command::cargo_bin("agent-exporter")
        .expect("binary should build")
        .current_dir(workspace.path())
        .arg("evidence")
        .arg("baseline")
        .arg("promote")
        .arg("--report")
        .arg(&report)
        .arg("--name")
        .arg("codex-main")
        .arg("--policy")
        .arg("codex")
        .assert()
        .success()
        .stdout(predicate::str::contains("Integration baseline promoted"))
        .stdout(predicate::str::contains("codex-main"))
        .stdout(predicate::str::contains("Policy       : codex v1.0.0"));

    Command::cargo_bin("agent-exporter")
        .expect("binary should build")
        .current_dir(workspace.path())
        .arg("evidence")
        .arg("baseline")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("Baselines    : 1"))
        .stdout(predicate::str::contains("codex-main"));

    Command::cargo_bin("agent-exporter")
        .expect("binary should build")
        .current_dir(workspace.path())
        .arg("evidence")
        .arg("baseline")
        .arg("show")
        .arg("--name")
        .arg("codex-main")
        .assert()
        .success()
        .stdout(predicate::str::contains("Integration baseline"))
        .stdout(predicate::str::contains("Identity"))
        .stdout(predicate::str::contains("Policy       : codex v1.0.0"));

    assert!(baseline_registry_path(workspace.path()).is_file());
    assert!(decision_history_path(workspace.path()).is_file());
}

#[test]
fn evidence_policy_list_and_show_surface_repo_owned_packs() {
    Command::cargo_bin("agent-exporter")
        .expect("binary should build")
        .arg("evidence")
        .arg("policy")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("Policies"))
        .stdout(predicate::str::contains("default v1.0.0"))
        .stdout(predicate::str::contains("codex v1.0.0"));

    Command::cargo_bin("agent-exporter")
        .expect("binary should build")
        .arg("evidence")
        .arg("policy")
        .arg("show")
        .arg("--name")
        .arg("codex")
        .assert()
        .success()
        .stdout(predicate::str::contains("Integration governance policy"))
        .stdout(predicate::str::contains("\"name\": \"codex\""))
        .stdout(predicate::str::contains("\"allowed_verdicts\""));
}

#[test]
fn evidence_diff_reports_readiness_and_check_changes() {
    let workspace = tempdir().expect("workspace");
    let target = tempdir().expect("target dir");

    Command::cargo_bin("agent-exporter")
        .expect("binary should build")
        .current_dir(workspace.path())
        .arg("integrate")
        .arg("codex")
        .arg("--target")
        .arg(target.path())
        .assert()
        .success();

    Command::cargo_bin("agent-exporter")
        .expect("binary should build")
        .current_dir(workspace.path())
        .arg("doctor")
        .arg("integrations")
        .arg("--platform")
        .arg("codex")
        .arg("--target")
        .arg(target.path())
        .arg("--save-report")
        .assert()
        .success();

    let config_path = target.path().join(".codex").join("config.toml");
    let broken = read(&config_path).replace("args = [", "# args = [");
    fs::write(&config_path, broken).expect("write broken config");

    Command::cargo_bin("agent-exporter")
        .expect("binary should build")
        .current_dir(workspace.path())
        .arg("doctor")
        .arg("integrations")
        .arg("--platform")
        .arg("codex")
        .arg("--target")
        .arg(target.path())
        .arg("--save-report")
        .assert()
        .success();

    let reports_root = workspace
        .path()
        .join(".agents")
        .join("Integration")
        .join("Reports");
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
                        name != "index.json" && name.starts_with("integration-report-")
                    })
        })
        .collect::<Vec<_>>();
    report_jsons.sort();
    assert_eq!(report_jsons.len(), 2);

    let mut command = Command::cargo_bin("agent-exporter").expect("binary should build");
    command
        .arg("evidence")
        .arg("diff")
        .arg("--left")
        .arg(&report_jsons[0])
        .arg("--right")
        .arg(&report_jsons[1]);

    command
        .assert()
        .success()
        .stdout(predicate::str::contains("Integration evidence diff"))
        .stdout(predicate::str::contains("Readiness    : ready -> partial"))
        .stdout(predicate::str::contains("codex_config_shape"))
        .stdout(predicate::str::contains("Added Next Steps"));
}

#[test]
fn evidence_baseline_policy_promotion_and_history_commands_work() {
    let workspace = tempdir().expect("workspace");
    let target = tempdir().expect("target dir");

    Command::cargo_bin("agent-exporter")
        .expect("binary should build")
        .current_dir(workspace.path())
        .arg("onboard")
        .arg("codex")
        .arg("--target")
        .arg(target.path())
        .arg("--save-report")
        .assert()
        .success();

    let config_path = target.path().join(".codex").join("config.toml");
    let original_config = read(&config_path);
    fs::write(
        &config_path,
        "[mcp_servers.agent_exporter]\ncommand = \"python3\"\n",
    )
    .expect("break config");

    Command::cargo_bin("agent-exporter")
        .expect("binary should build")
        .current_dir(workspace.path())
        .arg("doctor")
        .arg("integrations")
        .arg("--platform")
        .arg("codex")
        .arg("--target")
        .arg(target.path())
        .arg("--save-report")
        .assert()
        .success();

    fs::write(&config_path, original_config).expect("restore config");

    Command::cargo_bin("agent-exporter")
        .expect("binary should build")
        .current_dir(workspace.path())
        .arg("doctor")
        .arg("integrations")
        .arg("--platform")
        .arg("codex")
        .arg("--target")
        .arg(target.path())
        .arg("--save-report")
        .assert()
        .success();

    let report_jsons = collect_integration_report_jsons(workspace.path());
    assert_eq!(report_jsons.len(), 3);

    let ready_reports = report_jsons
        .iter()
        .filter(|path| report_readiness(path) == "ready")
        .collect::<Vec<_>>();
    let partial_report = report_jsons
        .iter()
        .find(|path| report_readiness(path) == "partial")
        .expect("partial report should exist");
    assert_eq!(ready_reports.len(), 2);
    let baseline_seed = ready_reports[0];
    let promotion_candidate = ready_reports[1];

    Command::cargo_bin("agent-exporter")
        .expect("binary should build")
        .current_dir(workspace.path())
        .arg("evidence")
        .arg("baseline")
        .arg("promote")
        .arg("--report")
        .arg(baseline_seed)
        .arg("--name")
        .arg("codex-main")
        .arg("--policy")
        .arg("codex")
        .assert()
        .success()
        .stdout(predicate::str::contains("Integration baseline promoted"))
        .stdout(predicate::str::contains("codex-main"));

    Command::cargo_bin("agent-exporter")
        .expect("binary should build")
        .current_dir(workspace.path())
        .arg("evidence")
        .arg("baseline")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("Integration baseline registry"))
        .stdout(predicate::str::contains("codex-main"));

    Command::cargo_bin("agent-exporter")
        .expect("binary should build")
        .current_dir(workspace.path())
        .arg("evidence")
        .arg("policy")
        .arg("show")
        .arg("--name")
        .arg("codex")
        .assert()
        .success()
        .stdout(predicate::str::contains("Integration governance policy"))
        .stdout(predicate::str::contains("codex"));

    Command::cargo_bin("agent-exporter")
        .expect("binary should build")
        .current_dir(workspace.path())
        .arg("evidence")
        .arg("gate")
        .arg("--baseline")
        .arg("codex-main")
        .arg("--candidate")
        .arg(partial_report)
        .arg("--policy")
        .arg("codex")
        .assert()
        .success()
        .stdout(predicate::str::contains("Verdict      : fail"));

    Command::cargo_bin("agent-exporter")
        .expect("binary should build")
        .current_dir(workspace.path())
        .arg("evidence")
        .arg("promote")
        .arg("--candidate")
        .arg(partial_report)
        .arg("--baseline-name")
        .arg("codex-main")
        .arg("--policy")
        .arg("codex")
        .assert()
        .success()
        .stdout(predicate::str::contains("Promoted     : no"));

    Command::cargo_bin("agent-exporter")
        .expect("binary should build")
        .current_dir(workspace.path())
        .arg("evidence")
        .arg("promote")
        .arg("--candidate")
        .arg(promotion_candidate)
        .arg("--baseline-name")
        .arg("codex-main")
        .arg("--policy")
        .arg("codex")
        .assert()
        .success()
        .stdout(predicate::str::contains("Promoted     : yes"))
        .stdout(predicate::str::contains("Summary"));

    Command::cargo_bin("agent-exporter")
        .expect("binary should build")
        .current_dir(workspace.path())
        .arg("evidence")
        .arg("current")
        .arg("--baseline-name")
        .arg("codex-main")
        .assert()
        .success()
        .stdout(predicate::str::contains("Integration current decision"))
        .stdout(predicate::str::contains("Baseline     : codex-main"))
        .stdout(predicate::str::contains("Promotion    : eligible"));

    Command::cargo_bin("agent-exporter")
        .expect("binary should build")
        .current_dir(workspace.path())
        .arg("evidence")
        .arg("baseline")
        .arg("show")
        .arg("--name")
        .arg("codex-main")
        .assert()
        .success()
        .stdout(predicate::str::contains("Integration baseline"))
        .stdout(predicate::str::contains("codex-main"))
        .stdout(predicate::str::contains("Verdict      : pass"));

    Command::cargo_bin("agent-exporter")
        .expect("binary should build")
        .current_dir(workspace.path())
        .arg("evidence")
        .arg("history")
        .arg("--baseline-name")
        .arg("codex-main")
        .assert()
        .success()
        .stdout(predicate::str::contains("Integration decision history"))
        .stdout(predicate::str::contains("codex-main"))
        .stdout(predicate::str::contains("promoted yes"));
}

#[test]
fn evidence_baseline_and_policy_commands_cover_phase31_governance_surfaces() {
    let workspace = tempdir().expect("workspace");
    let target = tempdir().expect("target dir");

    Command::cargo_bin("agent-exporter")
        .expect("binary should build")
        .current_dir(workspace.path())
        .arg("onboard")
        .arg("codex")
        .arg("--target")
        .arg(target.path())
        .arg("--save-report")
        .assert()
        .success();

    let reports = collect_integration_report_jsons(workspace.path());
    assert_eq!(reports.len(), 1);

    Command::cargo_bin("agent-exporter")
        .expect("binary should build")
        .current_dir(workspace.path())
        .arg("evidence")
        .arg("baseline")
        .arg("promote")
        .arg("--report")
        .arg(&reports[0])
        .arg("--name")
        .arg("codex-main")
        .arg("--policy")
        .arg("codex")
        .arg("--verdict")
        .arg("bootstrap")
        .assert()
        .success()
        .stdout(predicate::str::contains("Integration baseline promoted"))
        .stdout(predicate::str::contains("codex-main"))
        .stdout(predicate::str::contains("Policy       : codex v1.0.0"));

    let registry = read(&integration_reports_root(workspace.path()).join("baseline-registry.json"));
    assert!(registry.contains("\"name\": \"codex-main\""));
    assert!(registry.contains("\"policy_name\": \"codex\""));

    Command::cargo_bin("agent-exporter")
        .expect("binary should build")
        .current_dir(workspace.path())
        .arg("evidence")
        .arg("baseline")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("Integration baseline registry"))
        .stdout(predicate::str::contains("codex-main"));

    Command::cargo_bin("agent-exporter")
        .expect("binary should build")
        .current_dir(workspace.path())
        .arg("evidence")
        .arg("baseline")
        .arg("show")
        .arg("--name")
        .arg("codex-main")
        .assert()
        .success()
        .stdout(predicate::str::contains("Identity"))
        .stdout(predicate::str::contains("codex-main"))
        .stdout(predicate::str::contains("bootstrap"));

    Command::cargo_bin("agent-exporter")
        .expect("binary should build")
        .arg("evidence")
        .arg("policy")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("default v1.0.0"))
        .stdout(predicate::str::contains("codex v1.0.0"));

    Command::cargo_bin("agent-exporter")
        .expect("binary should build")
        .arg("evidence")
        .arg("policy")
        .arg("show")
        .arg("--name")
        .arg("codex")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"name\": \"codex\""))
        .stdout(predicate::str::contains("allowed_verdicts"))
        .stdout(predicate::str::contains("allowed_candidate_readiness"));
}

#[test]
fn evidence_promote_uses_baseline_name_and_records_decision_history() {
    let workspace = tempdir().expect("workspace");
    let target = tempdir().expect("target dir");

    Command::cargo_bin("agent-exporter")
        .expect("binary should build")
        .current_dir(workspace.path())
        .arg("onboard")
        .arg("codex")
        .arg("--target")
        .arg(target.path())
        .arg("--save-report")
        .assert()
        .success();

    let baseline_report = collect_integration_report_jsons(workspace.path())
        .into_iter()
        .next()
        .expect("baseline report");

    Command::cargo_bin("agent-exporter")
        .expect("binary should build")
        .current_dir(workspace.path())
        .arg("evidence")
        .arg("baseline")
        .arg("promote")
        .arg("--report")
        .arg(&baseline_report)
        .arg("--name")
        .arg("codex-main")
        .arg("--policy")
        .arg("codex")
        .arg("--verdict")
        .arg("bootstrap")
        .assert()
        .success();

    Command::cargo_bin("agent-exporter")
        .expect("binary should build")
        .current_dir(workspace.path())
        .arg("onboard")
        .arg("codex")
        .arg("--target")
        .arg(target.path())
        .arg("--save-report")
        .assert()
        .success();

    let reports = collect_integration_report_jsons(workspace.path());
    assert_eq!(reports.len(), 2);
    let candidate = reports
        .into_iter()
        .find(|path| path != &baseline_report)
        .expect("candidate report");

    Command::cargo_bin("agent-exporter")
        .expect("binary should build")
        .current_dir(workspace.path())
        .arg("evidence")
        .arg("gate")
        .arg("--baseline")
        .arg("codex-main")
        .arg("--candidate")
        .arg(&candidate)
        .assert()
        .success()
        .stdout(predicate::str::contains("Integration evidence gate"))
        .stdout(predicate::str::contains("Verdict      : pass"));

    Command::cargo_bin("agent-exporter")
        .expect("binary should build")
        .current_dir(workspace.path())
        .arg("evidence")
        .arg("promote")
        .arg("--candidate")
        .arg(&candidate)
        .arg("--baseline-name")
        .arg("codex-main")
        .assert()
        .success()
        .stdout(predicate::str::contains("Integration evidence promote"))
        .stdout(predicate::str::contains("Promoted     : yes"));

    let history_path = integration_reports_root(workspace.path()).join("decision-history.json");
    let history = read(&history_path);
    assert!(history.contains("\"baseline_name\": \"codex-main\""));
    assert!(history.contains("\"promoted\": true"));

    Command::cargo_bin("agent-exporter")
        .expect("binary should build")
        .current_dir(workspace.path())
        .arg("evidence")
        .arg("history")
        .arg("--baseline-name")
        .arg("codex-main")
        .assert()
        .success()
        .stdout(predicate::str::contains("Integration decision history"))
        .stdout(predicate::str::contains("promoted yes"));
}

#[test]
fn integrate_codex_rejects_live_codex_home_root() {
    let home = tempdir().expect("home dir");
    let forbidden_target = home.path().join(".codex");

    let mut command = Command::cargo_bin("agent-exporter").expect("binary should build");
    command
        .env("HOME", home.path())
        .arg("integrate")
        .arg("codex")
        .arg("--target")
        .arg(&forbidden_target);

    command.assert().failure().stderr(predicate::str::contains(
        "choose a staging pack directory instead of the live Codex home root `~/.codex`",
    ));
}

#[test]
fn onboard_claude_code_rejects_live_claude_home_root() {
    let home = tempdir().expect("home dir");
    let forbidden_target = home.path().join(".claude-testing");

    let mut command = Command::cargo_bin("agent-exporter").expect("binary should build");
    command
        .env("HOME", home.path())
        .arg("onboard")
        .arg("claude-code")
        .arg("--target")
        .arg(&forbidden_target);

    command.assert().failure().stderr(predicate::str::contains(
        "choose a staging pack directory instead of a live Claude home root such as `~/.claude*`",
    ));
}

#[test]
fn integrate_openclaw_rejects_direct_bundle_root_targets() {
    let target = tempdir().expect("target dir");
    let forbidden_target = target.path().join("openclaw-codex-bundle");

    let mut command = Command::cargo_bin("agent-exporter").expect("binary should build");
    command
        .arg("integrate")
        .arg("openclaw")
        .arg("--target")
        .arg(&forbidden_target);

    command.assert().failure().stderr(predicate::str::contains(
        "point `--target` at a neutral staging directory above the bundle/plugin roots",
    ));
}

#[test]
fn integrate_openclaw_rejects_bundles_child_targets() {
    let target = tempdir().expect("target dir");
    let forbidden_target = target.path().join("bundles").join("live-bundle");

    let mut command = Command::cargo_bin("agent-exporter").expect("binary should build");
    command
        .arg("integrate")
        .arg("openclaw")
        .arg("--target")
        .arg(&forbidden_target);

    command.assert().failure().stderr(predicate::str::contains(
        "point `--target` at a neutral staging directory above the bundle/plugin roots",
    ));
}

#[test]
fn onboard_openclaw_rejects_plugins_child_targets() {
    let target = tempdir().expect("target dir");
    let forbidden_target = target.path().join("plugins").join("live-plugin");

    let mut command = Command::cargo_bin("agent-exporter").expect("binary should build");
    command
        .arg("onboard")
        .arg("openclaw")
        .arg("--target")
        .arg(&forbidden_target);

    command.assert().failure().stderr(predicate::str::contains(
        "point `--target` at a neutral staging directory above the bundle/plugin roots",
    ));
}

#[test]
fn integrate_claude_code_refuses_to_overwrite_existing_files() {
    let target = tempdir().expect("target dir");
    let conflict = target.path().join(".mcp.json");
    fs::write(&conflict, "{ \"existing\": true }\n").expect("write conflict");

    let mut command = Command::cargo_bin("agent-exporter").expect("binary should build");
    command
        .arg("integrate")
        .arg("claude-code")
        .arg("--target")
        .arg(target.path());

    command.assert().failure().stderr(predicate::str::contains(
        "integration materializer refuses to overwrite existing file",
    ));
}

#[test]
fn doctor_integrations_reports_codex_ready_after_materialization() {
    let target = tempdir().expect("target dir");

    Command::cargo_bin("agent-exporter")
        .expect("binary should build")
        .arg("integrate")
        .arg("codex")
        .arg("--target")
        .arg(target.path())
        .assert()
        .success();

    let mut command = Command::cargo_bin("agent-exporter").expect("binary should build");
    command
        .arg("doctor")
        .arg("integrations")
        .arg("--platform")
        .arg("codex")
        .arg("--target")
        .arg(target.path());

    command
        .assert()
        .success()
        .stdout(predicate::str::contains("Integration doctor completed"))
        .stdout(predicate::str::contains("Readiness    : ready"))
        .stdout(predicate::str::contains(
            "Summary      : codex pack looks ready",
        ))
        .stdout(predicate::str::contains("Next Steps"))
        .stdout(predicate::str::contains("bridge_script [ready]"))
        .stdout(predicate::str::contains("target_files [ready]"))
        .stdout(predicate::str::contains("3/3 expected files present"))
        .stdout(predicate::str::contains("target_content_sync [ready]"))
        .stdout(predicate::str::contains("launcher_probe [ready]"))
        .stdout(predicate::str::contains("codex_config_shape [ready]"));
}

#[test]
fn doctor_integrations_reports_missing_when_target_is_absent() {
    let target = tempdir().expect("target dir");
    let missing = target.path().join("missing-codex-pack");

    let mut command = Command::cargo_bin("agent-exporter").expect("binary should build");
    command
        .arg("doctor")
        .arg("integrations")
        .arg("--platform")
        .arg("claude-code")
        .arg("--target")
        .arg(&missing);

    command
        .assert()
        .success()
        .stdout(predicate::str::contains("Readiness    : missing"))
        .stdout(predicate::str::contains("Next Steps"))
        .stdout(predicate::str::contains(
            "Run `agent-exporter integrate claude-code --target",
        ))
        .stdout(predicate::str::contains("target_root [missing]"));
}

#[test]
fn doctor_integrations_reports_claude_ready_after_materialization() {
    let target = tempdir().expect("target dir");

    Command::cargo_bin("agent-exporter")
        .expect("binary should build")
        .arg("integrate")
        .arg("claude-code")
        .arg("--target")
        .arg(target.path())
        .assert()
        .success();

    let mut command = Command::cargo_bin("agent-exporter").expect("binary should build");
    command
        .arg("doctor")
        .arg("integrations")
        .arg("--platform")
        .arg("claude-code")
        .arg("--target")
        .arg(target.path());

    command
        .assert()
        .success()
        .stdout(predicate::str::contains("Readiness    : ready"))
        .stdout(predicate::str::contains(
            "Summary      : claude-code pack looks ready",
        ))
        .stdout(predicate::str::contains("claude_project_shape [ready]"))
        .stdout(predicate::str::contains("claude_pack_shape [ready]"));
}

#[test]
fn doctor_integrations_reports_claude_partial_when_mcp_json_is_invalid() {
    let target = tempdir().expect("target dir");

    Command::cargo_bin("agent-exporter")
        .expect("binary should build")
        .arg("integrate")
        .arg("claude-code")
        .arg("--target")
        .arg(target.path())
        .assert()
        .success();

    fs::write(target.path().join(".mcp.json"), "{ invalid json\n").expect("write invalid json");

    let mut command = Command::cargo_bin("agent-exporter").expect("binary should build");
    command
        .arg("doctor")
        .arg("integrations")
        .arg("--platform")
        .arg("claude-code")
        .arg("--target")
        .arg(target.path());

    command
        .assert()
        .success()
        .stdout(predicate::str::contains("Readiness    : partial"))
        .stdout(predicate::str::contains("Next Steps"))
        .stdout(predicate::str::contains("claude_project_shape [partial]"));
}

#[test]
fn integrate_openclaw_materializes_both_bundle_variants() {
    let target = tempdir().expect("target dir");

    let mut command = Command::cargo_bin("agent-exporter").expect("binary should build");
    command
        .arg("integrate")
        .arg("openclaw")
        .arg("--target")
        .arg(target.path());

    command
        .assert()
        .success()
        .stdout(predicate::str::contains("Platform     : openclaw"));

    let codex_plugin = target
        .path()
        .join("openclaw-codex-bundle")
        .join(".codex-plugin")
        .join("plugin.json");
    let claude_plugin = target
        .path()
        .join("openclaw-claude-bundle")
        .join(".claude-plugin")
        .join("plugin.json");
    let codex_skill = target
        .path()
        .join("openclaw-codex-bundle")
        .join("skills")
        .join("export-archive")
        .join("SKILL.md");
    let claude_mcp = target
        .path()
        .join("openclaw-claude-bundle")
        .join(".mcp.json");

    assert!(codex_plugin.is_file());
    assert!(claude_plugin.is_file());
    assert!(codex_skill.is_file());
    assert!(claude_mcp.is_file());

    let skill_content = read(&codex_skill);
    let mcp_content = read(&claude_mcp);
    assert!(skill_content.contains(&expected_launcher_fragment()));
    assert!(!mcp_content.contains(MCP_PLACEHOLDER));
}

#[test]
fn doctor_integrations_reports_openclaw_ready_after_materialization() {
    let target = tempdir().expect("target dir");

    Command::cargo_bin("agent-exporter")
        .expect("binary should build")
        .arg("integrate")
        .arg("openclaw")
        .arg("--target")
        .arg(target.path())
        .assert()
        .success();

    let mut command = Command::cargo_bin("agent-exporter").expect("binary should build");
    command
        .arg("doctor")
        .arg("integrations")
        .arg("--platform")
        .arg("openclaw")
        .arg("--target")
        .arg(target.path());

    command
        .assert()
        .success()
        .stdout(predicate::str::contains("Readiness    : ready"))
        .stdout(predicate::str::contains(
            "Summary      : openclaw pack looks ready",
        ))
        .stdout(predicate::str::contains("openclaw_bundle_shape [ready]"));
}

#[test]
fn doctor_integrations_reports_codex_partial_when_config_shape_is_incomplete() {
    let target = tempdir().expect("target dir");

    Command::cargo_bin("agent-exporter")
        .expect("binary should build")
        .arg("integrate")
        .arg("codex")
        .arg("--target")
        .arg(target.path())
        .assert()
        .success();

    fs::write(
        target.path().join(".codex").join("config.toml"),
        "[mcp_servers.agent_exporter]\ncommand = \"python3\"\n",
    )
    .expect("write incomplete codex config");

    let mut command = Command::cargo_bin("agent-exporter").expect("binary should build");
    command
        .arg("doctor")
        .arg("integrations")
        .arg("--platform")
        .arg("codex")
        .arg("--target")
        .arg(target.path());

    command
        .assert()
        .success()
        .stdout(predicate::str::contains("Readiness    : partial"))
        .stdout(predicate::str::contains("Next Steps"))
        .stdout(predicate::str::contains("codex_config_shape [partial]"));
}

#[test]
fn doctor_integrations_reports_claude_partial_when_pack_shape_is_incomplete() {
    let target = tempdir().expect("target dir");

    Command::cargo_bin("agent-exporter")
        .expect("binary should build")
        .arg("integrate")
        .arg("claude-code")
        .arg("--target")
        .arg(target.path())
        .assert()
        .success();

    fs::write(
        target
            .path()
            .join(".claude")
            .join("commands")
            .join("publish-archive.md"),
        "Run:\n\nagent-exporter publish archive-index --workspace-root .\n",
    )
    .expect("write incomplete claude command");

    let mut command = Command::cargo_bin("agent-exporter").expect("binary should build");
    command
        .arg("doctor")
        .arg("integrations")
        .arg("--platform")
        .arg("claude-code")
        .arg("--target")
        .arg(target.path());

    command
        .assert()
        .success()
        .stdout(predicate::str::contains("Readiness    : partial"))
        .stdout(predicate::str::contains("Next Steps"))
        .stdout(predicate::str::contains("claude_pack_shape [partial]"));
}

#[test]
fn doctor_integrations_reports_partial_when_target_drifted() {
    let target = tempdir().expect("target dir");

    Command::cargo_bin("agent-exporter")
        .expect("binary should build")
        .arg("integrate")
        .arg("codex")
        .arg("--target")
        .arg(target.path())
        .assert()
        .success();

    let drifted_agents = target.path().join("AGENTS.md");
    let original = read(&drifted_agents);
    let drifted = original.replace(&expected_launcher_fragment(), "agent-exporter");
    fs::write(&drifted_agents, drifted).expect("write drifted agents");

    let mut command = Command::cargo_bin("agent-exporter").expect("binary should build");
    command
        .arg("doctor")
        .arg("integrations")
        .arg("--platform")
        .arg("codex")
        .arg("--target")
        .arg(target.path());

    command
        .assert()
        .success()
        .stdout(predicate::str::contains("Readiness    : partial"))
        .stdout(predicate::str::contains("Next Steps"))
        .stdout(predicate::str::contains("target_content_sync [partial]"));
}
