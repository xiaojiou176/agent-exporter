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
  <div class="ae-hero-side ae-panel">
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
  </div>
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

<div class="ae-proof-grid">
  <article class="ae-proof-card">
    <p class="ae-mini-label">front door</p>
    <h3>How do I get one successful result?</h3>
    <p><a href="https://github.com/xiaojiou176-open/agent-exporter">Open the GitHub front door</a> because the flagship packet still lives there.</p>
  </article>
  <article class="ae-proof-card">
    <p class="ae-mini-label">visual companion</p>
    <h3>Can I get the shortest visual walkthrough first?</h3>
    <p><a href="./promo-reel.html">Open the promo reel</a> when a short motion pass will orient you faster than more prose.</p>
  </article>
  <article class="ae-proof-card">
    <p class="ae-mini-label">sharing lane</p>
    <h3>What should I share once the product story is already clear?</h3>
    <p><a href="./launch-kit.html">Open the launch kit</a> for second-ring distribution-prep assets and copy.</p>
  </article>
  <article class="ae-proof-card">
    <p class="ae-mini-label">proof</p>
    <h3>What does the first proof actually prove?</h3>
    <p><a href="./archive-shell-proof.html">Open archive shell proof</a> when the proof boundary itself is the question.</p>
  </article>
  <article class="ae-proof-card">
    <p class="ae-mini-label">renderers</p>
    <h3>Where are transcripts, reports, and evidence rendered?</h3>
    <p><a href="https://github.com/xiaojiou176-open/agent-exporter/tree/main/src/output"><code>src/output/</code></a> is the visible shell layer.</p>
  </article>
  <article class="ae-proof-card">
    <p class="ae-mini-label">connectors</p>
    <h3>Where are connector boundaries defined?</h3>
    <p><a href="https://github.com/xiaojiou176-open/agent-exporter/tree/main/src/connectors"><code>src/connectors/</code></a> is the source adapter layer.</p>
  </article>
  <article class="ae-proof-card">
    <p class="ae-mini-label">contracts</p>
    <h3>Where does archive and governance logic live?</h3>
    <p><a href="https://github.com/xiaojiou176-open/agent-exporter/tree/main/src/core"><code>src/core/</code></a> is the contract and decision layer.</p>
  </article>
  <article class="ae-proof-card">
    <p class="ae-mini-label">integration lane</p>
    <h3>Where is the repo-owned integration lane?</h3>
    <p><a href="https://github.com/xiaojiou176-open/agent-exporter/tree/main/docs/integrations"><code>docs/integrations/</code></a> is the integration pack surface.</p>
  </article>
  <article class="ae-proof-card">
    <p class="ae-mini-label">reference shelf</p>
    <h3>Where are the truth boundaries and upstream reading lists?</h3>
    <p><a href="https://github.com/xiaojiou176-open/agent-exporter/tree/main/docs/reference"><code>docs/reference/</code></a> is the constraint shelf.</p>
  </article>
</div>

</section>

<section class="ae-section">
  <div class="ae-section-head">
    <p class="ae-kicker">repository layout</p>
    <h2>Think in lanes, not just folders.</h2>
  </div>
  <div class="ae-surface-grid">
    <article class="ae-surface-card">
      <p class="ae-mini-label">Entry</p>
      <h3><code>src/cli.rs</code></h3>
      <p>The CLI entrypoint and command routing. This is still the operational front door.</p>
    </article>
    <article class="ae-surface-card">
      <p class="ae-mini-label">Local execution UI</p>
      <h3><code>src/ui/</code></h3>
      <p>The local Export Cockpit WebUI. It auto-discovers workspace-relevant Codex threads and workspace-local Claude sessions, runs the matching export path, then opens the archive shell.</p>
    </article>
    <article class="ae-surface-card">
      <p class="ae-mini-label">Source adapters</p>
      <h3><code>src/connectors/</code></h3>
      <p>Codex and Claude Code source boundaries. This is where transcript intake rules live.</p>
    </article>
  <article class="ae-surface-card">
    <p class="ae-mini-label">Contracts</p>
    <h3><code>src/core/</code></h3>
    <p>Archive, search, evidence, workbench case stitching, official-answer lifecycle, and host-safety rules. This is the product logic shelf.</p>
  </article>
  <article class="ae-surface-card">
    <p class="ae-mini-label">Readable surfaces</p>
    <h3><code>src/output/</code></h3>
    <p>Archive shell, family case views, official-answer workflow cards, search report, and integration evidence rendering. This is the visible workbench layer.</p>
  </article>
    <article class="ae-surface-card">
      <p class="ae-mini-label">Public docs</p>
      <h3><code>docs/</code></h3>
      <p>The companion public docs surface. It explains, routes, and proves, but does not replace the CLI front door.</p>
    </article>
    <article class="ae-surface-card">
      <p class="ae-mini-label">Promo lane</p>
      <h3><code>docs/promo-reel.md</code></h3>
      <p>The compact visual companion for first-time reviewers who want the shortest proof-aligned walkthrough.</p>
      <div class="ae-inline-links">
        <a class="ae-button" href="./promo-reel.html">Open promo reel</a>
      </div>
    </article>
    <article class="ae-surface-card">
      <p class="ae-mini-label">Launch lane</p>
      <h3><code>docs/launch-kit.md</code></h3>
      <p>The second-ring distribution-prep shelf for channel-ready copy, asset routing, and packet-safe sharing.</p>
      <div class="ae-inline-links">
        <a class="ae-button" href="./launch-kit.html">Open launch kit</a>
      </div>
    </article>
    <article class="ae-surface-card">
      <p class="ae-mini-label">Design law</p>
      <h3><code>design-system/</code></h3>
      <p>The repo-owned visual and IA doctrine for front door and workbench shells.</p>
    </article>
    <article class="ae-surface-card">
      <p class="ae-mini-label">Integration lane</p>
      <h3><code>docs/integrations/</code></h3>
      <p>Repo-owned integration pack guidance for Codex, Claude Code, and related side lanes.</p>
    </article>
    <article class="ae-surface-card">
      <p class="ae-mini-label">Reference shelf</p>
      <h3><code>docs/reference/</code></h3>
      <p>Upstream contracts, reading lists, and host-safety boundaries. Use this when you need truth-source detail.</p>
    </article>
    <article class="ae-surface-card">
      <p class="ae-mini-label">Governance artifacts</p>
      <h3><code>policies/</code></h3>
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
