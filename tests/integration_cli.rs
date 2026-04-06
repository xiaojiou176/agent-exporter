use std::fs;
use std::path::{Path, PathBuf};

use assert_cmd::Command;
use predicates::prelude::*;
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
        .stdout(predicate::str::contains("target_content_sync [partial]"));
}
