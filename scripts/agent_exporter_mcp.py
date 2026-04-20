#!/usr/bin/env python3
import json
import os
import shlex
import subprocess
import sys
from pathlib import Path
from typing import Any

PROTOCOL_VERSION = "2025-03-26"
REPO_ROOT = Path(__file__).resolve().parent.parent


def read_message() -> dict[str, Any] | None:
    content_length = None
    while True:
        line = sys.stdin.buffer.readline()
        if not line:
            return None
        if line in (b"\r\n", b"\n"):
            break
        if line.lower().startswith(b"content-length:"):
            content_length = int(line.split(b":", 1)[1].strip())
    if content_length is None:
        return None
    body = sys.stdin.buffer.read(content_length)
    if not body:
        return None
    return json.loads(body.decode("utf-8"))


def send_message(message: dict[str, Any]) -> None:
    body = json.dumps(message, ensure_ascii=False).encode("utf-8")
    sys.stdout.buffer.write(f"Content-Length: {len(body)}\r\n\r\n".encode("ascii"))
    sys.stdout.buffer.write(body)
    sys.stdout.buffer.flush()


def success_text(text: str) -> dict[str, Any]:
    return {"content": [{"type": "text", "text": text}], "isError": False}


def error_text(text: str) -> dict[str, Any]:
    return {"content": [{"type": "text", "text": text}], "isError": True}


def tool_specs() -> list[dict[str, Any]]:
    common_args = {
        "workspace_root": {"type": "string", "description": "Workspace root path"},
        "query": {"type": "string", "description": "Search query"},
        "top_k": {"type": "integer", "description": "Maximum hits to return", "default": 5},
        "model_dir": {"type": "string", "description": "Optional local model directory"},
        "save_report": {"type": "boolean", "description": "Save a retrieval report", "default": False},
    }
    return [
        {
            "name": "publish_archive_index",
            "description": "Generate the local archive shell and reports shell for a workspace",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "workspace_root": {"type": "string", "description": "Workspace root path"}
                },
                "required": ["workspace_root"],
                "additionalProperties": False,
            },
        },
        {
            "name": "search_semantic",
            "description": "Run semantic retrieval over local archive transcripts",
            "inputSchema": {
                "type": "object",
                "properties": common_args,
                "required": ["workspace_root", "query"],
                "additionalProperties": False,
            },
        },
        {
            "name": "search_hybrid",
            "description": "Run hybrid retrieval over local archive transcripts",
            "inputSchema": {
                "type": "object",
                "properties": common_args,
                "required": ["workspace_root", "query"],
                "additionalProperties": False,
            },
        },
        {
            "name": "integration_evidence_diff",
            "description": "Diff two saved integration evidence snapshots",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "left": {"type": "string", "description": "Left report path"},
                    "right": {"type": "string", "description": "Right report path"},
                },
                "required": ["left", "right"],
                "additionalProperties": False,
            },
        },
        {
            "name": "integration_evidence_gate",
            "description": "Gate a candidate integration evidence snapshot against a baseline snapshot",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "baseline": {"type": "string", "description": "Baseline report path"},
                    "candidate": {"type": "string", "description": "Candidate report path"},
                    "policy": {"type": "string", "description": "Optional local JSON policy path"},
                    "workspace_root": {"type": "string", "description": "Optional workspace root when baseline is a registered name"},
                },
                "required": ["baseline", "candidate"],
                "additionalProperties": False,
            },
        },
        {
            "name": "integration_evidence_explain",
            "description": "Explain the ordered remediation sequence for a saved integration evidence report",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "report": {"type": "string", "description": "Saved report path"},
                },
                "required": ["report"],
                "additionalProperties": False,
            },
        },
        {
            "name": "integration_evidence_remediation",
            "description": "Read the remediation bundle for a saved integration evidence report",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "report": {"type": "string", "description": "Saved report path"},
                },
                "required": ["report"],
                "additionalProperties": False,
            },
        },
        {
            "name": "integration_evidence_baseline_list",
            "description": "Read the saved baseline registry from a workspace",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "workspace_root": {"type": "string", "description": "Workspace root path"},
                },
                "required": ["workspace_root"],
                "additionalProperties": False,
            },
        },
        {
            "name": "integration_evidence_baseline_show",
            "description": "Read one saved baseline entry by name from a workspace",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "workspace_root": {"type": "string", "description": "Workspace root path"},
                    "name": {"type": "string", "description": "Registered baseline name"},
                },
                "required": ["workspace_root", "name"],
                "additionalProperties": False,
            },
        },
        {
            "name": "integration_evidence_policy_list",
            "description": "Read the repo-owned governance policy pack list",
            "inputSchema": {
                "type": "object",
                "properties": {},
                "additionalProperties": False,
            },
        },
        {
            "name": "integration_evidence_policy_show",
            "description": "Read one repo-owned governance policy pack by name",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "name": {"type": "string", "description": "Policy pack name"},
                },
                "required": ["name"],
                "additionalProperties": False,
            },
        },
        {
            "name": "integration_evidence_decision_history",
            "description": "Read saved decision history entries from a workspace",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "workspace_root": {"type": "string", "description": "Workspace root path"},
                    "baseline_name": {"type": "string", "description": "Optional baseline filter"},
                    "limit": {"type": "integer", "description": "Maximum history entries to return", "default": 10},
                },
                "required": ["workspace_root"],
                "additionalProperties": False,
            },
        },
        {
            "name": "integration_evidence_current_decision",
            "description": "Automatically summarize the current decision for one official baseline",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "workspace_root": {"type": "string", "description": "Workspace root path"},
                    "baseline_name": {"type": "string", "description": "Baseline registry name"},
                },
                "required": ["workspace_root", "baseline_name"],
                "additionalProperties": False,
            },
        },
    ]


def default_base_command() -> list[str]:
    for candidate in (
        REPO_ROOT / "target" / "release" / "agent-exporter",
        REPO_ROOT / "target" / "debug" / "agent-exporter",
    ):
        if candidate.is_file() and os.access(candidate, os.X_OK):
            return [str(candidate)]
    return [
        "cargo",
        "run",
        "--quiet",
        "--manifest-path",
        str(REPO_ROOT / "Cargo.toml"),
        "--bin",
        "agent-exporter",
        "--",
    ]


def base_command() -> list[str]:
    command = os.environ.get("AGENT_EXPORTER_BIN")
    extra_args = os.environ.get("AGENT_EXPORTER_ARGS", "")
    parts = [command] if command is not None else default_base_command()
    if extra_args.strip():
        parts.extend(shlex.split(extra_args))
    return parts


def run_cli(args: list[str], cwd: str | None = None) -> dict[str, Any]:
    completed = subprocess.run(
        base_command() + args,
        text=True,
        capture_output=True,
        check=False,
        cwd=cwd,
    )
    output = (completed.stdout or "") + (("\n" + completed.stderr.strip()) if completed.stderr.strip() else "")
    if completed.returncode == 0:
        return success_text(output.strip())
    return error_text(output.strip() or f"command failed with exit code {completed.returncode}")


def resolve_integration_reports_dir(workspace_root: str) -> Path:
    return Path(workspace_root) / ".agents" / "Integration" / "Reports"


def resolve_baseline_registry_path(workspace_root: str) -> Path:
    return resolve_integration_reports_dir(workspace_root) / "baseline-registry.json"


def resolve_decision_history_path(workspace_root: str) -> Path:
    return resolve_integration_reports_dir(workspace_root) / "decision-history.json"


def resolve_policy_dir() -> Path:
    return REPO_ROOT / "policies" / "integration-evidence"


def read_json_file(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def handle_tool_call(name: str, arguments: dict[str, Any]) -> dict[str, Any]:
    if name == "publish_archive_index":
        return run_cli(["publish", "archive-index", "--workspace-root", arguments["workspace_root"]])

    if name in {"search_semantic", "search_hybrid"}:
        command = ["search", "semantic" if name == "search_semantic" else "hybrid"]
        command.extend(["--workspace-root", arguments["workspace_root"]])
        command.extend(["--query", arguments["query"]])
        if "top_k" in arguments and arguments["top_k"] is not None:
            command.extend(["--top-k", str(arguments["top_k"])])
        if arguments.get("model_dir"):
            command.extend(["--model-dir", arguments["model_dir"]])
        if arguments.get("save_report"):
            command.append("--save-report")
        return run_cli(command)

    if name == "integration_evidence_diff":
        return run_cli(
            [
                "evidence",
                "diff",
                "--left",
                arguments["left"],
                "--right",
                arguments["right"],
            ]
        )

    if name == "integration_evidence_gate":
        command = [
            "evidence",
            "gate",
            "--baseline",
            arguments["baseline"],
            "--candidate",
            arguments["candidate"],
        ]
        if arguments.get("policy"):
            command.extend(["--policy", arguments["policy"]])
        return run_cli(command, cwd=arguments.get("workspace_root"))

    if name == "integration_evidence_explain":
        return run_cli(["evidence", "explain", "--report", arguments["report"]])

    if name == "integration_evidence_remediation":
        return run_cli(["evidence", "remediation", "--report", arguments["report"]])

    if name == "integration_evidence_baseline_list":
        registry_path = resolve_baseline_registry_path(arguments["workspace_root"])
        if not registry_path.is_file():
            return error_text(f"missing baseline registry: {registry_path}")
        registry = read_json_file(registry_path)
        payload = {
            "generated_at": registry.get("generated_at"),
            "baseline_count": len(registry.get("baselines", [])),
            "baselines": registry.get("baselines", []),
        }
        return success_text(json.dumps(payload, ensure_ascii=False, indent=2))

    if name == "integration_evidence_baseline_show":
        registry_path = resolve_baseline_registry_path(arguments["workspace_root"])
        if not registry_path.is_file():
            return error_text(f"missing baseline registry: {registry_path}")
        registry = read_json_file(registry_path)
        name_filter = arguments["name"]
        baseline = next(
            (
                entry
                for entry in registry.get("baselines", [])
                if entry.get("name") == name_filter
            ),
            None,
        )
        if baseline is None:
            return error_text(f"baseline not found: {name_filter}")
        return success_text(json.dumps(baseline, ensure_ascii=False, indent=2))

    if name == "integration_evidence_policy_list":
        policy_dir = resolve_policy_dir()
        policies = []
        if policy_dir.is_dir():
            for path in sorted(policy_dir.glob("*.json")):
                document = read_json_file(path)
                policies.append(
                    {
                        "name": document.get("name"),
                        "version": document.get("version"),
                        "path": str(path),
                    }
                )
        payload = {"policy_count": len(policies), "policies": policies}
        return success_text(json.dumps(payload, ensure_ascii=False, indent=2))

    if name == "integration_evidence_policy_show":
        policy_path = resolve_policy_dir() / f"{arguments['name']}.json"
        if not policy_path.is_file():
            return error_text(f"missing policy pack: {policy_path}")
        return success_text(
            json.dumps(read_json_file(policy_path), ensure_ascii=False, indent=2)
        )

    if name == "integration_evidence_decision_history":
        history_path = resolve_decision_history_path(arguments["workspace_root"])
        if not history_path.is_file():
            return error_text(f"missing decision history: {history_path}")
        history = read_json_file(history_path)
        entries = history.get("decisions", [])
        baseline_name = arguments.get("baseline_name")
        if baseline_name:
            entries = [
                entry
                for entry in entries
                if entry.get("baseline_name") == baseline_name
            ]
        limit = int(arguments.get("limit", 10))
        payload = {
            "generated_at": history.get("generated_at"),
            "decision_count": len(entries),
            "decisions": entries[:limit],
        }
        return success_text(json.dumps(payload, ensure_ascii=False, indent=2))

    if name == "integration_evidence_current_decision":
        return run_cli(
            [
                "evidence",
                "current",
                "--baseline-name",
                arguments["baseline_name"],
            ],
            cwd=arguments["workspace_root"],
        )

    return error_text(f"unknown tool: {name}")


def handle_request(message: dict[str, Any]) -> dict[str, Any] | None:
    method = message.get("method")
    request_id = message.get("id")

    if request_id is None:
        return None

    if method == "initialize":
        return {
            "jsonrpc": "2.0",
            "id": request_id,
            "result": {
                "protocolVersion": PROTOCOL_VERSION,
                "capabilities": {"tools": {"listChanged": False}},
                "serverInfo": {"name": "agent-exporter-mcp", "version": "0.1.3"},
            },
        }

    if method == "ping":
        return {"jsonrpc": "2.0", "id": request_id, "result": {}}

    if method == "tools/list":
        return {"jsonrpc": "2.0", "id": request_id, "result": {"tools": tool_specs()}}

    if method == "tools/call":
        params = message.get("params", {})
        result = handle_tool_call(params.get("name", ""), params.get("arguments", {}))
        return {"jsonrpc": "2.0", "id": request_id, "result": result}

    return {
        "jsonrpc": "2.0",
        "id": request_id,
        "error": {"code": -32601, "message": f"Method not found: {method}"},
    }


def main() -> int:
    while True:
        message = read_message()
        if message is None:
            return 0
        response = handle_request(message)
        if response is not None:
            send_message(response)


if __name__ == "__main__":
    raise SystemExit(main())
