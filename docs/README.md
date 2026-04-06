# Documentation Index

这是 `agent-exporter` 的文档总入口。

如果你第一次读这个仓，推荐先按下面顺序走，不要直接跳到某个实现文件里。

## Current Phase Snapshot

当前这套文档已经进入 **Phase 18 reports-shell search landed**。

你可以先把它理解成：这套文档现在要同时解释“正门、侧门、以及第二种输出格式”，但不能把它们说成同一种真相。

- Codex 默认主路径仍然是 `app-server`
- Codex `local direct-read` 已经 landed，属于第二条已落地入口
- `Claude Code` 最小 `--session-path` connector 已 landed，证明架构能接第二个来源
- `--format json` 已经 landed，证明 shared transcript/core/output abstraction 能接第二种结构化输出
- `--format html` 已经 landed，证明同一份 transcript 也能输出成静态可读页面
- `publish archive-index` 已经 landed，证明这些页面已经可以本地浏览和静态发布
- `publish archive-index` 现在已经升级成 local multi-agent archive shell，自带本地 metadata filter、connector/completeness facets 和 retrieval lane 说明
- `search semantic` 已经 landed，证明这个仓现在已经有真实 embedding-based retrieval 命令
- `search semantic` 现在已经会按模型资产身份持久化并复用本地 semantic index sidecar
- `search hybrid` 已经 landed，证明 lexical metadata signal 和 semantic retrieval 已经能在本地 CLI 里组合
- `search semantic --save-report` / `search hybrid --save-report` 已经 landed，证明 retrieval 结果现在能保存成 local static reports
- workspace conversations HTML transcript 现在已经会带回 archive shell 的本地导航 backlink
- `.agents/Search/Reports/index.html` 现在已经会作为 local reports shell 组织这些 saved reports
- local reports shell 现在也已经支持本地 report search 和 report-kind filter
- `local` 和 `claude-code` 当前都按 **degraded** 理解，不能冒充 canonical parity
- 当前已进入 post-Phase-18 产品裁决区，而不是直接膨胀成 hosted / 平台壳

## Read Order

1. `../README.md`
2. `../AGENTS.md`
3. `../CLAUDE.md`
4. `./adr/ADR-0001-source-layering.md`
5. `./adr/ADR-0002-codex-first-delivery.md`
6. `./reference/host-safety-contract.md`
7. `./reference/codexmonitor-export-contract.md`
8. `./reference/codex-upstream-reading-list.md`
9. `./reference/external-repo-reading-list.md`
10. `./reference/codex-thread-archive-blueprint.md`

---

## Documentation Layers

### 1. `docs/adr/*`

记录架构决策，回答“为什么这么选”。

当前已有：

- `ADR-0001-source-layering.md`
- `ADR-0002-codex-first-delivery.md`

### 2. `docs/reference/*`

记录实现前必须参考的真理面和外部参考仓，回答“要参考谁、借哪一层、不该抄哪一层”，同时把当前 dual-source reality、second connector reality、以及双输出现实讲清楚。

当前已有：

- `host-safety-contract.md`
- `codexmonitor-export-contract.md`
- `codex-upstream-reading-list.md`
- `external-repo-reading-list.md`
- `codex-thread-archive-blueprint.md`
- `integrations/*`

---

## 当前最重要的 4 份文档

如果你只读 4 份，先读：

1. `reference/host-safety-contract.md`
2. `reference/codexmonitor-export-contract.md`
3. `reference/codex-upstream-reading-list.md`
4. `reference/codex-thread-archive-blueprint.md`

这四份分别回答：

- 这个仓允许碰宿主机的最远边界在哪里
- 默认 Codex `app-server` 主路径的 contract 是什么
- 为什么 `local` 和 `claude-code` 已 landed 但仍然只能算 degraded / archival-class reality
- `agent-exporter` 现在该如何同时容纳 multiple connectors 与 shared archive contract，以及 Markdown / JSON / HTML / archive index 这几层输出
