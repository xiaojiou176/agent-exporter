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

| Packet | What it really is | Current truth |
| --- | --- | --- |
| Flagship public packet | repo front door + CLI quickstart + archive shell proof | primary public box |
| Release shelf | tagged public packet and release notes | published shelf |
| Host-native public skill packet | reviewer-facing folder for the local stdio governance bridge | secondary lane |
| Integration pack | repo-owned companion lane for Codex / Claude Code / OpenClaw wiring | secondary lane |
| Governance MCP bridge | read-only local bridge for archive / retrieval / governance evidence | secondary lane, not product identity |

## Canonical packet map

| Packet slice | Exact repo paths | Current status | What it does not prove |
| --- | --- | --- | --- |
| Flagship CLI-first packet | `README.md`, `docs/README.md`, `docs/archive-shell-proof.md` | active front door | hosted platform, remote runtime, or repo-wide MCP-first identity |
| Release shelf packet | release/tag plus release notes linked from `README.md` and `docs/README.md` | published shelf | latest `main` packet truth |
| Host-native public skill packet | `public-skills/README.md`, `public-skills/agent-exporter-archive-governance-workbench/README.md`, `public-skills/agent-exporter-archive-governance-workbench/manifest.yaml` | secondary lane; ClawHub live, OpenHands review-pending | flagship packet replacement or generic registry acceptance |
| Integration pack | `docs/integrations/README.md`, `docs/integrations/templates/README.md` | repo-owned companion lane | host-native runtime proof |
| Governance MCP bridge | `public-skills/agent-exporter-archive-governance-workbench/references/INSTALL.md`, `docs/integrations/README.md` | local stdio bridge only | hosted MCP endpoint or container runtime lane |

## Lane truth that stays honest

Keep these claims in their own buckets:

1. **CLI-first flagship**
   - connector readback
   - transcript export receipt
   - archive shell proof
2. **Host-native packet**
   - ClawHub live as a packet lane
   - OpenHands still review-pending
3. **Integration pack**
   - repo-owned wiring templates and doctor/onboard flows
4. **Governance bridge**
   - read-only local stdio bridge for archive/governance tasks

None of those four facts turn the repo into:

- a hosted archive platform
- a container/runtime repo
- a generic MCP-first product

## Heavy-lane order that fits the repo

If a later wave wants to push distribution harder, the honest order is:

1. keep the flagship CLI-first packet stable
2. keep archive shell proof as the first visible proof layer
3. keep host-native packet receipts scoped to that packet
4. keep integration and governance as companion lanes
5. do **not** invent Docker/runtime/container work to make the repo look more "platform-like"

## Misreadings to block early

Do **not** flatten current packet shape into:

- `submit-ready` for unrelated registries
- `listed-live` beyond the actual host-native packet lanes
- `MCP-first` product positioning
- runtime/container expectations for `agent-exporter`
