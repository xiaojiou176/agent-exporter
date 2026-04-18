#!/usr/bin/env python3
"""Run local-first and live public-surface smoke checks for agent-exporter.

This script is intentionally manual and repo-owned:
- it proves the local first-success path still works
- it proves the generated local workbench surfaces still land where expected
- it checks that the current public front door, proof page, release shelf,
  and raw MCP descriptor are reachable

It does not publish anything by itself.
"""

from __future__ import annotations

import argparse
import json
import shutil
import subprocess
import sys
import tempfile
import urllib.request
from pathlib import Path


def run(cmd: list[str], cwd: Path) -> str:
    print(f"$ {' '.join(cmd)}")
    completed = subprocess.run(
        cmd,
        cwd=cwd,
        check=True,
        text=True,
        capture_output=True,
    )
    if completed.stdout.strip():
        print(completed.stdout.strip())
    if completed.stderr.strip():
        print(completed.stderr.strip())
    print()
    return completed.stdout


def ensure_exists(path: Path, label: str) -> None:
    if not path.exists():
        raise SystemExit(f"{label} missing: {path}")
    print(f"[ok] {label}: {path}")


def fetch(url: str) -> tuple[str, int, str]:
    request = urllib.request.Request(url, headers={"User-Agent": "Mozilla/5.0"})
    with urllib.request.urlopen(request, timeout=20) as response:
        body = response.read(4000).decode("utf-8", "replace")
        return response.geturl(), response.status, body


def local_smoke(repo_root: Path) -> None:
    run(["cargo", "fmt", "--check"], cwd=repo_root)
    run(["cargo", "clippy", "--all-targets", "--all-features", "--", "-D", "warnings"], cwd=repo_root)
    run(["cargo", "test"], cwd=repo_root)
    run(["cargo", "run", "--quiet", "--", "scaffold"], cwd=repo_root)
    run(["cargo", "run", "--quiet", "--", "connectors"], cwd=repo_root)
    run(
        ["cargo", "run", "--quiet", "--", "publish", "archive-index", "--workspace-root", str(repo_root)],
        cwd=repo_root,
    )

    workbench_paths = [
        repo_root / ".agents" / "Conversations" / "index.html",
        repo_root / ".agents" / "Search" / "Reports" / "index.html",
        repo_root / ".agents" / "Integration" / "Reports" / "index.html",
    ]
    for path in workbench_paths:
        ensure_exists(path, "generated workbench surface")

    temp_root = Path(tempfile.mkdtemp(prefix="agent-exporter-public-smoke-"))
    try:
        codex_pack = temp_root / "codex-pack"
        run(
            [
                "cargo",
                "run",
                "--quiet",
                "--",
                "onboard",
                "codex",
                "--target",
                str(codex_pack),
                "--save-report",
            ],
            cwd=repo_root,
        )
        ensure_exists(codex_pack / "AGENTS.md", "materialized codex AGENTS")
        ensure_exists(codex_pack / ".agents" / "skills" / "export-archive" / "SKILL.md", "materialized codex skill")
        ensure_exists(codex_pack / ".codex" / "config.toml", "materialized codex config")
    finally:
        shutil.rmtree(temp_root, ignore_errors=True)


def live_smoke(repo_root: Path) -> None:
    server = json.loads((repo_root / "server.json").read_text())
    repo_url = server["repository"]["url"]
    website_url = server["websiteUrl"].rstrip("/")
    urls = [
        ("repo front door", repo_url),
        ("pages landing", website_url + "/"),
        ("archive shell proof", website_url + "/archive-shell-proof.html"),
        ("latest release shelf", repo_url + "/releases/latest"),
        (
            "raw server descriptor",
            repo_url.replace("https://github.com/", "https://raw.githubusercontent.com/")
            + "/main/server.json",
        ),
    ]

    for label, url in urls:
        final_url, status, body = fetch(url)
        if status != 200:
            raise SystemExit(f"{label} did not return 200: {url} -> {status}")
        print(f"[ok] {label}: {final_url}")
        if label == "raw server descriptor":
            live_descriptor = json.loads(body)
            if live_descriptor["websiteUrl"] != server["websiteUrl"]:
                raise SystemExit("live server.json websiteUrl drifted from repo truth")
            if live_descriptor["repository"]["url"] != server["repository"]["url"]:
                raise SystemExit("live server.json repository url drifted from repo truth")
        print()


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Run public-surface smoke checks for agent-exporter.")
    parser.add_argument(
        "--workspace-root",
        default=".",
        help="Repo root to smoke test. Defaults to the current directory.",
    )
    parser.add_argument(
        "--skip-live",
        action="store_true",
        help="Skip live URL checks and only run local-first smoke.",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    repo_root = Path(args.workspace_root).resolve()
    print(f"== local smoke for {repo_root} ==")
    local_smoke(repo_root)
    if not args.skip_live:
        print("== live public-surface smoke ==")
        live_smoke(repo_root)
    print("== public surface smoke: all checks passed ==")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
