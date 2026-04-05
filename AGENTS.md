# agent-exporter Agent Guide

## Repo Summary

`agent-exporter` 是一个本地优先的 AI Agent CLI 会话导出仓库。

- 当前目标：**Codex canonical transcript/archive 导出**
- 未来目标：Claude Code、以及其他本地 CLI
- 当前阶段：**Codex dual-source Phase 2 已落地，默认主路径仍是 app-server**

---

## Core Rules

1. **文档先于代码**
   - 行为语义、边界、状态语义，先写进 `docs/`，再写实现。

2. **先守住上游真理源**
   - 当前完整导出的主真源，先按 CodexMonitor 的 `thread/read` contract 理解。
   - 官方 Codex 源码决定 thread/source 真相层，不要凭印象猜。

3. **source / core / output 分层**
   - `connectors/`：负责取数
   - `core/`：负责 transcript / archive contract
   - `output/`：负责 Markdown / JSON / HTML

4. **先小后大**
   - v1 只做 Codex
   - v1 不做 search/index/MCP server
   - v1 不做 GUI
   - v1 不做多 connector 并行上线

5. **可扩展，但不提前过度设计**
   - connector 边界要预留
   - 但不要为了未来十个 connector，把今天的实现搞得过重

---

## Read Order

1. `README.md`
2. `CLAUDE.md`
3. `docs/README.md`
4. `docs/adr/ADR-0001-source-layering.md`
5. `docs/adr/ADR-0002-codex-first-delivery.md`
6. `docs/reference/codexmonitor-export-contract.md`
7. `docs/reference/codex-upstream-reading-list.md`
8. `docs/reference/external-repo-reading-list.md`
9. `docs/reference/codex-thread-archive-blueprint.md`

---

## Near-Term Scope

### 当前实现目标

- `codex app-server source`
- `codex local direct-read source`
- typed archive core
- markdown export
- `export codex --thread-id ...` 真实 CLI 主链
- `--source app-server|local`
- `degraded` archival semantics for local source

### 当前明确非目标

- Claude Code connector
- Search / index
- MCP server
- Web UI / GUI
- Hosted service

---

## Validation

当前最小验证命令：

```bash
cargo fmt
cargo test
cargo run -- connectors
cargo run -- scaffold
cargo run -- export codex --thread-id <thread-id>
cargo run -- export codex --source local --thread-id <thread-id>
```
