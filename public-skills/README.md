# Public Skills

This directory holds public-facing skill packets for `agent-exporter`.

## Quick Truth

If you only need the fast verdict, use this table first:

| Question | Current answer |
| --- | --- |
| What is this lane? | a host-native packet lane, not the flagship front door |
| Which packet owns first-time orientation? | GitHub repo + CLI quickstart + archive shell proof |
| Which live external lanes exist? | Official MCP Registry and ClawHub |
| Which lanes are still not done? | Goose review-pending, awesome-agent-skills open, Smithery build-contract-blocked |

## What This Lane Is

Think of this directory as the **local stdio host packet lane**, not a second homepage.

It exists for people or platforms asking:

- "Is there a host-native packet I can inspect?"
- "Which packet is currently live, pending, blocked, or closed?"
- "Where is the public-facing skill folder for this workbench?"

It does **not** exist to replace the main product story.

These packets are **secondary public lanes**.
They do not replace the flagship public packet:

- GitHub repo front door
- CLI quickstart
- archive shell proof

The product itself remains an **archive and governance workbench**.
Its primary surface is still the **quickstart path**.

## Use This Lane Only After The Front Door

The healthy order is:

1. understand the product from the GitHub repo front door
2. understand the first proof from the archive shell proof page
3. only then inspect host-native packet lanes here

That order matters because packet status is narrower than product identity.

If you need the repo-wide packet map, open
[`docs/distribution-packet-ledger.md`](../docs/distribution-packet-ledger.md)
first. That page keeps the flagship public packet separate from these
host-native side lanes.

Current packet:

- `agent-exporter-archive-governance-workbench/`

## Current Lane Map

| If you need to know... | Open this |
| --- | --- |
| the flagship public packet | `../README.md` |
| what the proof page means | `../docs/archive-shell-proof.md` |
| the full packet/listing ledger | `../docs/distribution-packet-ledger.md` |
| the current host-native skill packet | `./agent-exporter-archive-governance-workbench/` |

Current lane truth:

- `Official MCP Registry`: `listed-live` for `io.github.xiaojiou176-open/agent-exporter-mcp`
- `ClawHub`: `listed-live`
- `Goose Skills Marketplace`: `review-pending` via `block/Agent-Skills#24`
- `agent-skill.co source repo`: `platform-not-accepted-yet` via `heilcheng/awesome-agent-skills#180` while Vercel team authorization is pending
- `OpenHands/extensions`: `closed-not-accepted`; maintainer pointed to a custom `marketplace.json` distribution path instead
- `Smithery`: `build-contract-blocked`; `smithery mcp publish . -n xiaojiou176-open/agent-exporter-mcp --json` failed while building an shttp bundle from the Rust repo root
- `awesome-opencode`: `exact_blocker_with_fresh_evidence`; the current packet is a host-native skill folder for an archive/governance workbench, not an honest opencode-native project/resource entry today

## Registry Outcome Map

| Lane | Current state | Why it matters |
| --- | --- | --- |
| Official MCP Registry | `listed-live` | canonical MCP registry lane for the local stdio bridge |
| ClawHub | `listed-live` | host-native packet lane remains externally discoverable |
| Goose Skills Marketplace | `review-pending` | submission exists but maintainer acceptance is still pending |
| awesome-agent-skills | `platform-not-accepted-yet` | source-repo lane is still open and blocked upstream |
| OpenHands/extensions | `closed-not-accepted` | this is no longer an active upstream listing lane |
| Smithery | `build-contract-blocked` | current repo-root publish path fails before publication |
| awesome-opencode | `exact_blocker_with_fresh_evidence` | current packet is not an honest opencode-native fit |

## What This Lane Must Not Overclaim

- not the flagship front door
- not the primary product identity
- not a hosted runtime
- not a repo-wide `MCP-first` product position
