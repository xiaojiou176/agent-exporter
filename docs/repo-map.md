---
title: Repo Map
description: Public repo map for the agent-exporter workbench, with clear routing between primary and secondary surfaces.
---

# Repo Map

This page answers one simple question:

> where does everything live in this repo?

## Front Door Hierarchy

- **Primary front door:** GitHub repo + CLI quickstart
- **First public proof layer:** archive shell proof page
- **Secondary surfaces:** reports shell, integration pack, read-only governance MCP bridge

## At A Glance

| Area | What lives there | Why it matters |
| --- | --- | --- |
| Primary path | CLI quickstart | gives a first-time visitor the fastest path to one successful result |
| Proof layer | archive shell proof page | explains what the repo can already prove |
| Secondary surfaces | reports shell / integration pack / governance MCP bridge | these lanes are real, but they do not take over the first screen |

## Repository Layout

- `src/cli.rs`: CLI entrypoint and command routing
- `src/connectors/`: Codex / Claude Code source adapters
- `src/core/`: transcript / archive contract and host-safety enforcement
- `src/output/`: archive shell, search report, and integration evidence rendering
- `design-system/`: front-door hierarchy, proof ladder, and boundary wording
- `docs/`: public docs companion surface
- `docs/integrations/`: repo-owned integration pack guidance
- `docs/reference/`: upstream contracts, reading lists, and host-safety boundaries
- `policies/`: integration evidence policy packs and governance baselines

## Public Truth Boundary

- Pages is a public companion surface, not a hosted runtime
- archive shell proof is a tracked public explanation page, not a live app
- integration pack and governance MCP bridge remain secondary surfaces and must clear their own truth/readiness lanes
