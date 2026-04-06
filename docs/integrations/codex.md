# Codex Integration

## 适用场景

如果你的团队主要在 Codex 里工作，当前最稳的接入方式不是等一个未来的 MCP server，
而是先把 `agent-exporter` 作为一个 **CLI-first repo utility** 接进项目协作协议。

## 当前最稳接法

1. 在项目里保留或合并 `AGENTS.md`
2. 给团队一个固定约定：当需要导出 / 发布 / 检索 / 保存 report 时，直接调用 `agent-exporter`
3. 如果你的 Codex 运行时支持 MCP，再补 `config.toml` 里的 MCP server 配置

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

## 当前诚实边界

- 这条接法今天是 **真实可用** 的
- 当前 repo 已经内建一个最小 stdio MCP bridge：`scripts/agent_exporter_mcp.py`
- 当前 bridge 暴露的是 publish/search 工具面，不是全量 CLI
