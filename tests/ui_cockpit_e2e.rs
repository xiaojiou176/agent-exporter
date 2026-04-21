use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread::sleep;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use assert_cmd::cargo::cargo_bin;
use rusqlite::{Connection, params};
use serde_json::Value;
use tempfile::tempdir;

fn cockpit_binary() -> PathBuf {
    cargo_bin("agent-exporter")
}

fn seed_codex_state(codex_home: &Path, workspace_root: &Path) {
    fs::create_dir_all(codex_home.join("sessions")).expect("create sessions");
    fs::write(
        codex_home.join("sessions").join("thread-1.jsonl"),
        format!(
            "{{\"timestamp\":\"2026-04-05T02:39:26.735Z\",\"type\":\"session_meta\",\"payload\":{{\"id\":\"thread-1\",\"timestamp\":\"2026-04-05T02:39:25.341Z\",\"cwd\":\"{}\",\"cli_version\":\"0.1.0\",\"source\":\"cli\",\"model_provider\":\"openai\"}}}}\n",
            workspace_root.display()
        ),
    )
    .expect("write rollout");

    let connection = Connection::open(codex_home.join("state_5.sqlite")).expect("sqlite db");
    connection
        .execute_batch(
            "CREATE TABLE threads (
                id TEXT PRIMARY KEY,
                rollout_path TEXT,
                cwd TEXT,
                model_provider TEXT,
                created_at INTEGER,
                updated_at INTEGER,
                source TEXT,
                archived INTEGER NOT NULL DEFAULT 0,
                cli_version TEXT,
                title TEXT,
                first_user_message TEXT
            );",
        )
        .expect("schema");
    connection
        .execute(
            "INSERT INTO threads (id, rollout_path, cwd, model_provider, created_at, updated_at, source, archived, cli_version, title, first_user_message)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                "thread-1",
                "sessions/thread-1.jsonl",
                workspace_root.display().to_string(),
                "openai",
                1000_i64,
                1001_i64,
                "vscode",
                0_i64,
                "0.1.0",
                "Renamed thread",
                "hello cockpit"
            ],
        )
        .expect("insert");
}

fn spawn_cockpit(
    workspace_root: &Path,
    codex_home: &Path,
    log_path: &Path,
    extra_path_entry: Option<&Path>,
    extra_env: &[(&str, &Path)],
) -> Child {
    let stdout = File::create(log_path).expect("create log");
    let stderr = stdout.try_clone().expect("clone log");
    let mut path_segments = Vec::new();
    if let Some(extra) = extra_path_entry {
        path_segments.push(extra.display().to_string());
    }
    if let Some(existing) = std::env::var_os("PATH") {
        path_segments.push(existing.to_string_lossy().into_owned());
    }

    let mut command = Command::new(cockpit_binary());
    command
        .arg("ui")
        .arg("cockpit")
        .arg("--workspace-root")
        .arg(workspace_root)
        .arg("--codex-home")
        .arg(codex_home)
        .arg("--open-browser")
        .arg("false")
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr));
    if !path_segments.is_empty() {
        command.env("PATH", path_segments.join(":"));
    }
    for (key, value) in extra_env {
        command.env(key, value);
    }

    command.spawn().expect("spawn cockpit")
}

struct ChildGuard(Child);

impl Drop for ChildGuard {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn wait_for_url(log_path: &Path) -> Result<String> {
    for _ in 0..40 {
        sleep(Duration::from_millis(250));
        let text = fs::read_to_string(log_path).unwrap_or_default();
        if let Some(line) = text.lines().find(|line| line.contains("URL"))
            && let Some((_, url)) = line.split_once(':')
        {
            return Ok(url.trim().to_string());
        }
    }
    bail!(
        "timed out waiting for cockpit url in `{}`",
        log_path.display()
    )
}

fn get_json(url: &str) -> Value {
    let output = Command::new("curl")
        .arg("-sS")
        .arg(url)
        .output()
        .expect("curl discovery should run");
    assert!(
        output.status.success(),
        "curl discovery failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("json body")
}

fn post_json(url: &str, payload: &Value) -> Value {
    let output = Command::new("curl")
        .arg("-sS")
        .arg("-X")
        .arg("POST")
        .arg(url)
        .arg("-H")
        .arg("Content-Type: application/json")
        .arg("-d")
        .arg(payload.to_string())
        .output()
        .expect("curl export should run");
    assert!(
        output.status.success(),
        "curl export failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("json body")
}

fn wait_for_job(url: &str, job_id: &str) -> Value {
    for _ in 0..80 {
        sleep(Duration::from_millis(250));
        let body = get_json(&format!("{url}/api/export/jobs/{job_id}"));
        let status = body["status"].as_str().unwrap_or_default();
        if matches!(status, "completed" | "completed_with_warnings" | "failed") {
            return body;
        }
    }
    panic!("timed out waiting for export job {job_id}");
}

fn write_fake_codex(binary_path: &Path) {
    fs::write(
        binary_path,
        r##"#!/usr/bin/env python3
import os
import sys
import time

args_log = os.environ["FAKE_CODEX_ARGS_LOG"]
stdin_log = os.environ["FAKE_CODEX_STDIN_LOG"]

with open(args_log, "w", encoding="utf-8") as handle:
    handle.write("\n".join(sys.argv[1:]))

fd = sys.stdin.fileno()
os.set_blocking(fd, False)
time.sleep(0.2)
chunks = []
while True:
    try:
        chunk = os.read(fd, 65536)
    except BlockingIOError:
        break
    if not chunk:
        break
    chunks.append(chunk)
    time.sleep(0.05)

with open(stdin_log, "wb") as handle:
    handle.write(b"".join(chunks))

output_path = ""
previous = None
for arg in sys.argv[1:]:
    if previous in ("-o", "--output-last-message"):
        output_path = arg
        break
    previous = arg

if not output_path:
    print("missing output path", file=sys.stderr)
    sys.exit(1)

os.makedirs(os.path.dirname(output_path), exist_ok=True)
with open(output_path, "w", encoding="utf-8") as handle:
    handle.write("# AI 梳理\n\nfake summary\n")
"##,
    )
    .expect("write fake codex");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let mut permissions = fs::metadata(binary_path)
            .expect("fake codex metadata")
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(binary_path, permissions).expect("chmod fake codex");
    }
}

fn seed_workspace_claude_session(workspace_root: &Path) -> PathBuf {
    let session_path = workspace_root
        .join(".agents")
        .join("fixtures")
        .join("claude")
        .join("session-demo.jsonl");
    fs::create_dir_all(session_path.parent().expect("claude session parent"))
        .expect("create claude session dir");
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("claude_session_minimal.jsonl");
    fs::copy(fixture, &session_path).expect("copy claude fixture");
    session_path
}

#[test]
fn cockpit_discovery_and_export_work_end_to_end() -> Result<()> {
    let workspace = tempdir().context("workspace tempdir")?;
    let codex_home = tempdir().context("codex tempdir")?;
    seed_codex_state(codex_home.path(), workspace.path());

    let log_path = workspace.path().join("cockpit.log");
    let _child = ChildGuard(spawn_cockpit(
        workspace.path(),
        codex_home.path(),
        &log_path,
        None,
        &[],
    ));
    let url = wait_for_url(&log_path)?.trim_end_matches('/').to_string();

    let discovery = get_json(&format!("{url}/api/discovery"));
    assert_eq!(
        discovery["discoveryMode"],
        "persisted-codex-state-vscode-fallback"
    );
    assert_eq!(
        discovery["threads"]
            .as_array()
            .expect("threads array")
            .len(),
        1
    );
    assert_eq!(discovery["threads"][0]["threadId"], "thread-1");
    assert_eq!(discovery["threads"][0]["displayName"], "Renamed thread");

    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("mock_codex_app_server.py");
    let export = post_json(
        &format!("{url}/api/export"),
        &serde_json::json!({
            "selections": [{
                "threadId": "thread-1",
                "workspacePath": workspace.path().display().to_string(),
                "workspaceLabel": "workspace"
            }],
            "appServerCommand": std::env::var("PYTHON").unwrap_or_else(|_| "python3".to_string()),
            "appServerArgs": [fixture.display().to_string()],
        }),
    );

    assert_eq!(export["status"], "started");
    let job = wait_for_job(&url, export["jobId"].as_str().expect("job id"));
    let archive_shell = workspace
        .path()
        .join(".agents")
        .join("Conversations")
        .join("index.html");
    let reports_shell = workspace
        .path()
        .join(".agents")
        .join("Search")
        .join("Reports")
        .join("index.html");
    let evidence_shell = workspace
        .path()
        .join(".agents")
        .join("Integration")
        .join("Reports")
        .join("index.html");
    let archive_shell_str = archive_shell.display().to_string();

    assert!(archive_shell.exists(), "archive shell should exist");
    assert!(reports_shell.exists(), "reports shell should exist");
    assert!(
        evidence_shell.exists(),
        "integration evidence shell should exist"
    );
    assert_eq!(
        job["workspaces"][0]["archiveShellPath"].as_str(),
        Some(archive_shell_str.as_str())
    );
    let transcript_paths = job["workspaces"][0]["threads"][0]["transcriptPaths"]
        .as_array()
        .expect("transcript paths array");
    let thread_display_name = job["workspaces"][0]["threads"][0]["displayName"]
        .as_str()
        .expect("thread display name");
    assert!(
        transcript_paths[0]
            .as_str()
            .is_some_and(|path| path.ends_with(".md")),
        "markdown transcript should be listed first"
    );
    let copy_bundle = job["workspaces"][0]["copyBundleText"]
        .as_str()
        .expect("copy bundle text");
    let transcript_lines = transcript_paths
        .iter()
        .map(|value| value.as_str().expect("transcript path"))
        .collect::<Vec<_>>()
        .join("\n");
    let expected_copy_bundle = format!(
        "# 工作区: workspace\n{}\n\n# 会话: {}\n{}\n\n# 当前Repo全局工作台 Index：\n{}\n{}\n{}",
        workspace.path().display(),
        thread_display_name,
        transcript_lines,
        archive_shell.display(),
        reports_shell.display(),
        evidence_shell.display(),
    );
    assert_eq!(copy_bundle, expected_copy_bundle);

    Ok(())
}

#[test]
fn cockpit_ai_summary_controls_reach_backend_summary_command() -> Result<()> {
    let workspace = tempdir().context("workspace tempdir")?;
    let codex_home = tempdir().context("codex tempdir")?;
    seed_codex_state(codex_home.path(), workspace.path());

    let bin_dir = workspace.path().join("bin");
    fs::create_dir_all(&bin_dir).context("create bin dir")?;
    write_fake_codex(&bin_dir.join("codex"));
    let fake_args_log = workspace.path().join("fake-codex-args.log");
    let fake_stdin_log = workspace.path().join("fake-codex-stdin.log");

    let log_path = workspace.path().join("cockpit-ai-summary.log");
    let _child = ChildGuard(spawn_cockpit(
        workspace.path(),
        codex_home.path(),
        &log_path,
        Some(&bin_dir),
        &[
            ("FAKE_CODEX_ARGS_LOG", fake_args_log.as_path()),
            ("FAKE_CODEX_STDIN_LOG", fake_stdin_log.as_path()),
        ],
    ));
    let url = wait_for_url(&log_path)?.trim_end_matches('/').to_string();

    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("mock_codex_app_server.py");
    let export = post_json(
        &format!("{url}/api/export"),
        &serde_json::json!({
            "selections": [{
                "threadId": "thread-1",
                "workspacePath": workspace.path().display().to_string(),
                "workspaceLabel": "workspace"
            }],
            "appServerCommand": std::env::var("PYTHON").unwrap_or_else(|_| "python3".to_string()),
            "appServerArgs": [fixture.display().to_string()],
            "aiSummary": true,
            "aiSummaryInstructions": "请特别关注 blocker。",
            "aiSummaryProfile": "workbench",
            "aiSummaryModel": "gpt-5",
            "aiSummaryProvider": "openai"
        }),
    );

    assert_eq!(export["status"], "started");
    let job = wait_for_job(&url, export["jobId"].as_str().expect("job id"));
    assert_eq!(job["status"], "completed");

    let args = fs::read_to_string(&fake_args_log).context("read fake codex args")?;
    assert!(args.contains("--profile"));
    assert!(args.contains("workbench"));
    assert!(args.contains("--model"));
    assert!(args.contains("gpt-5"));
    assert!(args.contains("model_provider=\"openai\""));

    let prompt = fs::read_to_string(&fake_stdin_log).context("read fake codex prompt")?;
    assert!(prompt.contains("请特别关注 blocker。"));

    Ok(())
}

#[test]
fn cockpit_discovery_and_export_support_workspace_local_claude_sessions() -> Result<()> {
    let workspace = tempdir().context("workspace tempdir")?;
    let codex_home = tempdir().context("codex tempdir")?;
    seed_codex_state(codex_home.path(), workspace.path());
    let claude_session_path = seed_workspace_claude_session(workspace.path());

    let log_path = workspace.path().join("cockpit-claude.log");
    let _child = ChildGuard(spawn_cockpit(
        workspace.path(),
        codex_home.path(),
        &log_path,
        None,
        &[],
    ));
    let url = wait_for_url(&log_path)?.trim_end_matches('/').to_string();

    let discovery = get_json(&format!("{url}/api/discovery"));
    let threads = discovery["threads"].as_array().expect("threads array");
    let claude_entry = threads
        .iter()
        .find(|entry| entry["connectorKind"] == "claude-code")
        .expect("claude discovery entry");
    let canonical_claude_session_path = claude_session_path
        .canonicalize()
        .expect("canonical claude session path");
    assert_eq!(claude_entry["sourceKind"], "claude-session-path");
    assert_eq!(
        claude_entry["sessionPath"].as_str(),
        Some(canonical_claude_session_path.display().to_string().as_str())
    );

    let export = post_json(
        &format!("{url}/api/export"),
        &serde_json::json!({
            "selections": [{
                "connectorKind": "claude-code",
                "threadId": claude_entry["threadId"].as_str().expect("thread id"),
                "workspacePath": workspace.path().display().to_string(),
                "workspaceLabel": "workspace",
                "sessionPath": claude_session_path.display().to_string()
            }]
        }),
    );

    assert_eq!(export["status"], "started");
    let job = wait_for_job(&url, export["jobId"].as_str().expect("job id"));
    assert_eq!(job["status"], "completed");

    let transcript_paths = job["workspaces"][0]["threads"][0]["transcriptPaths"]
        .as_array()
        .expect("transcript paths array");
    assert!(
        transcript_paths
            .iter()
            .any(|value| value.as_str().is_some_and(|path| path.ends_with(".html"))),
        "claude cockpit export should produce html transcript output"
    );
    assert!(
        workspace
            .path()
            .join(".agents")
            .join("Conversations")
            .join("index.html")
            .exists(),
        "cockpit should publish archive shell after claude export"
    );

    Ok(())
}
