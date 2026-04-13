# Documentation Index

This is the documentation hub for `agent-exporter`.

If you are new to the repo, follow the reading order here first instead of jumping straight into implementation files.

## Product Snapshot

Lock these four sentences in first:

- `Product Kernel`: `agent-exporter` is a **local-first archive and governance workbench for AI agent transcripts**
- `Primary Surface`: **CLI-first**
- `Secondary Surfaces`: local archive shell / reports shell, repo-owned integration pack, read-only governance MCP bridge
- `Flagship Public Packet`: **GitHub repo + CLI quickstart + archive shell proof**

One more ordering rule matters:

> The front door starts with the CLI quickstart.
> The archive shell proof is the first visible proof layer.
> Integration pack and governance lanes stay visible, but they do not own the first screen.

## First Success Orientation

Use the docs entry in the same order:

1. `cargo run -- scaffold`
1. `cargo run -- connectors`
2. `cargo run -- export codex --thread-id <thread-id> --format html --destination workspace-conversations --workspace-root /absolute/path/to/repo`
3. `cargo run -- publish archive-index --workspace-root /absolute/path/to/repo`

After that succeeds, you should see:

- the scaffolded workbench shape and expected local surfaces
- `.agents/Conversations/*.html` transcript exports
- `.agents/Conversations/index.html` archive shell proof
- a **local-only HTML receipt**, not a hosted page

## Public Docs Entry Points

- Pages landing: `https://xiaojiou176-open.github.io/agent-exporter/`
- Archive shell proof page: `https://xiaojiou176-open.github.io/agent-exporter/archive-shell-proof.html`
- Repo map: `https://xiaojiou176-open.github.io/agent-exporter/repo-map/`
- Latest release shelf: `https://github.com/xiaojiou176-open/agent-exporter/releases/latest`
- Secondary packet / listing ledger: `./distribution-packet-ledger.md`

## Release Shelf Truth

Use the latest release shelf for the newest **published** packet:

- the tagged binary cut
- release notes for the shipped packet
- the frozen public packet state that already made it into a release

Use the repo front door and Pages docs for the newest **repository-side truth**
on `main`:

- front-door wording
- packet / lane truth after the latest tag
- docs or governance hardening that moved ahead of the current release

Those surfaces should agree conceptually, but they do not mean the same thing.
The release shelf is the newest tagged cut; the repo/docs surface can move
forward before the next release is published.

## Current Surface Snapshot

The current docs already absorb the latest repo-local capability phase, but the easiest way to understand the product is still this map:

| Layer | Current truth | First proof / entry |
| --- | --- | --- |
| CLI core | Codex `app-server` remains the canonical path; `local` and `claude-code` are landed | CLI quickstart in `../README.md` |
| Archive shell proof | `publish archive-index` generates the transcript browser, workspace backlinks, and the archive shell | `.agents/Conversations/index.html` |
| Reports shell | `search semantic|hybrid --save-report` generates retrieval receipts and the reports shell | `.agents/Search/Reports/index.html` |
| Integration pack | `integrate`, `doctor integrations`, and `onboard` are repo-owned companion lanes | `.agents/Integration/Reports/index.html` |
| Governance lane | evidence, baselines, policy packs, and remediation now live in the local workbench | archive shell Decision Desk + integration evidence reports |

What this table really says:

> The primary door is fixed as CLI.
> Secondary doors are named and real, but each one still has to pass its own review line.

## Secondary Lane Truth

Host-native packet and listing truth still matters, but it is a second-ring
surface. Keep it behind the CLI-first front door.

Use these files when you actually need lane truth instead of product identity:

- `./distribution-packet-ledger.md`
- `../public-skills/README.md`

Those pages hold the current `listed-live`, `review-pending`,
`platform-not-accepted-yet`, `closed-not-accepted`, and
`not_honest_cargo_yet` states without turning packet status into the first
screen.

## Public Read Order

If you are evaluating the product from the outside, keep the route this short:

1. `../README.md`
2. `./index.md`
3. `./archive-shell-proof.md`
4. `./distribution-packet-ledger.md`
5. `../public-skills/README.md` only if you actually need the host-native packet lane

That order keeps the first screen on:

- what the product is
- how the first success works
- what the archive shell really proves
- where second-ring lane truth lives

## Documentation Layers

### 1. `docs/adr/*`

Architecture decisions that answer "why did we choose this?"

Current files:

- `ADR-0001-source-layering.md`
- `ADR-0002-codex-first-delivery.md`

### 2. `docs/reference/*`

Reference material that answers "who are we borrowing from, which layer are we borrowing, and which layer must not be copied?"

Current files:

- `host-safety-contract.md`
- `codexmonitor-export-contract.md`
- `codex-upstream-reading-list.md`
- `external-repo-reading-list.md`
- `codex-thread-archive-blueprint.md`
- `integrations/*`

## Maintainer-Only Internal References

The files below are still important, but they are **not** part of the public
front door. Read them when you are doing maintainer work, implementation
archaeology, or upstream contract review:

1. `../AGENTS.md`
2. `../CLAUDE.md`
3. `./adr/ADR-0001-source-layering.md`
4. `./adr/ADR-0002-codex-first-delivery.md`
5. `./reference/host-safety-contract.md`
6. `./reference/codexmonitor-export-contract.md`
7. `./reference/codex-upstream-reading-list.md`
8. `./reference/external-repo-reading-list.md`
9. `./reference/codex-thread-archive-blueprint.md`

Those files answer maintainer questions such as:

- where the host-system boundary stops
- what the default Codex `app-server` primary path contract really is
- why `local` and `claude-code` are landed but still degraded / archival-class realities
- how `agent-exporter` currently holds multiple connectors and shared archive outputs across Markdown, JSON, HTML, and archive index layers
