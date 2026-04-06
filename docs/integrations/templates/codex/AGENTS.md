# agent-exporter Project Workflow

当你在这个项目里需要：

- 导出 Codex / Claude Code transcript
- 更新 archive shell
- 保存 semantic / hybrid retrieval reports

优先调用这些本地命令：

```bash
agent-exporter publish archive-index --workspace-root .
agent-exporter search semantic --workspace-root . --query "<query>" --save-report
agent-exporter search hybrid --workspace-root . --query "<query>" --save-report
```

规则：

1. archive shell 和 reports shell 都是本地静态工件
2. retrieval execution 仍然只在 CLI
3. 不要把浏览器页面当成检索执行器
