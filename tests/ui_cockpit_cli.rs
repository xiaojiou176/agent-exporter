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
    assert_eq!(html.matches("id=\"ai-summary-model\"").count(), 1);
    assert_eq!(html.matches("id=\"ai-summary-provider\"").count(), 1);
    assert!(js.contains("aiSummaryProfile"));
    assert!(js.contains("aiSummaryModel"));
    assert!(js.contains("aiSummaryProvider"));
    assert!(js.contains("aiSummaryProfile:"));
    assert!(js.contains("aiSummaryModel:"));
    assert!(js.contains("aiSummaryProvider:"));
    assert!(js.contains("--ai-summary-profile"));
    assert!(js.contains("--ai-summary-model"));
    assert!(js.contains("--ai-summary-provider"));
}
