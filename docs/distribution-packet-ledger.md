---
title: Distribution Packet Ledger
description: Truth shelf for agent-exporter packet, registry, listing, and published-release status.
image: /assets/media/agent-exporter-social-card.png
---

<main id="main-content" role="main" markdown="1">

<section class="ae-hero">
  <div class="ae-hero-main">
    <p class="ae-kicker">distribution packet ledger</p>
    <h1>Use this page when packet, registry, or release-shelf truth matters more than first-time product orientation.</h1>
    <p class="ae-lead">
      This page is the repo-wide shipping label for <code>agent-exporter</code>.
      It answers one narrow but important question:
      which packet is the flagship public box, which lanes are secondary, and which external registries are live, pending, blocked, or not an honest fit.
    </p>
    <div class="ae-actions">
      <a class="ae-button ae-button-primary" href="https://github.com/xiaojiou176-open/agent-exporter/releases/latest">Open latest release shelf</a>
      <a class="ae-button" href="https://github.com/xiaojiou176-open/agent-exporter">Open GitHub front door</a>
      <a class="ae-button" href="./archive-shell-proof.html">Open archive shell proof</a>
    </div>
    <p class="ae-caption">
      Keep this page in the second ring.
      The first screen still belongs to the flagship packet:
      <strong>GitHub repo + CLI quickstart + archive shell proof.</strong>
    </p>
  </div>
  <div class="ae-hero-side ae-panel">
    <p class="ae-kicker">at a glance</p>
    <dl class="ae-glance-list">
      <div>
        <dt>Flagship packet</dt>
        <dd>GitHub repo + CLI quickstart + archive shell proof</dd>
      </div>
      <div>
        <dt>Published shelf</dt>
        <dd><code>v0.1.4</code> is the newest frozen packet</dd>
      </div>
      <div>
        <dt>Live external lane</dt>
        <dd>Official MCP Registry and ClawHub</dd>
      </div>
      <div>
        <dt>Still blocked</dt>
        <dd>Smithery repo-root publish build contract</dd>
      </div>
    </dl>
  </div>
</section>

<section class="ae-section">
  <div class="ae-section-head">
    <p class="ae-kicker">packet truth in one screen</p>
    <h2>Read the packet hierarchy before you read the detailed ledger.</h2>
    <p class="ae-lead">
      Think of the repo like a workshop.
      The CLI workbench is the main bench in the middle.
      Integration, governance, and host packet lanes are side stations.
      They help the same workbench; they do not turn the shop into a different factory.
    </p>
  </div>
  <div class="ae-proof-grid">
    <article class="ae-proof-card">
      <p class="ae-mini-label">primary public box</p>
      <h3>Flagship packet</h3>
      <p>GitHub repo front door, CLI quickstart, and archive shell proof still own the product identity.</p>
    </article>
    <article class="ae-proof-card">
      <p class="ae-mini-label">frozen shelf</p>
      <h3>Latest release</h3>
      <p><code>v0.1.4</code> is the newest published shelf. It should match the current packet truth for this cut, even if later work moves ahead again on <code>main</code>.</p>
    </article>
    <article class="ae-proof-card">
      <p class="ae-mini-label">live listing</p>
      <h3>Official MCP Registry</h3>
      <p>The local stdio bridge is now published as <code>io.github.xiaojiou176-open/agent-exporter-mcp</code>.</p>
    </article>
    <article class="ae-proof-card">
      <p class="ae-mini-label">live listing</p>
      <h3>ClawHub</h3>
      <p>The host-native skill packet remains publicly discoverable as a secondary packet lane.</p>
    </article>
    <article class="ae-proof-card">
      <p class="ae-mini-label">still pending</p>
      <h3>Goose and awesome-agent-skills</h3>
      <p>Those lanes still depend on external maintainer acceptance and have not flipped to listed-live.</p>
    </article>
    <article class="ae-proof-card">
      <p class="ae-mini-label">real blocker</p>
      <h3>Smithery</h3>
      <p>The current repo-root publish path fails during Smithery's shttp bundle build, so this is a build-contract blocker, not a vague missing status.</p>
    </article>
  </div>
</section>

<section class="ae-section">
  <div class="ae-section-head">
    <p class="ae-kicker">how to use this page</p>
    <h2>Open the summary first. Drop into the ledger only when you need exact evidence.</h2>
  </div>
  <div class="ae-split">
    <article class="ae-split-card">
      <p class="ae-kicker">open this page when</p>
      <ul class="ae-bullet-list">
        <li>you need to know which packet is actually public</li>
        <li>you need registry/listing truth for host or reviewer lanes</li>
        <li>you need to separate published shelf truth from repository-side truth</li>
      </ul>
    </article>
    <article class="ae-split-card">
      <p class="ae-kicker">do not use this page for</p>
      <ul class="ae-bullet-list">
        <li>first-time product orientation</li>
        <li>the shortest try-it-once path</li>
        <li>turning packet status into product identity</li>
      </ul>
    </article>
  </div>
</section>

## Current packet split

> Freshness note:
> treat this ledger as repo-side truth that must be rechecked before any new public packet cut.
> The latest release can lag behind `main`, and that lag is expected as long as it is explicitly disclosed.
> On `2026-04-20`, repo-owned local smoke, public-surface contract tests, promo asset rebuild, live public URL rereads, and fresh external distribution readbacks all ran again.

| Packet | What it really is | Current truth |
| --- | --- | --- |
| Flagship public packet | repo front door + CLI quickstart + archive shell proof | primary public box |
| Promo reel | short visual walkthrough linked from the flagship surfaces | supporting orientation lane |
| Launch kit | second-ring distribution-prep page with share-ready copy and asset routing | supporting prep lane |
| Release shelf | tagged release notes plus frozen packet links | published shelf |
| Host-native public skill packet | reviewer-facing folder for the local stdio host packet | secondary lane |
| Integration pack | repo-owned companion lane for Codex / Claude Code / OpenClaw wiring | secondary lane |
| Governance MCP bridge | read-only local bridge for archive / retrieval / governance evidence | secondary lane, not product identity |

## Canonical packet map

| Packet slice | Exact repo paths | Current status | What it does not prove |
| --- | --- | --- | --- |
| Flagship CLI packet | `README.md`, `docs/README.md`, `docs/archive-shell-proof.md` | active front door | hosted platform, remote runtime, or repo-wide MCP product identity |
| Promo reel | `docs/promo-reel.md`, `docs/assets/media/agent-exporter-promo.mp4`, `docs/assets/media/agent-exporter-promo-poster.png`, `studio/agent-exporter-promo/**` | supporting orientation lane | proof boundary replacement, hosted demo, or public claim inflation |
| Launch kit | `docs/launch-kit.md`, `docs/assets/media/agent-exporter-social-card.png` | supporting distribution-prep lane | flagship packet replacement, channel-ready claim inflation, or release shelf truth override |
| Release shelf packet | release/tag plus release notes linked from `README.md` and `docs/README.md` | published shelf aligned to `v0.1.4` | latest `main` packet truth after later commits; future drift must still be disclosed explicitly |
| Host-native public skill packet | `public-skills/README.md`, `public-skills/agent-exporter-archive-governance-workbench/README.md`, `public-skills/agent-exporter-archive-governance-workbench/manifest.yaml` | secondary lane; Official MCP Registry live, ClawHub live, Goose review-pending, agent-skill.co blocked upstream, OpenHands closed-not-accepted, Smithery build-contract-blocked, awesome-opencode not_honest_cargo_yet | flagship packet replacement, generic registry acceptance, or an opencode-native project/resource claim this repo does not honestly fit today |
| Integration pack | `docs/integrations/README.md`, `docs/integrations/templates/README.md` | repo-owned companion lane | host-native runtime proof |
| Governance MCP bridge | `public-skills/agent-exporter-archive-governance-workbench/references/INSTALL.md`, `docs/integrations/README.md` | local stdio bridge only | hosted MCP endpoint or container runtime lane |

## Fresh Verification Anchors

Use this ledger like a shipping log, not a myth shelf.

| Surface | Last verified (UTC) | Evidence handle | Owner boundary |
| --- | --- | --- | --- |
| GitHub repo front door | `2026-04-20` | public repo URL + repo description/homepage live readback | repo-owned |
| Pages landing | `2026-04-20` | live `https://xiaojiou176-open.github.io/agent-exporter/` smoke + live Pages reread after `#42/#43` | repo-owned |
| Archive shell proof page | `2026-04-20` | live `https://xiaojiou176-open.github.io/agent-exporter/archive-shell-proof.html` smoke + proof-page reread | repo-owned |
| Latest release shelf | `2026-04-20` | `gh release view v0.1.4` + live `/releases/latest` readback for the aligned published packet | repo-owned wording, platform-hosted chrome |
| Local stdio host packet descriptor | `2026-04-20` | raw `server.json` readback + `llms-install.md` truth check | repo-owned |
| Official MCP Registry status | `2026-04-20` | `mcp-publisher validate server.json` + successful `mcp-publisher publish server.json` for `io.github.xiaojiou176-open/agent-exporter-mcp` | external platform outcome after repo packet |
| ClawHub status | `2026-04-20` | fresh public search hit for `clawhub.ai/plugins/@openclaw/agent-exporter` plus packet-manifest readback | external platform outcome after repo packet |
| Goose / agent-skill.co / OpenHands / Smithery / awesome-opencode | `2026-04-20` | fresh GitHub PR readback for `block/agent-skills#24`, `heilcheng/awesome-agent-skills#180`, `OpenHands/extensions#162`, plus a fresh Smithery publish attempt that failed during repo-root bundle build | platform / reviewer later |

## Lane truth that stays honest

Keep these claims in their own buckets:

1. **CLI flagship**
   - connector readback
   - transcript export receipt
   - archive shell proof
2. **Host-native packet**
   - Official MCP Registry is now live for `io.github.xiaojiou176-open/agent-exporter-mcp`
   - ClawHub live as a packet lane
   - Goose is still review-pending
   - agent-skill.co is waiting on external Vercel authorization
   - OpenHands is closed-not-accepted and no longer an active upstream listing lane
   - Smithery is not listed yet because the current repo-root publish path fails during the shttp bundle build
   - awesome-opencode is still not_honest_cargo_yet because this packet is not an opencode-native project/resource entry
3. **Integration pack**
   - repo-owned wiring templates and doctor/onboard flows
4. **Governance bridge**
   - read-only local stdio bridge for archive/governance tasks

None of those four facts turn the repo into:

- a hosted archive platform
- a container/runtime repo
- a generic MCP product

## Heavy-lane order that fits the repo

If a later wave wants to push distribution harder, the honest order is:

1. keep the flagship CLI packet stable
2. keep archive shell proof as the first visible proof layer
3. keep host-native packet receipts scoped to that packet
4. keep integration and governance as companion lanes
5. do **not** invent Docker/runtime/container work to make the repo look more "platform-like"

## Pre-Distribution Smoke

Before you cut a new public-facing packet, run the repo-owned smoke path instead
of trusting memory:

```bash
python3 scripts/public_surface_smoke.py --workspace-root /absolute/path/to/agent-exporter
```

What this smoke covers:

- local first-success path still works
- archive / reports / integration workbench shells still generate
- `onboard codex --save-report` still lands a ready local pack
- the promo reel page plus poster/video assets still exist in the repo-owned public packet
- current public front door, proof page, release shelf, and raw `server.json`
  are reachable

What it does **not** do:

- publish a new release
- submit to registries or marketplaces
- override the need for human taste review on public-facing copy and visuals

## Misreadings to block early

Do **not** flatten current packet shape into:

- `submit-ready` for unrelated registries
- `listed-live` beyond the actual host-native packet lanes
- `MCP`-led product positioning
- runtime/container expectations for `agent-exporter`

## Public-facing packet naming

Keep these names aligned in every public-facing surface:

- **Flagship packet** = GitHub repo + CLI quickstart + archive shell proof
- **Published packet** = latest tagged release notes plus frozen packet links
- **Local stdio host packet** = `llms-install.md` + `server.json` + reviewer-facing skill packet
- **Distribution packet ledger** = the truth shelf for packet/listing status

</main>
