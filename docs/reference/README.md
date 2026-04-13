# Maintainer Reference Hub

This folder is the maintainer-only reference layer for `agent-exporter`.

It is not part of the public first-run route.
The English-first public route stops at:

1. `README.md`
2. `docs/index.md`
3. `docs/archive-shell-proof.md`
4. `docs/distribution-packet-ledger.md`

The materials below are still active and important, but they belong to
implementation contracts, upstream archaeology, and maintainer judgment rather
than the public front door.

Mixed-language notes are allowed here on purpose because this folder is a
maintainer reference shelf, not a public product landing page.

## Current Reality

当前 reference 层已经对齐 **Phase 32-34 local governance workbench landed**：

- 默认主路径仍然是 `app-server`
- `local direct-read` 已经 landed，不再是 future plan
- `claude-code --session-path` 已经 landed，不再是 future plan
- `--format json` 已经 landed，不再是 future plan
- `--format html` 已经 landed，不再是 future plan
- `publish archive-index` 已经 landed，不再是 future plan
- `publish archive-index` 现在已经是 local multi-agent archive shell，不再只是 flat index
- local metadata search 已经 landed，不再是 future plan
- `search semantic` 已经 landed，不再是 future plan
- persistent local semantic index 已经 landed，不再是 future plan
- persistent semantic index sidecar 只会在相同模型资产身份下复用
- `search hybrid` 已经 landed，不再是 future plan
- retrieval reports under `.agents/Search/Reports` 已经 landed，不再是 future plan
- workspace-only transcript backlinks 已经 landed，不再是 future plan
- local reports shell 已经 landed，不再是 future plan
- reports-shell metadata search 已经 landed，不再是 future plan
- minimal stdio MCP bridge 已经 landed，不再是 future plan
- repo-owned integration materializer 已经 landed，不再是 future plan
- repo-owned integration doctor 已经 landed，不再是 future plan
- integration doctor 的 target drift checks + launcher probe 已经 landed，不再是 future plan
- integration doctor 的 platform-aware config/bundle diagnostics 已经 landed，不再是 future plan
- integration doctor 的 pack-shape hardening 已经 landed，不再是 future plan
- integration onboarding experience 已经 landed，不再是 future plan
- integration materialization now rejects obvious live host/global roots instead of relying on docs-only warnings
- integration evidence reports under `.agents/Integration/Reports` 已经 landed，不再是 future plan
- integration evidence shell search/facets 已经 landed，不再是 future plan
- machine-readable integration evidence via `report.json + index.json` 已经 landed，不再是 future plan
- integration evidence timeline/diff via `agent-exporter evidence diff` 已经 landed，不再是 future plan
- evidence gate / explain 已经 landed，不再是 future plan
- baseline registry via `agent-exporter evidence baseline ...` 已经 landed，不再是 future plan
- policy packs via `agent-exporter evidence policy ...` 已经 landed，不再是 future plan
- decision promotion/history via `agent-exporter evidence promote` / `evidence history` 已经 landed，不再是 future plan
- remediation bundle studio via `agent-exporter evidence remediation` 已经 landed，不再是 future plan
- read-only governance MCP surface + current-decision automation 已经 landed，不再是 future plan
- local-first governance workbench 已经 landed，不再是 future plan
- hosted / browser executor / cloud backend 仍然不是当前允许方向
- `local` 只代表 **archival truth**
- `claude-code` 当前也只代表 **degraded local import truth**
- `local` / `claude-code` 的导出状态都不能写成 complete 或 canonical

说得更直白一点：

> `app-server` 像正式成绩单，`local` 和 `claude-code` 更像档案室复印件。
> 它们都是真的，但不是同一种“真”。

## Current Documents

1. `codexmonitor-export-contract.md`
   - 当前默认 `app-server` 主路径要继承的导出格式、路径、`complete/incomplete` 语义，以及 local second entrance 不能反客为主的边界
2. `codex-upstream-reading-list.md`
   - 官方 Codex `thread/read` / sqlite / rollout / `turn-history` 的核心锚点，解释为什么 local source 有价值，但仍然只是 archival layer
3. `external-repo-reading-list.md`
   - 外部开源仓的分层借鉴矩阵
4. `codex-thread-archive-blueprint.md`
  - 当前 `agent-exporter` 的产品/架构蓝图，明确 multi-connector 命令面、默认 source、输出格式顺序、以及 `degraded` 状态语义

## Suggested Reading Order

1. `codexmonitor-export-contract.md`
2. `codex-upstream-reading-list.md`
3. `external-repo-reading-list.md`
4. `codex-thread-archive-blueprint.md`

这条顺序现在的目的，不只是“读参考资料”，而是尽快把下面四件事读成同一个口径：

1. 为什么 `app-server` 仍然是默认主路径
2. 为什么 `local` 与 `claude-code` 已经 landed 但仍然不是 canonical replacement
3. 为什么 reports-shell metadata search landed 后仍然还不是 hosted / semantic platform
4. 为什么 `degraded` 是正确披露，而不是保守措辞
5. 为什么 integration evidence 现在既有 human-facing HTML receipt，也有 machine-readable JSON receipt
