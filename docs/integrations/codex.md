# Codex Integration

## 适用场景

如果你的团队主要在 Codex 里工作，当前最稳的接入方式不是等一个未来的 MCP server，
而是先把 `agent-exporter` 作为一个 **CLI-first repo utility** 接进项目协作协议。

## 当前最稳接法

1. 先运行 `agent-exporter integrate codex --target <DIR>` 材料化一份可审计的 Codex pack
   - `<DIR>` 应该是 staging/pack 目录，不要直接指向 `~/.codex`
2. 在项目里保留或合并 `AGENTS.md`
3. 如果你要把 repo-shared skill 也一起带上，就连同 `.agents/skills/export-archive/` 一起放进项目
4. 给团队一个固定约定：当需要导出 / 发布 / 检索 / 保存 report 时，直接调用 `agent-exporter`
5. 如果你的 Codex 运行时支持 MCP，再补 project-scoped `.codex/config.toml`

## Doctor

材料化后，可以用：

```bash
agent-exporter doctor integrations --platform codex --target <DIR>
```

去确认 target 里的 `AGENTS.md`、`.codex/config.toml`、`.agents/skills/export-archive/`、bridge script 路径和 launcher readiness 是否已经到 `ready`。

## Onboard

如果你更想要一条一次性更顺手的 first-run 路径，可以直接用：

```bash
agent-exporter onboard codex --target <DIR>
```

这条主链也会拒绝直接把材料写进 `~/.codex` 这类 live host/global root。

它会把 materialize + doctor + next steps 串在一起。

## MCP first-run 说明

Codex 这条线现在已经不要求你先硬编码一个 release binary 路径。

当前模板默认只需要：

1. 一个 repo checkout
2. `python3`
3. repo 内的 `scripts/agent_exporter_mcp.py`
4. 至少满足下面三条中的一条
   - repo-local `target/release/agent-exporter`
   - repo-local `target/debug/agent-exporter`
   - 本机可用的 `cargo`

如果你已经有自己固定的安装 binary，再显式覆盖：

- `AGENT_EXPORTER_BIN`
- `AGENT_EXPORTER_ARGS`

## 推荐命令

```bash
agent-exporter export codex --thread-id <thread-id>
agent-exporter publish archive-index --workspace-root <repo>
agent-exporter search semantic --workspace-root <repo> --query "login issues" --save-report
agent-exporter search hybrid --workspace-root <repo> --query "thread-1" --save-report
```

## 模板

见：

- `templates/codex/AGENTS.md`
- `templates/codex/config.toml`
- `templates/codex/.agents/skills/export-archive/SKILL.md`

## 当前诚实边界

- 这条接法今天是 **真实可用** 的
- 当前 repo 已经内建一个最小 stdio MCP bridge：`scripts/agent_exporter_mcp.py`
- 当前 bridge 暴露的是 publish/search 工具面，不是全量 CLI
- CLI-first 仍然是最稳 front door；MCP 是对 publish/search 的轻量接线，不是第二套产品面
