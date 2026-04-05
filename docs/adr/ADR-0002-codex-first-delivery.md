# ADR-0002: Codex-First Delivery, App-Server-First Semantics

## Status

Accepted

## Context

`agent-exporter` 最终会扩展到 Claude Code 和其他本地 CLI。

但当前如果同时推进：

- Codex
- Claude Code
- local direct-read
- app-server parity

那范围会瞬间膨胀，最后得到的不是一个稳的 v1，而是一堆半成品。

同时，当前 CodexMonitor 已经把完整导出的主 contract 写死为：

- `thread/read` 是 primary source
- `thread/resume` 是 fallback
- fallback 结果必须标 `incomplete`

## Decision

v1 的交付顺序固定为：

1. **先做 Codex**
2. **先做 app-server source**
3. **先把 canonical export 路径做对**
4. **local direct-read 放到第二阶段**
5. **Claude Code connector 放到后续阶段**

## Why

### 这样做的好处

- 最贴当前 CodexMonitor contract
- 最容易验证 complete / incomplete 语义
- 不会把 `.codex` 的 archival truth 误包装成 canonical truth
- 可以把 `source / core / output` 三层先稳定下来

### 不这么做的坏处

如果第一天就上：

- local direct-read
- 双 source 同步推进
- 多 connector 同步上线

会直接带来：

- truth surface 混淆
- 测试矩阵爆炸
- 文档边界失真
- v1 范围失控

## Consequences

### Positive

- v1 路线非常清楚
- 文档和实现都更容易收束
- local direct-read 和 Claude Code 扩展都有明确后手

### Negative

- v1 不会立刻获得“最独立”的 local source 能力
- 需要明确区分当前交付和后续计划，不能偷换概念

## Non-Goals

- 当前不把 local direct-read 当成默认主真源
- 当前不承诺 Claude Code 同期交付
- 当前不做跨 agent search / archive platform
