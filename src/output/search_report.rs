use std::collections::BTreeMap;

use crate::core::archive_index::ArchiveIndexEntry;
use crate::core::search_report::SearchReportEntry;

use super::html::escape_html;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SearchReportKind {
    Semantic,
    Hybrid,
}

impl SearchReportKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Semantic => "semantic",
            Self::Hybrid => "hybrid",
        }
    }

    fn title(self) -> &'static str {
        match self {
            Self::Semantic => "Semantic Retrieval Report",
            Self::Hybrid => "Hybrid Retrieval Report",
        }
    }

    fn description(self) -> &'static str {
        match self {
            Self::Semantic => {
                "这份 report 记录的是纯语义检索结果。它像一次被保存下来的查询回执：检索仍然发生在 CLI，页面只负责把当时的结果和上下文保存下来。"
            }
            Self::Hybrid => {
                "这份 report 记录的是 hybrid retrieval 结果。它会把 semantic score 和 lexical metadata score 一起展示出来，方便你回看当时为什么是这个排序。"
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SearchReportHit {
    pub entry: ArchiveIndexEntry,
    pub primary_score: f32,
    pub semantic_score: Option<f32>,
    pub lexical_score: Option<f32>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SearchReportDocument {
    pub kind: SearchReportKind,
    pub query: String,
    pub generated_at: String,
    pub workspace_root: String,
    pub model_dir: String,
    pub index_path: String,
    pub total_documents: usize,
    pub reused_documents: usize,
    pub embedded_documents: usize,
    pub hits: Vec<SearchReportHit>,
}

pub fn render_search_report_document(report: &SearchReportDocument) -> String {
    let hit_cards = if report.hits.is_empty() {
        "<section class=\"empty-state\"><h2>No hits</h2><p>这次检索没有命中任何 transcript。你可以调整 query，或者回到 archive shell 再选择别的 lane。</p></section>".to_string()
    } else {
        report
            .hits
            .iter()
            .map(|hit| render_hit_card(report.kind, hit))
            .collect::<Vec<_>>()
            .join("\n")
    };

    format!(
        concat!(
            "<!DOCTYPE html>\n",
            "<html lang=\"zh-CN\">\n",
            "<head>\n",
            "  <meta charset=\"utf-8\">\n",
            "  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n",
            "  <title>{title}</title>\n",
            "  <meta name=\"agent-exporter:report-title\" content=\"{title}\">\n",
            "  <meta name=\"agent-exporter:report-kind\" content=\"{kind}\">\n",
            "  <meta name=\"agent-exporter:search-query\" content=\"{query}\">\n",
            "  <meta name=\"agent-exporter:generated-at\" content=\"{generated_at}\">\n",
            "  <style>\n{style}\n  </style>\n",
            "</head>\n",
            "<body>\n",
            "  <main class=\"page-shell\">\n",
            "    <header class=\"hero-card\">\n",
            "      <p class=\"eyebrow\">agent-exporter retrieval report</p>\n",
            "      <h1>{title}</h1>\n",
            "      <p class=\"hero-copy\">{description} 这页属于 local archive shell / reports shell 这条 secondary surface：检索动作仍然从 CLI 发起，这里保存的是可回看的 receipt，不是产品主门本身。真正的 primary front door 仍然是 CLI quickstart，archive shell proof 则是第一层可浏览证明。</p>\n",
            "      <dl class=\"meta-grid\">\n",
            "        <div><dt>Query</dt><dd><code>{query}</code></dd></div>\n",
            "        <div><dt>Generated</dt><dd><code>{generated_at}</code></dd></div>\n",
            "        <div><dt>Workspace</dt><dd><code>{workspace_root}</code></dd></div>\n",
            "        <div><dt>Model Dir</dt><dd><code>{model_dir}</code></dd></div>\n",
            "        <div><dt>Index Path</dt><dd><code>{index_path}</code></dd></div>\n",
            "        <div><dt>Documents</dt><dd><code>{documents}</code></dd></div>\n",
            "        <div><dt>Reused</dt><dd><code>{reused}</code></dd></div>\n",
            "        <div><dt>Embedded</dt><dd><code>{embedded}</code></dd></div>\n",
            "      </dl>\n",
            "      <div class=\"link-row\">\n",
            "        <a class=\"open-link\" href=\"../../Conversations/index.html\">Open archive shell</a>\n",
            "        <a class=\"open-link\" href=\"index.html\">Open reports shell</a>\n",
            "        <a class=\"open-link\" href=\"../../Integration/Reports/index.html\">Open integration reports</a>\n",
            "      </div>\n",
            "    </header>\n",
            "    <section class=\"card-grid\">\n",
            "{hit_cards}\n",
            "    </section>\n",
            "  </main>\n",
            "</body>\n",
            "</html>\n"
        ),
        title = escape_html(report.kind.title()),
        kind = escape_html(report.kind.as_str()),
        query = escape_html(&report.query),
        generated_at = escape_html(&report.generated_at),
        description = escape_html(report.kind.description()),
        workspace_root = escape_html(&report.workspace_root),
        model_dir = escape_html(&report.model_dir),
        index_path = escape_html(&report.index_path),
        documents = report.total_documents,
        reused = report.reused_documents,
        embedded = report.embedded_documents,
        hit_cards = hit_cards,
        style = search_report_style(),
    )
}

pub fn render_search_reports_index_document(
    archive_title: &str,
    generated_at: &str,
    reports: &[SearchReportEntry],
) -> String {
    let report_kind_facets = render_filter_buttons(
        "report-kind",
        "Report kind",
        summarize_reports_by(reports, |entry| {
            entry.report_kind.as_deref().unwrap_or("unknown")
        }),
    );
    let body = if reports.is_empty() {
        "<section class=\"empty-state\"><h2>No saved reports yet</h2><p>运行 <code>search semantic --save-report</code> 或 <code>search hybrid --save-report</code> 之后，这里会出现可重复打开的 retrieval report cards。</p></section>".to_string()
    } else {
        reports
            .iter()
            .map(render_report_index_card)
            .collect::<Vec<_>>()
            .join("\n")
    };

    format!(
        concat!(
            "<!DOCTYPE html>\n",
            "<html lang=\"zh-CN\">\n",
            "<head>\n",
            "  <meta charset=\"utf-8\">\n",
            "  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n",
            "  <title>{title}</title>\n",
            "  <style>\n{style}\n  </style>\n",
            "</head>\n",
            "<body>\n",
            "  <main class=\"page-shell\">\n",
            "    <header class=\"hero-card\">\n",
            "      <p class=\"eyebrow\">retrieval lane</p>\n",
            "      <p class=\"hero-kicker\">{title}</p>\n",
            "      <h1>Revisit saved retrieval work without leaving the archive workbench.</h1>\n",
            "      <p class=\"hero-copy\">当你已经做过 semantic 或 hybrid retrieval，想回看 query、排序和 report receipt 时，再来这页。它是 archive workbench 的侧门，不是主门；检索动作仍然发生在 CLI，这里只负责把已经保存下来的 retrieval evidence 组织成可回看的 shelf。</p>\n",
            "      <dl class=\"meta-grid\">\n",
            "        <div><dt>Generated</dt><dd><code>{generated_at}</code></dd></div>\n",
            "        <div><dt>Saved reports</dt><dd><code>{report_count}</code></dd></div>\n",
            "      </dl>\n",
            "      <div class=\"link-row\">\n",
            "        <a class=\"open-link\" href=\"../../Conversations/index.html\">Open archive shell</a>\n",
            "        <a class=\"open-link\" href=\"../../Integration/Reports/index.html\">Open integration reports</a>\n",
            "      </div>\n",
            "    </header>\n",
            "    <section class=\"route-grid\" aria-label=\"retrieval lane framing\">\n",
            "{route_cards}\n",
            "    </section>\n",
            "    <section class=\"search-bar\" aria-label=\"reports search\">\n",
            "      <label class=\"search-label\" for=\"report-search\">Report search</label>\n",
            "      <input id=\"report-search\" class=\"search-input\" type=\"search\" placeholder=\"Search title, query, report kind...\" autocomplete=\"off\">\n",
            "      <div class=\"facet-grid\">\n",
            "{report_kind_facets}\n",
            "      </div>\n",
            "      <p id=\"report-search-status\" class=\"search-status\">Showing <strong>{report_count}</strong> reports.</p>\n",
            "    </section>\n",
            "    <section class=\"card-grid\">\n",
            "{body}\n",
            "    </section>\n",
            "    <p id=\"report-empty-result\" class=\"empty-result\" hidden>No reports matched the current search.</p>\n",
            "  </main>\n",
            "  <script>\n{script}\n  </script>\n",
            "</body>\n",
            "</html>\n"
        ),
        title = escape_html(archive_title),
        generated_at = escape_html(generated_at),
        report_count = reports.len(),
        route_cards = render_reports_lane_glance(reports),
        report_kind_facets = report_kind_facets,
        body = body,
        style = search_report_style(),
        script = search_report_script(),
    )
}

fn render_reports_lane_glance(reports: &[SearchReportEntry]) -> String {
    let latest = reports
        .iter()
        .max_by_key(|entry| entry.generated_at.as_deref().unwrap_or(""));

    let latest_card = if let Some(latest) = latest {
        let query = latest.query.as_deref().unwrap_or("saved retrieval");
        let generated = latest.generated_at.as_deref().unwrap_or("unknown");
        format!(
            concat!(
                "<article class=\"route-card\">",
                "<p class=\"eyebrow\">Latest saved report</p>",
                "<h2>{title}</h2>",
                "<p>如果你只是想知道最近一次 retrieval 到底留下了什么 receipt，从这里进最快。</p>",
                "<p class=\"mono-inline\">query: <code>{query}</code></p>",
                "<p class=\"mono-inline\">generated: <code>{generated}</code></p>",
                "</article>"
            ),
            title = escape_html(&latest.title),
            query = escape_html(query),
            generated = escape_html(generated),
        )
    } else {
        "<article class=\"route-card\"><p class=\"eyebrow\">Latest saved report</p><h2>No report yet</h2><p>先在 CLI 里保存一次 semantic 或 hybrid retrieval，才有值得回看的 shelf。</p></article>".to_string()
    };

    format!(
        concat!(
            "<article class=\"route-card\">",
            "<p class=\"eyebrow\">Use this page when</p>",
            "<h2>You want retrieval receipts, not transcript browsing.</h2>",
            "<p>这里回答的是：我当时搜了什么、为什么是这个排序、最近保存过哪些 report。它不负责替代 transcript browser，也不负责做 integration governance judgement。</p>",
            "</article>",
            "<article class=\"route-card\">",
            "<p class=\"eyebrow\">Do not use this page for</p>",
            "<h2>First contact with the product.</h2>",
            "<p>如果你还在找“从哪开始”，先回 archive shell；如果你要看 onboarding、doctor、baseline 或 policy，去 integration evidence lane，而不是在这里绕路。</p>",
            "</article>",
            "{latest_card}"
        ),
        latest_card = latest_card,
    )
}

fn render_hit_card(kind: SearchReportKind, hit: &SearchReportHit) -> String {
    let connector = hit.entry.connector.as_deref().unwrap_or("unknown");
    let completeness = hit.entry.completeness.as_deref().unwrap_or("unknown");
    let transcript_href = transcript_href(&hit.entry.relative_href);
    let score_lines = match kind {
        SearchReportKind::Semantic => format!(
            "<p class=\"score-line\">semantic score: <code>{:.4}</code></p>",
            hit.primary_score
        ),
        SearchReportKind::Hybrid => format!(
            concat!(
                "<p class=\"score-line\">hybrid score: <code>{:.4}</code></p>",
                "<p class=\"score-line\">semantic score: <code>{:.4}</code></p>",
                "<p class=\"score-line\">lexical score: <code>{:.4}</code></p>"
            ),
            hit.primary_score,
            hit.semantic_score.unwrap_or(0.0),
            hit.lexical_score.unwrap_or(0.0),
        ),
    };

    format!(
        concat!(
            "<article class=\"entry-card\">",
            "<p class=\"eyebrow\">Transcript hit</p>",
            "<h2>{title}</h2>",
            "<div class=\"chip-row\">",
            "<span class=\"chip\">{connector}</span>",
            "<span class=\"chip\">{completeness}</span>",
            "</div>",
            "{score_lines}",
            "<p class=\"mono-inline\">thread: <code>{thread_id}</code></p>",
            "<p class=\"mono-inline\">file: <code>{file_name}</code></p>",
            "<p><a class=\"open-link\" href=\"{href}\">Open transcript</a></p>",
            "</article>"
        ),
        title = escape_html(&hit.entry.title),
        connector = escape_html(connector),
        completeness = escape_html(completeness),
        score_lines = score_lines,
        thread_id = escape_html(hit.entry.thread_id.as_deref().unwrap_or("unknown")),
        file_name = escape_html(&hit.entry.file_name),
        href = escape_html(&transcript_href),
    )
}

fn render_report_index_card(entry: &SearchReportEntry) -> String {
    let query_line = entry
        .query
        .as_deref()
        .map(|query| {
            format!(
                "<p class=\"mono-inline\">query: <code>{}</code></p>",
                escape_html(query)
            )
        })
        .unwrap_or_default();
    let generated_line = entry
        .generated_at
        .as_deref()
        .map(|generated| {
            format!(
                "<p class=\"mono-inline\">generated: <code>{}</code></p>",
                escape_html(generated)
            )
        })
        .unwrap_or_default();
    let report_href = entry.relative_href.trim_start_matches("./");
    let searchable_text = [
        entry.title.as_str(),
        entry.report_kind.as_deref().unwrap_or(""),
        entry.query.as_deref().unwrap_or(""),
        entry.generated_at.as_deref().unwrap_or(""),
        entry.file_name.as_str(),
    ]
    .join(" ")
    .to_lowercase();

    format!(
        concat!(
            "<article class=\"entry-card\" data-search-text=\"{searchable_text}\" data-report-kind=\"{report_kind}\">",
            "<p class=\"eyebrow\">Saved report</p>",
            "<h2>{title}</h2>",
            "<div class=\"chip-row\">{kind_chip}</div>",
            "{query_line}",
            "{generated_line}",
            "<p><a class=\"open-link\" href=\"{href}\">Open report</a></p>",
            "</article>"
        ),
        title = escape_html(&entry.title),
        kind_chip = chip(entry.report_kind.as_deref().unwrap_or("unknown")),
        query_line = query_line,
        generated_line = generated_line,
        href = escape_html(report_href),
        searchable_text = escape_html(&searchable_text),
        report_kind = escape_html(entry.report_kind.as_deref().unwrap_or("unknown")),
    )
}

fn chip(value: &str) -> String {
    format!("<span class=\"chip\">{}</span>", escape_html(value))
}

fn summarize_reports_by<F>(entries: &[SearchReportEntry], label: F) -> Vec<(String, usize)>
where
    F: Fn(&SearchReportEntry) -> &str,
{
    let mut counts = BTreeMap::new();
    for entry in entries {
        *counts.entry(label(entry).to_string()).or_insert(0usize) += 1;
    }
    counts.into_iter().collect()
}

fn render_filter_buttons(group: &str, label: &str, items: Vec<(String, usize)>) -> String {
    let mut buttons = vec![format!(
        "<button type=\"button\" class=\"facet-button is-active\" data-filter-group=\"{group}\" data-filter-value=\"all\">All</button>"
    )];
    buttons.extend(items.into_iter().map(|(name, count)| {
        format!(
            "<button type=\"button\" class=\"facet-button\" data-filter-group=\"{group}\" data-filter-value=\"{value}\">{label} <span>{count}</span></button>",
            value = escape_html(&name),
            label = escape_html(&name),
        )
    }));

    format!(
        concat!(
            "<section class=\"facet-section\" aria-label=\"{group}\">",
            "<p class=\"search-label\">{label}</p>",
            "<div class=\"facet-row\">{buttons}</div>",
            "</section>"
        ),
        group = escape_html(group),
        label = escape_html(label),
        buttons = buttons.join(""),
    )
}

fn transcript_href(relative_href: &str) -> String {
    format!(
        "../../Conversations/{}",
        relative_href.trim_start_matches("./")
    )
}

fn search_report_style() -> &'static str {
    r#"    :root {
      color-scheme: light;
      --bg:
        radial-gradient(circle at 12% 16%, rgba(37, 99, 235, 0.12), transparent 0 28%),
        radial-gradient(circle at 88% 10%, rgba(14, 165, 233, 0.10), transparent 0 24%),
        linear-gradient(180deg, #f8fbff 0%, #eef3fb 52%, #e8eef8 100%);
      --surface: rgba(255, 255, 255, 0.88);
      --surface-strong: rgba(255, 255, 255, 0.96);
      --surface-muted: rgba(248, 250, 252, 0.82);
      --ink: #0f172a;
      --ink-soft: #334155;
      --muted: #64748b;
      --line: rgba(15, 23, 42, 0.08);
      --line-strong: rgba(37, 99, 235, 0.18);
      --accent: #2563eb;
      --accent-strong: #1d4ed8;
      --accent-soft: rgba(37, 99, 235, 0.10);
      --shadow-border: 0 0 0 1px rgba(15, 23, 42, 0.06);
      --shadow-panel:
        0 24px 60px rgba(15, 23, 42, 0.07),
        0 0 0 1px rgba(255, 255, 255, 0.72) inset;
      --shadow-hero:
        0 28px 80px rgba(15, 23, 42, 0.10),
        0 0 0 1px rgba(255, 255, 255, 0.80) inset;
      --mono: "JetBrains Mono", "SFMono-Regular", "Menlo", monospace;
      --sans: "IBM Plex Sans", "SF Pro Text", -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
      --display: "IBM Plex Sans", -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
    }

    * { box-sizing: border-box; }
    html { scroll-behavior: smooth; }
    body {
      margin: 0;
      font-family: var(--sans);
      color: var(--ink);
      background: var(--bg);
      background-attachment: fixed;
    }

    body::before {
      content: "";
      position: fixed;
      inset: 0;
      pointer-events: none;
      background-image:
        linear-gradient(rgba(148, 163, 184, 0.08) 1px, transparent 1px),
        linear-gradient(90deg, rgba(148, 163, 184, 0.08) 1px, transparent 1px);
      background-size: 52px 52px;
      mask-image: linear-gradient(180deg, rgba(0, 0, 0, 0.60), transparent 82%);
      opacity: 0.26;
    }

    .page-shell {
      width: min(1180px, calc(100vw - 32px));
      margin: 0 auto;
      padding: 28px 0 64px;
      position: relative;
      z-index: 1;
    }

    .hero-card,
    .entry-card,
    .empty-state {
      position: relative;
      overflow: hidden;
      background: var(--surface);
      border: 1px solid var(--line);
      border-radius: 28px;
      box-shadow: var(--shadow-panel);
      backdrop-filter: blur(14px);
      padding: 24px;
    }

    .hero-card::before,
    .entry-card::before {
      content: "";
      position: absolute;
      inset: 0;
      pointer-events: none;
      background: linear-gradient(180deg, rgba(255, 255, 255, 0.65), transparent 28%);
    }

    .hero-card {
      margin-bottom: 24px;
      padding: 30px;
      border-radius: 32px;
      background:
        radial-gradient(circle at top left, rgba(37, 99, 235, 0.14), transparent 34%),
        linear-gradient(135deg, rgba(255, 255, 255, 0.92), rgba(248, 250, 252, 0.84)),
        var(--surface);
      box-shadow: var(--shadow-hero);
    }

    .eyebrow {
      margin: 0 0 10px;
      text-transform: uppercase;
      letter-spacing: 0.12em;
      font-family: var(--mono);
      font-size: 11px;
      color: var(--accent);
    }

    .hero-kicker {
      margin: 0 0 12px;
      color: var(--muted);
      font-family: var(--mono);
      font-size: 13px;
      letter-spacing: 0.04em;
    }

    h1, h2 {
      margin: 0 0 12px;
      line-height: 1.04;
      font-weight: 700;
      letter-spacing: -0.03em;
      font-family: var(--display);
      color: var(--ink);
    }

    h1 { font-size: clamp(32px, 4.2vw, 52px); }
    h2 { font-size: clamp(22px, 2.6vw, 30px); }

    .hero-copy,
    p {
      margin: 0;
      line-height: 1.72;
      color: var(--ink-soft);
      word-break: break-word;
    }

    .meta-grid {
      display: grid;
      gap: 12px;
      grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
      margin: 18px 0 0;
    }

    .route-grid {
      display: grid;
      gap: 18px;
      grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
      margin: 0 0 22px;
    }

    .route-card {
      position: relative;
      overflow: hidden;
      padding: 22px;
      border-radius: 24px;
      border: 1px solid var(--line);
      background: rgba(255, 255, 255, 0.84);
      box-shadow: var(--shadow-panel);
    }

    .meta-grid div {
      padding: 14px 16px;
      background: rgba(255, 255, 255, 0.76);
      border-radius: 18px;
      box-shadow: var(--shadow-border);
    }

    dt {
      margin-bottom: 6px;
      font-size: 11px;
      letter-spacing: 0.08em;
      text-transform: uppercase;
      color: var(--muted);
      font-family: var(--mono);
    }

    dd { margin: 0; line-height: 1.5; }

    .link-row {
      display: flex;
      flex-wrap: wrap;
      gap: 10px;
      margin-top: 18px;
    }

    .search-bar {
      display: grid;
      gap: 10px;
      margin: 0 0 18px;
      padding: 20px 22px;
      border-radius: 24px;
      border: 1px solid var(--line);
      background: var(--surface-muted);
      box-shadow: var(--shadow-panel);
    }

    .search-label {
      font-family: var(--mono);
      font-size: 11px;
      letter-spacing: 0.1em;
      text-transform: uppercase;
      color: var(--accent);
    }

    .search-input {
      width: 100%;
      padding: 13px 15px;
      border-radius: 16px;
      border: 1px solid var(--line);
      background: rgba(255, 255, 255, 0.96);
      color: var(--ink);
      font-family: var(--mono);
      font-size: 14px;
    }

    .search-status,
    .empty-result {
      color: var(--muted);
      font-size: 14px;
      line-height: 1.65;
    }

    .facet-grid {
      display: grid;
      gap: 12px;
    }

    .facet-row {
      display: flex;
      flex-wrap: wrap;
      gap: 8px;
    }

    .facet-button {
      border: 1px solid var(--line);
      background: rgba(255, 255, 255, 0.86);
      border-radius: 999px;
      padding: 8px 12px;
      color: var(--ink);
      font-family: var(--mono);
      font-size: 12px;
      cursor: pointer;
      transition: transform 140ms ease, border-color 140ms ease, background 140ms ease;
    }

    .facet-button:hover {
      transform: translateY(-1px);
      border-color: var(--line-strong);
    }

    .facet-button.is-active {
      background: var(--accent-soft);
      color: var(--accent-strong);
      border-color: var(--line-strong);
    }

    .chip span,
    .facet-button span {
      margin-left: 6px;
      opacity: 0.7;
    }

    .card-grid {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
      gap: 18px;
    }

    .chip-row {
      display: flex;
      flex-wrap: wrap;
      gap: 8px;
      margin-bottom: 12px;
    }

    .chip {
      display: inline-flex;
      align-items: center;
      border-radius: 999px;
      padding: 6px 10px;
      background: var(--accent-soft);
      color: var(--accent-strong);
      border: 1px solid var(--line-strong);
      font-family: var(--mono);
      font-size: 12px;
    }

    .mono-inline,
    code,
    .open-link {
      font-family: var(--mono);
    }

    code {
      padding: 0.16em 0.42em;
      border-radius: 999px;
      color: var(--ink);
      background: rgba(15, 23, 42, 0.05);
    }

    .mono-inline,
    .score-line {
      margin-top: 10px;
      color: var(--ink);
    }

    .open-link {
      display: inline-flex;
      margin-top: 14px;
      min-height: 42px;
      padding: 10px 14px;
      border-radius: 999px;
      border: 1px solid var(--line);
      text-decoration: none;
      color: var(--ink);
      background: rgba(255, 255, 255, 0.76);
      box-shadow: var(--shadow-border);
      transition: transform 160ms ease, border-color 160ms ease, background 160ms ease;
    }

    .open-link:hover {
      transform: translateY(-1px);
      background: rgba(255, 255, 255, 0.96);
      border-color: var(--line-strong);
    }

    a:focus-visible,
    button:focus-visible,
    input:focus-visible {
      outline: 3px solid rgba(37, 99, 235, 0.28);
      outline-offset: 4px;
    }

    @media (max-width: 720px) {
      .page-shell {
        width: min(100vw - 20px, 1080px);
        padding: 16px 0 28px;
      }

      .hero-card,
      .entry-card,
      .route-card,
      .empty-state {
        border-radius: 20px;
        padding: 18px;
      }

      .card-grid {
        grid-template-columns: 1fr;
      }
    }"#
}

fn search_report_script() -> &'static str {
    r#"    const input = document.getElementById('report-search');
    const status = document.getElementById('report-search-status');
    const empty = document.getElementById('report-empty-result');
    const cards = Array.from(document.querySelectorAll('.entry-card'));
    const buttons = Array.from(document.querySelectorAll('.facet-button'));
    const activeFilters = {
      'report-kind': 'all',
    };

    if (input && status && empty) {
      const update = () => {
        const query = input.value.trim().toLowerCase();
        let visible = 0;
        for (const card of cards) {
          const haystack = (card.getAttribute('data-search-text') || '').toLowerCase();
          const reportKind = (card.getAttribute('data-report-kind') || 'unknown').toLowerCase();
          const matchesQuery = !query || haystack.includes(query);
          const matchesKind = activeFilters['report-kind'] === 'all' || reportKind === activeFilters['report-kind'];
          const match = matchesQuery && matchesKind;
          card.hidden = !match;
          if (match) visible += 1;
        }
        status.innerHTML = `Showing <strong>${visible}</strong> report${visible === 1 ? '' : 's'}.`;
        empty.hidden = visible !== 0;
      };
      for (const button of buttons) {
        button.addEventListener('click', () => {
          const group = button.getAttribute('data-filter-group');
          const value = (button.getAttribute('data-filter-value') || 'all').toLowerCase();
          if (!group) return;
          activeFilters[group] = value;
          for (const peer of buttons.filter((candidate) => candidate.getAttribute('data-filter-group') === group)) {
            peer.classList.toggle('is-active', peer === button);
          }
          update();
        });
      }
      input.addEventListener('input', update);
      update();
    }"#
}

#[cfg(test)]
mod tests {
    use super::{
        SearchReportDocument, SearchReportHit, SearchReportKind, render_search_report_document,
        render_search_reports_index_document,
    };
    use crate::core::archive_index::ArchiveIndexEntry;
    use crate::core::search_report::SearchReportEntry;

    fn sample_hit() -> SearchReportHit {
        SearchReportHit {
            entry: ArchiveIndexEntry {
                file_name: "demo.html".to_string(),
                relative_href: "demo.html".to_string(),
                title: "Demo transcript".to_string(),
                connector: Some("codex".to_string()),
                thread_id: Some("thread-1".to_string()),
                completeness: Some("complete".to_string()),
                source_kind: Some("app-server-thread-read".to_string()),
                exported_at: Some("2026-04-05T00:00:00Z".to_string()),
                ai_summary_href: None,
            },
            primary_score: 0.88,
            semantic_score: Some(0.81),
            lexical_score: Some(0.67),
        }
    }

    #[test]
    fn render_search_report_document_renders_semantic_report() {
        let html = render_search_report_document(&SearchReportDocument {
            kind: SearchReportKind::Semantic,
            query: "login issue".to_string(),
            generated_at: "2026-04-05T12:00:00Z".to_string(),
            workspace_root: "/tmp/workspace".to_string(),
            model_dir: "/tmp/model".to_string(),
            index_path: "/tmp/index.json".to_string(),
            total_documents: 2,
            reused_documents: 1,
            embedded_documents: 1,
            hits: vec![sample_hit()],
        });

        assert!(html.contains("Semantic Retrieval Report"));
        assert!(html.contains("agent-exporter:report-kind"));
        assert!(html.contains("semantic score"));
        assert!(html.contains("../../Conversations/demo.html"));
        assert!(html.contains("Open reports shell"));
    }

    #[test]
    fn render_search_report_document_renders_hybrid_scores() {
        let html = render_search_report_document(&SearchReportDocument {
            kind: SearchReportKind::Hybrid,
            query: "thread-1".to_string(),
            generated_at: "2026-04-05T12:00:00Z".to_string(),
            workspace_root: "/tmp/workspace".to_string(),
            model_dir: "/tmp/model".to_string(),
            index_path: "/tmp/index.json".to_string(),
            total_documents: 2,
            reused_documents: 2,
            embedded_documents: 0,
            hits: vec![sample_hit()],
        });

        assert!(html.contains("Hybrid Retrieval Report"));
        assert!(html.contains("hybrid score"));
        assert!(html.contains("lexical score"));
    }

    #[test]
    fn render_search_reports_index_document_lists_saved_reports() {
        let html = render_search_reports_index_document(
            "Saved retrieval reports",
            "2026-04-05T12:00:00Z",
            &[SearchReportEntry {
                file_name: "search-report-semantic-demo.html".to_string(),
                relative_href: "search-report-semantic-demo.html".to_string(),
                title: "Semantic Retrieval Report".to_string(),
                report_kind: Some("semantic".to_string()),
                query: Some("login issue".to_string()),
                generated_at: Some("2026-04-05T12:00:00Z".to_string()),
            }],
        );

        assert!(
            html.contains("Revisit saved retrieval work without leaving the archive workbench.")
        );
        assert!(html.contains("Open archive shell"));
        assert!(html.contains("Open integration reports"));
        assert!(html.contains("search-report-semantic-demo.html"));
        assert!(html.contains("login issue"));
        assert!(html.contains("report-search"));
        assert!(html.contains("data-report-kind"));
    }
}
