# ADR-0001: Source / Core / Output Layering

## Status

Accepted

## Context

`agent-exporter` 当前先做 Codex 对话记录导出，但未来明确要扩展到 Claude Code 和其他本地 CLI。

如果一开始把“取数逻辑”和“导出格式逻辑”写死在一起，后面接第二个 connector 时就会大拆。

## Decision

仓库从第一天开始按三层拆分：

1. `connectors/`
   - 负责从具体来源取数
2. `core/`
   - 负责 transcript / archive contract
3. `output/`
   - 负责 Markdown / JSON / HTML 渲染

CLI 只负责参数解析和 orchestration，不直接承担某个来源的私有解析逻辑。

## Consequences

### Positive

- 先做 Codex，不会阻断后续接 Claude Code
- 未来可以同时支持 canonical source 和 local archival source
- 文档和代码的边界更稳定

### Negative

- 初期会有一些“看起来像多写了一层”的骨架代码
- v1 交付前需要保持克制，避免把 connector 设计做得过重

## Current Scope

- `codex`: current
- `claude-code`: planned

## Non-Goals

- 当前不做 search/index
- 当前不做 GUI
- 当前不做远程服务
