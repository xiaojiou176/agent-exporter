#!/usr/bin/env python3

import json
import sys


def send(payload):
    sys.stdout.write(json.dumps(payload) + "\n")
    sys.stdout.flush()


def complete_thread(thread_id):
    return {
        "thread": {
            "id": thread_id,
            "preview": "Canonical preview from thread/read",
            "ephemeral": False,
            "modelProvider": "openai",
            "createdAt": 1712198400,
            "updatedAt": 1712198460,
            "status": {"type": "notLoaded"},
            "path": "/tmp/mock-rollout.jsonl",
            "cwd": "/tmp/mock-workspace",
            "cliVersion": "0.1.0",
            "source": "cli",
            "name": "Mock Complete Thread",
            "turns": [
                {
                    "id": "turn-1",
                    "status": "completed",
                    "items": [
                        {
                            "type": "userMessage",
                            "id": "user-1",
                            "content": [{"type": "text", "text": "hello from user"}],
                        },
                        {
                            "type": "agentMessage",
                            "id": "assistant-1",
                            "text": "hello from assistant",
                        },
                        {
                            "type": "commandExecution",
                            "id": "command-1",
                            "command": "pwd",
                            "cwd": "/tmp/mock-workspace",
                            "status": "completed",
                            "aggregatedOutput": "/tmp/mock-workspace\n",
                            "exitCode": 0,
                        },
                    ],
                }
            ],
        }
    }


def fallback_thread(thread_id):
    return {
        "thread": {
            "id": thread_id,
            "preview": "Preview recovered through resume fallback",
            "ephemeral": True,
            "modelProvider": "openai",
            "createdAt": 1712198400,
            "updatedAt": 1712198460,
            "status": {"type": "notLoaded"},
            "path": None,
            "cwd": "/tmp/mock-workspace",
            "cliVersion": "0.1.0",
            "source": "cli",
            "name": "Mock Fallback Thread",
            "turns": [
                {
                    "id": "turn-1",
                    "status": "completed",
                    "items": [
                        {
                            "type": "agentMessage",
                            "id": "assistant-1",
                            "text": "Recovered from resume fallback",
                        }
                    ],
                }
            ],
        }
    }


for raw_line in sys.stdin:
    line = raw_line.strip()
    if not line:
        continue

    message = json.loads(line)
    method = message.get("method")

    if method == "initialize":
        send(
            {
                "id": message["id"],
                "result": {
                    "userAgent": "mock-agent-exporter",
                    "codexHome": "/tmp/mock-codex-home",
                    "platformFamily": "unix",
                    "platformOs": "darwin",
                },
            }
        )
        continue

    if method == "initialized":
        continue

    if method == "thread/read":
        thread_id = message["params"]["threadId"]
        if thread_id == "fallback-thread":
            send(
                {
                    "id": message["id"],
                    "error": {
                        "code": -32000,
                        "message": "ephemeral threads do not support includeTurns",
                    },
                }
            )
        else:
            send({"id": message["id"], "result": complete_thread(thread_id)})
        continue

    if method == "thread/resume":
        thread_id = message["params"]["threadId"]
        if thread_id == "fallback-thread":
            send({"id": message["id"], "result": fallback_thread(thread_id)})
        else:
            send(
                {
                    "id": message["id"],
                    "error": {
                        "code": -32000,
                        "message": f"unsupported resume thread `{thread_id}`",
                    },
                }
            )
        continue

    send(
        {
            "id": message.get("id"),
            "error": {
                "code": -32601,
                "message": f"unsupported method `{method}`",
            },
        }
    )
