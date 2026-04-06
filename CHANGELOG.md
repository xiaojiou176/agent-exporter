# Changelog

All notable changes to this project will be documented in this file.

The format is based on Keep a Changelog.

## [Unreleased]

### Added
- Initial Rust CLI scaffold
- Connector-aware repository layout
- Documentation entrypoints and first ADR
- Full repository documentation skeleton:
  - CodexMonitor export contract notes
  - official Codex upstream reading list
  - external repo comparison matrix
  - codex-thread-archive blueprint
  - Codex-first delivery ADR
- One-shot Codex app-server client for canonical `thread/read` exports
- Typed archive core for transcript / round / item / completeness modeling
- Round-based Markdown renderer with part splitting on round boundaries
- Real `export codex --thread-id ...` CLI flow
- Export file writing with Downloads / workspace conversations target semantics
- Local direct-read Codex source via sqlite `threads.rollout_path` lookup
- Local direct-read Codex source via direct `--rollout-path`
- `--source app-server|local` and `--codex-home` command surface
- `degraded` completeness semantics for archival local exports
- Local source CLI integration tests and app-server vs local structure comparison tests
- Minimal Claude Code connector via `export claude-code --session-path <PATH>`
- Claude session-path CLI integration tests
- Minimal shared JSON exporter via `--format json`
- Codex JSON export CLI integration tests
- Claude JSON export CLI integration tests
- JSON renderer unit tests and JSON writer conflict-suffix coverage
- Minimal shared HTML exporter via `--format html`
- Codex HTML export CLI integration tests
- Claude HTML export CLI integration tests
- HTML renderer unit tests and HTML writer conflict-suffix coverage
- Local `publish archive-index` command for workspace conversations
- Archive index unit tests and publish CLI integration tests
- Local metadata search inside archive index
- Archive index search UI unit coverage
- `search semantic` command for embedding-based retrieval
- Semantic retrieval unit tests and CLI coverage
- Persistent local semantic index sidecar reuse
- Semantic index reuse unit coverage
- `search hybrid` command for blended semantic + lexical metadata retrieval
- Hybrid retrieval unit tests and CLI coverage
- Multi-agent local archive shell sections, facets, and retrieval-lane guidance inside `publish archive-index`
- Archive shell rendering coverage
- Static retrieval report artifacts for semantic / hybrid searches
- Retrieval report rendering coverage and report-link coverage in archive shell
- Workspace-local transcript backlinks for HTML exports
- HTML workspace-navigation coverage
- Local reports shell generation under `.agents/Search/Reports/index.html`
- Reports-shell rendering coverage
- Reports-shell metadata search and report-kind facets
- Reports-shell search coverage
- Integration pack docs and templates for Codex / Claude Code / OpenClaw
- LICENSE / SECURITY / CODE_OF_CONDUCT baseline for GitHub/open-source closeout
- Minimal stdio MCP bridge for publish/search workflows
- Evidence gate / explain, read-only evidence MCP surface, and local evidence decision desk
- MCP bridge smoke coverage and MCP config templates
- Integration template README for Codex / Claude Code / OpenClaw first-run setup
- Repo-owned integration materializer for Codex / Claude Code / OpenClaw explicit targets
- Repo-owned integration doctor for target-scoped readiness checks
- Drift-aware integration doctor checks for target-content sync and launcher probing
- Integration doctor now keeps `cargo run` launcher fallback in `partial` rather than probing it in read-only mode
- Platform-aware integration doctor diagnostics for Codex config, Claude project MCP config, and OpenClaw bundle manifests
- Integration pack-shape hardening for Codex `command/args` and Claude project pack structure
- Repo-owned onboarding command that stitches materialize + doctor + next steps into one first-run path
- Forbidden-target guards for `integrate` / `onboard`, rejecting live Codex/Claude home roots and direct OpenClaw bundle/plugin roots such as `bundles/<name>` / `plugins/<name>`
- Integration evidence reports via `doctor/onboard --save-report`, with a dedicated `.agents/Integration/Reports` front door
- Integration evidence shell search/facets for `platform` and `readiness`
- Machine-readable integration evidence via paired `report.json + index.json`
- `evidence diff` for comparing saved integration evidence snapshots
- Baseline registry, policy packs, decision promotion/history, and read-only governance MCP tools
- Decision governance rendering inside the local archive front door

### Changed
- `scaffold` now reports the real v1 export path instead of a plan-only placeholder
- README / AGENTS / CLAUDE now describe the landed Codex-only v1 implementation
- CLI help and docs now describe dual-source Phase 2 while keeping `app-server` as the default front door
- README / blueprint order now place Claude Code connector ahead of JSON / HTML follow-ups
- CLI export surface now keeps Markdown as the default while exposing `--format markdown|json|html`
- CLI now also exposes `publish archive-index --workspace-root <repo>`
- Archive index now includes local metadata search without adding a hosted search layer
- Semantic retrieval now lands as a separate command instead of masquerading as metadata search
- Semantic retrieval now reuses a persistent local semantic index sidecar
- Hybrid retrieval now lands as a separate command instead of mutating `search semantic`
- `publish archive-index` now renders a richer local archive shell instead of a flat archive list
- Search commands can now persist HTML retrieval reports under `.agents/Search/Reports`
- Workspace conversations HTML exports now link back to the local archive shell without changing Downloads behavior
- `publish archive-index` now also generates a local reports shell for saved retrieval reports
- Local reports shell now includes static metadata search/filter without changing retrieval execution semantics
- A minimal stdio MCP bridge now exposes publish/search workflows to external agent clients
- Codex / Claude connector summaries now describe the shared archive transcript contract instead of a Markdown-only contract
- MCP bridge templates now default to repo-local launcher discovery instead of requiring a hard-coded release binary path
- Integration docs now spell out MCP first-run prerequisites and keep OpenClaw at bundle-content honesty
- Integration pack is no longer docs-only; it now has explicit `integrate` and `doctor integrations` entry points
- Integration doctor now detects stale materialized targets and probes the current repo-local launcher
- Integration evidence now writes paired HTML + JSON receipts while keeping integration artifacts out of transcript/search corpora
