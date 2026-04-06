# Integration Pack

这一层不是产品新功能，而是给开发者的“接线包”。

你可以先把它理解成：

> `agent-exporter` 现在已经是一个稳定的本地 CLI / artifact graph，
> 这一层要做的是把它更容易接进 **Codex / Claude Code / OpenClaw** 的日常工作流。

## 当前交付边界

当前 integration pack 里，已经真实准备好的部分有：

- Codex 的项目级 `AGENTS.md` 接入模板
- Claude Code 的 project skills / commands 接入模板
- OpenClaw 的 bundle/plugin 模板

当前仍需开发者自己补的部分有：

- 如果你想走 **MCP server** 集成，当前 repo 还没有内建 `agent-exporter mcp serve` 这类原生 MCP server 命令面
- 所以本轮模板优先走 **CLI-first / skill-first / bundle-first** 路线，而不是伪造一个并不存在的 MCP server

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
