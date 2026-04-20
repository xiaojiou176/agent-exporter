# Distribution Packet Ledger

This page is the repo-wide shipping label for `agent-exporter`.

Use it when you need a clean answer to this question:

> Which packet is the flagship public box, which packets are only secondary
> lanes, and why this repo should not be mistaken for a runtime/container
> warehouse?

Think of the repo like a workshop.
The CLI workbench is the main bench in the middle. Integration and governance
lanes are side stations. They help the same workbench; they do not turn the
shop into a different factory.

## Current packet split

> Freshness note:
> treat this ledger as repo-side truth that must be rechecked before any new public packet cut.
> The latest release can lag behind `main`, and that lag is expected as long as it is explicitly disclosed.
> On `2026-04-20`, repo-owned local smoke, public-surface contract tests, and promo asset rebuild all ran fresh again; live public URL reachability remains pinned to the last published verification until the next Pages-facing publish loop.

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
| Release shelf packet | release/tag plus release notes linked from `README.md` and `docs/README.md` | published shelf | latest `main` packet truth; this shelf can lag behind `main` and must say so explicitly |
| Host-native public skill packet | `public-skills/README.md`, `public-skills/agent-exporter-archive-governance-workbench/README.md`, `public-skills/agent-exporter-archive-governance-workbench/manifest.yaml` | secondary lane; ClawHub live, Goose review-pending, agent-skill.co blocked upstream, OpenHands closed-not-accepted, awesome-opencode not_honest_cargo_yet | flagship packet replacement, generic registry acceptance, or an opencode-native project/resource claim this repo does not honestly fit today |
| Integration pack | `docs/integrations/README.md`, `docs/integrations/templates/README.md` | repo-owned companion lane | host-native runtime proof |
| Governance MCP bridge | `public-skills/agent-exporter-archive-governance-workbench/references/INSTALL.md`, `docs/integrations/README.md` | local stdio bridge only | hosted MCP endpoint or container runtime lane |

## Fresh Verification Anchors

Use this ledger like a shipping log, not a myth shelf.

| Surface | Last verified (UTC) | Evidence handle | Owner boundary |
| --- | --- | --- | --- |
| GitHub repo front door | `2026-04-19` | public repo URL + repo description/homepage live readback | repo-owned |
| Pages landing | `2026-04-19` | live `https://xiaojiou176-open.github.io/agent-exporter/` smoke | repo-owned |
| Archive shell proof page | `2026-04-19` | live `https://xiaojiou176-open.github.io/agent-exporter/archive-shell-proof.html` smoke | repo-owned |
| Latest release shelf | `2026-04-19` | `gh release view v0.1.1` + live `/releases/latest` readback after frozen-shelf wording correction | repo-owned wording, platform-hosted chrome |
| Local stdio host packet descriptor | `2026-04-19` | raw `server.json` readback + `llms-install.md` truth check | repo-owned |
| ClawHub status | `2026-04-18` | packet manifest + host read-back receipt lane | external platform outcome after repo packet |
| Goose / agent-skill.co / OpenHands / awesome-opencode | `2026-04-18` | submission refs and packet ledger notes | platform / reviewer later |

## Lane truth that stays honest

Keep these claims in their own buckets:

1. **CLI flagship**
   - connector readback
   - transcript export receipt
   - archive shell proof
2. **Host-native packet**
   - ClawHub live as a packet lane
   - Goose is still review-pending
   - agent-skill.co is waiting on external Vercel authorization
   - OpenHands is closed-not-accepted and no longer an active upstream listing lane
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
