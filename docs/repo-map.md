---
title: Repo Map
description: Public repo map for the agent-exporter workbench, with clear routing between primary and secondary surfaces.
---

# Repo Map

这张页的目标是回答一句很朴素的话：

> 这仓的东西都放在哪里？

## Front Door Hierarchy

- **Primary front door:** GitHub repo + CLI quickstart
- **First public proof layer:** archive shell proof page
- **Secondary surfaces:** reports shell、integration pack、read-only governance MCP bridge

## Repository Layout

- `src/cli.rs`: CLI entrypoint and command routing
- `src/connectors/`: Codex / Claude Code source adapters
- `src/core/`: transcript / archive contract and host-safety enforcement
- `src/output/`: archive shell, search report, integration evidence rendering
- `docs/`: public docs companion surface
- `docs/integrations/`: repo-owned integration pack guidance
- `docs/reference/`: upstream contracts, reading lists, host-safety boundary
- `policies/`: integration evidence policy packs and governance baselines

## Public Truth Boundary

- Pages 是公开 companion surface，不是 hosted runtime
- archive shell proof 是 tracked public explanation page，不是 live app
- integration pack 与 governance MCP bridge 仍然是 secondary surfaces，必须各自记账、各自过线
