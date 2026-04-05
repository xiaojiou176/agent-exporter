# Reference Reading List

这里放的是实现 `agent-exporter` 时必须参考的上游资料、当前 contract、以及外部开源仓矩阵。

这不是“以后再补”的目录，而是当前已经生效的参考入口。

## Current Reality

当前 reference 层已经对齐 **Phase 3 second-connector proof**：

- 默认主路径仍然是 `app-server`
- `local direct-read` 已经 landed，不再是 future plan
- `claude-code --session-path` 已经 landed，不再是 future plan
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
  - 当前 `agent-exporter` 的产品/架构蓝图，明确 multi-connector 命令面、默认 source、以及 `degraded` 状态语义

## Suggested Reading Order

1. `codexmonitor-export-contract.md`
2. `codex-upstream-reading-list.md`
3. `external-repo-reading-list.md`
4. `codex-thread-archive-blueprint.md`

这条顺序现在的目的，不只是“读参考资料”，而是尽快把下面三件事读成同一个口径：

1. 为什么 `app-server` 仍然是默认主路径
2. 为什么 `local` 与 `claude-code` 已经 landed 但仍然不是 canonical replacement
3. 为什么 `degraded` 是正确披露，而不是保守措辞
