# Documentation Index

这是 `agent-exporter` 的文档总入口。

如果你第一次读这个仓，推荐先按下面顺序走，不要直接跳到某个实现文件里。

## Product Snapshot

先记住这 4 句，再继续往下读：

- `Product Kernel`：`agent-exporter` 是一个 **local-first archive and governance workbench for AI agent transcripts**
- `Primary Surface`：**CLI-first**
- `Secondary Surfaces`：local archive shell / reports shell、repo-owned integration pack、read-only governance MCP bridge
- `Flagship Public Packet`：**GitHub repo + CLI quickstart + archive shell proof**

再补一句顺序感：

> front door 先讲 CLI quickstart，archive shell proof 是第一层可见证明；
> integration pack 和 governance lane 继续保留，但不抢第一屏。

## First Success Orientation

把 docs 入口也按同一个顺序理解：

1. `cargo run -- connectors`
2. `cargo run -- export codex --thread-id <thread-id> --format html --destination workspace-conversations --workspace-root /absolute/path/to/repo`
3. `cargo run -- publish archive-index --workspace-root /absolute/path/to/repo`

成功后你会看到：

- `.agents/Conversations/*.html` transcript export
- `.agents/Conversations/index.html` archive shell proof
- 这份 proof 是 **local-only HTML receipt**，不是 hosted page

## Public Docs Entry Points

- Pages landing: `https://xiaojiou176-open.github.io/agent-exporter/`
- Archive shell proof page: `https://xiaojiou176-open.github.io/agent-exporter/archive-shell-proof.html`
- Repo map: `https://xiaojiou176-open.github.io/agent-exporter/repo-map/`
- Latest release shelf: `https://github.com/xiaojiou176-open/agent-exporter/releases/latest`

## Current Surface Snapshot

当前这套文档已经吸收 **Phase 32-34 的 repo-local capability**，并且产品身份已经收口。

与其用一大串 landed 项目压人，不如先记住下面这张图：

| Layer | 当前真相 | first proof / entry |
| --- | --- | --- |
| CLI core | Codex `app-server` 仍是 canonical 主路径；`local` 与 `claude-code` 已 landed | `../README.md` 的 CLI quickstart |
| Archive shell proof | `publish archive-index` 会生成 transcript browser、workspace backlinks 和 archive shell | `.agents/Conversations/index.html` |
| Reports shell | `search semantic|hybrid --save-report` 会生成 retrieval receipts 与 reports shell | `.agents/Search/Reports/index.html` |
| Integration pack | `integrate` / `doctor integrations` / `onboard` 已是 repo-owned companion lane | `.agents/Integration/Reports/index.html` |
| Governance lane | evidence / baseline / policy / remediation 已进入本地 workbench | archive shell Decision Desk + integration evidence reports |

这张表想表达的其实只有一句话：

> 正门已经固定是 CLI。
> 侧门都已经被命名，但每一扇侧门都还要各自记账、各自过线，不能混成一句“都已经 ready”。

## Public Boundary Right Now

- Pages 是 **public companion docs surface**，但 archive shell / reports shell / integration evidence shell 仍然是 **repo-local proof**
- `local` 和 `claude-code` 继续按 **degraded** 理解，不能冒充 canonical parity
- integration pack 与 governance lane 可以继续长，但 docs landing page 不会把它们写成第一主角
- archive shell proof page 是 tracked public explanation page，不是 hosted archive shell runtime
- 如果要看完整 capability ledger，请去 `../README.md` 和 `../CHANGELOG.md`，不要把 docs landing page 当 release history

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
