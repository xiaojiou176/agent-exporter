---
title: agent-exporter
description: Local-first archive and governance workbench for AI agent transcripts, with CLI-first onboarding and archive-shell proof routing.
---

# agent-exporter

`agent-exporter` 是一个 **local-first archive and governance workbench for AI agent transcripts**。

这张 Pages 首页不是第二扇主门。
它只是一个更轻的公开 companion surface，帮助第一次路过的人先看懂：

1. 这仓到底是什么
2. 第一条成功路径怎么跑
3. archive shell proof 到底证明了什么

## Front Door Rule

- **Primary Surface:** `CLI-first`
- **Flagship Public Packet:** GitHub repo + CLI quickstart + archive shell proof
- **Secondary Surfaces:** local archive shell / reports shell、repo-owned integration pack、read-only governance MCP bridge

换句话说：

> Pages 负责把门口的话说清楚。
> 真正的主门还是 GitHub repo 里的 CLI quickstart。

## First Success In 3 Steps

1. 查看当前 connector 路线图

```bash
cargo run -- connectors
```

2. 导出一份 HTML transcript 到当前 workspace

```bash
cargo run -- export codex \
  --thread-id <thread-id> \
  --format html \
  --destination workspace-conversations \
  --workspace-root /absolute/path/to/repo
```

3. 生成 archive shell proof

```bash
cargo run -- publish archive-index --workspace-root /absolute/path/to/repo
```

成功信号：

- `.agents/Conversations/*.html` transcript export
- `.agents/Conversations/index.html` archive shell
- 这份 archive shell 是 **local-first HTML receipt**，不是 hosted service

## Start Here

- [GitHub repo front door](https://github.com/xiaojiou176-open/agent-exporter)
- [Archive shell proof](./archive-shell-proof.md)
- [Repo map](./repo-map.md)
- [Latest release](https://github.com/xiaojiou176-open/agent-exporter/releases/latest)

## Current Public Boundary

- Pages 是 **companion docs surface**，不是另一套 primary surface
- Archive shell proof page 是 **公开解释页**，不是 live hosted archive shell
- reports shell、integration pack、read-only governance MCP bridge 仍然是 secondary surfaces
- 当前不能 claim：`submit-ready`、`already approved`、`MCP-first`
