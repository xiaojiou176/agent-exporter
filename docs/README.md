# Documentation Index

这是 `agent-exporter` 的文档总入口。

如果你第一次读这个仓，推荐先按下面顺序走，不要直接跳到某个实现文件里。

## Current Phase Snapshot

当前这套文档已经进入 **Codex dual-source Phase 2**。

你可以先把它理解成：这套文档现在要同时解释“正门”和“侧门”，但不能把两者说反。

- 默认主路径仍然是 `app-server`
- `local direct-read` 已经 landed，属于第二条已落地入口
- `local` 代表 **archival truth**
- `local` 的结果要按 **degraded** 理解，不能冒充 canonical parity

## Read Order

1. `../README.md`
2. `../AGENTS.md`
3. `../CLAUDE.md`
4. `./adr/ADR-0001-source-layering.md`
5. `./adr/ADR-0002-codex-first-delivery.md`
6. `./reference/codexmonitor-export-contract.md`
7. `./reference/codex-upstream-reading-list.md`
8. `./reference/external-repo-reading-list.md`
9. `./reference/codex-thread-archive-blueprint.md`

---

## Documentation Layers

### 1. `docs/adr/*`

记录架构决策，回答“为什么这么选”。

当前已有：

- `ADR-0001-source-layering.md`
- `ADR-0002-codex-first-delivery.md`

### 2. `docs/reference/*`

记录实现前必须参考的真理面和外部参考仓，回答“要参考谁、借哪一层、不该抄哪一层”，同时把当前 dual-source 现实讲清楚。

当前已有：

- `codexmonitor-export-contract.md`
- `codex-upstream-reading-list.md`
- `external-repo-reading-list.md`
- `codex-thread-archive-blueprint.md`

---

## 当前最重要的 3 份文档

如果你只读 3 份，先读：

1. `reference/codexmonitor-export-contract.md`
2. `reference/codex-upstream-reading-list.md`
3. `reference/codex-thread-archive-blueprint.md`

这三份分别回答：

- 默认 `app-server` 主路径的 contract 是什么
- 为什么 `local` 已 landed 但仍然只能算 archival truth
- `agent-exporter` 现在该如何同时容纳 canonical 与 degraded 这两层 reality
