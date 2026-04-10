---
title: Archive Shell Proof
description: What the archive shell proves, what it does not prove, and how to reproduce it locally with agent-exporter.
---

# Archive Shell Proof

这页的用途很简单：

> 让公开访客先看懂 `archive shell proof` 是什么，
> 但不把它误写成 hosted 平台或已经 live 的远程服务。

![agent-exporter archive shell proof diagram](./assets/archive-shell-proof.svg)

![agent-exporter proof ladder from CLI to transcript receipt to archive shell](./assets/proof-ladder.svg)

## What This Proof Actually Shows

- transcript export 可以写成可浏览的 HTML receipt
- `publish archive-index` 可以把 transcript/browser、reports shell、integration evidence 组织成一个 local-first archive shell
- archive shell 是 **workbench proof**，说明这仓已经能把 transcript 组织成可回看、可导航、可继续工作的本地前厅

## What This Proof Does Not Show

- 这不是 hosted product demo
- 这不是 GitHub Pages live archive shell
- 这不是远程多用户平台
- 这也不等于 `submit-ready`、`listed-live` 或 `already approved`

## How To Reproduce It Locally

```bash
cargo run -- connectors
cargo run -- export codex \
  --thread-id <thread-id> \
  --format html \
  --destination workspace-conversations \
  --workspace-root /absolute/path/to/repo
cargo run -- publish archive-index --workspace-root /absolute/path/to/repo
```

本地成功后，你会看到：

- `.agents/Conversations/*.html`
- `.agents/Conversations/index.html`
- 从 transcript/browser 继续走向 reports shell 和 integration evidence 的导航入口

## Proof Ladder

| Level | It proves | Current public artifact |
| --- | --- | --- |
| `L1` | CLI 命令能把 transcript 导出来 | README quickstart |
| `L2` | transcript 会留下可浏览 HTML receipt | `.agents/Conversations/*.html` |
| `L3` | archive shell 会把本地 workbench 组织成可导航前厅 | `archive-shell-proof.svg` + `proof-ladder.svg` |

## Why This Matters

`agent-exporter` 现在不是“只有导出”的小工具。
但今天它对外最应该先证明的，不是所有 side lane 都完成了，而是：

1. CLI quickstart 能跑
2. transcript 能导出
3. archive shell proof 能生成

这就是当前旗舰公开包的第一层可信证明。
