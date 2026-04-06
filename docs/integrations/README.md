# Integration Pack

这一层不是产品新功能，而是给开发者的“接线包”。

你可以先把它理解成：

> `agent-exporter` 现在已经是一个稳定的本地 CLI / artifact graph，
> 这一层要做的是把它更容易接进 **Codex / Claude Code / OpenClaw** 的日常工作流。

## 当前交付边界

当前 integration pack 里，已经真实准备好的部分有：

- Codex 的项目级 `AGENTS.md` 接入模板
- Codex 的 `config.toml` MCP snippet
- Claude Code 的 project skills / commands 接入模板
- Claude Code 的 `.mcp.json` MCP snippet
- OpenClaw 的 bundle/plugin 模板
- OpenClaw bundle 内可直接放入的 `.mcp.json` snippet

当前 integration pack 已经额外内建了一个最小 **stdio MCP bridge**：

- `scripts/agent_exporter_mcp.py`

当前 MCP bridge 暴露的是最小 publish/search 工具面，不是全量 CLI 面。

## 目录

- `codex.md`
- `claude-code.md`
- `openclaw.md`
- `templates/`

## 设计原则

1. **先交付真实可用的接线方式**
   - 不把“未来可能做的 MCP server”写成今天已经存在

2. **继续保持 local-first**
   - 模板默认调用本地 `agent-exporter` CLI

3. **让 artifact graph 可复用**
   - archive shell、reports shell、transcript、saved reports 继续是本地静态工件

4. **不要把集成模板写成平台壳**
   - 这层是接线包，不是新产品前台
