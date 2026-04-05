use crate::core::archive_index::ArchiveIndexEntry;
use crate::output::html::escape_html;

pub fn render_archive_index_document(
    archive_title: &str,
    generated_at: &str,
    entries: &[ArchiveIndexEntry],
) -> String {
    let body = if entries.is_empty() {
        "<section class=\"empty-state\"><h2>还没有 HTML transcript exports</h2><p>先运行 `agent-exporter export ... --format html`，再回来生成 archive index。</p></section>".to_string()
    } else {
        entries
            .iter()
            .map(render_entry)
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
            "      <p class=\"eyebrow\">agent-exporter archive index</p>\n",
            "      <h1>{title}</h1>\n",
            "      <p class=\"hero-copy\">这是一张 workspace conversations 的本地目录页。它像一张书架标签卡，只负责带你走到已经导出的 HTML transcript，不负责搜索、分页或托管发布。</p>\n",
            "      <dl class=\"meta-grid\">\n",
            "        <div><dt>生成时间</dt><dd><code>{generated_at}</code></dd></div>\n",
            "        <div><dt>HTML transcripts</dt><dd><code>{entry_count}</code></dd></div>\n",
            "      </dl>\n",
            "    </header>\n",
            "    <section class=\"search-bar\" aria-label=\"archive search\">\n",
            "      <label class=\"search-label\" for=\"archive-search\">Search archive</label>\n",
            "      <input id=\"archive-search\" class=\"search-input\" type=\"search\" placeholder=\"Search title, connector, thread id, completeness...\" autocomplete=\"off\">\n",
            "      <p id=\"archive-search-status\" class=\"search-status\">Showing <strong>{entry_count}</strong> transcripts.</p>\n",
            "    </section>\n",
            "    <section class=\"card-grid\">\n",
            "{body}\n",
            "    </section>\n",
            "    <p id=\"archive-empty-result\" class=\"empty-result\" hidden>No transcripts matched the current search.</p>\n",
            "  </main>\n",
            "  <script>\n{script}\n  </script>\n",
            "</body>\n",
            "</html>\n"
        ),
        title = escape_html(archive_title),
        generated_at = escape_html(generated_at),
        entry_count = entries.len(),
        body = body,
        style = archive_index_style(),
        script = archive_index_script(),
    )
}

fn render_entry(entry: &ArchiveIndexEntry) -> String {
    let mut meta_lines = Vec::new();
    if let Some(connector) = entry.connector.as_deref() {
        meta_lines.push(chip(connector));
    }
    if let Some(completeness) = entry.completeness.as_deref() {
        meta_lines.push(chip(completeness));
    }
    if let Some(source_kind) = entry.source_kind.as_deref() {
        meta_lines.push(chip(source_kind));
    }

    let thread_line = entry
        .thread_id
        .as_deref()
        .map(|thread_id| {
            format!(
                "<p class=\"mono-inline\">thread: <code>{}</code></p>",
                escape_html(thread_id)
            )
        })
        .unwrap_or_default();
    let exported_line = entry
        .exported_at
        .as_deref()
        .map(|timestamp| {
            format!(
                "<p class=\"mono-inline\">exported: <code>{}</code></p>",
                escape_html(timestamp)
            )
        })
        .unwrap_or_default();
    let searchable_text = [
        entry.title.as_str(),
        entry.connector.as_deref().unwrap_or(""),
        entry.thread_id.as_deref().unwrap_or(""),
        entry.completeness.as_deref().unwrap_or(""),
        entry.source_kind.as_deref().unwrap_or(""),
        entry.file_name.as_str(),
    ]
    .join(" ")
    .to_lowercase();

    format!(
        concat!(
            "<article class=\"entry-card\" data-search-text=\"{searchable_text}\">",
            "<p class=\"eyebrow\">HTML transcript</p>",
            "<h2>{title}</h2>",
            "<div class=\"chip-row\">{chips}</div>",
            "{thread_line}",
            "{exported_line}",
            "<p class=\"mono-inline\">file: <code>{file_name}</code></p>",
            "<p><a class=\"open-link\" href=\"{href}\">Open transcript</a></p>",
            "</article>"
        ),
        title = escape_html(&entry.title),
        chips = meta_lines.join(" "),
        thread_line = thread_line,
        exported_line = exported_line,
        file_name = escape_html(&entry.file_name),
        href = escape_html(&entry.relative_href),
        searchable_text = escape_html(&searchable_text),
    )
}

fn chip(value: &str) -> String {
    format!("<span class=\"chip\">{}</span>", escape_html(value))
}

fn archive_index_style() -> &'static str {
    r#"    :root {
      --page-bg: linear-gradient(180deg, #f4efe5 0%, #ebe2d0 100%);
      --panel: rgba(255, 251, 244, 0.93);
      --panel-strong: #fffdf9;
      --ink: #20303b;
      --muted: #61717c;
      --border: #d4c6b2;
      --accent: #9b5b23;
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
    }

    .hero-card,
    .entry-card,
    .empty-state {
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
      grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
      margin: 18px 0 0;
    }

    .meta-grid div {
      padding: 12px 14px;
      background: var(--panel-strong);
      border: 1px solid rgba(212, 198, 178, 0.9);
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
      background: rgba(155, 91, 35, 0.12);
      color: var(--accent);
      font-family: var(--mono);
      font-size: 12px;
    }

    .mono-inline,
    code,
    .open-link {
      font-family: var(--mono);
    }

    .mono-inline {
      margin-top: 10px;
      color: var(--ink);
    }

    .open-link {
      display: inline-flex;
      margin-top: 14px;
      padding: 10px 14px;
      border-radius: 999px;
      border: 1px solid rgba(155, 91, 35, 0.25);
      text-decoration: none;
      color: var(--accent);
      background: rgba(255, 255, 255, 0.7);
    }

    .open-link:hover {
      background: rgba(255, 255, 255, 0.95);
    }

    .search-bar {
      display: grid;
      gap: 10px;
      margin: 0 0 18px;
      padding: 18px 20px;
      border-radius: 18px;
      border: 1px solid rgba(212, 198, 178, 0.95);
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
      border: 1px solid rgba(155, 91, 35, 0.25);
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

fn archive_index_script() -> &'static str {
    r#"    const input = document.getElementById('archive-search');
    const status = document.getElementById('archive-search-status');
    const empty = document.getElementById('archive-empty-result');
    const cards = Array.from(document.querySelectorAll('.entry-card'));

    if (input && status && empty) {
      const update = () => {
        const query = input.value.trim().toLowerCase();
        let visible = 0;
        for (const card of cards) {
          const haystack = (card.getAttribute('data-search-text') || '').toLowerCase();
          const match = !query || haystack.includes(query);
          card.hidden = !match;
          if (match) visible += 1;
        }
        status.innerHTML = `Showing <strong>${visible}</strong> transcript${visible === 1 ? '' : 's'}.`;
        empty.hidden = visible !== 0;
      };
      input.addEventListener('input', update);
      update();
    }"#
}

#[cfg(test)]
mod tests {
    use super::render_archive_index_document;
    use crate::core::archive_index::ArchiveIndexEntry;

    #[test]
    fn render_archive_index_document_lists_entries() {
        let html = render_archive_index_document(
            "Demo archive",
            "2026-04-05T00:00:00Z",
            &[ArchiveIndexEntry {
                file_name: "demo.html".to_string(),
                relative_href: "demo.html".to_string(),
                title: "Demo transcript".to_string(),
                connector: Some("codex".to_string()),
                thread_id: Some("thread-1".to_string()),
                completeness: Some("complete".to_string()),
                source_kind: Some("app-server-thread-read".to_string()),
                exported_at: Some("2026-04-05T00:00:00Z".to_string()),
            }],
        );

        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("Demo archive"));
        assert!(html.contains("demo.html"));
        assert!(html.contains("Open transcript"));
    }

    #[test]
    fn render_archive_index_document_handles_empty_state() {
        let html = render_archive_index_document("Demo archive", "2026-04-05T00:00:00Z", &[]);
        assert!(html.contains("还没有 HTML transcript exports"));
    }

    #[test]
    fn render_archive_index_document_embeds_search_ui() {
        let html = render_archive_index_document("Demo archive", "2026-04-05T00:00:00Z", &[]);
        assert!(html.contains("archive-search"));
        assert!(html.contains("data-search-text"));
        assert!(html.contains("No transcripts matched the current search."));
    }
}
