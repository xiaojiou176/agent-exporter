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

### Changed
- `scaffold` now reports the real v1 export path instead of a plan-only placeholder
- README / AGENTS / CLAUDE now describe the landed Codex-only v1 implementation
- CLI help and docs now describe dual-source Phase 2 while keeping `app-server` as the default front door
- README / blueprint order now place Claude Code connector ahead of JSON / HTML follow-ups
- CLI export surface now keeps Markdown as the default while exposing `--format markdown|json|html`
- Codex / Claude connector summaries now describe the shared archive transcript contract instead of a Markdown-only contract
