# OpenClaw Integration

## 适用场景

OpenClaw 当前最稳的接法，是走它已经支持的 **bundle/plugin** 兼容层。

说得更直白一点：

> OpenClaw 能吃 Codex / Claude 生态的 bundle 结构，
> 所以我们先给开发者一套本地 bundle 模板，
> 而不是硬造一个还不存在的 repo-native OpenClaw runtime。

## 当前最稳接法

1. 使用 bundle/plugin 模板目录
2. 让 bundle 内的 skills / commands 继续调用本地 `agent-exporter`
3. 如果你要 OpenClaw 直接吃 MCP，再把 bundle 内的 `.mcp.json` 一起带上

## 模板

- `templates/openclaw-codex-bundle/`
- `templates/openclaw-claude-bundle/`

## 当前诚实边界

- 这些模板今天可以作为 **bundle content** 使用
- 当前 repo 已经内建最小 stdio MCP bridge，所以模板里附带了 `.mcp.json` snippet
- 但当前 bridge 只覆盖 publish/search，不代表整个 CLI 全量变成 MCP
