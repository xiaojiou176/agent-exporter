use std::collections::BTreeMap;

use crate::core::archive_index::ArchiveIndexEntry;
use crate::core::integration_report::{IntegrationEvidenceGateOutcome, IntegrationReportEntry};
use crate::core::search_report::SearchReportEntry;
use crate::output::html::escape_html;

#[derive(Clone, Debug, PartialEq)]
pub struct DecisionDeskSnapshot {
    pub title: String,
    pub kind: String,
    pub platform: String,
    pub readiness: String,
    pub target: String,
    pub generated_at: String,
    pub html_href: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DecisionDeskSummary {
    pub evidence_report_count: usize,
    pub evidence_shell_href: String,
    pub baseline: Option<DecisionDeskSnapshot>,
    pub candidate: Option<DecisionDeskSnapshot>,
    pub gate: Option<IntegrationEvidenceGateOutcome>,
}

pub fn render_archive_index_document(
    archive_title: &str,
    generated_at: &str,
    entries: &[ArchiveIndexEntry],
    reports: &[SearchReportEntry],
    integration_reports: &[IntegrationReportEntry],
    decision_desk: Option<&DecisionDeskSummary>,
) -> String {
    let distinct_connectors = count_distinct_connectors(entries);
    let connector_facets = render_filter_buttons(
        "connector",
        "Connector",
        summarize_by(entries, |entry| {
            entry.connector.as_deref().unwrap_or("unknown")
        }),
    );
    let completeness_facets = render_filter_buttons(
        "completeness",
        "Completeness",
        summarize_by(entries, |entry| {
            entry.completeness.as_deref().unwrap_or("unknown")
        }),
    );
    let connector_summary = render_summary_section(
        "Connectors in this workspace",
        summarize_by(entries, |entry| {
            entry.connector.as_deref().unwrap_or("unknown")
        }),
    );
    let completeness_summary = render_summary_section(
        "Truth states currently visible",
        summarize_by(entries, |entry| {
            entry.completeness.as_deref().unwrap_or("unknown")
        }),
    );
    let source_summary = render_summary_section(
        "Sources represented in the archive",
        summarize_by(entries, |entry| {
            entry.source_kind.as_deref().unwrap_or("unknown")
        }),
    );
    let report_summary = render_summary_section(
        "Saved retrieval reports",
        summarize_by_reports(reports, |report| {
            report.report_kind.as_deref().unwrap_or("unknown")
        }),
    );
    let integration_summary = render_summary_section(
        "Integration evidence reports",
        summarize_by_integration_reports(integration_reports, |report| {
            report.platform.as_deref().unwrap_or("unknown")
        }),
    );
    let body = if entries.is_empty() {
        "<section class=\"empty-state\"><h2>还没有 HTML transcript exports</h2><p>先运行 `agent-exporter export ... --format html`，再回来生成 archive index。</p></section>".to_string()
    } else {
        entries
            .iter()
            .map(render_entry)
            .collect::<Vec<_>>()
            .join("\n")
    };
    let report_cards = if reports.is_empty() {
        "<article class=\"summary-card\"><p class=\"eyebrow\">Retrieval reports</p><h2>No saved reports yet</h2><p>下一次运行 <code>search semantic</code> 或 <code>search hybrid</code> 时，加上 <code>--save-report</code>，这里就会出现可直接打开的 report links。</p></article>".to_string()
    } else {
        reports
            .iter()
            .map(render_report_entry)
            .collect::<Vec<_>>()
            .join("\n")
    };
    let decision_section = render_decision_desk(decision_desk);

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
            "      <p class=\"eyebrow\">agent-exporter local archive shell</p>\n",
            "      <h1>{title}</h1>\n",
            "      <p class=\"hero-copy\">这是一张 workspace conversations 的本地 archive shell。你可以把它理解成一个多 agent 档案前厅：页内 metadata filter 负责快速翻卡片，CLI 的 semantic / hybrid retrieval 负责更深一层的检索。它仍然是本地静态页面，不会替你走远程服务。</p>\n",
            "      <dl class=\"meta-grid\">\n",
            "        <div><dt>生成时间</dt><dd><code>{generated_at}</code></dd></div>\n",
            "        <div><dt>HTML transcripts</dt><dd><code>{entry_count}</code></dd></div>\n",
            "        <div><dt>Connectors</dt><dd><code>{connector_count}</code></dd></div>\n",
            "        <div><dt>Retrieval lanes</dt><dd><code>metadata / semantic / hybrid</code></dd></div>\n",
            "        <div><dt>Saved reports</dt><dd><code>{report_count}</code></dd></div>\n",
            "        <div><dt>Integration evidence</dt><dd><code>{integration_report_count}</code></dd></div>\n",
            "      </dl>\n",
            "      <div class=\"link-row\">\n",
            "        <a class=\"open-link\" href=\"../Search/Reports/index.html\">Open reports shell</a>\n",
            "        <a class=\"open-link\" href=\"../Integration/Reports/index.html\">Open integration reports</a>\n",
            "      </div>\n",
            "    </header>\n",
            "{decision_section}\n",
            "    <section class=\"lane-grid\" aria-label=\"retrieval lanes\">\n",
            "      <article class=\"lane-card\">\n",
            "        <p class=\"eyebrow\">Lane 1</p>\n",
            "        <h2>Metadata filter</h2>\n",
            "        <p>用下面的搜索框和 facet buttons 先做本地 metadata filter。它像翻书架标签卡，只看标题、connector、thread id、completeness 和 source。</p>\n",
            "      </article>\n",
            "      <article class=\"lane-card\">\n",
            "        <p class=\"eyebrow\">Lane 2</p>\n",
            "        <h2>Semantic retrieval</h2>\n",
            "        <p>如果你要按语义找“内容相近”的 transcript，继续用当前纯语义命令面；加上 <code>--save-report</code> 就能把这次查询留成可重复打开的本地 report：</p>\n",
            "        <pre><code>agent-exporter search semantic --workspace-root &lt;repo-root&gt; --query \"login issues\" --save-report</code></pre>\n",
            "      </article>\n",
            "      <article class=\"lane-card\">\n",
            "        <p class=\"eyebrow\">Lane 3</p>\n",
            "        <h2>Hybrid retrieval</h2>\n",
            "        <p>如果你既想保留 semantic ranking，又想吃到 metadata signal，就走 blended lane；同样可以把结果保存成 local report：</p>\n",
            "        <pre><code>agent-exporter search hybrid --workspace-root &lt;repo-root&gt; --query \"thread-1\" --save-report</code></pre>\n",
            "      </article>\n",
            "    </section>\n",
            "    <section class=\"summary-grid\" aria-label=\"archive summaries\">\n",
            "{connector_summary}\n",
            "{completeness_summary}\n",
            "{source_summary}\n",
            "{report_summary}\n",
            "{integration_summary}\n",
            "    </section>\n",
            "    <section class=\"search-bar\" aria-label=\"archive search\">\n",
            "      <label class=\"search-label\" for=\"archive-search\">Metadata search</label>\n",
            "      <input id=\"archive-search\" class=\"search-input\" type=\"search\" placeholder=\"Search title, connector, thread id, completeness, source...\" autocomplete=\"off\">\n",
            "      <div class=\"facet-grid\">\n",
            "{connector_facets}\n",
            "{completeness_facets}\n",
            "      </div>\n",
            "      <p id=\"archive-search-status\" class=\"search-status\">Showing <strong>{entry_count}</strong> transcripts.</p>\n",
            "    </section>\n",
            "    <section class=\"report-grid\" aria-label=\"retrieval reports\">\n",
            "{report_cards}\n",
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
        connector_count = distinct_connectors,
        report_count = reports.len(),
        integration_report_count = integration_reports.len(),
        decision_section = decision_section,
        connector_facets = connector_facets,
        completeness_facets = completeness_facets,
        connector_summary = connector_summary,
        completeness_summary = completeness_summary,
        source_summary = source_summary,
        report_summary = report_summary,
        integration_summary = integration_summary,
        report_cards = report_cards,
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
            "<article class=\"entry-card\" data-search-text=\"{searchable_text}\" data-connector=\"{connector}\" data-completeness=\"{completeness}\">",
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
        connector = escape_html(entry.connector.as_deref().unwrap_or("unknown")),
        completeness = escape_html(entry.completeness.as_deref().unwrap_or("unknown")),
    )
}

fn chip(value: &str) -> String {
    format!("<span class=\"chip\">{}</span>", escape_html(value))
}

fn render_report_entry(entry: &SearchReportEntry) -> String {
    let report_href = format!("../Search/Reports/{}", entry.relative_href);
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

    format!(
        concat!(
            "<article class=\"summary-card\">",
            "<p class=\"eyebrow\">Retrieval report</p>",
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
        href = escape_html(&report_href),
    )
}

fn count_distinct_connectors(entries: &[ArchiveIndexEntry]) -> usize {
    summarize_by(entries, |entry| {
        entry.connector.as_deref().unwrap_or("unknown")
    })
    .len()
}

fn summarize_by<F>(entries: &[ArchiveIndexEntry], label: F) -> Vec<(String, usize)>
where
    F: Fn(&ArchiveIndexEntry) -> &str,
{
    let mut counts = BTreeMap::new();
    for entry in entries {
        *counts.entry(label(entry).to_string()).or_insert(0usize) += 1;
    }
    counts.into_iter().collect()
}

fn summarize_by_reports<F>(entries: &[SearchReportEntry], label: F) -> Vec<(String, usize)>
where
    F: Fn(&SearchReportEntry) -> &str,
{
    let mut counts = BTreeMap::new();
    for entry in entries {
        *counts.entry(label(entry).to_string()).or_insert(0usize) += 1;
    }
    counts.into_iter().collect()
}

fn summarize_by_integration_reports<F>(
    entries: &[IntegrationReportEntry],
    label: F,
) -> Vec<(String, usize)>
where
    F: Fn(&IntegrationReportEntry) -> &str,
{
    let mut counts = BTreeMap::new();
    for entry in entries {
        *counts.entry(label(entry).to_string()).or_insert(0usize) += 1;
    }
    counts.into_iter().collect()
}

fn render_decision_desk(summary: Option<&DecisionDeskSummary>) -> String {
    let Some(summary) = summary else {
        return String::new();
    };

    let verdict_label = summary
        .gate
        .as_ref()
        .map(|gate| gate.verdict.as_str())
        .unwrap_or("insufficient");
    let regression_label = summary
        .gate
        .as_ref()
        .map(|gate| {
            if gate.regression {
                "regression"
            } else {
                "stable"
            }
        })
        .unwrap_or("awaiting-pair");

    let baseline_card = render_decision_snapshot("Baseline", summary.baseline.as_ref());
    let candidate_card = render_decision_snapshot("Candidate", summary.candidate.as_ref());
    let remediation = render_decision_remediation(summary);
    let changed_checks = render_decision_changes(summary);

    format!(
        concat!(
            "<section class=\"decision-desk\" aria-label=\"decision desk\">\n",
            "  <header class=\"decision-header\">\n",
            "    <p class=\"eyebrow\">Local Evidence Decision Desk</p>\n",
            "    <h2>Baseline / Candidate / Verdict</h2>\n",
            "    <p class=\"hero-copy\">这张总控台只读当前 workspace 下已经保存好的 integration evidence。它负责告诉你现在的 verdict、变化项和修复顺序，但不会在浏览器里执行 doctor、onboard 或 gate。</p>\n",
            "    <div class=\"verdict-strip\">\n",
            "      <span class=\"chip verdict-chip\">{verdict}</span>\n",
            "      <span class=\"chip\">{regression}</span>\n",
            "      <span class=\"chip\">evidence reports <span>{count}</span></span>\n",
            "      <a class=\"open-link\" href=\"{evidence_shell_href}\">Open integration reports</a>\n",
            "    </div>\n",
            "  </header>\n",
            "  <div class=\"decision-grid\">\n",
            "{baseline_card}\n",
            "{candidate_card}\n",
            "{remediation}\n",
            "  </div>\n",
            "{changed_checks}\n",
            "  <section class=\"summary-card decision-nav\">\n",
            "    <p class=\"eyebrow\">Cross-shell navigation</p>\n",
            "    <h2>Stay in one front door, keep three corpora</h2>\n",
            "    <div class=\"link-row\">\n",
            "      <a class=\"open-link\" href=\"index.html\">Open transcript shell</a>\n",
            "      <a class=\"open-link\" href=\"../Search/Reports/index.html\">Open search reports shell</a>\n",
            "      <a class=\"open-link\" href=\"../Integration/Reports/index.html\">Open integration reports shell</a>\n",
            "    </div>\n",
            "  </section>\n",
            "</section>\n"
        ),
        verdict = escape_html(verdict_label),
        regression = escape_html(regression_label),
        count = summary.evidence_report_count,
        evidence_shell_href = escape_html(&summary.evidence_shell_href),
        baseline_card = baseline_card,
        candidate_card = candidate_card,
        remediation = remediation,
        changed_checks = changed_checks,
    )
}

fn render_decision_snapshot(label: &str, snapshot: Option<&DecisionDeskSnapshot>) -> String {
    let Some(snapshot) = snapshot else {
        return format!(
            concat!(
                "<article class=\"summary-card decision-card\">",
                "<p class=\"eyebrow\">{label}</p>",
                "<h2>No artifact selected</h2>",
                "<p>This side of the comparison is currently unavailable. Keep the shell-level navigation visible, and avoid inventing a verdict from one-sided input.</p>",
                "</article>"
            ),
            label = escape_html(label),
        );
    };

    format!(
        concat!(
            "<article class=\"summary-card decision-card\">",
            "<p class=\"eyebrow\">{label}</p>",
            "<h2>{title}</h2>",
            "<div class=\"chip-row\">",
            "<span class=\"chip\">{kind}</span>",
            "<span class=\"chip\">{platform}</span>",
            "<span class=\"chip\">{readiness}</span>",
            "</div>",
            "<p class=\"mono-inline\">target: <code>{target}</code></p>",
            "<p class=\"mono-inline\">generated: <code>{generated_at}</code></p>",
            "<p><a class=\"open-link\" href=\"{href}\">Open evidence report</a></p>",
            "</article>"
        ),
        label = escape_html(label),
        title = escape_html(&snapshot.title),
        kind = escape_html(&snapshot.kind),
        platform = escape_html(&snapshot.platform),
        readiness = escape_html(&snapshot.readiness),
        target = escape_html(&snapshot.target),
        generated_at = escape_html(&snapshot.generated_at),
        href = escape_html(&snapshot.html_href),
    )
}

fn render_decision_remediation(summary: &DecisionDeskSummary) -> String {
    let steps = summary
        .gate
        .as_ref()
        .map(|gate| gate.remediation_steps.clone())
        .unwrap_or_default();

    let body = if steps.is_empty() {
        "<p class=\"empty-inline\">No remediation steps are available yet. Save at least one candidate evidence report with actionable next steps before relying on this panel for ordering.</p>".to_string()
    } else {
        format!(
            "<ol class=\"step-list\">{}</ol>",
            steps
                .iter()
                .map(|step| {
                    format!(
                        concat!(
                            "<li>",
                            "<strong>{title}</strong>",
                            "<p>{why}</p>",
                            "<p class=\"mono-inline\">recheck: <code>{recheck}</code></p>",
                            "</li>"
                        ),
                        title = escape_html(&step.title),
                        why = escape_html(&step.why),
                        recheck = escape_html(&step.recheck),
                    )
                })
                .collect::<Vec<_>>()
                .join("")
        )
    };

    format!(
        concat!(
            "<article class=\"summary-card decision-card remediation-card\">",
            "<p class=\"eyebrow\">Remediation order</p>",
            "<h2>Fix this first</h2>",
            "{body}",
            "</article>"
        ),
        body = body,
    )
}

fn render_decision_changes(summary: &DecisionDeskSummary) -> String {
    let Some(gate) = summary.gate.as_ref() else {
        return "<section class=\"summary-card decision-changes\"><p class=\"eyebrow\">Changed checks</p><h2>Insufficient comparison input</h2><p>Save at least two related evidence reports before expecting a changed-checks ledger.</p></section>".to_string();
    };

    let mut items = Vec::new();

    for change in &gate.blocking_changes {
        items.push(render_change_item("blocking", change));
    }
    for change in &gate.warning_changes {
        items.push(render_change_item("warning", change));
    }
    for change in &gate.ignored_changes {
        items.push(render_change_item("ignorable", change));
    }

    let body = if items.is_empty() {
        "<p class=\"empty-inline\">No changed checks. Baseline and candidate are aligned at the current evidence depth.</p>".to_string()
    } else {
        format!("<ul class=\"check-list\">{}</ul>", items.join(""))
    };

    format!(
        concat!(
            "<section class=\"summary-card decision-changes\">",
            "<p class=\"eyebrow\">Changed checks</p>",
            "<h2>What moved between baseline and candidate</h2>",
            "{body}",
            "</section>"
        ),
        body = body,
    )
}

fn render_change_item(
    severity: &str,
    change: &crate::core::integration_report::IntegrationEvidenceCheckDiff,
) -> String {
    format!(
        concat!(
            "<li class=\"change-item\">",
            "<div class=\"chip-row\">",
            "<span class=\"chip\">{severity}</span>",
            "<span class=\"chip\">{label}</span>",
            "</div>",
            "<p class=\"mono-inline\">{left} -&gt; {right}</p>",
            "</li>"
        ),
        severity = escape_html(severity),
        label = escape_html(&change.label),
        left = escape_html(change.left_readiness.as_deref().unwrap_or("missing")),
        right = escape_html(change.right_readiness.as_deref().unwrap_or("missing")),
    )
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

fn render_summary_section(title: &str, items: Vec<(String, usize)>) -> String {
    let chips = if items.is_empty() {
        "<span class=\"chip\">none yet</span>".to_string()
    } else {
        items
            .into_iter()
            .map(|(name, count)| format!("{} <span>{count}</span>", chip(&name)))
            .collect::<Vec<_>>()
            .join(" ")
    };

    format!(
        concat!(
            "<article class=\"summary-card\">",
            "<p class=\"eyebrow\">Archive summary</p>",
            "<h2>{title}</h2>",
            "<div class=\"chip-row\">{chips}</div>",
            "</article>"
        ),
        title = escape_html(title),
        chips = chips,
    )
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
    .lane-card,
    .summary-card,
    .entry-card,
    .empty-state,
    .decision-header,
    .decision-changes {
      background: var(--panel);
      border: 1px solid var(--border);
      border-radius: 24px;
      box-shadow: var(--shadow);
      backdrop-filter: blur(14px);
    }

    .hero-card,
    .lane-card,
    .summary-card,
    .entry-card,
    .empty-state {
      padding: 24px;
    }

    .hero-card { margin-bottom: 24px; }

    .lane-grid,
    .report-grid,
    .summary-grid,
    .decision-grid {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
      gap: 18px;
      margin-bottom: 20px;
    }

    .decision-desk { margin-bottom: 24px; }

    .decision-header,
    .decision-changes {
      padding: 24px;
      box-shadow: var(--shadow);
      backdrop-filter: blur(14px);
      margin-bottom: 18px;
    }

    .verdict-strip {
      display: flex;
      flex-wrap: wrap;
      gap: 10px;
      margin-top: 16px;
      align-items: center;
    }

    .decision-card,
    .remediation-card {
      min-height: 100%;
    }

    .check-list,
    .step-list {
      margin: 0;
      padding-left: 20px;
      color: var(--muted);
    }

    .change-item {
      margin-bottom: 12px;
    }

    .empty-inline {
      color: var(--muted);
      line-height: 1.7;
    }

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

    .chip span,
    .facet-button span {
      margin-left: 6px;
      opacity: 0.7;
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
      border: 1px solid rgba(155, 91, 35, 0.25);
      background: rgba(255, 255, 255, 0.85);
      border-radius: 999px;
      padding: 8px 12px;
      color: var(--ink);
      font-family: var(--mono);
      font-size: 12px;
      cursor: pointer;
    }

    .facet-button.is-active {
      background: rgba(155, 91, 35, 0.12);
      color: var(--accent);
      border-color: rgba(155, 91, 35, 0.35);
    }

    pre {
      margin: 14px 0 0;
      padding: 14px;
      overflow-x: auto;
      border-radius: 16px;
      background: rgba(32, 48, 59, 0.92);
      color: #f9f4eb;
    }

    @media (max-width: 720px) {
      .page-shell {
        width: min(100vw - 20px, 1080px);
        padding: 16px 0 28px;
      }

      .hero-card,
      .lane-card,
      .summary-card,
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
    const buttons = Array.from(document.querySelectorAll('.facet-button'));
    const activeFilters = {
      connector: 'all',
      completeness: 'all',
    };

    if (input && status && empty) {
      const update = () => {
        const query = input.value.trim().toLowerCase();
        let visible = 0;
        for (const card of cards) {
          const haystack = (card.getAttribute('data-search-text') || '').toLowerCase();
          const connector = (card.getAttribute('data-connector') || 'unknown').toLowerCase();
          const completeness = (card.getAttribute('data-completeness') || 'unknown').toLowerCase();
          const matchesQuery = !query || haystack.includes(query);
          const matchesConnector = activeFilters.connector === 'all' || connector === activeFilters.connector;
          const matchesCompleteness = activeFilters.completeness === 'all' || completeness === activeFilters.completeness;
          const match = matchesQuery && matchesConnector && matchesCompleteness;
          card.hidden = !match;
          if (match) visible += 1;
        }
        status.innerHTML = `Showing <strong>${visible}</strong> transcript${visible === 1 ? '' : 's'}.`;
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
            &[],
            &[],
            None,
        );

        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("Demo archive"));
        assert!(html.contains("demo.html"));
        assert!(html.contains("Open transcript"));
    }

    #[test]
    fn render_archive_index_document_handles_empty_state() {
        let html = render_archive_index_document(
            "Demo archive",
            "2026-04-05T00:00:00Z",
            &[],
            &[],
            &[],
            None,
        );
        assert!(html.contains("还没有 HTML transcript exports"));
    }

    #[test]
    fn render_archive_index_document_embeds_search_ui() {
        let html = render_archive_index_document(
            "Demo archive",
            "2026-04-05T00:00:00Z",
            &[],
            &[],
            &[],
            None,
        );
        assert!(html.contains("archive-search"));
        assert!(html.contains("data-search-text"));
        assert!(html.contains("No transcripts matched the current search."));
    }

    #[test]
    fn render_archive_index_document_embeds_multi_agent_shell_sections() {
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
            &[],
            &[],
            None,
        );

        assert!(html.contains("agent-exporter local archive shell"));
        assert!(html.contains("Open reports shell"));
        assert!(html.contains(
            "search semantic --workspace-root &lt;repo-root&gt; --query \"login issues\" --save-report"
        ));
        assert!(html.contains(
            "search hybrid --workspace-root &lt;repo-root&gt; --query \"thread-1\" --save-report"
        ));
        assert!(html.contains("data-filter-group=\"connector\""));
        assert!(html.contains("data-filter-group=\"completeness\""));
        assert!(html.contains("Open integration reports"));
    }
}
