---
title: agent-exporter
description: Local-first archive and governance workbench for AI agent transcripts, with CLI-first onboarding and archive-shell proof routing.
---

<style>
.markdown-body a {
  color: #0550ae;
  text-decoration: underline;
  text-underline-offset: 0.16em;
}

.markdown-body .highlight .nb {
  color: #0a3069;
}
</style>

<main id="main-content" role="main" markdown="1">

# agent-exporter

`agent-exporter` is a **local-first archive and governance workbench for AI agent transcripts**.

This Pages home is not a second primary door.
It is a lighter public companion surface that helps a first-time visitor answer:

1. what this repo actually is
2. how to get the first successful result
3. what the archive shell proof really proves

## Front Door Rule

- **Primary Surface:** `CLI-first`
- **Flagship Public Packet:** GitHub repo + CLI quickstart + archive shell proof
- **Secondary Surfaces:** local archive shell / reports shell, repo-owned integration pack, read-only governance MCP bridge

In plain language:

> Pages explains the door.
> The real primary entrance is still the CLI quickstart in the GitHub repo.

## Run This First

If you only want to try the repo once, do not start by reading every surface.
Run these three steps first:

1. `cargo run -- connectors`
2. `cargo run -- export codex --thread-id <thread-id> --format html --destination workspace-conversations --workspace-root /absolute/path/to/repo`
3. `cargo run -- publish archive-index --workspace-root /absolute/path/to/repo`

## First Success In 3 Steps

1. Inspect the current connector surface

```bash
cargo run -- connectors
```

2. Export one HTML transcript into the current workspace

```bash
cargo run -- export codex \
  --thread-id <thread-id> \
  --format html \
  --destination workspace-conversations \
  --workspace-root /absolute/path/to/repo
```

3. Publish the archive shell proof

```bash
cargo run -- publish archive-index --workspace-root /absolute/path/to/repo
```

Success signals:

- `.agents/Conversations/*.html` transcript exports
- `.agents/Conversations/index.html` archive shell
- a **local-first HTML receipt**, not a hosted service

## You Will Get

- one HTML transcript receipt
- one local archive shell entrypoint
- one navigation chain from the transcript to reports shell and integration evidence

## This Does Not Mean

- not a hosted archive platform
- not a live multi-user service
- not already `submit-ready`
- not already `listed-live` across every secondary lane

## Host-Native Packet Status

The public skill packet already has lane-specific truth:

- **ClawHub:** `listed-live`
- **Goose Skills Marketplace:** `review-pending` via `block/Agent-Skills#24`
- **agent-skill.co source repo:** `platform-not-accepted-yet` via `heilcheng/awesome-agent-skills#180`
- **OpenHands/extensions:** `closed-not-accepted` via `OpenHands/extensions#162`

That does **not** change the product hierarchy.
The host-native packet remains a secondary public lane.

## Release Shelf Truth

Use the latest release entrypoint when you want the newest **published**
release packet.

Use the repo front door and Pages docs when you want the newest
**repository-side truth** on `main`.

Those are neighboring shelves, not the same shelf. A newer `main` can sharpen
public wording or packet truth before the next tagged release exists.

## Start Here

- [GitHub repo front door](https://github.com/xiaojiou176-open/agent-exporter)
- [Archive shell proof](https://xiaojiou176-open.github.io/agent-exporter/archive-shell-proof.html)
- [Repo map](./repo-map.md)
- [Latest release](https://github.com/xiaojiou176-open/agent-exporter/releases/latest)

## Current Public Boundary

- Pages is a **companion docs surface**, not another primary surface
- the archive shell proof page is a **public explanation page**, not a live hosted archive shell
- reports shell, integration pack, and the read-only governance MCP bridge remain secondary surfaces
- current public language must not claim `submit-ready`, `already approved`, or `MCP-first`

</main>
