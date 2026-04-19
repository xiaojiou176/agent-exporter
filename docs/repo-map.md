---
title: Repo Map
description: Public repo map for the agent-exporter workbench, with clear routing between primary and secondary surfaces.
---

<main id="main-content" role="main" markdown="1">

<section class="ae-hero">
  <div class="ae-hero-main">
    <p class="ae-kicker">repo map</p>
    <h1>Use this map when you already understand the product and now need to know where each lane actually lives.</h1>
    <p class="ae-lead">
      This page answers one practical question:
      <strong>where does everything live in this repo?</strong>
      It is a routing page, not a second homepage.
      In other words, open this after the front door has already done its job.
    </p>
    <div class="ae-actions">
      <a class="ae-button ae-button-primary" href="./">Return to docs home</a>
      <a class="ae-button" href="./archive-shell-proof.html">Open archive proof</a>
      <a class="ae-button" href="https://github.com/xiaojiou176-open/agent-exporter">Open GitHub front door</a>
    </div>
  </div>
  <aside class="ae-hero-side ae-panel">
    <p class="ae-kicker">use this page for</p>
    <dl class="ae-glance-list">
      <div>
        <dt>Question</dt>
        <dd>Which file or lane should I open next?</dd>
      </div>
      <div>
        <dt>Best moment</dt>
        <dd>after you already understand the product sentence and first success path</dd>
      </div>
      <div>
        <dt>Not for</dt>
        <dd>first-time product orientation</dd>
      </div>
    </dl>
  </aside>
</section>

<section class="ae-section">
  <div class="ae-section-head">
    <p class="ae-kicker">front door hierarchy</p>
    <h2>The map only works if the order stays honest.</h2>
  </div>
  <div class="ae-proof-grid">
    <article class="ae-proof-card">
      <p class="ae-mini-label">Primary front door</p>
      <h3>GitHub repo + CLI quickstart</h3>
      <p>This is still the first path for a new visitor.</p>
    </article>
    <article class="ae-proof-card">
      <p class="ae-mini-label">First visible proof</p>
      <h3>Archive shell proof page</h3>
      <p>This explains what the repo can already prove without turning proof into hosted-product theatre.</p>
    </article>
    <article class="ae-proof-card">
      <p class="ae-mini-label">Secondary surfaces</p>
      <h3>Reports, integration, governance</h3>
      <p>These lanes are real and useful, but they still should not own the first screen.</p>
    </article>
  </div>
</section>

<section class="ae-section">
  <div class="ae-section-head">
    <p class="ae-kicker">choose by intent</p>
    <h2>Open the lane that matches the question you actually have.</h2>
  </div>

| If your question is... | Open this next | Why |
| --- | --- | --- |
| "How do I get one successful result?" | [GitHub front door](https://github.com/xiaojiou176-open/agent-exporter) | the flagship front door stays there |
| "Can I get the shortest visual walkthrough first?" | [Promo reel](./promo-reel.html) | this is the compact visual companion |
| "What should I share once the product story is already clear?" | [Launch kit](./launch-kit.html) | this is the second-ring distribution-prep lane |
| "What does the first proof actually prove?" | [Archive shell proof](./archive-shell-proof.html) | this is the proof explanation page |
| "Where are transcripts, reports, and evidence rendered?" | [`src/output/`](https://github.com/xiaojiou176-open/agent-exporter/tree/main/src/output) | this is the output shell layer |
| "Where are connector boundaries defined?" | [`src/connectors/`](https://github.com/xiaojiou176-open/agent-exporter/tree/main/src/connectors) | this is the source adapter layer |
| "Where does archive and governance logic live?" | [`src/core/`](https://github.com/xiaojiou176-open/agent-exporter/tree/main/src/core) | this is the contract and decision layer |
| "Where is the repo-owned integration lane?" | [`docs/integrations/`](https://github.com/xiaojiou176-open/agent-exporter/tree/main/docs/integrations) | this is the integration pack surface |
| "Where are the truth boundaries and upstream reading lists?" | [`docs/reference/`](https://github.com/xiaojiou176-open/agent-exporter/tree/main/docs/reference) | this is the constraint shelf |

</section>

<section class="ae-section">
  <div class="ae-section-head">
    <p class="ae-kicker">repository layout</p>
    <h2>Think in lanes, not just folders.</h2>
  </div>
  <div class="ae-surface-grid">
    <article class="ae-surface-card">
      <p class="ae-mini-label">Entry</p>
      <h3>`src/cli.rs`</h3>
      <p>The CLI entrypoint and command routing. This is still the operational front door.</p>
    </article>
    <article class="ae-surface-card">
      <p class="ae-mini-label">Source adapters</p>
      <h3>`src/connectors/`</h3>
      <p>Codex and Claude Code source boundaries. This is where transcript intake rules live.</p>
    </article>
    <article class="ae-surface-card">
      <p class="ae-mini-label">Contracts</p>
      <h3>`src/core/`</h3>
      <p>Archive, search, evidence, and host-safety rules. This is the product logic shelf.</p>
    </article>
    <article class="ae-surface-card">
      <p class="ae-mini-label">Readable surfaces</p>
      <h3>`src/output/`</h3>
      <p>Archive shell, search report, and integration evidence rendering. This is the visible workbench layer.</p>
    </article>
    <article class="ae-surface-card">
      <p class="ae-mini-label">Public docs</p>
      <h3>`docs/`</h3>
      <p>The companion public docs surface. It explains, routes, and proves, but does not replace the CLI front door.</p>
    </article>
    <article class="ae-surface-card">
      <p class="ae-mini-label">Promo lane</p>
      <h3>`docs/promo-reel.md`</h3>
      <p>The compact visual companion for first-time reviewers who want the shortest proof-aligned walkthrough.</p>
      <div class="ae-inline-links">
        <a class="ae-button" href="./promo-reel.html">Open promo reel</a>
      </div>
    </article>
    <article class="ae-surface-card">
      <p class="ae-mini-label">Launch lane</p>
      <h3>`docs/launch-kit.md`</h3>
      <p>The second-ring distribution-prep shelf for channel-ready copy, asset routing, and packet-safe sharing.</p>
      <div class="ae-inline-links">
        <a class="ae-button" href="./launch-kit.html">Open launch kit</a>
      </div>
    </article>
    <article class="ae-surface-card">
      <p class="ae-mini-label">Design law</p>
      <h3>`design-system/`</h3>
      <p>The repo-owned visual and IA doctrine for front door and workbench shells.</p>
    </article>
    <article class="ae-surface-card">
      <p class="ae-mini-label">Integration lane</p>
      <h3>`docs/integrations/`</h3>
      <p>Repo-owned integration pack guidance for Codex, Claude Code, and related side lanes.</p>
    </article>
    <article class="ae-surface-card">
      <p class="ae-mini-label">Reference shelf</p>
      <h3>`docs/reference/`</h3>
      <p>Upstream contracts, reading lists, and host-safety boundaries. Use this when you need truth-source detail.</p>
    </article>
    <article class="ae-surface-card">
      <p class="ae-mini-label">Governance artifacts</p>
      <h3>`policies/`</h3>
      <p>Integration evidence policy packs and governance baselines. This is the local rulebook shelf.</p>
    </article>
  </div>
</section>

<section class="ae-section">
  <div class="ae-split">
    <article class="ae-split-card">
      <p class="ae-kicker">public truth boundary</p>
      <h2>What this map should help you avoid misunderstanding.</h2>
      <ul class="ae-bullet-list">
        <li>Pages is a public companion surface, not a hosted runtime</li>
        <li>archive shell proof is a tracked explanation page, not a live app</li>
        <li>integration pack and governance MCP bridge remain secondary surfaces</li>
      </ul>
    </article>
    <article class="ae-split-card">
      <p class="ae-kicker">what to do next</p>
      <h2>Leave the map as soon as it has done its job.</h2>
      <ul class="ae-bullet-list">
        <li>go back to the <a href="https://github.com/xiaojiou176-open/agent-exporter">GitHub front door</a> for first success</li>
        <li>open <a href="./archive-shell-proof.html">archive shell proof</a> for proof interpretation</li>
        <li>open <a href="./promo-reel.html">promo reel</a> for the shortest visual walkthrough</li>
        <li>open <a href="./launch-kit.html">launch kit</a> for second-ring sharing and distribution-prep</li>
        <li>open <a href="./distribution-packet-ledger.html">distribution packet ledger</a> only when lane truth matters</li>
      </ul>
    </article>
  </div>
</section>

</main>
