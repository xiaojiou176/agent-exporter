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
- `agent-exporter onboard codex --target <DIR>`
- `agent-exporter onboard claude-code --target <DIR>`
- `agent-exporter onboard openclaw --target <DIR>`
- `agent-exporter doctor integrations --platform <platform> --target <DIR> --save-report`
- `agent-exporter onboard codex --target <DIR> --save-report`
- `agent-exporter onboard claude-code --target <DIR> --save-report`
- `agent-exporter onboard openclaw --target <DIR> --save-report`

你可以把它理解成：

> `integrate` 负责把 repo-owned 接线材料材料化到一个显式 target，
> `doctor` 负责用只读方式检查这份材料和 bridge/launcher readiness 到底处在 `ready / partial / missing` 哪一层。
> `onboard` 则把这两步再加上人话 next steps 串成一条 first-run 主链。
> `--save-report` 则把这次体检/引导结果保存成独立的 integration evidence artifact。

这两条入口都守住同一个硬边界：

- 不静默写用户 `Home`
- 不自动扫描全局安装目录
- 不偷偷替你 build / install / mutate shell
- 不把 OpenClaw bundle content 夸大成 repo-native runtime
- `integrate` / `onboard` 会直接拒绝明显的 live host/global roots，例如 `~/.codex`、`~/.claude*`，以及 direct OpenClaw bundle/plugin roots（例如 `bundles/<name>`、`plugins/<name>`）

## Doctor Hardening

当前 `doctor integrations` 已经不只是“看文件在不在”。

它现在还会额外检查两件更值钱的事：

1. 当前 target 里的材料，是否仍然和当前 repo 重新材料化后的版本一致
2. 当前 repo-local launcher，是否真的还能执行 `connectors`

如果当前 launcher 只能回退到 `cargo run`，doctor 会保守停在 `partial`。
原因很简单：这条命令可能触发 build，而 doctor 这条线承诺的是只读体检，不是顺手编译。

说得更直白一点：

> 如果你昨天材料化了一套接线包，今天这仓已经换了 launcher 路径或 bridge 路径，
> doctor 现在会更早把这种“接线包过期”揪出来。

## Platform-Aware Diagnostics

当前 doctor 还已经更进一步，不再只是做通用检查。

它现在会按平台补最关键的 shape checks：

- Codex：`.codex/config.toml` 是否真包含 project-scoped `mcp_servers.agent_exporter`
- Claude Code：`.mcp.json` 是否真是可解析的 project-scoped MCP config
- OpenClaw：bundle/plugin manifests 与 `.mcp.json` 是否真像一个合法 bundle

这仍然是文件级、target 级、只读的 doctor。
它不是 host runtime 验证器，也不会替你执行 OpenClaw install。

当前这一层还进一步收紧了两件事：

- Codex：不只看 `mcp_servers.agent_exporter` 在不在，还会看 `command` 和非空 `args`
- Claude Code：不只看 `.mcp.json`，还会看 `CLAUDE.md` 与 `.claude/commands/*.md` 是否像一份项目 pack

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

## Integration Evidence Pack

Phase 26 之后，integration 主链已经不只会“写 target、打印状态”。

现在它还会在当前工作目录下把结果保存到：

- `.agents/Integration/Reports/`

这层 evidence pack 的正确理解是：

- 它是 integration-owned local artifact
- 它可以被后续本地 artifact graph 链接和回看
- 它不会回流 `.agents/Conversations` transcript corpus
- 它也不会混进 `.agents/Search/Reports` retrieval report 壳
