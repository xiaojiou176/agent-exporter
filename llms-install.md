# agent-exporter Local Stdio Host Packet Install

This page is the shortest truthful install path for host reviewers who want the
local stdio host packet instead of the CLI-first front door.

## What this is

- a **local-first stdio MCP bridge**
- launched from a repo checkout
- backed by `python3 scripts/agent_exporter_mcp.py`
- exposing read-mostly archive, retrieval, and governance tools

It is **not**:

- a hosted archive platform
- a remote transcript service
- a replacement for the CLI-first primary product surface

## Current Main Vs Published Shelf

This file tracks the **repo-side current host packet on `main`**.

If you need the newest **frozen published packet**, use the latest GitHub
release shelf instead of assuming this page and the latest tag are identical.

## Packet contents

Keep this reviewer packet together:

| Slice | File | Why it matters |
| --- | --- | --- |
| install note | `./llms-install.md` | shortest truthful attach path |
| canonical descriptor | `./server.json` | metadata for registry/read-back review lanes |
| square logo | `./docs/assets/marketplace/agent-exporter-cline-logo-400.png` | host/reviewer-facing logo asset |
| square proof tile | `./docs/assets/marketplace/archive-shell-proof.svg.png` | reviewer-facing proof art that still points back to the archive shell |

## Quick install

1. Clone the repo:

```bash
git clone https://github.com/xiaojiou176-open/agent-exporter.git
cd agent-exporter
```

2. Make sure one of these is available:

- `target/release/agent-exporter`
- `target/debug/agent-exporter`
- or a working `cargo` toolchain

The bridge script looks for the binary in this order:

1. `AGENT_EXPORTER_BIN` when you explicitly pin one
2. `target/release/agent-exporter`
3. `target/debug/agent-exporter`
4. `cargo run --quiet --manifest-path <repo>/Cargo.toml --bin agent-exporter --`

3. Point your MCP host at the bridge script:

```json
{
  "mcpServers": {
    "agent-exporter": {
      "command": "python3",
      "args": [
        "/absolute/path/to/agent-exporter/scripts/agent_exporter_mcp.py"
      ]
    }
  }
}
```

Optional environment overrides:

- `AGENT_EXPORTER_BIN`: pin one exact binary path when you do not want the
  bridge script to auto-detect release/debug builds
- `AGENT_EXPORTER_ARGS`: append extra fixed CLI arguments without changing the
  script path itself

## First smoke check

After attach, call one low-risk tool first:

- `integration_evidence_policy_list`

That proves the bridge is reachable before you ask it to publish, search, or
inspect workspace artifacts.

## First workspace-backed checks

Once the bridge responds, the next honest checks are:

1. `publish_archive_index` with a real `workspace_root`
2. `search_semantic` or `search_hybrid` against a workspace that already has
   transcript exports
3. one governance read, such as `integration_evidence_policy_show`

That order matters for the same reason a pilot checks the instruments before
takeoff: you confirm the bridge is alive first, then you confirm it can see a
real workspace, then you ask it for deeper governance work.

## Current descriptor

- canonical MCP descriptor: [`server.json`](./server.json)
- current public proof pages:
  - Docs landing: `https://xiaojiou176-open.github.io/agent-exporter/`
  - Archive shell proof: `https://xiaojiou176-open.github.io/agent-exporter/archive-shell-proof.html`

## Tool families

- archive publishing:
  - `publish_archive_index`
- retrieval:
  - `search_semantic`
  - `search_hybrid`
- governance evidence:
  - `integration_evidence_diff`
  - `integration_evidence_gate`
  - `integration_evidence_explain`
  - `integration_evidence_remediation`
  - `integration_evidence_policy_list`
  - `integration_evidence_policy_show`

## Truth boundary

This install page proves the repo-owned local bridge exists.
It does **not** prove:

- Official MCP Registry publication
- Smithery publication
- a hosted runtime
- a live marketplace listing beyond the intake receipts
- that the MCP bridge replaces the CLI-first flagship packet
