use std::io::{BufRead, BufReader, Read, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

use serde_json::{Value, json};
use tempfile::tempdir;

fn python_command() -> String {
    std::env::var("PYTHON").unwrap_or_else(|_| "python3".to_string())
}

fn mcp_script_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("scripts")
        .join("agent_exporter_mcp.py")
}

fn agent_exporter_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_agent-exporter"))
}

fn repo_local_debug_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("debug")
        .join("agent-exporter")
}

fn write_message(stdin: &mut impl Write, value: &Value) {
    let body = serde_json::to_vec(value).expect("json body");
    write!(stdin, "Content-Length: {}\r\n\r\n", body.len()).expect("write header");
    stdin.write_all(&body).expect("write body");
    stdin.flush().expect("flush");
}

fn read_message(stdout: &mut BufReader<impl Read>) -> Value {
    let mut content_length = None;
    loop {
        let mut line = String::new();
        stdout.read_line(&mut line).expect("read header");
        if line.is_empty() {
            panic!("unexpected EOF while reading MCP headers");
        }
        if line == "\r\n" || line == "\n" {
            break;
        }
        if line.to_ascii_lowercase().starts_with("content-length:") {
            content_length = Some(
                line.split_once(':')
                    .expect("content-length split")
                    .1
                    .trim()
                    .parse::<usize>()
                    .expect("content-length number"),
            );
        }
    }

    let content_length = content_length.expect("content-length header");
    let mut body = vec![0u8; content_length];
    stdout.read_exact(&mut body).expect("read body");
    serde_json::from_slice(&body).expect("parse json")
}

fn report_readiness(path: &PathBuf) -> String {
    let document: Value =
        serde_json::from_str(&std::fs::read_to_string(path).expect("report json should exist"))
            .expect("valid report json");
    document["readiness"]
        .as_str()
        .expect("top-level readiness")
        .to_string()
}

#[test]
fn mcp_bridge_lists_tools_and_can_publish_archive_index() {
    let workspace = tempdir().expect("workspace");
    let mut child = Command::new(python_command())
        .arg(mcp_script_path())
        .env("AGENT_EXPORTER_BIN", agent_exporter_bin())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("spawn mcp bridge");

    let mut stdin = child.stdin.take().expect("stdin");
    let mut stdout = BufReader::new(child.stdout.take().expect("stdout"));

    write_message(
        &mut stdin,
        &json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2025-03-26",
                "capabilities": {},
                "clientInfo": {"name": "test", "version": "0.1.0"}
            }
        }),
    );
    let initialize = read_message(&mut stdout);
    assert_eq!(
        initialize["result"]["serverInfo"]["name"],
        "agent-exporter-mcp"
    );

    write_message(
        &mut stdin,
        &json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/list",
            "params": {}
        }),
    );
    let tools = read_message(&mut stdout);
    let tool_names = tools["result"]["tools"]
        .as_array()
        .expect("tools array")
        .iter()
        .filter_map(|tool| tool["name"].as_str())
        .collect::<Vec<_>>();
    assert!(tool_names.contains(&"publish_archive_index"));
    assert!(tool_names.contains(&"search_semantic"));
    assert!(tool_names.contains(&"search_hybrid"));
    assert!(tool_names.contains(&"integration_evidence_list"));
    assert!(tool_names.contains(&"integration_evidence_diff"));
    assert!(tool_names.contains(&"integration_evidence_gate"));
    assert!(tool_names.contains(&"integration_evidence_explain"));

    write_message(
        &mut stdin,
        &json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": {
                "name": "publish_archive_index",
                "arguments": {
                    "workspace_root": workspace.path().display().to_string()
                }
            }
        }),
    );
    let publish = read_message(&mut stdout);
    let text = publish["result"]["content"][0]["text"]
        .as_str()
        .expect("text result");
    assert!(text.contains("Archive index published"));
    assert!(
        workspace
            .path()
            .join(".agents")
            .join("Conversations")
            .join("index.html")
            .exists()
    );

    drop(stdin);
    let status = child.wait().expect("wait child");
    assert!(status.success());
}

#[test]
fn mcp_bridge_reads_integration_evidence_tools() {
    let workspace = tempdir().expect("workspace");
    let target = workspace.path().join("codex-pack");

    let onboard = Command::new(agent_exporter_bin())
        .current_dir(workspace.path())
        .args([
            "onboard",
            "codex",
            "--target",
            target.display().to_string().as_str(),
            "--save-report",
        ])
        .output()
        .expect("run onboard");
    assert!(onboard.status.success(), "{onboard:?}");

    std::fs::write(
        target.join(".codex").join("config.toml"),
        "[mcp_servers.agent_exporter]\ncommand = \"python3\"\n",
    )
    .expect("break codex config");

    let doctor = Command::new(agent_exporter_bin())
        .current_dir(workspace.path())
        .args([
            "doctor",
            "integrations",
            "--platform",
            "codex",
            "--target",
            target.display().to_string().as_str(),
            "--save-report",
        ])
        .output()
        .expect("run doctor");
    assert!(doctor.status.success(), "{doctor:?}");

    let reports_root = workspace
        .path()
        .join(".agents")
        .join("Integration")
        .join("Reports");
    let mut report_jsons = std::fs::read_dir(&reports_root)
        .expect("read reports")
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| ext == "json")
        })
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name != "index.json")
        })
        .collect::<Vec<_>>();
    report_jsons.sort();
    assert_eq!(report_jsons.len(), 2);
    let (baseline, candidate) = if report_readiness(&report_jsons[0]) == "ready" {
        (&report_jsons[0], &report_jsons[1])
    } else {
        (&report_jsons[1], &report_jsons[0])
    };

    let mut child = Command::new(python_command())
        .arg(mcp_script_path())
        .env("AGENT_EXPORTER_BIN", agent_exporter_bin())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("spawn mcp bridge");

    let mut stdin = child.stdin.take().expect("stdin");
    let mut stdout = BufReader::new(child.stdout.take().expect("stdout"));

    write_message(
        &mut stdin,
        &json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2025-03-26",
                "capabilities": {},
                "clientInfo": {"name": "test", "version": "0.1.0"}
            }
        }),
    );
    let _ = read_message(&mut stdout);

    write_message(
        &mut stdin,
        &json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": {
                "name": "integration_evidence_list",
                "arguments": {
                    "workspace_root": workspace.path().display().to_string()
                }
            }
        }),
    );
    let list = read_message(&mut stdout);
    let list_text = list["result"]["content"][0]["text"]
        .as_str()
        .expect("list result text");
    assert!(list_text.contains("\"report_count\": 2"));

    write_message(
        &mut stdin,
        &json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": {
                "name": "integration_evidence_explain",
                "arguments": {
                    "report": candidate.display().to_string()
                }
            }
        }),
    );
    let explain = read_message(&mut stdout);
    let explain_text = explain["result"]["content"][0]["text"]
        .as_str()
        .expect("explain result text");
    assert!(explain_text.contains("Integration evidence explain"));

    write_message(
        &mut stdin,
        &json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": "tools/call",
            "params": {
                "name": "integration_evidence_diff",
                "arguments": {
                    "left": baseline.display().to_string(),
                    "right": candidate.display().to_string()
                }
            }
        }),
    );
    let diff = read_message(&mut stdout);
    let diff_text = diff["result"]["content"][0]["text"]
        .as_str()
        .expect("diff result text");
    assert!(diff_text.contains("Integration evidence diff"));

    write_message(
        &mut stdin,
        &json!({
            "jsonrpc": "2.0",
            "id": 5,
            "method": "tools/call",
            "params": {
                "name": "integration_evidence_gate",
                "arguments": {
                    "baseline": baseline.display().to_string(),
                    "candidate": candidate.display().to_string()
                }
            }
        }),
    );
    let gate = read_message(&mut stdout);
    let gate_text = gate["result"]["content"][0]["text"]
        .as_str()
        .expect("gate result text");
    assert!(gate_text.contains("Integration evidence gate"));
    assert!(gate_text.contains("Verdict      : fail"));

    drop(stdin);
    let status = child.wait().expect("wait child");
    assert!(status.success());
}

#[test]
fn mcp_bridge_uses_repo_local_default_launcher_without_explicit_bin_override() {
    let workspace = tempdir().expect("workspace");
    assert!(
        repo_local_debug_bin().exists(),
        "expected repo-local debug binary to exist for default launcher"
    );

    let mut child = Command::new(python_command())
        .arg(mcp_script_path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("spawn mcp bridge");

    let mut stdin = child.stdin.take().expect("stdin");
    let mut stdout = BufReader::new(child.stdout.take().expect("stdout"));

    write_message(
        &mut stdin,
        &json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2025-03-26",
                "capabilities": {},
                "clientInfo": {"name": "test", "version": "0.1.0"}
            }
        }),
    );
    let initialize = read_message(&mut stdout);
    assert_eq!(
        initialize["result"]["serverInfo"]["name"],
        "agent-exporter-mcp"
    );

    write_message(
        &mut stdin,
        &json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": {
                "name": "publish_archive_index",
                "arguments": {
                    "workspace_root": workspace.path().display().to_string()
                }
            }
        }),
    );
    let publish = read_message(&mut stdout);
    let text = publish["result"]["content"][0]["text"]
        .as_str()
        .expect("text result");
    assert!(text.contains("Archive index published"));
    assert!(
        workspace
            .path()
            .join(".agents")
            .join("Conversations")
            .join("index.html")
            .exists()
    );

    drop(stdin);
    let status = child.wait().expect("wait child");
    assert!(status.success());
}
