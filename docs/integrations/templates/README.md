# Integration Templates

这些模板不是“安装器”，而是已经整理好的接线材料。

你可以先这样理解：

> 这层负责把 `agent-exporter` 的命令、技能、bundle 内容和最小 MCP bridge
> 打包成开发者能直接复制的模板，
> 但不会替你猜宿主机上的最终安装目录。

## 目录怎么用

- `codex/`
  - `AGENTS.md`：可直接合并进项目级协作协议
  - `config.toml`：可直接摘进 Codex 的 MCP 配置
- `claude-code/`
  - `.claude/commands/*`：可直接复制进项目的 `.claude/commands/`
  - `.mcp.json`：Claude Code MCP snippet
- `openclaw-codex-bundle/`
  - 复制整个目录内容到你的 Codex-compatible OpenClaw bundle root
- `openclaw-claude-bundle/`
  - 复制整个目录内容到你的 Claude-compatible OpenClaw bundle root

## MCP bridge first-run contract

所有 `.mcp.json` / `config.toml` snippet 默认都只需要指向：

- `python3`
- `/absolute/path/to/agent-exporter/scripts/agent_exporter_mcp.py`

bridge 自己会按顺序尝试：

1. repo-local `target/release/agent-exporter`
2. repo-local `target/debug/agent-exporter`
3. `cargo run --manifest-path <repo>/Cargo.toml --bin agent-exporter --`

如果你有更稳定的安装方式，再额外设置：

- `AGENT_EXPORTER_BIN`
- `AGENT_EXPORTER_ARGS`

## OpenClaw 边界

OpenClaw 这层当前保证的是：

- bundle content 准备好了
- plugin metadata 准备好了
- bundle 内命令 / skill / `.mcp.json` 准备好了

OpenClaw 这层当前不保证的是：

- 自动发现你的 OpenClaw host 目录
- repo 内直接完成宿主安装
- host-specific runtime 已做 live proof
