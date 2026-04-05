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

### Changed
- `scaffold` now reports the real v1 export path instead of a plan-only placeholder
- README / AGENTS / CLAUDE now describe the landed Codex-only v1 implementation
