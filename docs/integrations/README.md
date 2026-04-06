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

## Repo-Owned Entry Points

现在 integration pack 不再只是“模板目录”。

当前仓里已经有两条 repo-owned 接入入口：

- `agent-exporter integrate codex --target <DIR>`
- `agent-exporter integrate claude-code --target <DIR>`
- `agent-exporter integrate openclaw --target <DIR>`
- `agent-exporter doctor integrations --platform <platform> --target <DIR>`

你可以把它理解成：

> `integrate` 负责把 repo-owned 接线材料材料化到一个显式 target，
> `doctor` 负责用只读方式检查这份材料和 bridge/launcher readiness 到底处在 `ready / partial / missing` 哪一层。

这两条入口都守住同一个硬边界：

- 不静默写用户 `Home`
- 不自动扫描全局安装目录
- 不偷偷替你 build / install / mutate shell
- 不把 OpenClaw bundle content 夸大成 repo-native runtime

## First-Run Contract

这一层最容易踩坑的地方，不是模板不存在，而是第一次接线时把前置条件想得太乐观。

你可以先把 MCP bridge 理解成一个“本地转接头”：

- 你需要保留 repo checkout，因为 bridge 本体就是 `scripts/agent_exporter_mcp.py`
- 默认 first-run 不再要求你先手写 `AGENT_EXPORTER_BIN=/absolute/path/to/target/release/agent-exporter`
- bridge 会按这个顺序找本地执行入口：
  1. repo-local `target/release/agent-exporter`
  2. repo-local `target/debug/agent-exporter`
  3. `cargo run --manifest-path <repo>/Cargo.toml --bin agent-exporter --`
- 如果你本机已经有一个更稳定的安装方式，再显式设置 `AGENT_EXPORTER_BIN` / `AGENT_EXPORTER_ARGS`

说得更直白一点：

> 现在的模板默认更像“拿着 repo 直接接线”，
> 而不是“先自己猜一套 release binary 绝对路径再去改配置”。

## OpenClaw Boundary

OpenClaw 这一层当前准备好的是：

- Codex-compatible bundle content
- Claude-compatible bundle content
- bundle 内可直接一起带上的 `.mcp.json`

OpenClaw 这一层当前**没有**声称的是：

- repo-native OpenClaw runtime
- hosted plugin registry
- 自动发现你的 OpenClaw 安装目录

所以这层的正确理解是：

> `agent-exporter` 已经把“可复制进 bundle 的内容”准备好了，
> 但具体复制到你哪一个 OpenClaw host 目录，仍然由你的本机安装方式决定。

## 目录

- `codex.md`
- `claude-code.md`
- `openclaw.md`
- `templates/`
- `templates/README.md`

## 设计原则

1. **先交付真实可用的接线方式**
   - 不把“未来可能做的 MCP server”写成今天已经存在

2. **继续保持 local-first**
   - 模板默认调用本地 `agent-exporter` CLI

3. **让 artifact graph 可复用**
   - archive shell、reports shell、transcript、saved reports 继续是本地静态工件

4. **不要把集成模板写成平台壳**
   - 这层是接线包，不是新产品前台
