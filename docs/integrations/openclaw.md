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
3. 先把 CLI-first workflow 接进去

## 模板

- `templates/openclaw-codex-bundle/`
- `templates/openclaw-claude-bundle/`

## 当前诚实边界

- 这些模板今天可以作为 **bundle content** 使用
- 当前 repo **还没有**原生 MCP server，所以模板里不伪造 `.mcp.json` server entry
- 如果后续 repo 真补了 MCP server，再给 OpenClaw 加 `.mcp.json` 会更准确
