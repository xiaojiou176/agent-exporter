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
            "      <p class=\"hero-copy\">{description}</p>\n",
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
            "      <p class=\"eyebrow\">agent-exporter reports shell</p>\n",
            "      <h1>{title}</h1>\n",
            "      <p class=\"hero-copy\">这是一张 retrieval reports 的本地目录页。你可以把它理解成 search receipts 的柜台：检索执行仍然在 CLI，页面只负责组织和回看这些已保存的 report。</p>\n",
            "      <dl class=\"meta-grid\">\n",
            "        <div><dt>Generated</dt><dd><code>{generated_at}</code></dd></div>\n",
            "        <div><dt>Saved reports</dt><dd><code>{report_count}</code></dd></div>\n",
            "      </dl>\n",
            "      <div class=\"link-row\">\n",
            "        <a class=\"open-link\" href=\"../../Conversations/index.html\">Open archive shell</a>\n",
            "      </div>\n",
            "    </header>\n",
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
        report_kind_facets = report_kind_facets,
        body = body,
        style = search_report_style(),
        script = search_report_script(),
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
      --page-bg: linear-gradient(180deg, #f3efe7 0%, #e8e2d7 100%);
      --panel: rgba(255, 252, 247, 0.92);
      --panel-strong: #fffdf9;
      --ink: #20303b;
      --muted: #5f707d;
      --border: #d8ccbc;
      --accent: #8c4f1f;
      --shadow: 0 18px 40px rgba(54, 42, 30, 0.12);
      --mono: "SFMono-Regular", "JetBrains Mono", "Menlo", monospace;
      --serif: "Iowan Old Style", "Palatino Linotype", "Book Antiqua", Georgia, serif;
    }

    * { box-sizing: border-box; }
    body {
      margin: 0;
      font-family: var(--serif);
      color: var(--ink);
      background: var(--page-bg);
    }

    .page-shell {
      width: min(1080px, calc(100vw - 32px));
      margin: 0 auto;
      padding: 28px 0 52px;
    }

    .hero-card,
    .entry-card,
    .empty-state {
      background: var(--panel);
      border: 1px solid var(--border);
      border-radius: 24px;
      box-shadow: var(--shadow);
      backdrop-filter: blur(14px);
      padding: 24px;
    }

    .hero-card { margin-bottom: 24px; }

    .eyebrow {
      margin: 0 0 10px;
      text-transform: uppercase;
      letter-spacing: 0.12em;
      font-family: var(--mono);
      font-size: 12px;
      color: var(--accent);
    }

    h1, h2 {
      margin: 0 0 12px;
      line-height: 1.2;
      font-weight: 700;
    }

    h1 { font-size: clamp(32px, 4vw, 46px); }
    h2 { font-size: clamp(22px, 3vw, 28px); }

    .hero-copy,
    p {
      margin: 0;
      line-height: 1.7;
      color: var(--muted);
      word-break: break-word;
    }

    .meta-grid {
      display: grid;
      gap: 12px;
      grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
      margin: 18px 0 0;
    }

    .meta-grid div {
      padding: 12px 14px;
      background: var(--panel-strong);
      border: 1px solid rgba(216, 204, 188, 0.9);
      border-radius: 16px;
    }

    dt {
      margin-bottom: 6px;
      font-size: 12px;
      letter-spacing: 0.08em;
      text-transform: uppercase;
      color: var(--muted);
      font-family: var(--mono);
    }

    dd { margin: 0; }

    .link-row {
      margin-top: 18px;
    }

    .search-bar {
      display: grid;
      gap: 10px;
      margin: 0 0 18px;
      padding: 18px 20px;
      border-radius: 18px;
      border: 1px solid rgba(216, 204, 188, 0.95);
      background: rgba(255, 251, 244, 0.85);
      box-shadow: var(--shadow);
    }

    .search-label {
      font-family: var(--mono);
      font-size: 12px;
      letter-spacing: 0.1em;
      text-transform: uppercase;
      color: var(--accent);
    }

    .search-input {
      width: 100%;
      padding: 12px 14px;
      border-radius: 14px;
      border: 1px solid rgba(140, 79, 31, 0.25);
      background: rgba(255, 255, 255, 0.9);
      color: var(--ink);
      font-family: var(--mono);
      font-size: 14px;
    }

    .search-status,
    .empty-result {
      color: var(--muted);
      font-size: 14px;
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
      border: 1px solid rgba(140, 79, 31, 0.25);
      background: rgba(255, 255, 255, 0.85);
      border-radius: 999px;
      padding: 8px 12px;
      color: var(--ink);
      font-family: var(--mono);
      font-size: 12px;
      cursor: pointer;
    }

    .facet-button.is-active {
      background: rgba(140, 79, 31, 0.12);
      color: var(--accent);
      border-color: rgba(140, 79, 31, 0.35);
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
      padding: 4px 10px;
      background: rgba(140, 79, 31, 0.12);
      color: var(--accent);
      font-family: var(--mono);
      font-size: 12px;
    }

    .mono-inline,
    code,
    .open-link {
      font-family: var(--mono);
    }

    .mono-inline,
    .score-line {
      margin-top: 10px;
      color: var(--ink);
    }

    .open-link {
      display: inline-flex;
      margin-top: 14px;
      padding: 10px 14px;
      border-radius: 999px;
      border: 1px solid rgba(140, 79, 31, 0.25);
      text-decoration: none;
      color: var(--accent);
      background: rgba(255, 255, 255, 0.7);
    }

    .open-link:hover {
      background: rgba(255, 255, 255, 0.95);
    }

    @media (max-width: 720px) {
      .page-shell {
        width: min(100vw - 20px, 1080px);
        padding: 16px 0 28px;
      }

      .hero-card,
      .entry-card,
      .empty-state {
        border-radius: 20px;
        padding: 18px;
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

        assert!(html.contains("agent-exporter reports shell"));
        assert!(html.contains("Open archive shell"));
        assert!(html.contains("search-report-semantic-demo.html"));
        assert!(html.contains("login issue"));
        assert!(html.contains("report-search"));
        assert!(html.contains("data-report-kind"));
    }
}
