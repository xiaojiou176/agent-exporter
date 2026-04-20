---
title: agent-exporter
description: Archive and governance workbench for AI agent transcripts, with a quickstart path and archive-shell proof routing.
image: /assets/media/agent-exporter-social-card.png
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

<section class="ae-hero">
  <div class="ae-hero-main">
    <p class="ae-kicker">archive and governance workbench</p>
    <h1>Put transcript export, archive proof, and governance on one desk without pretending to be a hosted platform.</h1>
    <p class="ae-lead">
      <code>agent-exporter</code> helps you export one AI agent transcript into a local receipt, then expand that receipt into an archive and governance workbench.
      The CLI stays the real front door because the first honest thing this product should do is generate visible proof.
      This Pages home exists to lower orientation cost: understand the result, run the shortest path, then open the right lane.
    </p>
    <div class="ae-actions">
      <a class="ae-button ae-button-primary" href="#first-success">Try the first success path</a>
      <a class="ae-button" href="./promo-reel.html">Watch the promo reel</a>
      <a class="ae-button" href="./archive-shell-proof.html">Inspect archive shell proof</a>
    </div>
    <div class="ae-inline-links">
      <a class="ae-button" href="./launch-kit.html">Open launch kit</a>
      <a class="ae-button" href="https://github.com/xiaojiou176-open/agent-exporter">Open GitHub front door</a>
    </div>
    <p class="ae-caption">
      Pages is a <strong>companion docs surface</strong>.
      The primary surface remains the <strong>quickstart path</strong>.
    </p>
    <p class="ae-note">
      Published shelf note:
      the latest release is the frozen public packet,
      while this Pages surface may move ahead with repository-side truth on <code>main</code>.
    </p>
  </div>
  <div class="ae-hero-side ae-panel">
    <p class="ae-kicker">at a glance</p>
    <dl class="ae-glance-list">
      <div>
        <dt>Primary entrypoint</dt>
        <dd>quickstart path</dd>
      </div>
      <div>
        <dt>First proof</dt>
        <dd>one HTML transcript receipt plus one archive shell entrypoint</dd>
      </div>
      <div>
        <dt>Start here</dt>
        <dd>run the first success path before opening side lanes</dd>
      </div>
      <div>
        <dt>Boundary</dt>
        <dd>local-first proof surface, not a hosted runtime</dd>
      </div>
    </dl>
  </div>
</section>

<section class="ae-section">
  <div class="ae-section-head">
    <p class="ae-kicker">visual walkthrough</p>
    <h2>Take the 20-second pass only when it helps you orient faster.</h2>
    <p class="ae-lead">
      The promo reel is a compact visual companion for first-time reviewers.
      It does not replace the CLI quickstart or the archive shell proof.
      It exists to lower orientation cost before you run the real path.
    </p>
  </div>
  <figure class="ae-media-card">
    <a href="./promo-reel.html">
      <img src="./assets/media/agent-exporter-promo-poster.png" alt="Poster frame from the agent-exporter promo reel">
    </a>
    <figcaption class="ae-caption">
      Promo reel: a short visual walkthrough of the transcript-first workbench, proof ladder, and lane hierarchy.
    </figcaption>
  </figure>
  <div class="ae-proof-grid">
    <article class="ae-proof-card">
      <p class="ae-mini-label">share-ready asset</p>
      <h3>Social card</h3>
      <p>Need a single-frame preview for a post, chat share, or reviewer packet? Use the <a href="./assets/media/agent-exporter-social-card.png">social card image</a>.</p>
    </article>
    <article class="ae-proof-card">
      <p class="ae-mini-label">distribution-prep</p>
      <h3>Launch kit</h3>
      <p>Once the product story is already clear, open the <a href="./launch-kit.html">launch kit</a> for truthful copy variants and asset routing.</p>
    </article>
  </div>
</section>

<section class="ae-section">
  <div class="ae-section-head">
    <p class="ae-kicker">front door rule</p>
    <h2>Start with the shortest truthful path, then disclose the rest.</h2>
    <p class="ae-lead">
      Think of the product like a workshop.
      First you turn on the bench light, then you test one tool, and only after that do you open the cabinets.
      That is why the opening route stays fixed:
      <strong>CLI quickstart first, archive shell proof second, secondary lanes after that.</strong>
    </p>
  </div>
  <div class="ae-surface-grid">
    <article class="ae-surface-card">
      <p class="ae-mini-label">Primary</p>
      <h3>CLI quickstart</h3>
      <p>The main door proves the product can actually run, not just describe itself.</p>
    </article>
    <article class="ae-surface-card">
      <p class="ae-mini-label">First visible proof</p>
      <h3>Archive shell proof</h3>
      <p>The proof page explains what the local workbench already organizes and what it still must not overclaim.</p>
    </article>
    <article class="ae-surface-card">
      <p class="ae-mini-label">Progressive disclosure</p>
      <h3>Open the next lane only when you need it</h3>
      <p>Reports shell, integration evidence, and governance stay visible, but they do not compete for the first screen.</p>
    </article>
  </div>
</section>

<section id="first-success" class="ae-section">
  <div class="ae-section-head">
    <p class="ae-kicker">first success path</p>
    <h2>Three steps to a real local receipt.</h2>
    <p class="ae-lead">
      If you only want to answer “is this worth trying once?”, do not read every lane first.
      Run these three steps, in order, and let the product prove itself.
    </p>
  </div>
  <div class="ae-step-grid">
    <article class="ae-step">
      <span class="ae-step-number">01</span>
      <h3>Read the bench shape</h3>
      <p>See the local workbench structure before you point the repo at a real transcript.</p>
      <div class="ae-command">
        <pre><code>cargo run -- scaffold
cargo run -- connectors</code></pre>
      </div>
      <p class="ae-command-caption">You confirm the workspace shape and the current connector surface.</p>
    </article>
    <article class="ae-step">
      <span class="ae-step-number">02</span>
      <h3>Export one HTML transcript</h3>
      <p>Create one browsable receipt instead of guessing what the output will look like.</p>
      <div class="ae-command">
        <pre><code>cargo run -- export codex \
  --thread-id &lt;thread-id&gt; \
  --format html \
  --destination workspace-conversations \
  --workspace-root /absolute/path/to/repo</code></pre>
      </div>
      <p class="ae-command-caption">The result is an inspectable HTML receipt inside <code>.agents/Conversations/</code>.</p>
    </article>
    <article class="ae-step">
      <span class="ae-step-number">03</span>
      <h3>Publish the archive shell</h3>
      <p>Organize transcript, reports, and evidence into one local navigation surface.</p>
      <div class="ae-command">
        <pre><code>cargo run -- publish archive-index --workspace-root /absolute/path/to/repo</code></pre>
      </div>
      <p class="ae-command-caption">Now you have <code>.agents/Conversations/index.html</code> as the archive shell entrypoint.</p>
    </article>
  </div>
</section>

<section class="ae-section">
  <div class="ae-split">
    <article class="ae-split-card">
      <p class="ae-kicker">you will get</p>
      <h2>Concrete artifacts, not abstract readiness.</h2>
      <ul class="ae-bullet-list">
        <li>one HTML transcript receipt</li>
        <li>one archive shell entrypoint that links transcripts, reports, and evidence</li>
        <li>one reproducible local path from export to archive browsing</li>
      </ul>
    </article>
    <article class="ae-split-card">
      <p class="ae-kicker">this does not mean</p>
      <h2>Proof is still not platform theatre.</h2>
      <ul class="ae-bullet-list">
        <li>not a hosted archive platform</li>
        <li>not a live multi-user service</li>
        <li>not already <code>submit-ready</code></li>
        <li>not already <code>listed-live</code> across every secondary lane</li>
      </ul>
    </article>
  </div>
</section>

<section class="ae-section">
  <div class="ae-section-head">
    <p class="ae-kicker">proof ladder</p>
    <h2>Read the product in three increasing layers of confidence.</h2>
  </div>
  <div class="ae-proof-grid">
    <article class="ae-proof-card">
      <p class="ae-mini-label">L1</p>
      <h3>CLI front door</h3>
      <p>The CLI can walk a new visitor through the truthful first path.</p>
    </article>
    <article class="ae-proof-card">
      <p class="ae-mini-label">L2</p>
      <h3>Transcript receipt</h3>
      <p>Transcript export leaves behind a browsable HTML receipt, not just a hidden file.</p>
    </article>
    <article class="ae-proof-card">
      <p class="ae-mini-label">L3</p>
      <h3>Archive shell</h3>
      <p>The archive shell organizes the local workbench into one navigable surface with clear side lanes.</p>
    </article>
  </div>
</section>

<section class="ae-section">
  <div class="ae-section-head">
    <p class="ae-kicker">open the right next door</p>
    <h2>Use progressive disclosure instead of opening every cabinet at once.</h2>
  </div>
  <details class="ae-disclosure">
    <summary>Archive shell proof</summary>
    <p>Open this when you want the shortest public explanation of what the archive shell proves and what it still must not claim.</p>
  </details>
  <details class="ae-disclosure">
    <summary>Repo map</summary>
    <p>Open this when you already understand the product sentence and now need to know where files, lanes, and shells live.</p>
  </details>
  <details class="ae-disclosure">
    <summary>Secondary packet and listing ledger</summary>
    <p>Use this only when lane truth matters. Packet and listing status belong in the second ring, not the first screen.</p>
  </details>
  <details class="ae-disclosure">
    <summary>Latest release shelf</summary>
    <p>Use the release shelf when you need the newest tagged packet rather than the newest repository-side wording on <code>main</code>.</p>
  </details>
</section>

## Release Shelf Truth

<section class="ae-section">
  <div class="ae-split">
    <article class="ae-split-card">
      <p class="ae-kicker">published shelf</p>
      <h2>Use the latest release when you need the newest published packet.</h2>
      <ul class="ae-bullet-list">
        <li>tagged release notes</li>
        <li>the frozen packet links for that tag</li>
        <li>release notes for the shipped cut</li>
        <li>the packet state already frozen into a release</li>
      </ul>
    </article>
    <article class="ae-split-card">
      <p class="ae-kicker">repository-side truth</p>
      <h2>Use the repo front door and Pages docs when you need the newest repository-side truth on <code>main</code>.</h2>
      <ul class="ae-bullet-list">
        <li>front-door wording and CTA order</li>
        <li>packet and lane truth that moved after the last tag</li>
        <li>docs or governance hardening not yet republished</li>
      </ul>
    </article>
  </div>
  <p class="ae-note">
    These are neighboring shelves, not the same shelf.
    A newer <code>main</code> can sharpen wording and proof hierarchy before the next release is cut.
  </p>
</section>
</main>
