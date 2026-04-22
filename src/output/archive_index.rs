use std::collections::BTreeMap;

use crate::core::archive_index::ArchiveIndexEntry;
use crate::core::integration_report::{
    IntegrationEvidenceGateOutcome, IntegrationEvidenceRemediationBundle, IntegrationReportEntry,
};
use crate::core::search_report::SearchReportEntry;
use crate::core::workbench::{
    OfficialAnswerPinView, ThreadFamilySummary, WorkbenchIndexJsonDocument, WorkspaceTimelineEvent,
};
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
    workbench_index: Option<&WorkbenchIndexJsonDocument>,
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
    let intelligence_section = render_workbench_projection(workbench_index);

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
            "      <p class=\"eyebrow\">archive workbench</p>\n",
            "      <p class=\"hero-kicker\">{title}</p>\n",
            "      <h1>Browse transcript evidence first.</h1>\n",
            "      <p class=\"hero-copy\">这页首先要解决的，不是治理、不是什么 shell inventory，而是“我有哪些 transcript、我该先从哪一条开始”。你可以把它理解成 archive shell proof 和 transcript/browser 正门；search reports、integration evidence 和 governance 仍然在同一张桌子上，但都应该在你先看过 transcript 之后再打开。</p>\n",
            "      <dl class=\"meta-grid\">\n",
            "        <div><dt>生成时间</dt><dd><code>{generated_at}</code></dd></div>\n",
            "        <div><dt>Transcript exports</dt><dd><code>{entry_count}</code></dd></div>\n",
            "        <div><dt>Connectors</dt><dd><code>{connector_count}</code></dd></div>\n",
            "        <div><dt>Primary front door</dt><dd><code>CLI quickstart</code></dd></div>\n",
            "        <div><dt>Workbench style</dt><dd><code>local-first / static / no executor</code></dd></div>\n",
            "      </dl>\n",
            "      <div class=\"hero-actions\">\n",
            "        <a class=\"open-link primary-link\" href=\"#archive-browser\">Start with transcripts</a>\n",
            "        <a class=\"open-link\" href=\"#tool-lanes\">See next doors</a>\n",
            "        <a class=\"open-link\" href=\"#governance-lane\">Open governance snapshot</a>\n",
            "      </div>\n",
            "      <div class=\"link-row compact-row\">\n",
            "        <a class=\"open-link\" href=\"../Search/Reports/index.html\">Open reports shell</a>\n",
            "        <a class=\"open-link\" href=\"../Integration/Reports/index.html\">Open integration reports</a>\n",
            "      </div>\n",
            "    </header>\n",
            "    <section class=\"section-card section-intro\" id=\"archive-browser\">\n",
            "      <p class=\"eyebrow\">Archive browser</p>\n",
            "      <h2>先从对话记录开始，而不是先看治理票据</h2>\n",
            "      <p class=\"hero-copy\">这里是这张 workbench 的主航道。先筛 transcript、打开对话、理解 archive 本身，再决定要不要跳去 search reports 或 integration evidence。换句话说：先看内容，再看 receipts，最后再进治理。</p>\n",
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
            "        <p class=\"eyebrow\">Next door</p>\n",
            "        <h2>Search reports</h2>\n",
            "        <p>当你已经知道要找什么，但想回看 semantic 或 hybrid retrieval 的 receipt 时，再进入这条 lane。它负责保存检索结果，不负责替代 transcript browser。</p>\n",
            "        <p><a class=\"open-link\" href=\"../Search/Reports/index.html\">Open retrieval lane</a></p>\n",
            "      </article>\n",
            "      <article class=\"lane-card\">\n",
            "        <p class=\"eyebrow\">Next door</p>\n",
            "        <h2>Integration evidence</h2>\n",
            "        <p>当你要确认 attach / onboard / doctor 结果时，再进入 integration evidence。它是一条 receipts shelf，不会重新执行诊断，也不会反过来盖过 transcript 主航道。</p>\n",
            "        <p><a class=\"open-link\" href=\"../Integration/Reports/index.html\">Open integration lane</a></p>\n",
            "      </article>\n",
            "      <article class=\"lane-card\">\n",
            "        <p class=\"eyebrow\">What this page proves</p>\n",
            "        <h2>This shell proves local browsing, not hosted orchestration.</h2>\n",
            "        <p>你现在看到的是 local-first 的静态前厅。它会把 transcript、retrieval receipts 和 governance receipts 组织成可浏览 workbench，但不会在浏览器里替你执行 doctor、onboard 或 gate。</p>\n",
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
            "    <section class=\"summary-grid\" aria-label=\"archive inventory summaries\">\n",
            "{connector_summary}\n",
            "{completeness_summary}\n",
            "{source_summary}\n",
            "{report_summary}\n",
            "{integration_summary}\n",
            "    </section>\n",
            "{intelligence_section}\n",
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
        entry_suffix = if entries.len() == 1 { "" } else { "s" },
        decision_section = decision_section,
        connector_facets = connector_facets,
        completeness_facets = completeness_facets,
        connector_summary = connector_summary,
        completeness_summary = completeness_summary,
        source_summary = source_summary,
        report_summary = report_summary,
        integration_summary = integration_summary,
        intelligence_section = intelligence_section,
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
    let ai_summary_line = entry
        .ai_summary_href
        .as_deref()
        .map(|href| {
            format!(
                "<p><a class=\"open-link\" href=\"{href}\">Open AI summary</a></p>",
                href = escape_html(href)
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
        if entry.ai_summary_href.is_some() {
            "ai summary"
        } else {
            ""
        },
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
            "{ai_summary_line}",
            "</article>"
        ),
        title = escape_html(&entry.title),
        chips = meta_lines.join(" "),
        thread_line = thread_line,
        exported_line = exported_line,
        file_name = escape_html(&entry.file_name),
        href = escape_html(&entry.relative_href),
        ai_summary_line = ai_summary_line,
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
    format!(
        concat!(
            "<section class=\"decision-desk\" aria-label=\"decision desk\">\n",
            "  <header class=\"decision-header\">\n",
            "    <p class=\"eyebrow\">Governance snapshot</p>\n",
            "    <h2>See governance only after you already know the content and receipts.</h2>\n",
            "    <p class=\"hero-copy\">这块只给 archive 首页一张压缩版治理快照：official baseline、candidate verdict、promotion status 和 remediation count。完整比较、历史和详细接线票据应该去 integration evidence lane，而不是继续在首页扩成第二个主舞台。</p>\n",
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
    )
}

fn render_workbench_projection(workbench_index: Option<&WorkbenchIndexJsonDocument>) -> String {
    let Some(workbench_index) = workbench_index else {
        return String::new();
    };

    let timeline = render_workspace_timeline(&workbench_index.timeline);
    let families = render_thread_families(&workbench_index.families);
    let official_answers = render_official_answers(&workbench_index.official_answers);
    let fleet = render_fleet_view(&workbench_index.fleet_view);

    format!(
        concat!(
            "<section class=\"decision-desk\" aria-label=\"workspace intelligence\">\n",
            "  <header class=\"decision-header\">\n",
            "    <p class=\"eyebrow\">Workspace intelligence</p>\n",
            "    <h2>See what changed across transcripts, summaries, reports, and pins.</h2>\n",
            "    <p class=\"hero-copy\">这一层把 transcript export、AI summary、retrieval receipt、integration evidence 和 pinned official answers 收成同一张 workspace intelligence 视图。它不替代 transcript browser，但会帮你更快回答：最近发生了什么、哪些导出属于同一案件、哪些官方答案已经 stale。</p>\n",
            "    <div class=\"verdict-strip\">\n",
            "      <span class=\"chip\">timeline <span>{timeline_count}</span></span>\n",
            "      <span class=\"chip\">families <span>{family_count}</span></span>\n",
            "      <span class=\"chip\">official answers <span>{pin_count}</span></span>\n",
            "      <a class=\"open-link\" href=\"share-safe-packet.md\">Open share-safe packet</a>\n",
            "      <a class=\"open-link\" href=\"share-safe-packet.public.md\">Open public-safe packet</a>\n",
            "      <a class=\"open-link\" href=\"fleet-view.html\">Open fleet board</a>\n",
            "      <a class=\"open-link\" href=\"action-packs/index.html\">Open action bridge</a>\n",
            "      <a class=\"open-link\" href=\"memory-lane.html\">Open memory lane</a>\n",
            "    </div>\n",
            "  </header>\n",
            "  <div class=\"decision-grid\">\n",
            "{timeline}\n",
            "{families}\n",
            "{official_answers}\n",
            "{fleet}\n",
            "  </div>\n",
            "</section>\n"
        ),
        timeline_count = workbench_index.timeline.len(),
        family_count = workbench_index.families.len(),
        pin_count = workbench_index.official_answers.len(),
        timeline = timeline,
        families = families,
        official_answers = official_answers,
        fleet = fleet,
    )
}

fn render_workspace_timeline(events: &[WorkspaceTimelineEvent]) -> String {
    let body = if events.is_empty() {
        "<p class=\"empty-inline\">No workspace activity has been projected yet.</p>".to_string()
    } else {
        format!(
            "<ol class=\"step-list\">{}</ol>",
            events
                .iter()
                .take(8)
                .map(|event| {
                    let summary = event
                        .summary
                        .as_deref()
                        .filter(|value| !value.trim().is_empty())
                        .map(|value| format!("<p>{}</p>", escape_html(value)))
                        .unwrap_or_default();
                    let href = event
                        .href
                        .as_deref()
                        .map(|href| {
                            format!(
                                "<p><a class=\"open-link\" href=\"{href}\">Open artifact</a></p>",
                                href = escape_html(href)
                            )
                        })
                        .unwrap_or_default();
                    format!(
                        concat!(
                            "<li>",
                            "<strong>{title}</strong>",
                            "<p class=\"mono-inline\">{kind} · <code>{occurred_at}</code></p>",
                            "{summary}",
                            "{href}",
                            "</li>"
                        ),
                        title = escape_html(&event.title),
                        kind = escape_html(&event.kind),
                        occurred_at = escape_html(&event.occurred_at),
                        summary = summary,
                        href = href,
                    )
                })
                .collect::<Vec<_>>()
                .join("")
        )
    };

    format!(
        concat!(
            "<article class=\"summary-card decision-card\">",
            "<p class=\"eyebrow\">Workspace timeline</p>",
            "<h2>Recent activity across the workbench</h2>",
            "{body}",
            "</article>"
        ),
        body = body,
    )
}

fn render_thread_families(families: &[ThreadFamilySummary]) -> String {
    let body = if families.is_empty() {
        "<p class=\"empty-inline\">No stitched thread families yet.</p>".to_string()
    } else {
        families
            .iter()
            .take(6)
            .map(|family| {
                let profiles = if family.profiles.is_empty() {
                    String::new()
                } else {
                    format!(
                        "<p class=\"mono-inline\">profiles: <code>{}</code></p>",
                        escape_html(&family.profiles.join(", "))
                    )
                };
                let files = if family.files_touched.is_empty() {
                    String::new()
                } else {
                    format!(
                        "<p>files: {}</p>",
                        escape_html(&family.files_touched.join(", "))
                    )
                };
                let tests = if family.tests_run.is_empty() {
                    String::new()
                } else {
                    format!(
                        "<p>tests: {}</p>",
                        escape_html(&family.tests_run.join(", "))
                    )
                };
                let risks = if family.risks.is_empty() {
                    String::new()
                } else {
                    format!(
                        "<p>risks: {}</p>",
                        escape_html(&family.risks.join(", "))
                    )
                };
                let blockers = if family.blockers.is_empty() {
                    String::new()
                } else {
                    format!(
                        "<p>blockers: {}</p>",
                        escape_html(&family.blockers.join(", "))
                    )
                };
                let latest_summary = family
                    .latest_summary
                    .as_ref()
                    .map(|summary| {
                        format!(
                            concat!(
                                "<section class=\"nested-card\">",
                                "<p class=\"eyebrow\">Case view</p>",
                                "<h3>{title}</h3>",
                                "<p>{overview}</p>",
                                "<p class=\"mono-inline\">profile: <code>{profile}</code> · generated: <code>{generated_at}</code></p>",
                                "<p><a class=\"open-link\" href=\"{href}\">Open latest summary</a></p>",
                                "<p class=\"mono-inline\">Pin as official answer: <code>agent-exporter publish pin-answer --workspace-root &lt;repo&gt; --artifact ./{json_href} --label &quot;{label}&quot;</code></p>",
                                "</section>"
                            ),
                            title = escape_html(&summary.title),
                            overview = escape_html(&summary.overview),
                            profile = escape_html(&summary.profile_id),
                            generated_at = escape_html(&summary.generated_at),
                            href = escape_html(&summary.href),
                            json_href = escape_html(&summary.json_href),
                            label = escape_html(&family.label),
                        )
                    })
                    .unwrap_or_default();
                let official_answer = family
                    .official_answer
                    .as_ref()
                    .map(|official| {
                        format!(
                            concat!(
                                "<section class=\"nested-card\">",
                                "<p class=\"eyebrow\">Official answer</p>",
                                "<h3>{label}</h3>",
                                "<div class=\"chip-row\">",
                                "<span class=\"chip\">{status}</span>",
                                "<span class=\"chip\">{stale}</span>",
                                "</div>",
                                "<p>{summary}</p>",
                                "{note}",
                                "<p><a class=\"open-link\" href=\"{href}\">Open pinned answer</a></p>",
                                "</section>"
                            ),
                            label = escape_html(&official.label),
                            status = escape_html(&official.status),
                            stale = escape_html(if official.stale { "stale" } else { "fresh" }),
                            summary = escape_html(&official.summary),
                            note = official
                                .note
                                .as_deref()
                                .map(|note| format!("<p>{}</p>", escape_html(note)))
                                .unwrap_or_default(),
                            href = escape_html(&official.href),
                        )
                    })
                    .unwrap_or_default();
                let latest_vs_pinned = family
                    .latest_vs_pinned
                    .as_ref()
                    .map(|diff| {
                        let new_files = if diff.new_files_touched.is_empty() {
                            String::new()
                        } else {
                            format!(
                                "<p>new files: {}</p>",
                                escape_html(&diff.new_files_touched.join(", "))
                            )
                        };
                        let new_tests = if diff.new_tests_run.is_empty() {
                            String::new()
                        } else {
                            format!(
                                "<p>new tests: {}</p>",
                                escape_html(&diff.new_tests_run.join(", "))
                            )
                        };
                        let new_risks = if diff.new_risks.is_empty() {
                            String::new()
                        } else {
                            format!(
                                "<p>new risks: {}</p>",
                                escape_html(&diff.new_risks.join(", "))
                            )
                        };
                        let new_blockers = if diff.new_blockers.is_empty() {
                            String::new()
                        } else {
                            format!(
                                "<p>new blockers: {}</p>",
                                escape_html(&diff.new_blockers.join(", "))
                            )
                        };
                        let new_next_steps = if diff.new_next_steps.is_empty() {
                            String::new()
                        } else {
                            format!(
                                "<p>new next steps: {}</p>",
                                escape_html(&diff.new_next_steps.join(", "))
                            )
                        };
                        format!(
                            concat!(
                                "<section class=\"nested-card\">",
                                "<p class=\"eyebrow\">Latest vs pinned answer</p>",
                                "<h3>{status}</h3>",
                                "<p>{summary}</p>",
                                "{new_files}",
                                "{new_tests}",
                                "{new_risks}",
                                "{new_blockers}",
                                "{new_next_steps}",
                                "</section>"
                            ),
                            status = escape_html(&diff.status),
                            summary = escape_html(&diff.summary),
                            new_files = new_files,
                            new_tests = new_tests,
                            new_risks = new_risks,
                            new_blockers = new_blockers,
                            new_next_steps = new_next_steps,
                        )
                    })
                    .unwrap_or_default();
                let href = family
                    .latest_href
                    .as_deref()
                    .map(|href| {
                        format!(
                            "<p><a class=\"open-link\" href=\"{href}\">Open latest artifact</a></p>",
                            href = escape_html(href)
                        )
                    })
                    .unwrap_or_default();
                format!(
                    concat!(
                        "<article class=\"entry-card\">",
                        "<p class=\"eyebrow\">Thread family</p>",
                        "<h2>{label}</h2>",
                        "<div class=\"chip-row\">",
                        "<span class=\"chip\">transcripts <span>{transcript_count}</span></span>",
                        "<span class=\"chip\">summaries <span>{summary_count}</span></span>",
                        "</div>",
                        "<p class=\"mono-inline\">latest: <code>{latest_at}</code></p>",
                        "{profiles}",
                        "{files}",
                        "{tests}",
                        "{risks}",
                        "{blockers}",
                        "{latest_summary}",
                        "{official_answer}",
                        "{latest_vs_pinned}",
                        "{href}",
                        "</article>"
                    ),
                    label = escape_html(&family.label),
                    transcript_count = family.transcript_count,
                    summary_count = family.summary_count,
                    latest_at = escape_html(&family.latest_at),
                    profiles = profiles,
                    files = files,
                    tests = tests,
                    risks = risks,
                    blockers = blockers,
                    latest_summary = latest_summary,
                    official_answer = official_answer,
                    latest_vs_pinned = latest_vs_pinned,
                    href = href,
                )
            })
            .collect::<Vec<_>>()
            .join("")
    };

    format!(
        concat!(
            "<article class=\"summary-card decision-card\">",
            "<p class=\"eyebrow\">Thread families</p>",
            "<h2>Stitch repeated exports into cases instead of fragments</h2>",
            "{body}",
            "</article>"
        ),
        body = body,
    )
}

fn render_official_answers(pins: &[OfficialAnswerPinView]) -> String {
    let body = if pins.is_empty() {
        "<p class=\"empty-inline\">No official answers pinned yet.</p>".to_string()
    } else {
        pins.iter()
            .take(6)
            .map(|pin| {
                let stale = if pin.stale { "stale" } else { "fresh" };
                let stale_reason = pin
                    .stale_reason
                    .as_deref()
                    .map(|reason| format!("<p>{}</p>", escape_html(reason)))
                    .unwrap_or_default();
                let lifecycle_summary = pin
                    .lifecycle_summary
                    .as_deref()
                    .map(|summary| format!("<p>{}</p>", escape_html(summary)))
                    .unwrap_or_default();
                let note = pin
                    .note
                    .as_deref()
                    .map(|note| format!("<p>{}</p>", escape_html(note)))
                    .unwrap_or_default();
                let artifact_json = std::path::Path::new(&pin.artifact_json_path)
                    .file_name()
                    .and_then(|value| value.to_str())
                    .unwrap_or(pin.artifact_json_path.as_str());
                format!(
                    concat!(
                        "<article class=\"entry-card\">",
                        "<p class=\"eyebrow\">Official answers</p>",
                        "<h2>{label}</h2>",
                        "<div class=\"chip-row\">",
                        "<span class=\"chip\">{kind}</span>",
                        "<span class=\"chip\">{status}</span>",
                        "<span class=\"chip\">{stale}</span>",
                        "</div>",
                        "<p>{summary}</p>",
                        "<p class=\"mono-inline\">generated: <code>{generated_at}</code></p>",
                        "{lifecycle_summary}",
                        "{stale_reason}",
                        "{note}",
                        "<p><a class=\"open-link\" href=\"{href}\">Open pinned artifact</a></p>",
                        "<p class=\"mono-inline\">Resolve via CLI: <code>agent-exporter publish resolve-answer --workspace-root &lt;repo&gt; --label &quot;{label}&quot;</code></p>",
                        "<p class=\"mono-inline\">Unpin via CLI: <code>agent-exporter publish unpin-answer --workspace-root &lt;repo&gt; --label &quot;{label}&quot;</code></p>",
                        "<p class=\"mono-inline\">Supersede via CLI: <code>agent-exporter publish pin-answer --workspace-root &lt;repo&gt; --artifact ./{artifact_json} --label &quot;next-{label}&quot; --supersedes &quot;{label}&quot;</code></p>",
                        "</article>"
                    ),
                    label = escape_html(&pin.label),
                    kind = escape_html(&pin.artifact_kind),
                    status = escape_html(&pin.status),
                    stale = escape_html(stale),
                    summary = escape_html(&pin.summary),
                    generated_at = escape_html(&pin.generated_at),
                    lifecycle_summary = lifecycle_summary,
                    stale_reason = stale_reason,
                    note = note,
                    href = escape_html(&pin.href),
                    artifact_json = escape_html(artifact_json),
                )
            })
            .collect::<Vec<_>>()
            .join("")
    };

    format!(
        concat!(
            "<article class=\"summary-card decision-card\">",
            "<p class=\"eyebrow\">Official answers</p>",
            "<h2>Keep the current official answer visible and auditable</h2>",
            "{body}",
            "</article>"
        ),
        body = body,
    )
}

fn render_fleet_view(entries: &[crate::core::workbench::FleetViewEntry]) -> String {
    let body = if entries.is_empty() {
        "<p class=\"empty-inline\">No integration fleet snapshot yet.</p>".to_string()
    } else {
        entries
            .iter()
            .take(6)
            .map(|entry| {
                let href = entry
                    .html_href
                    .as_deref()
                    .map(|href| {
                        format!(
                            "<p><a class=\"open-link\" href=\"{href}\">Open latest fleet report</a></p>",
                            href = escape_html(href)
                        )
                    })
                    .unwrap_or_default();
                format!(
                    concat!(
                        "<article class=\"entry-card\">",
                        "<p class=\"eyebrow\">Fleet view</p>",
                        "<h2>{platform}</h2>",
                        "<div class=\"chip-row\">",
                        "<span class=\"chip\">{readiness}</span>",
                        "<span class=\"chip\">reports <span>{report_count}</span></span>",
                        "</div>",
                        "<p class=\"mono-inline\">target: <code>{target}</code></p>",
                        "<p class=\"mono-inline\">latest: <code>{latest}</code></p>",
                        "{href}",
                        "</article>"
                    ),
                    platform = escape_html(&entry.platform),
                    readiness = escape_html(&entry.latest_readiness),
                    report_count = entry.report_count,
                    target = escape_html(&entry.target),
                    latest = escape_html(&entry.latest_generated_at),
                    href = href,
                )
            })
            .collect::<Vec<_>>()
            .join("")
    };

    format!(
        concat!(
            "<article class=\"summary-card decision-card\">",
            "<p class=\"eyebrow\">Fleet view</p>",
            "<h2>See the latest readiness snapshot across targets</h2>",
            "{body}",
            "</article>"
        ),
        body = body,
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
      background: rgba(255, 255, 255, 0.9);
      box-shadow: var(--shadow-panel);
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

    .hero-kicker {
      margin: 0 0 12px;
      color: var(--muted);
      font-family: var(--mono);
      font-size: 13px;
      letter-spacing: 0.04em;
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
                ai_summary_href: Some("demo-ai-summary.md".to_string()),
            }],
            &[],
            &[],
            None,
            None,
        );

        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("Demo archive"));
        assert!(html.contains("demo.html"));
        assert!(html.contains("Open transcript"));
        assert!(html.contains("Open AI summary"));
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
                ai_summary_href: None,
            }],
            &[],
            &[],
            None,
            None,
        );

        assert!(html.contains("Start with transcripts"));
        assert!(html.contains("Open governance snapshot"));
        assert!(html.contains("Open reports shell"));
        assert!(html.contains("Open retrieval lane"));
        assert!(html.contains("Open integration lane"));
        assert!(html.contains("data-filter-group=\"connector\""));
        assert!(html.contains("data-filter-group=\"completeness\""));
        assert!(html.contains("Open integration reports"));
    }
}
