---
title: Archive Shell Proof
description: What the archive shell proves, what it does not prove, and how to reproduce it locally with agent-exporter.
---

# Archive Shell Proof

This page has one job:

> help a public visitor understand what the `archive shell proof` is,
> without turning it into a hosted platform claim or a fake live runtime.

![agent-exporter archive shell proof diagram](./assets/archive-shell-proof.svg)

![agent-exporter proof ladder from CLI to transcript receipt to archive shell](./assets/proof-ladder.svg)

## What This Proof Actually Shows

- transcript export can become a browsable HTML receipt
- `publish archive-index` can organize transcripts, reports shell, and integration evidence into one local-first archive shell
- the archive shell is **workbench proof**, meaning the repo can already organize transcript artifacts into a local reading-and-routing surface

## What This Proof Does Not Show

- this is not a hosted product demo
- this is not a GitHub Pages live archive shell
- this is not a remote multi-user platform
- this does not automatically mean `submit-ready`, `listed-live`, or `already approved`

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

After a successful local run, you should see:

- `.agents/Conversations/*.html`
- `.agents/Conversations/index.html`
- navigation paths from the transcript browser into reports shell and integration evidence

## Proof Ladder

| Level | It proves | Current public artifact |
| --- | --- | --- |
| `L1` | the CLI can export a transcript through the front door path | README quickstart |
| `L2` | transcript export leaves a browsable HTML receipt | `.agents/Conversations/*.html` |
| `L3` | the archive shell organizes the local workbench into one navigable surface | `archive-shell-proof.svg` + `proof-ladder.svg` |

## Why This Matters

`agent-exporter` is no longer a tiny "export only" utility.
But the first thing it should prove in public is still:

1. the CLI quickstart works
2. transcript export works
3. archive shell proof can be generated

That is the first trustworthy proof layer of the current flagship public packet.
