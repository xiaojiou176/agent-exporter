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
- Current implementation delivery is **Codex dual-source export + minimal Claude Code session-path export + shared JSON/HTML export + local archive index + local metadata search + semantic retrieval + hybrid retrieval + local multi-agent archive shell + local retrieval report artifacts + workspace-local transcript backlinks + local reports shell + reports-shell metadata search + repo-owned integration materializer/doctor + integration doctor hardening + platform-aware integration doctor diagnostics**.
- Current integration-pack semantics:
  - Codex and Claude Code are ready through CLI-first templates plus an optional minimal stdio MCP bridge
  - the MCP bridge resolves repo-local launcher paths before any explicit `AGENT_EXPORTER_BIN` / `AGENT_EXPORTER_ARGS` override
  - OpenClaw is prepared as bundle content and plugin skeletons, not as a repo-native runtime
  - `integrate <platform> --target <dir>` materializes repo-owned assets into explicit targets only
  - `doctor integrations --platform <platform> --target <dir>` stays read-only and reports `ready / partial / missing`
  - doctor now also checks target-content drift against the current repo-generated content and runs a launcher probe
  - if the launcher can only fall back to `cargo run`, doctor keeps the result conservative instead of triggering a build in read-only mode
  - doctor now also validates platform-specific config/bundle shape for Codex, Claude Code, and OpenClaw
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
  - `--format markdown|json|html`
  - `markdown` stays the default
  - `json` is a second output over the same transcript core
  - `html` is a third output over the same transcript core
  - `html` is a static readable transcript document, not a browse shell
  - workspace conversations HTML now includes a backlink to the local archive shell
  - Downloads HTML stays free of workspace-only links
- Current archive browsing / publish semantics:
  - `publish archive-index --workspace-root <repo>`
  - scans workspace conversations for existing HTML transcript exports
  - writes one local archive shell `index.html` with relative links
  - now supports local metadata filtering plus retrieval-lane guidance for semantic / hybrid CLI search
  - now also writes a local reports shell at `.agents/Search/Reports/index.html`
  - local reports shell now also supports report search and report-kind filtering
  - keeps semantic / hybrid retrieval in CLI instead of moving them into browser-side execution
  - does not add hosted publish / remote search behavior
- Current semantic retrieval semantics:
  - `search semantic --workspace-root <repo> --query "<text>"`
  - uses embedding-based retrieval over the local archive corpus
  - requires local model assets for live retrieval
  - does not silently fall back to lexical search
- Current hybrid retrieval semantics:
  - `search hybrid --workspace-root <repo> --query "<text>"`
  - blends semantic ranking with lexical metadata signals
  - reuses the persistent semantic index from the semantic path
  - does not mutate `search semantic` into a hidden hybrid command
- Current retrieval report semantics:
  - `search semantic --save-report` and `search hybrid --save-report`
  - write static HTML reports under `.agents/Search/Reports`
  - reports are search-owned local artifacts, not transcript HTML inputs
  - archive shell and reports shell may link them, but retrieval execution stays in CLI
- Current highest-value next step:
  - a new post-Phase-23 product decision, still local-first and non-hosted
- Current host-safety semantics:
  - the repo may spawn one direct app-server child
  - the repo may only terminate that directly owned child handle
  - host-control utilities, shell launchers, desktop automation, and inline-eval launcher overrides are rejected

## Current Document Surfaces

- `docs/reference/codexmonitor-export-contract.md`
- `docs/reference/codex-upstream-reading-list.md`
- `docs/reference/external-repo-reading-list.md`
- `docs/reference/codex-thread-archive-blueprint.md`
- `docs/integrations/README.md`

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
cargo run -- export codex --thread-id <thread-id> --format html
cargo run -- export claude-code --session-path /absolute/path/to/session.jsonl --format html
cargo run -- publish archive-index --workspace-root /absolute/path/to/repo
cargo run -- search semantic --workspace-root /absolute/path/to/repo --query "how do I fix login issues"
cargo run -- search hybrid --workspace-root /absolute/path/to/repo --query "thread-1"
cargo run -- search semantic --workspace-root /absolute/path/to/repo --query "how do I fix login issues" --save-report
cargo run -- search hybrid --workspace-root /absolute/path/to/repo --query "thread-1" --save-report
cargo run -- integrate codex --target /absolute/path/to/codex-pack
cargo run -- integrate claude-code --target /absolute/path/to/claude-pack
cargo run -- integrate openclaw --target /absolute/path/to/openclaw-pack
cargo run -- doctor integrations --platform codex --target /absolute/path/to/codex-pack
```

`cargo test` now also acts as the repo's host-safety gate.
