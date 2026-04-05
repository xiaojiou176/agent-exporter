# Documentation Index

这是 `agent-exporter` 的文档总入口。

如果你第一次读这个仓，推荐先按下面顺序走，不要直接跳到某个实现文件里。

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

记录实现前必须参考的真理面和外部参考仓，回答“要参考谁、借哪一层、不该抄哪一层”。

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

- 当前导出 contract 是什么
- 官方 thread/source 真相层是什么
- `agent-exporter` 该怎么做
