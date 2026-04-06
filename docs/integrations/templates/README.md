# Integration Templates

这些模板不是“安装器”，而是已经整理好的接线材料。

你可以先这样理解：

> 这层负责把 `agent-exporter` 的命令、技能、bundle 内容和最小 MCP bridge
> 打包成开发者能直接复制的模板，
> 但不会替你猜宿主机上的最终安装目录。

## 目录怎么用

- `codex/`
  - `AGENTS.md`：可直接合并进项目级协作协议
  - `.codex/config.toml`：可直接摘进 Codex 的 project-scoped MCP 配置
  - `.agents/skills/export-archive/SKILL.md`：可直接放进 repo-shared Codex skills
- `claude-code/`
  - `CLAUDE.md`：可直接复制进项目根目录
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

当前 bridge 除了 publish/search，也已经扩到 evidence 只读消费：

- `integration_evidence_list`
- `integration_evidence_diff`
- `integration_evidence_gate`
- `integration_evidence_explain`

## Repo-Owned Materializer

如果你不想手工复制模板目录，现在已经可以直接让仓库替你把这些材料化到一个显式 target：

```bash
agent-exporter integrate codex --target /absolute/path/to/codex-pack
agent-exporter integrate claude-code --target /absolute/path/to/claude-pack
agent-exporter integrate openclaw --target /absolute/path/to/openclaw-pack
```

它只会写到你明确给出的 target 下，不会静默改你的 `~/.codex`、`~/.claude` 或 OpenClaw host 目录。
现在它还会直接拒绝明显的 live host/global roots；请先使用一个中性的 staging target。

如果你需要把这次接线/体检结果保存下来，当前仓还支持把 evidence report 单独写到当前工作目录下的：

- `.agents/Integration/Reports/`

这样 report 会留在 integration 自己的抽屉里，而不是混进 transcript 或 retrieval report 抽屉。

## Repo-Owned Doctor

材料化之后，可以再用 doctor 做只读验收：

```bash
agent-exporter doctor integrations --platform codex --target /absolute/path/to/codex-pack
```

doctor 会检查：

- target 里该有的模板文件是否齐
- MCP bridge script 是否存在
- repo-local launcher 解析是否可用
- target 内容是否已经和当前 repo 重新材料化后的版本漂移
- 当前 launcher 是否真的还能执行 `connectors`
- 当前 readiness 是 `ready / partial / missing` 哪一层

如果当前 launcher 只能回退到 `cargo run`，doctor 会保守停在 `partial`。
它不会为了给你一个更好看的状态，而在只读模式下偷偷触发 build。

另外，doctor 现在还会按平台补最关键的 shape checks：

- Codex：`.codex/config.toml`
- Claude Code：`.mcp.json`
- OpenClaw：bundle/plugin manifests 和 `.mcp.json`

当前这层还进一步收紧了：

- Codex `command` + 非空 `args` 数组
- Claude `CLAUDE.md` + `.claude/commands/*.md` 的 pack 形状

## Repo-Owned Onboarding

如果你更想要一条一次性更顺手的 first-run 路径，现在已经可以直接用：

```bash
agent-exporter onboard codex --target /absolute/path/to/codex-pack
agent-exporter onboard claude-code --target /absolute/path/to/claude-pack
agent-exporter onboard openclaw --target /absolute/path/to/openclaw-pack
```

它会：

1. 先材料化 pack
2. 再跑只读 doctor
3. 最后打印更清楚的人话 summary 和 next steps

## OpenClaw 边界

OpenClaw 这层当前保证的是：

- bundle content 准备好了
- plugin metadata 准备好了
- bundle 内命令 / skill / `.mcp.json` 准备好了

OpenClaw 这层当前不保证的是：

- 自动发现你的 OpenClaw host 目录
- repo 内直接完成宿主安装
- host-specific runtime 已做 live proof
