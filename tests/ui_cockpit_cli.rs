use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;

#[test]
fn ui_cockpit_help_lists_local_webui_command() {
    let mut command = Command::cargo_bin("agent-exporter").expect("binary should build");
    command.arg("ui").arg("cockpit").arg("--help");
    command
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Start the local Export Cockpit WebUI",
        ))
        .stdout(predicate::str::contains("--workspace-root"))
        .stdout(predicate::str::contains("--codex-home"))
        .stdout(predicate::str::contains("--open-browser"));
}

#[test]
fn cockpit_assets_expose_ai_summary_profile_model_provider_controls() {
    let html = fs::read_to_string("src/ui/assets/cockpit.html").expect("read cockpit.html");
    let js = fs::read_to_string("src/ui/assets/cockpit.js").expect("read cockpit.js");

    assert_eq!(html.matches("id=\"ai-summary-profile\"").count(), 1);
    assert_eq!(html.matches("id=\"ai-summary-preset\"").count(), 1);
    assert_eq!(html.matches("id=\"ai-summary-model\"").count(), 1);
    assert_eq!(html.matches("id=\"ai-summary-provider\"").count(), 1);
    assert_eq!(html.matches("id=\"ai-summary-timeout-seconds\"").count(), 1);
    assert!(js.contains("aiSummaryProfile"));
    assert!(js.contains("aiSummaryPreset"));
    assert!(js.contains("aiSummaryModel"));
    assert!(js.contains("aiSummaryProvider"));
    assert!(js.contains("aiSummaryTimeoutSeconds"));
    assert!(js.contains("aiSummaryProfile:"));
    assert!(js.contains("aiSummaryPreset:"));
    assert!(js.contains("aiSummaryModel:"));
    assert!(js.contains("aiSummaryProvider:"));
    assert!(js.contains("aiSummaryTimeoutSeconds:"));
    assert!(js.contains("--ai-summary-profile"));
    assert!(js.contains("--ai-summary-preset"));
    assert!(js.contains("--ai-summary-model"));
    assert!(js.contains("--ai-summary-provider"));
    assert!(js.contains("--ai-summary-timeout-seconds"));
    assert!(js.contains("connectorKind"));
    assert!(js.contains("sessionPath"));
}

#[test]
fn cockpit_action_copy_stays_connector_neutral_in_both_locales() {
    let html = fs::read_to_string("src/ui/assets/cockpit.html").expect("read cockpit.html");
    let js = fs::read_to_string("src/ui/assets/cockpit.js").expect("read cockpit.js");

    assert!(html.contains("Export a Codex or Claude session"));
    assert!(html.contains("selected sessions"));
    assert!(html.contains("Select one or more sessions"));
    assert!(html.contains("Loading active local sessions"));
    assert!(!html.contains("Export one Codex thread"));
    assert!(!html.contains("selected thread"));
    assert!(!html.contains("Select one thread"));
    assert!(!html.contains("Loading persisted Codex threads"));
    assert!(html.contains("Uses the canonical export path"));
    assert!(html.contains("matching workspace shell"));
    assert!(!html.contains("canonical Codex export path"));
    assert!(js.contains(
        "\"action.note\":\n      \"Exports through the canonical path, then refreshes archive / reports / evidence shells for affected workspaces.\""
    ));
    assert!(js.contains(
        "\"action.note\":\n      \"会沿通用导出主链导出，并刷新受影响 workspace 的 archive / reports / evidence shell。\""
    ));
    assert!(!js.contains("会沿 canonical Codex export path 导出"));
    assert!(js.contains("resultView"));
    assert!(js.contains("rerenderResultView"));
    assert!(js.contains("setResultView(\"idle\""));
    assert!(js.contains("setResultView(\"error\""));
    assert!(js.contains("setResultView(\"job\""));
    assert!(js.contains("setResultView(\"success\""));
}
