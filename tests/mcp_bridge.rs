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
