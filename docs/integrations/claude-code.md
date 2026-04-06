# Claude Code Integration

## 适用场景

Claude Code 当前最稳的接法，是把 `agent-exporter` 作为 **project skills / commands**
接进 `.claude/` 目录。

## 当前最稳接法

1. 在项目里加入 `.claude/commands/` 或 `.claude/skills/`
2. 让命令直接调用本地 `agent-exporter`
3. 如果你想走 MCP，再加 `.mcp.json` 指向 `scripts/agent_exporter_mcp.py`

## 推荐命令面

```bash
agent-exporter publish archive-index --workspace-root .
agent-exporter search semantic --workspace-root . --query "$ARGUMENTS" --save-report
agent-exporter search hybrid --workspace-root . --query "$ARGUMENTS" --save-report
```

## 模板

见：

- `templates/claude-code/.claude/commands/publish-archive.md`
- `templates/claude-code/.claude/commands/search-semantic-report.md`
- `templates/claude-code/.claude/commands/search-hybrid-report.md`
- `templates/claude-code/.mcp.json`

## 当前诚实边界

- 这些命令模板今天就能用
- 当前 repo 已经内建最小 stdio MCP bridge，但当前工具面只覆盖 publish/search
