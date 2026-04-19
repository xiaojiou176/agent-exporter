---
title: Docs Index
description: Documentation hub for agent-exporter public pages, proof shelves, and companion lanes.
image: /assets/media/agent-exporter-social-card.png
permalink: /docs-index.html
---

# Documentation Index

<main id="main-content" role="main" markdown="1">

<section class="ae-hero">
  <div class="ae-hero-main">
    <p class="ae-kicker">documentation hub</p>
    <h1>Use this hub when you need the right explanation shelf, not when you need to rediscover the front door.</h1>
    <p class="ae-lead">
      This is the documentation hub for `agent-exporter`.
      It is designed to reduce navigation overhead:
      first decide what kind of question you have, then open the matching shelf.
      If you are still trying to understand the product itself, go back to the flagship front door first.
    </p>
    <div class="ae-actions">
      <a class="ae-button ae-button-primary" href="https://github.com/xiaojiou176-open/agent-exporter">Open GitHub front door</a>
      <a class="ae-button" href="./promo-reel.html">Open promo reel</a>
      <a class="ae-button" href="./launch-kit.html">Open launch kit</a>
      <a class="ae-button" href="./archive-shell-proof.html">Open archive proof</a>
      <a class="ae-button" href="./repo-map.html">Open repo map</a>
    </div>
    <p class="ae-note">
      Published shelf note:
      the latest release is the frozen public packet,
      while this docs surface may move ahead with repository-side truth on `main`.
    </p>
  </div>
  <aside class="ae-hero-side ae-panel">
    <p class="ae-kicker">this hub is best for</p>
    <dl class="ae-glance-list">
      <div>
        <dt>Main use</dt>
        <dd>finding the right lane after you already know what the product is</dd>
      </div>
      <div>
        <dt>Not for</dt>
        <dd>first-time product orientation</dd>
      </div>
      <div>
        <dt>Ground rule</dt>
        <dd>quickstart path first, archive proof second, side lanes after that</dd>
      </div>
    </dl>
  </aside>
</section>

<section class="ae-section">
  <div class="ae-section-head">
    <p class="ae-kicker">docs hub in one screen</p>
    <h2>Pick the question before you pick the file.</h2>
  </div>

| Question | Open this | Why |
| --- | --- | --- |
| "What is the product and what should I do first?" | [GitHub front door](https://github.com/xiaojiou176-open/agent-exporter) | this is still the flagship front door |
| "Can I get a fast visual walkthrough before I run it?" | [Promo reel](./promo-reel.html) | this is the compact visual companion, not the proof boundary |
| "What should I share after the product story is clear?" | [Launch kit](./launch-kit.html) | this is the second-ring distribution-prep lane |
| "What does the first proof actually prove?" | [Archive shell proof](./archive-shell-proof.html) | this is the first public proof layer |
| "Where do the surfaces and files live?" | [Repo map](./repo-map.html) | this is the repo-side map |
| "What is the packet/listing status?" | [Distribution packet ledger](./distribution-packet-ledger.html) | packet truth belongs in the second ring |

</section>

<section class="ae-section">
  <div class="ae-section-head">
    <p class="ae-kicker">product snapshot</p>
    <h2>Lock the product truth first, then branch.</h2>
  </div>
  <div class="ae-surface-grid">
    <article class="ae-surface-card">
      <p class="ae-mini-label">Product kernel</p>
      <h3>Archive and governance workbench</h3>
      <p>`agent-exporter` is an <strong>archive and governance workbench</strong> that stays grounded in inspectable artifacts and proof rather than a hosted platform story.</p>
    </article>
    <article class="ae-surface-card">
      <p class="ae-mini-label">Primary surface</p>
      <h3>Quickstart path</h3>
      <p>The operational door is still the CLI quickstart, not a browser runtime.</p>
    </article>
    <article class="ae-surface-card">
      <p class="ae-mini-label">Secondary surfaces</p>
      <h3>Archive, reports, integrations, governance</h3>
      <p>These lanes are real and landed, but they still do not replace the first screen.</p>
    </article>
    <article class="ae-surface-card">
      <p class="ae-mini-label">Flagship packet</p>
      <h3>GitHub repo + CLI quickstart + archive shell proof</h3>
      <p>This remains the most honest public packet for the current stage.</p>
    </article>
  </div>
  <p class="ae-note">
    The front door starts with the CLI quickstart.
    The archive shell proof is the first visible proof layer.
    Integration pack and governance lanes stay visible, but they do not own the first screen.
  </p>
</section>

<section class="ae-section">
  <div class="ae-section-head">
    <p class="ae-kicker">reading order by intent</p>
    <h2>Do not read the docs like a phone book.</h2>
  </div>

| If you are trying to... | Read this next |
| --- | --- |
| decide whether the repo is worth trying once | [GitHub front door](https://github.com/xiaojiou176-open/agent-exporter) |
| understand the first truthful visible proof | [Archive shell proof](./archive-shell-proof.html) |
| understand the structure of the workbench | [Repo map](./repo-map.html) |
| inspect registry / packet / listing truth | [Distribution packet ledger](./distribution-packet-ledger.html) |
| inspect host-native packet lanes | [Public skills packet](https://github.com/xiaojiou176-open/agent-exporter/blob/main/public-skills/README.md) |

</section>

<section class="ae-section">
  <div class="ae-section-head">
    <p class="ae-kicker">first success orientation</p>
    <h2>Prove the desk stands up locally before opening side lanes.</h2>
  </div>
  <div class="ae-step-grid">
    <article class="ae-step">
      <span class="ae-step-number">01</span>
      <h3>Scaffold the workbench</h3>
      <div class="ae-command">
        <pre><code>cargo run -- scaffold</code></pre>
      </div>
    </article>
    <article class="ae-step">
      <span class="ae-step-number">02</span>
      <h3>Read the connector surface</h3>
      <div class="ae-command">
        <pre><code>cargo run -- connectors</code></pre>
      </div>
    </article>
    <article class="ae-step">
      <span class="ae-step-number">03</span>
      <h3>Export one transcript receipt</h3>
      <div class="ae-command">
        <pre><code>cargo run -- export codex --thread-id &lt;thread-id&gt; --format html --destination workspace-conversations --workspace-root /absolute/path/to/repo</code></pre>
      </div>
    </article>
    <article class="ae-step">
      <span class="ae-step-number">04</span>
      <h3>Publish the archive shell</h3>
      <div class="ae-command">
        <pre><code>cargo run -- publish archive-index --workspace-root /absolute/path/to/repo</code></pre>
      </div>
    </article>
  </div>
  <p class="ae-note">
    After that succeeds, you should see the scaffolded workbench shape, `.agents/Conversations/*.html` transcript receipts,
    `.agents/Conversations/index.html`, and a local-only HTML proof path rather than a hosted page.
  </p>
</section>

<section class="ae-section">
  <div class="ae-section-head">
    <p class="ae-kicker">public docs entry points</p>
    <h2>Each page answers a different class of question.</h2>
  </div>
  <div class="ae-proof-grid">
    <article class="ae-proof-card">
      <p class="ae-mini-label">Companion docs home</p>
      <h3>Pages landing</h3>
      <p>`https://xiaojiou176-open.github.io/agent-exporter/`</p>
      <div class="ae-inline-links">
        <a class="ae-button" href="./">Open Pages landing</a>
      </div>
    </article>
    <article class="ae-proof-card">
      <p class="ae-mini-label">Proof page</p>
      <h3>Archive shell proof</h3>
      <p>`https://xiaojiou176-open.github.io/agent-exporter/archive-shell-proof.html`</p>
      <div class="ae-inline-links">
        <a class="ae-button" href="./archive-shell-proof.html">Open archive shell proof</a>
      </div>
    </article>
    <article class="ae-proof-card">
      <p class="ae-mini-label">Structure page</p>
      <h3>Repo map</h3>
      <p>`https://xiaojiou176-open.github.io/agent-exporter/repo-map.html`</p>
      <div class="ae-inline-links">
        <a class="ae-button" href="./repo-map.html">Open repo map</a>
      </div>
    </article>
    <article class="ae-proof-card">
      <p class="ae-mini-label">Published shelf</p>
      <h3>Latest release</h3>
      <p>`https://github.com/xiaojiou176-open/agent-exporter/releases/latest`</p>
      <div class="ae-inline-links">
        <a class="ae-button" href="https://github.com/xiaojiou176-open/agent-exporter/releases/latest">Open latest release</a>
      </div>
    </article>
  </div>
</section>

<section class="ae-section">
  <div class="ae-split">
    <article class="ae-split-card">
      <p class="ae-kicker">secondary local stdio host packet</p>
      <h2>Keep this packet small, narrow, and honest.</h2>
      <ul class="ae-bullet-list">
        <li><a href="https://github.com/xiaojiou176-open/agent-exporter/blob/main/llms-install.md">llms-install.md on GitHub</a> for the shortest attach path</li>
        <li><a href="https://github.com/xiaojiou176-open/agent-exporter/blob/main/server.json">server.json on GitHub</a> for the canonical descriptor</li>
        <li>marketplace logo/proof tile only for reviewer lanes</li>
      </ul>
    </article>
    <article class="ae-split-card">
      <p class="ae-kicker">why it stays second-ring</p>
      <h2>Packet truth is narrower than product identity.</h2>
      <p>
        Use the bridge packet when the question is about registry or read-back lanes.
        If the question is still "What is this repo?" or "How do I get one successful result?",
        stay on the README + archive shell proof path.
      </p>
    </article>
  </div>
</section>

## Release Shelf Truth

<section class="ae-section">
  <div class="ae-split">
    <article class="ae-split-card">
      <p class="ae-kicker">published packet</p>
      <h2>Use the latest release shelf for the newest published packet.</h2>
      <ul class="ae-bullet-list">
        <li>the tagged release notes</li>
        <li>the frozen packet links for that tag</li>
        <li>release notes for the shipped packet</li>
        <li>the frozen packet state already inside a release</li>
      </ul>
    </article>
    <article class="ae-split-card">
      <p class="ae-kicker">repository-side truth</p>
      <h2>Use the repo/docs surface for the newest truth on `main`.</h2>
      <ul class="ae-bullet-list">
        <li>front-door wording</li>
        <li>packet and lane truth after the latest tag</li>
        <li>docs or governance hardening that moved ahead of the current release</li>
      </ul>
    </article>
  </div>
  <p class="ae-note">
    These shelves should stay conceptually aligned, but they are not the same shelf.
  </p>
</section>

<section class="ae-section">
  <div class="ae-section-head">
    <p class="ae-kicker">current surface snapshot</p>
    <h2>Read the product as a stack of proof lanes.</h2>
  </div>

| Layer | Current truth | First proof / entry |
| --- | --- | --- |
| CLI core | Codex `app-server` remains the canonical path; `local` and `claude-code` are landed | CLI quickstart in the [GitHub front door](https://github.com/xiaojiou176-open/agent-exporter) |
| Archive shell proof | `publish archive-index` generates the transcript browser, workspace backlinks, and the archive shell | `.agents/Conversations/index.html` |
| Reports shell | `search semantic|hybrid --save-report` generates retrieval receipts and the reports shell | `.agents/Search/Reports/index.html` |
| Integration pack | `integrate`, `doctor integrations`, and `onboard` are repo-owned companion lanes | `.agents/Integration/Reports/index.html` |
| MCP descriptor | root `server.json` is the canonical registry/read-back descriptor for the local stdio bridge | [`server.json`](https://github.com/xiaojiou176-open/agent-exporter/blob/main/server.json) |
| Governance lane | evidence, baselines, policy packs, and remediation now live in the local workbench | archive shell Decision Desk + integration evidence reports |

</section>

<section class="ae-section">
  <div class="ae-split">
    <article class="ae-split-card">
      <p class="ae-kicker">secondary lane truth</p>
      <h2>Use packet/listing files only when lane truth matters.</h2>
      <ul class="ae-bullet-list">
        <li><a href="./distribution-packet-ledger.html">distribution packet ledger</a></li>
        <li><a href="https://github.com/xiaojiou176-open/agent-exporter/blob/main/public-skills/README.md">public-skills packet README</a></li>
      </ul>
      <p>Those files hold `listed-live`, `review-pending`, `platform-not-accepted-yet`, `closed-not-accepted`, and `exact_blocker_with_fresh_evidence` states without turning packet status into the first screen.</p>
    </article>
    <article class="ae-split-card">
      <p class="ae-kicker">maintainer-only references</p>
      <h2>These files matter, but not for a first-time public reader.</h2>
      <ul class="ae-bullet-list">
        <li>`../AGENTS.md`</li>
        <li>`../CLAUDE.md`</li>
        <li>`./adr/ADR-0001-source-layering.md`</li>
        <li>`./adr/ADR-0002-codex-first-delivery.md`</li>
        <li>`./reference/*`</li>
      </ul>
    </article>
  </div>
</section>

</main>
