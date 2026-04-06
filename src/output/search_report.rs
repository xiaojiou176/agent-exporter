use crate::core::archive_index::ArchiveIndexEntry;

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

#[cfg(test)]
mod tests {
    use super::{
        SearchReportDocument, SearchReportHit, SearchReportKind, render_search_report_document,
    };
    use crate::core::archive_index::ArchiveIndexEntry;

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
}
