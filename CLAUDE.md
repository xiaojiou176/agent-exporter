# agent-exporter CLAUDE Guide

## Quick Index

1. `AGENTS.md`
2. `README.md`
3. `docs/README.md`
4. `docs/adr/ADR-0001-source-layering.md`
5. `docs/adr/ADR-0002-codex-first-delivery.md`
6. `docs/reference/host-safety-contract.md`

## Current Truth

- This repo is a **Rust CLI-first exporter**.
- Current implementation delivery is **Codex dual-source export + minimal Claude Code session-path export + shared JSON export**.
- The repository is designed to grow into multiple connectors later, but not all at once.
- Current export semantics stay aligned with CodexMonitor:
  - `thread/read` primary
  - `thread/resume` fallback only
  - fallback exports are `incomplete`
  - Markdown stays round-based and remains the default output
  - output target semantics remain explicit
- Current local direct-read semantics:
  - `--source local`
  - `thread-id -> sqlite -> rollout_path` or direct `--rollout-path`
  - result is `degraded`
  - local source is archival truth, not canonical truth
- Current Claude Code semantics:
  - `export claude-code --session-path <PATH>`
  - local session-file import into the shared archive core
  - result is `degraded`
  - Claude Phase 3 is a second-connector proof, not a second Markdown dialect
- Current output-format semantics:
  - `--format markdown|json`
  - `markdown` stays the default
  - `json` is a second output over the same transcript core
  - `html` is still future-only
- Current highest-value next step:
  - minimal HTML renderer before browse / publish layers
- Current host-safety semantics:
  - the repo may spawn one direct app-server child
  - the repo may only terminate that directly owned child handle
  - host-control utilities, shell launchers, desktop automation, and inline-eval launcher overrides are rejected

## Current Document Surfaces

- `docs/reference/codexmonitor-export-contract.md`
- `docs/reference/codex-upstream-reading-list.md`
- `docs/reference/external-repo-reading-list.md`
- `docs/reference/codex-thread-archive-blueprint.md`

## Fast Commands

```bash
cargo fmt
cargo test
cargo run -- connectors
cargo run -- scaffold
cargo run -- export codex --thread-id <thread-id>
cargo run -- export codex --source local --thread-id <thread-id>
cargo run -- export claude-code --session-path /absolute/path/to/session.jsonl
cargo run -- export codex --thread-id <thread-id> --format json
cargo run -- export claude-code --session-path /absolute/path/to/session.jsonl --format json
```

`cargo test` now also acts as the repo's host-safety gate.
