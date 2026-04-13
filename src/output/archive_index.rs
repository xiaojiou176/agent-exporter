use std::collections::BTreeMap;

use crate::core::archive_index::ArchiveIndexEntry;
use crate::core::integration_report::{
    IntegrationEvidenceGateOutcome, IntegrationEvidenceRemediationBundle, IntegrationReportEntry,
};
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
pub struct DecisionDeskPolicySummary {
    pub name: String,
    pub version: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DecisionDeskPromotionSummary {
    pub state: String,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DecisionDeskHistoryEntry {
    pub decided_at: String,
    pub baseline_name: String,
    pub verdict: String,
    pub promoted: bool,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DecisionDeskSummary {
    pub evidence_report_count: usize,
    pub evidence_shell_href: String,
    pub baseline_name: Option<String>,
    pub baseline: Option<DecisionDeskSnapshot>,
    pub candidate: Option<DecisionDeskSnapshot>,
    pub active_policy: DecisionDeskPolicySummary,
    pub promotion: DecisionDeskPromotionSummary,
    pub history: Vec<DecisionDeskHistoryEntry>,
    pub remediation_bundle: Option<IntegrationEvidenceRemediationBundle>,
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
            "      <p class=\"eyebrow\">agent-exporter local archive shell proof</p>\n",
            "      <h1>{title}</h1>\n",
            "      <p class=\"hero-copy\">产品总正门仍然是 CLI quickstart；如果你已经来到这里，就把这页理解成 archive shell proof 和 transcript/browser 侧厅。这里的主航道仍然是 transcript/archive 浏览与搜索，search reports 和 integration evidence 是并列工作线，Decision Desk 则是右手边的治理 lane，而不是整个产品的唯一主角。这里仍然是 local-first 的静态前厅，不会替你调用远程服务，也不会在浏览器里执行 doctor/onboard。</p>\n",
            "      <dl class=\"meta-grid\">\n",
            "        <div><dt>生成时间</dt><dd><code>{generated_at}</code></dd></div>\n",
            "        <div><dt>Transcript exports</dt><dd><code>{entry_count}</code></dd></div>\n",
            "        <div><dt>Connectors</dt><dd><code>{connector_count}</code></dd></div>\n",
            "        <div><dt>Saved search reports</dt><dd><code>{report_count}</code></dd></div>\n",
            "        <div><dt>Integration evidence</dt><dd><code>{integration_report_count}</code></dd></div>\n",
            "        <div><dt>Primary front door</dt><dd><code>CLI quickstart</code></dd></div>\n",
            "        <div><dt>Archive lanes</dt><dd><code>browse / search / governance</code></dd></div>\n",
            "        <div><dt>Workbench style</dt><dd><code>local-first / static / no executor</code></dd></div>\n",
            "      </dl>\n",
            "      <div class=\"hero-actions\">\n",
            "        <a class=\"open-link primary-link\" href=\"#archive-browser\">Start with transcripts</a>\n",
            "        <a class=\"open-link\" href=\"#tool-lanes\">See side lanes</a>\n",
            "        <a class=\"open-link\" href=\"#saved-reports\">Open saved reports</a>\n",
            "      </div>\n",
            "      <div class=\"link-row compact-row\">\n",
            "        <a class=\"open-link\" href=\"../Search/Reports/index.html\">Open reports shell</a>\n",
            "        <a class=\"open-link\" href=\"../Integration/Reports/index.html\">Open integration reports</a>\n",
            "      </div>\n",
            "    </header>\n",
            "    <section class=\"summary-grid\" aria-label=\"workspace summaries\">\n",
            "{connector_summary}\n",
            "{completeness_summary}\n",
            "{source_summary}\n",
            "{report_summary}\n",
            "{integration_summary}\n",
            "    </section>\n",
            "    <section class=\"section-card section-intro\" id=\"archive-browser\">\n",
            "      <p class=\"eyebrow\">Archive browser</p>\n",
            "      <h2>先从对话记录开始，而不是先看治理票据</h2>\n",
            "      <p class=\"hero-copy\">这里是这张侧厅里的主航道。先筛 transcript、打开对话、理解 archive 本身，再决定要不要跳去 search reports 或 governance lane。换句话说：先看内容，再看诊断，再看制度。</p>\n",
            "    </section>\n",
            "    <section class=\"search-bar\" aria-label=\"archive search\">\n",
            "      <label class=\"search-label\" for=\"archive-search\">Transcript search</label>\n",
            "      <input id=\"archive-search\" class=\"search-input\" type=\"search\" placeholder=\"Search title, connector, thread id, completeness, source...\" autocomplete=\"off\">\n",
            "      <div class=\"facet-grid\">\n",
            "{connector_facets}\n",
            "{completeness_facets}\n",
            "      </div>\n",
            "      <p id=\"archive-search-status\" class=\"search-status\">Showing <strong>{entry_count}</strong> transcript{entry_suffix}.</p>\n",
            "    </section>\n",
            "    <section class=\"card-grid transcript-grid\" aria-label=\"transcript browser\">\n",
            "{body}\n",
            "    </section>\n",
            "    <p id=\"archive-empty-result\" class=\"empty-result\" hidden>No transcripts matched the current search.</p>\n",
            "    <section class=\"lane-grid\" id=\"tool-lanes\" aria-label=\"tool lanes\">\n",
            "      <article class=\"lane-card\">\n",
            "        <p class=\"eyebrow\">Lane A</p>\n",
            "        <h2>Browse transcripts</h2>\n",
            "        <p>先把 transcript 当主角。用前面的搜索框和 facets 快速定位 conversation export，再决定要不要跳去更深的 report 或 governance 工具。</p>\n",
            "      </article>\n",
            "      <article class=\"lane-card\">\n",
            "        <p class=\"eyebrow\">Lane B</p>\n",
            "        <h2>Semantic retrieval</h2>\n",
            "        <p>如果你要按语义找“内容相近”的 transcript，继续用 CLI 的 semantic / hybrid retrieval。search reports 仍然是独立抽屉，但从这里可以顺着跳过去，不会迷路。</p>\n",
            "        <pre tabindex=\"0\" aria-label=\"semantic retrieval command example\"><code>agent-exporter search semantic --workspace-root &lt;repo-root&gt; --query \"login issues\" --save-report</code></pre>\n",
            "        <pre tabindex=\"0\" aria-label=\"hybrid retrieval command example\"><code>agent-exporter search hybrid --workspace-root &lt;repo-root&gt; --query \"thread-1\" --save-report</code></pre>\n",
            "      </article>\n",
            "      <article class=\"lane-card\">\n",
            "        <p class=\"eyebrow\">Lane C</p>\n",
            "        <h2>Governance lane</h2>\n",
            "        <p>Decision Desk 很重要，但它不是整张桌子的主语。它负责 official baseline、policy、promotion、history 和 remediation；当你处理“接线是否健康”时，再从这里切过去。</p>\n",
            "        <pre tabindex=\"0\" aria-label=\"governance lane command example\"><code>agent-exporter evidence current --baseline-name &lt;name&gt;</code></pre>\n",
            "      </article>\n",
            "    </section>\n",
            "    <section class=\"section-card section-intro report-intro\" id=\"saved-reports\">\n",
            "      <p class=\"eyebrow\">Saved reports</p>\n",
            "      <h2>Search 和 evidence 仍然是两条独立 lane</h2>\n",
            "      <p class=\"hero-copy\">search reports 帮你回看检索，integration evidence 帮你回看接线结果和治理账本。它们都挂在这个 archive workbench 下面，但不会混成同一份 corpus，也不会反过来盖过 transcript/archive 主航道。</p>\n",
            "    </section>\n",
            "    <section class=\"report-grid\" aria-label=\"retrieval reports\">\n",
            "{report_cards}\n",
            "    </section>\n",
            "<div id=\"governance-lane\">{decision_section}</div>\n",
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
        entry_suffix = if entries.len() == 1 { "" } else { "s" },
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

    let baseline_card = render_decision_snapshot("Official baseline", summary.baseline.as_ref());
    let candidate_card = render_decision_snapshot("Candidate", summary.candidate.as_ref());
    let governance_card = render_decision_governance(summary);
    let remediation = render_decision_remediation(summary);
    let changed_checks = render_decision_changes(summary);
    let history = render_decision_history(summary);

    format!(
        concat!(
            "<section class=\"decision-desk\" aria-label=\"decision desk\">\n",
            "  <header class=\"decision-header\">\n",
            "    <p class=\"eyebrow\">Local Governance Workbench</p>\n",
            "    <h2>Official standard / candidate / action bundle</h2>\n",
            "    <p class=\"hero-copy\">这张工作台只读当前 workspace 下已经保存好的 integration evidence、baseline registry、policy packs 和 decision history。它负责告诉你现在的 official standard、candidate verdict、promotion status 和 remediation bundle，但在层级上仍然是 transcript/archive 主航道旁边的一条治理 lane，不会在浏览器里执行 doctor、onboard 或 gate。</p>\n",
            "    <div class=\"verdict-strip\">\n",
            "      <span class=\"chip verdict-chip\">{verdict}</span>\n",
            "      <span class=\"chip\">{regression}</span>\n",
            "      <span class=\"chip\">evidence reports <span>{count}</span></span>\n",
            "      <span class=\"chip\">history <span>{history_count}</span></span>\n",
            "      <a class=\"open-link\" href=\"{evidence_shell_href}\">Open integration reports</a>\n",
            "    </div>\n",
            "  </header>\n",
            "  <div class=\"decision-grid\">\n",
            "{baseline_card}\n",
            "{candidate_card}\n",
            "{governance_card}\n",
            "{remediation}\n",
            "  </div>\n",
            "{changed_checks}\n",
            "{history}\n",
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
        history_count = summary.history.len(),
        evidence_shell_href = escape_html(&summary.evidence_shell_href),
        baseline_card = baseline_card,
        candidate_card = candidate_card,
        governance_card = governance_card,
        remediation = remediation,
        changed_checks = changed_checks,
        history = history,
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
    let bundle = summary.remediation_bundle.as_ref();
    let steps = bundle
        .map(|bundle| bundle.steps.clone())
        .or_else(|| {
            summary
                .gate
                .as_ref()
                .map(|gate| gate.remediation_steps.clone())
        })
        .unwrap_or_default();

    let body = if steps.is_empty() {
        "<p class=\"empty-inline\">No remediation steps are available yet. Save at least one candidate evidence report with actionable next steps before relying on this panel for ordering.</p>".to_string()
    } else {
        let summary_line = bundle
            .map(|bundle| {
                format!(
                    "<p class=\"mono-inline\">status: <code>{}</code> | steps: <code>{}</code></p><p>{}</p>",
                    escape_html(&bundle.bundle_status),
                    bundle.step_count,
                    escape_html(&bundle.summary),
                )
            })
            .unwrap_or_default();
        format!(
            "{summary_line}<ol class=\"step-list\">{}</ol>",
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
                .join(""),
            summary_line = summary_line
        )
    };

    format!(
        concat!(
            "<article class=\"summary-card decision-card remediation-card\">",
            "<p class=\"eyebrow\">Remediation bundle</p>",
            "<h2>What to fix next</h2>",
            "{body}",
            "</article>"
        ),
        body = body,
    )
}

fn render_decision_governance(summary: &DecisionDeskSummary) -> String {
    let baseline_name = summary.baseline_name.as_deref().unwrap_or("unregistered");
    format!(
        concat!(
            "<article class=\"summary-card decision-card governance-card\">",
            "<p class=\"eyebrow\">Governance</p>",
            "<h2>Official baseline / active policy / promotion</h2>",
            "<p class=\"mono-inline\">baseline name: <code>{baseline_name}</code></p>",
            "<p class=\"mono-inline\">active policy: <code>{policy_name}</code> <code>{policy_version}</code></p>",
            "<p class=\"mono-inline\">promotion status: <code>{promotion_state}</code></p>",
            "<p>{promotion_summary}</p>",
            "</article>"
        ),
        baseline_name = escape_html(baseline_name),
        policy_name = escape_html(&summary.active_policy.name),
        policy_version = escape_html(&summary.active_policy.version),
        promotion_state = escape_html(&summary.promotion.state),
        promotion_summary = escape_html(&summary.promotion.summary),
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
            "<h2>Policy-aware change ledger</h2>",
            "{body}",
            "</section>"
        ),
        body = body,
    )
}

fn render_decision_history(summary: &DecisionDeskSummary) -> String {
    let body = if summary.history.is_empty() {
        "<p class=\"empty-inline\">No governance decisions have been recorded for this platform/target yet.</p>".to_string()
    } else {
        format!(
            "<ul class=\"check-list\">{}</ul>",
            summary
                .history
                .iter()
                .map(|entry| {
                    format!(
                        concat!(
                            "<li class=\"change-item\">",
                            "<div class=\"chip-row\">",
                            "<span class=\"chip\">{verdict}</span>",
                            "<span class=\"chip\">promoted {promoted}</span>",
                            "<span class=\"chip\">{baseline_name}</span>",
                            "</div>",
                            "<p class=\"mono-inline\">{decided_at}</p>",
                            "<p>{summary}</p>",
                            "</li>"
                        ),
                        verdict = escape_html(&entry.verdict),
                        promoted = if entry.promoted { "yes" } else { "no" },
                        baseline_name = escape_html(&entry.baseline_name),
                        decided_at = escape_html(&entry.decided_at),
                        summary = escape_html(&entry.summary),
                    )
                })
                .collect::<Vec<_>>()
                .join("")
        )
    };

    format!(
        concat!(
            "<section class=\"summary-card decision-changes\">",
            "<p class=\"eyebrow\">Decision history</p>",
            "<h2>Recent governance ledger</h2>",
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
      --success: #0f766e;
      --warning: #b45309;
      --danger: #b91c1c;
      --shadow-border: 0 0 0 1px rgba(15, 23, 42, 0.06);
      --shadow-panel:
        0 24px 60px rgba(15, 23, 42, 0.07),
        0 0 0 1px rgba(255, 255, 255, 0.72) inset;
      --shadow-hero:
        0 28px 80px rgba(15, 23, 42, 0.10),
        0 0 0 1px rgba(255, 255, 255, 0.80) inset;
      --mono: "JetBrains Mono", "SFMono-Regular", "Menlo", monospace;
      --sans: -apple-system, BlinkMacSystemFont, "IBM Plex Sans", "Segoe UI", sans-serif;
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
      width: min(1240px, calc(100vw - 32px));
      margin: 0 auto;
      padding: 28px 0 72px;
      position: relative;
      z-index: 1;
    }

    .hero-card,
    .lane-card,
    .summary-card,
    .entry-card,
    .empty-state,
    .decision-header,
    .decision-changes,
    .section-card {
      position: relative;
      overflow: hidden;
      background: var(--surface);
      border: 1px solid var(--line);
      border-radius: 28px;
      box-shadow: var(--shadow-panel);
      backdrop-filter: blur(14px);
    }

    .hero-card::before,
    .lane-card::before,
    .summary-card::before,
    .entry-card::before,
    .decision-header::before,
    .decision-changes::before,
    .section-card::before {
      content: "";
      position: absolute;
      inset: 0;
      pointer-events: none;
      background: linear-gradient(180deg, rgba(255, 255, 255, 0.65), transparent 28%);
    }

    .hero-card,
    .lane-card,
    .summary-card,
    .entry-card,
    .empty-state,
    .section-card {
      padding: 24px;
    }

    .hero-card {
      margin-bottom: 22px;
      padding: 32px;
      border-radius: 32px;
      background:
        radial-gradient(circle at top left, rgba(37, 99, 235, 0.14), transparent 34%),
        linear-gradient(135deg, rgba(255, 255, 255, 0.92), rgba(248, 250, 252, 0.84)),
        var(--surface);
      box-shadow: var(--shadow-hero);
    }

    .section-card {
      margin-bottom: 18px;
    }

    .summary-grid,
    .lane-grid,
    .report-grid,
    .decision-grid,
    .card-grid,
    .transcript-grid {
      display: grid;
      gap: 18px;
      margin-bottom: 22px;
    }

    .summary-grid {
      grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
    }

    .lane-grid,
    .report-grid,
    .decision-grid {
      grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
    }

    .transcript-grid {
      grid-template-columns: repeat(auto-fit, minmax(320px, 1fr));
    }

    .decision-desk {
      margin: 6px 0 28px;
    }

    .decision-header,
    .decision-changes {
      padding: 24px;
      margin-bottom: 18px;
      border-radius: 26px;
    }

    .decision-header {
      background:
        radial-gradient(circle at top left, rgba(37, 99, 235, 0.14), transparent 36%),
        linear-gradient(135deg, rgba(255, 255, 255, 0.90), rgba(248, 250, 252, 0.82)),
        var(--surface);
      box-shadow: var(--shadow-hero);
    }

    .decision-card,
    .remediation-card {
      min-height: 100%;
    }

    .eyebrow {
      margin: 0 0 10px;
      text-transform: uppercase;
      letter-spacing: 0.14em;
      font-family: var(--mono);
      font-size: 11px;
      color: var(--accent);
    }

    h1, h2 {
      margin: 0 0 12px;
      line-height: 1.02;
      font-weight: 700;
      letter-spacing: -0.03em;
      font-family: var(--display);
      color: var(--ink);
    }

    h1 { font-size: clamp(34px, 4.8vw, 62px); }
    h2 { font-size: clamp(24px, 2.7vw, 34px); }

    .hero-copy,
    p {
      margin: 0;
      line-height: 1.72;
      color: var(--ink-soft);
      word-break: break-word;
    }

    .hero-copy {
      max-width: 78ch;
      font-size: 16px;
    }

    .meta-grid {
      display: grid;
      gap: 12px;
      grid-template-columns: repeat(auto-fit, minmax(170px, 1fr));
      margin: 22px 0 0;
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

    dd {
      margin: 0;
      font-size: 15px;
      color: var(--ink);
      font-weight: 600;
      line-height: 1.5;
    }

    .hero-actions,
    .link-row {
      display: flex;
      flex-wrap: wrap;
      gap: 10px;
      margin-top: 18px;
    }

    .compact-row {
      margin-top: 12px;
    }

    .chip-row {
      display: flex;
      flex-wrap: wrap;
      gap: 8px;
      margin: 0 0 14px;
    }

    .chip {
      display: inline-flex;
      align-items: center;
      gap: 6px;
      border-radius: 999px;
      padding: 6px 11px;
      background: var(--accent-soft);
      color: var(--accent-strong);
      border: 1px solid var(--line-strong);
      font-family: var(--mono);
      font-size: 12px;
    }

    .chip span,
    .facet-button span {
      opacity: 0.72;
    }

    .verdict-chip {
      background: rgba(15, 118, 110, 0.10);
      color: var(--success);
      border-color: rgba(15, 118, 110, 0.20);
    }

    .mono-inline,
    code,
    .open-link,
    pre {
      font-family: var(--mono);
    }

    code {
      padding: 0.16em 0.42em;
      border-radius: 999px;
      color: var(--ink);
      background: rgba(15, 23, 42, 0.05);
    }

    .mono-inline {
      margin-top: 10px;
      color: var(--ink);
      font-size: 13px;
    }

    .open-link {
      display: inline-flex;
      align-items: center;
      justify-content: center;
      min-height: 42px;
      padding: 10px 14px;
      border-radius: 999px;
      border: 1px solid var(--line);
      text-decoration: none;
      color: var(--ink);
      background: rgba(255, 255, 255, 0.76);
      box-shadow: var(--shadow-border);
      transition: transform 160ms ease, background 160ms ease, border-color 160ms ease;
    }

    .open-link:hover {
      transform: translateY(-1px);
      background: rgba(255, 255, 255, 0.96);
      border-color: var(--line-strong);
    }

    .primary-link {
      color: #ffffff;
      background: linear-gradient(135deg, #2563eb, #1d4ed8);
      border-color: transparent;
      box-shadow:
        0 18px 36px rgba(37, 99, 235, 0.24),
        0 0 0 1px rgba(255, 255, 255, 0.18) inset;
    }

    .primary-link:hover {
      background: linear-gradient(135deg, #1d4ed8, #1e40af);
    }

    .search-bar {
      display: grid;
      gap: 12px;
      margin: 0 0 20px;
      padding: 20px 22px;
      border-radius: 24px;
      border: 1px solid var(--line);
      background: var(--surface-muted);
      box-shadow: var(--shadow-panel);
    }

    .search-label {
      font-family: var(--mono);
      font-size: 11px;
      letter-spacing: 0.12em;
      text-transform: uppercase;
      color: var(--accent);
    }

    .search-input {
      width: 100%;
      padding: 14px 16px;
      border-radius: 16px;
      border: 1px solid var(--line);
      background: rgba(255, 255, 255, 0.96);
      color: var(--ink);
      font-family: var(--mono);
      font-size: 14px;
    }

    .search-status,
    .empty-result,
    .empty-inline {
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
      padding: 9px 13px;
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

    .entry-card h2,
    .summary-card h2,
    .lane-card h2,
    .decision-card h2 {
      font-size: clamp(20px, 2vw, 28px);
    }

    .section-intro h2,
    .decision-header h2 {
      max-width: 20ch;
    }

    .section-intro p {
      max-width: 72ch;
    }

    .decision-nav {
      margin-top: 4px;
    }

    .check-list,
    .step-list {
      margin: 0;
      padding-left: 20px;
      color: var(--ink-soft);
    }

    .step-list li,
    .check-list li {
      margin-bottom: 12px;
    }

    .change-item {
      margin-bottom: 12px;
    }

    pre {
      margin: 14px 0 0;
      padding: 16px;
      overflow-x: auto;
      border-radius: 20px;
      background: linear-gradient(180deg, #0f172a 0%, #111827 100%);
      color: #e2e8f0;
      border: 1px solid rgba(255, 255, 255, 0.08);
      box-shadow:
        0 18px 40px rgba(15, 23, 42, 0.16),
        0 0 0 1px rgba(255, 255, 255, 0.04) inset;
    }

    pre code {
      padding: 0;
      color: inherit;
      background: transparent;
    }

    .empty-state {
      padding: 28px;
      background: rgba(255, 255, 255, 0.92);
      text-align: center;
    }

    .empty-result {
      margin-top: -4px;
      margin-bottom: 18px;
    }

    a:focus-visible,
    button:focus-visible,
    input:focus-visible {
      outline: 3px solid rgba(37, 99, 235, 0.28);
      outline-offset: 4px;
    }

    @media (max-width: 860px) {
      .page-shell {
        width: min(100vw - 20px, 1240px);
      }

      .hero-card {
        padding: 22px;
      }

      .hero-card,
      .lane-card,
      .summary-card,
      .entry-card,
      .empty-state,
      .section-card,
      .decision-header,
      .decision-changes {
        border-radius: 22px;
      }

      .transcript-grid {
        grid-template-columns: 1fr;
      }
    }

    @media (max-width: 640px) {
      .page-shell {
        padding: 16px 0 28px;
      }

      .hero-card,
      .lane-card,
      .summary-card,
      .entry-card,
      .empty-state,
      .section-card,
      .decision-header,
      .decision-changes {
        padding: 18px;
      }

      h1 { font-size: clamp(30px, 10vw, 44px); }
      h2 { font-size: clamp(22px, 7vw, 30px); }
      .hero-copy { font-size: 15px; }
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

        assert!(html.contains("Start with transcripts"));
        assert!(html.contains("Open saved reports"));
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
