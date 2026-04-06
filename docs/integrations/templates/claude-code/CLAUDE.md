# agent-exporter Project Workflow

When you need to:

- publish the local archive shell
- save semantic retrieval reports
- save hybrid retrieval reports

prefer these local commands:

```bash
agent-exporter publish archive-index --workspace-root .
agent-exporter search semantic --workspace-root . --query "$ARGUMENTS" --save-report
agent-exporter search hybrid --workspace-root . --query "$ARGUMENTS" --save-report
```

Rules:

1. archive shell and reports shell stay local static artifacts
2. retrieval execution still stays in the CLI
3. browser pages organize and link artifacts; they do not execute retrieval
