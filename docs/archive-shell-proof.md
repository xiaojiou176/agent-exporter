---
title: Archive Shell Proof
description: What the archive shell proves, what it does not prove, and how to reproduce it locally with agent-exporter.
---

<main id="main-content" role="main" markdown="1">

<section class="ae-hero">
  <div class="ae-hero-main">
    <p class="ae-kicker">proof, not platform theatre</p>
    <h1>Use this page to understand what the archive shell really proves before you assign it more authority than it has earned.</h1>
    <p class="ae-lead">
      The archive shell proof is the first public explanation layer after the CLI front door.
      It shows that transcript export, report routing, and governance evidence can already be organized into one
      <strong>inspectable</strong> reading surface.
      It does <strong>not</strong> turn the repo into a hosted product or a live remote service.
    </p>
    <div class="ae-actions">
      <a class="ae-button ae-button-primary" href="https://github.com/xiaojiou176-open/agent-exporter">Back to GitHub front door</a>
      <a class="ae-button" href="./repo-map.html">Open repo map</a>
      <a class="ae-button" href="./">Return to docs home</a>
    </div>
  </div>
  <div class="ae-hero-side ae-panel">
    <p class="ae-kicker">what this page is for</p>
    <dl class="ae-glance-list">
      <div>
        <dt>Audience</dt>
        <dd>a first-time reviewer trying to separate proof from overclaim</dd>
      </div>
      <div>
        <dt>Main question</dt>
        <dd>What does the archive shell already prove today?</dd>
      </div>
      <div>
        <dt>Boundary</dt>
        <dd>local workbench proof, not a hosted runtime</dd>
      </div>
    </dl>
  </div>
</section>

<section class="ae-section">
  <div class="ae-section-head">
    <p class="ae-kicker">what you should see</p>
    <h2>Read the artifacts first, then use diagrams as map legends.</h2>
    <p class="ae-lead">
      Before this page earns the right to show diagrams, it should tell you what concrete things the shortest truthful path is supposed to leave behind.
    </p>
  </div>
  <div class="ae-proof-grid">
    <article class="ae-proof-card">
      <p class="ae-mini-label">artifact 01</p>
      <h3>One HTML transcript receipt</h3>
      <p>You should get a browsable transcript receipt inside <code>.agents/Conversations/</code>, not just a hidden export file.</p>
    </article>
    <article class="ae-proof-card">
      <p class="ae-mini-label">artifact 02</p>
      <h3>One archive shell entrypoint</h3>
      <p>You should then get <code>.agents/Conversations/index.html</code> as the local navigation surface for transcripts, reports, and evidence.</p>
    </article>
    <article class="ae-proof-card">
      <p class="ae-mini-label">proof boundary</p>
      <h3>Still local-only proof</h3>
      <p>Those artifacts prove a local workbench path. They still do not prove a hosted runtime, a remote service, or a live app shell.</p>
    </article>
  </div>
</section>

<section class="ae-section">
  <div class="ae-section-head">
    <p class="ae-kicker">visual proof assets</p>
    <h2>Two diagrams, two jobs.</h2>
    <p class="ae-lead">
      The first diagram shows the workbench shape.
      The second shows the proof ladder from CLI to transcript receipt to archive shell.
      Read them like map legends after the artifacts, not like a product hype montage.
    </p>
  </div>
  <div class="ae-media-grid">
    <figure class="ae-media-card">
      <img src="./assets/archive-shell-proof.svg" alt="agent-exporter archive shell proof diagram">
      <figcaption class="ae-caption">Archive shell proof map: how transcripts, retrieval receipts, and governance evidence sit on the same local desk.</figcaption>
    </figure>
    <figure class="ae-media-card">
      <img src="./assets/proof-ladder.svg" alt="agent-exporter proof ladder from CLI to transcript receipt to archive shell">
      <figcaption class="ae-caption">Proof ladder: the order in which confidence should increase.</figcaption>
    </figure>
  </div>
</section>

<section class="ae-section">
  <div class="ae-split">
    <article class="ae-split-card">
      <p class="ae-kicker">what this proof actually shows</p>
      <h2>Real local workbench proof.</h2>
      <ul class="ae-proof-list">
        <li>transcript export can become a browsable HTML receipt</li>
        <li><code>publish archive-index</code> can organize transcripts, reports shell, and integration evidence into one inspectable archive shell</li>
        <li>the archive shell is already <strong>workbench proof</strong>: it can route a reader through local artifacts without pretending to be a hosted platform</li>
      </ul>
    </article>
    <article class="ae-split-card">
      <p class="ae-kicker">what this proof does not show</p>
      <h2>Do not promote proof into product theatre.</h2>
      <ul class="ae-proof-list">
        <li>this is not a hosted product demo</li>
        <li>this is not a GitHub Pages live archive shell</li>
        <li>this is not a remote multi-user platform</li>
        <li>this does not automatically mean <code>submit-ready</code>, <code>listed-live</code>, or <code>already approved</code></li>
      </ul>
    </article>
  </div>
</section>

<section class="ae-section">
  <div class="ae-section-head">
    <p class="ae-kicker">how to reproduce it locally</p>
    <h2>Three commands, one honest result.</h2>
    <p class="ae-lead">
      Treat this like checking a lab result for yourself.
      Do not trust the diagram alone; run the path and inspect the artifacts it leaves behind.
    </p>
  </div>
  <div class="ae-step-grid">
    <article class="ae-step">
      <span class="ae-step-number">01</span>
      <h3>Confirm source availability</h3>
      <div class="ae-command">
        <pre><code>cargo run -- connectors</code></pre>
      </div>
      <p class="ae-command-caption">You confirm which transcript sources are actually available before you export anything.</p>
    </article>
    <article class="ae-step">
      <span class="ae-step-number">02</span>
      <h3>Export one HTML transcript</h3>
      <div class="ae-command">
        <pre><code>cargo run -- export codex \
  --thread-id &lt;thread-id&gt; \
  --format html \
  --destination workspace-conversations \
  --workspace-root /absolute/path/to/repo</code></pre>
      </div>
      <p class="ae-command-caption">This leaves behind a concrete HTML receipt in <code>.agents/Conversations/</code>.</p>
    </article>
    <article class="ae-step">
      <span class="ae-step-number">03</span>
      <h3>Publish the archive shell</h3>
      <div class="ae-command">
        <pre><code>cargo run -- publish archive-index --workspace-root /absolute/path/to/repo</code></pre>
      </div>
      <p class="ae-command-caption">Now the transcript, reports shell, and integration evidence can be browsed as one local navigation surface.</p>
    </article>
  </div>
  <p class="ae-note">
    After a successful local run you should see <code>.agents/Conversations/*.html</code>, <code>.agents/Conversations/index.html</code>,
    and navigation paths from the transcript browser into reports shell and integration evidence.
  </p>
</section>

<section class="ae-section">
  <div class="ae-section-head">
    <p class="ae-kicker">proof ladder</p>
    <h2>Confidence should climb in order.</h2>
  </div>
  <div class="ae-proof-grid">
    <article class="ae-proof-card">
      <p class="ae-mini-label">L1</p>
      <h3>CLI front door</h3>
      <p>The CLI can export a transcript through the truthful front door path.</p>
    </article>
    <article class="ae-proof-card">
      <p class="ae-mini-label">L2</p>
      <h3>Transcript receipt</h3>
      <p>The export leaves a browsable HTML receipt rather than a hidden one-off file.</p>
    </article>
    <article class="ae-proof-card">
      <p class="ae-mini-label">L3</p>
      <h3>Archive shell</h3>
      <p>The archive shell organizes transcript, reports, and evidence into one navigable local surface.</p>
    </article>
  </div>
</section>

<section class="ae-section">
  <div class="ae-section-head">
    <p class="ae-kicker">next doors</p>
    <h2>After proof, choose the right frozen or reviewer-facing shelf.</h2>
    <p class="ae-lead">
      This page explains what the archive shell proves.
      Once that question is answered, the next question is usually one of four:
      do you need the visual companion, the launch kit, the latest published packet, or the wider packet/listing ledger?
    </p>
  </div>
  <div class="ae-proof-grid">
    <article class="ae-proof-card">
      <p class="ae-mini-label">visual companion</p>
      <h3>Promo reel</h3>
      <p>Use this when you want the shortest visual walkthrough before you open the proof or quickstart layers in detail.</p>
      <div class="ae-inline-links">
        <a class="ae-button" href="./promo-reel.html">Open promo reel</a>
      </div>
    </article>
    <article class="ae-proof-card">
      <p class="ae-mini-label">distribution-prep</p>
      <h3>Launch kit</h3>
      <p>Use this when the product story is already clear and you need truthful share-ready copy, asset routing, and packet-prep guidance.</p>
      <div class="ae-inline-links">
        <a class="ae-button" href="./launch-kit.html">Open launch kit</a>
      </div>
    </article>
    <article class="ae-proof-card">
      <p class="ae-mini-label">Published shelf</p>
      <h3>Latest release</h3>
      <p>Use this when you need the newest frozen public packet rather than the newest repository-side wording on <code>main</code>.</p>
      <div class="ae-inline-links">
        <a class="ae-button" href="https://github.com/xiaojiou176-open/agent-exporter/releases/latest">Open latest release</a>
      </div>
    </article>
    <article class="ae-proof-card">
      <p class="ae-mini-label">Host reviewer lane</p>
      <h3>Local stdio host packet</h3>
      <p>Use <code>llms-install.md</code> and <code>server.json</code> when the question is specifically about host-side wiring and review packet truth.</p>
      <div class="ae-inline-links">
        <a class="ae-button" href="https://github.com/xiaojiou176-open/agent-exporter/blob/main/llms-install.md">Open install note on GitHub</a>
        <a class="ae-button" href="https://github.com/xiaojiou176-open/agent-exporter/blob/main/server.json">Open server.json on GitHub</a>
      </div>
    </article>
    <article class="ae-proof-card">
      <p class="ae-mini-label">Packet truth</p>
      <h3>Distribution packet ledger</h3>
      <p>Use the ledger when you need platform/listing status, not when you are still trying to understand the product itself.</p>
      <div class="ae-inline-links">
        <a class="ae-button" href="./distribution-packet-ledger.html">Open packet ledger</a>
      </div>
    </article>
  </div>
</section>

<section class="ae-section">
  <div class="ae-note-grid">
    <article class="ae-note-card">
      <p class="ae-kicker">when to open this page</p>
      <h3>You need a proof explanation, not a product tour.</h3>
      <p>Open this page when someone needs to understand the current proof boundary before evaluating reports shell, integration evidence, or governance lanes.</p>
    </article>
    <article class="ae-note-card">
      <p class="ae-kicker">why this matters</p>
      <h3>Truthful product positioning depends on ordering.</h3>
      <p><code>agent-exporter</code> is already more than an export utility, but its first public proof still has to start with CLI quickstart, transcript export, and archive shell generation.</p>
    </article>
  </div>
</section>

</main>
