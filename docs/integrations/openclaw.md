# OpenClaw Integration

## 适用场景

OpenClaw 当前最稳的接法，是走它已经支持的 **bundle/plugin** 兼容层。

说得更直白一点：

> OpenClaw 能吃 Codex / Claude 生态的 bundle 结构，
> 所以我们先给开发者一套本地 bundle 模板，
> 而不是硬造一个还不存在的 repo-native OpenClaw runtime。

## 当前最稳接法

1. 先运行 `agent-exporter integrate openclaw --target <DIR>` 材料化一份 OpenClaw bundle pack
   - `<DIR>` 应该是 staging/pack 目录本身，不要直接指向 live OpenClaw bundle/plugin root
2. 在材料化结果里选一个 bundle 变体
   - `openclaw-codex-bundle/`
   - `openclaw-claude-bundle/`
3. 把对应目录里的内容复制进你的 OpenClaw bundle/plugin root
4. 让 bundle 内的 skills / commands 继续调用本地 `agent-exporter`
5. 如果你要 OpenClaw 直接吃 MCP，再把 bundle 内的 `.mcp.json` 一起带上

## Doctor

材料化后，可以用：

```bash
agent-exporter doctor integrations --platform openclaw --target <DIR>
```

去检查 bundle/plugin 元数据、命令/skill 文件、`.mcp.json` 以及 repo-local bridge readiness。

## Onboard

如果你更想要一条一次性更顺手的 first-run 路径，可以直接用：

```bash
agent-exporter onboard openclaw --target <DIR>
```

它会把 materialize + doctor + next steps 串在一起，但仍然不会替你静默装进 OpenClaw host。
同时，它也会拒绝把 `--target` 直接指到 OpenClaw bundle/plugin root。

## Bundle first-run 说明

OpenClaw 这条线最重要的不是“有没有模板”，而是“不要把模板误当成一个已经 host-verified 的 runtime”。

当前 repo 已经准备好的，是：

- bundle metadata
- Codex-compatible skills / commands content
- Claude-compatible commands content
- 可一起打包的 `.mcp.json`

当前你仍然需要自己决定的，是：

- 你的 OpenClaw 安装把 bundle/plugin 根目录放在哪里
- 你要让 `.mcp.json` 走 repo-local build、还是显式绑定 `AGENT_EXPORTER_BIN` / `AGENT_EXPORTER_ARGS`

换句话说：

> 这层已经是“可复制进宿主的 bundle content”，
> 但不是“repo 直接替你发现并安装到 OpenClaw 宿主里”。

## 模板

- `templates/openclaw-codex-bundle/`
- `templates/openclaw-claude-bundle/`

## 当前诚实边界

- 这些模板今天可以作为 **bundle content** 使用
- 当前 repo 已经内建最小 stdio MCP bridge，所以模板里附带了 `.mcp.json` snippet
- 但当前 bridge 只覆盖 publish/search，不代表整个 CLI 全量变成 MCP
- 当前 repo 声称的是 **bundle-content readiness**，不是 OpenClaw host runtime 已经 repo-local live proof
