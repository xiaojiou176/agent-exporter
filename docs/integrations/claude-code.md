# Claude Code Integration

## 适用场景

Claude Code 当前最稳的接法，是把 `agent-exporter` 作为 **project skills / commands**
接进 `.claude/` 目录。

## 当前最稳接法

1. 先运行 `agent-exporter integrate claude-code --target <DIR>` 材料化一份可审计的 Claude Code pack
   - `<DIR>` 应该是 staging/pack 目录，不要直接指向 `~/.claude*`
2. 在项目里加入 `CLAUDE.md`
3. 在项目里加入 `.claude/commands/` 或 `.claude/skills/`
4. 让命令直接调用本地 `agent-exporter`
5. 如果你想走 MCP，再加 `.mcp.json` 指向 `scripts/agent_exporter_mcp.py`

## Doctor

材料化后，可以用：

```bash
agent-exporter doctor integrations --platform claude-code --target <DIR>
```

去检查 `CLAUDE.md`、`.claude/commands/*`、`.mcp.json`、bridge script 路径和 launcher readiness。

## Onboard

如果你更想要一条一次性更顺手的 first-run 路径，可以直接用：

```bash
agent-exporter onboard claude-code --target <DIR>
```

这条主链也会拒绝直接把材料写进 `~/.claude*` 这类 live Claude home roots。

它会把 materialize + doctor + next steps 串在一起。

如果你想把这次接线结果留成一张可复查的本地 evidence report，可以再加：

```bash
agent-exporter onboard claude-code --target <DIR> --save-report
```

或：

```bash
agent-exporter doctor integrations --platform claude-code --target <DIR> --save-report
```

报告会写到当前工作目录下的 `.agents/Integration/Reports/`，不会混进 transcript 或 retrieval report 壳。

## MCP first-run 说明

Claude Code 这条线现在默认按“repo checkout 直接接线”来理解。

也就是说，模板不再把 first-run 绑死在一条预构建 release binary 路径上。  
bridge 会优先尝试 repo-local build 产物；如果你还没提前 build，它会继续尝试 `cargo run`。

你真正需要提前确认的只有：

1. `python3` 可用
2. `scripts/agent_exporter_mcp.py` 路径写对
3. 你的本机要么已经有 repo-local binary，要么能跑 `cargo`

如果你更想绑定一个稳定安装好的 executable，再显式设置：

- `AGENT_EXPORTER_BIN`
- `AGENT_EXPORTER_ARGS`

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
- `templates/claude-code/CLAUDE.md`
- `templates/claude-code/.mcp.json`

## 当前诚实边界

- 这些命令模板今天就能用
- 当前 repo 已经内建最小 stdio MCP bridge，但当前工具面只覆盖 publish/search
- Claude Code 当前最稳的入口仍然是 commands / skills；MCP 是可选接线层，不是替代整个命令面
