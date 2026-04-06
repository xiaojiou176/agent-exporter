use std::collections::BTreeMap;

use crate::core::integration_report::IntegrationReportEntry;
use crate::integrations::IntegrationDoctorCheck;

use super::html::escape_html;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IntegrationReportKind {
    Doctor,
    Onboard,
}

impl IntegrationReportKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Doctor => "doctor",
            Self::Onboard => "onboard",
        }
    }

    fn title(self) -> &'static str {
        match self {
            Self::Doctor => "Integration Doctor Report",
            Self::Onboard => "Integration Onboarding Report",
        }
    }

    fn description(self) -> &'static str {
        match self {
            Self::Doctor => {
                "这份 report 是一张 integration 体检单。它会把当时 doctor 看到的 readiness、关键检查和下一步建议静态保存下来，方便复查和留痕。"
            }
            Self::Onboard => {
                "这份 report 是一张 onboarding 结果单。它会把 materialize、doctor 和记下来的 next steps 收成一页静态证据，方便后续回看或分享。"
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntegrationReportDocument {
    pub kind: IntegrationReportKind,
    pub platform: String,
    pub target_root: String,
    pub generated_at: String,
    pub readiness: String,
    pub summary: String,
    pub launcher_kind: String,
    pub launcher_command: String,
    pub written_files: Vec<String>,
    pub unchanged_files: Vec<String>,
    pub checks: Vec<IntegrationDoctorCheck>,
    pub next_steps: Vec<String>,
}

pub fn render_integration_report_document(report: &IntegrationReportDocument) -> String {
    let written_list = if report.written_files.is_empty() {
        "<p class=\"empty-inline\">No new files were written in this run.</p>".to_string()
    } else {
        format!(
            "<ul class=\"mono-list\">{}</ul>",
            report
                .written_files
                .iter()
                .map(|path| format!("<li><code>{}</code></li>", escape_html(path)))
                .collect::<Vec<_>>()
                .join("")
        )
    };

    let unchanged_list = if report.unchanged_files.is_empty() {
        "<p class=\"empty-inline\">No unchanged files were reported.</p>".to_string()
    } else {
        format!(
            "<ul class=\"mono-list\">{}</ul>",
            report
                .unchanged_files
                .iter()
                .map(|path| format!("<li><code>{}</code></li>", escape_html(path)))
                .collect::<Vec<_>>()
                .join("")
        )
    };

    let checks = if report.checks.is_empty() {
        "<p class=\"empty-inline\">No doctor checks were captured.</p>".to_string()
    } else {
        report
            .checks
            .iter()
            .map(render_check)
            .collect::<Vec<_>>()
            .join("\n")
    };

    let next_steps = if report.next_steps.is_empty() {
        "<p class=\"empty-inline\">No extra next steps were generated.</p>".to_string()
    } else {
        format!(
            "<ol class=\"step-list\">{}</ol>",
            report
                .next_steps
                .iter()
                .map(|step| format!("<li>{}</li>", escape_html(step)))
                .collect::<Vec<_>>()
                .join("")
        )
    };

    let title = format!("{} - {}", report.platform, report.kind.title());
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
            "  <meta name=\"agent-exporter:integration-platform\" content=\"{platform}\">\n",
            "  <meta name=\"agent-exporter:integration-readiness\" content=\"{readiness}\">\n",
            "  <meta name=\"agent-exporter:integration-target\" content=\"{target_root}\">\n",
            "  <meta name=\"agent-exporter:generated-at\" content=\"{generated_at}\">\n",
            "  <style>\n{style}\n  </style>\n",
            "</head>\n",
            "<body>\n",
            "  <main class=\"page-shell\">\n",
            "    <header class=\"hero-card\">\n",
            "      <p class=\"eyebrow\">agent-exporter integration evidence</p>\n",
            "      <h1>{title}</h1>\n",
            "      <p class=\"hero-copy\">{description}</p>\n",
            "      <dl class=\"meta-grid\">\n",
            "        <div><dt>Platform</dt><dd><code>{platform}</code></dd></div>\n",
            "        <div><dt>Kind</dt><dd><code>{kind}</code></dd></div>\n",
            "        <div><dt>Generated</dt><dd><code>{generated_at}</code></dd></div>\n",
            "        <div><dt>Readiness</dt><dd><code>{readiness}</code></dd></div>\n",
            "        <div><dt>Target</dt><dd><code>{target_root}</code></dd></div>\n",
            "        <div><dt>Launcher</dt><dd><code>{launcher_kind}</code></dd></div>\n",
            "      </dl>\n",
            "      <p class=\"summary-card\">{summary}</p>\n",
            "      <p class=\"mono-inline\">command: <code>{launcher_command}</code></p>\n",
            "      <div class=\"link-row\">\n",
            "        <a class=\"open-link\" href=\"index.html\">Open integration reports</a>\n",
            "      </div>\n",
            "    </header>\n",
            "    <section class=\"section-card\">\n",
            "      <h2>Checks</h2>\n",
            "      <div class=\"check-grid\">{checks}</div>\n",
            "    </section>\n",
            "    <section class=\"section-card\">\n",
            "      <h2>Next Steps</h2>\n",
            "      {next_steps}\n",
            "    </section>\n",
            "    <section class=\"section-card two-col\">\n",
            "      <div>\n",
            "        <h2>Files Written</h2>\n",
            "        {written_list}\n",
            "      </div>\n",
            "      <div>\n",
            "        <h2>Files Unchanged</h2>\n",
            "        {unchanged_list}\n",
            "      </div>\n",
            "    </section>\n",
            "  </main>\n",
            "</body>\n",
            "</html>\n"
        ),
        title = escape_html(&title),
        kind = escape_html(report.kind.as_str()),
        platform = escape_html(&report.platform),
        readiness = escape_html(&report.readiness),
        target_root = escape_html(&report.target_root),
        generated_at = escape_html(&report.generated_at),
        launcher_kind = escape_html(&report.launcher_kind),
        launcher_command = escape_html(&report.launcher_command),
        summary = escape_html(&report.summary),
        description = escape_html(report.kind.description()),
        checks = checks,
        next_steps = next_steps,
        written_list = written_list,
        unchanged_list = unchanged_list,
        style = integration_report_style(),
    )
}

pub fn render_integration_reports_index_document(
    title: &str,
    generated_at: &str,
    reports: &[IntegrationReportEntry],
) -> String {
    let body = if reports.is_empty() {
        "<section class=\"empty-state\"><h2>No integration reports yet</h2><p>Run `doctor integrations --save-report` or `onboard --save-report` to save a local integration evidence page here.</p></section>".to_string()
    } else {
        reports
            .iter()
            .map(render_index_card)
            .collect::<Vec<_>>()
            .join("\n")
    };

    let platform_facets = render_filter_buttons(
        "platform",
        "Platform",
        summarize_reports_by(reports, |entry| {
            entry.platform.as_deref().unwrap_or("unknown")
        }),
    );
    let readiness_facets = render_filter_buttons(
        "readiness",
        "Readiness",
        summarize_reports_by(reports, |entry| {
            entry.readiness.as_deref().unwrap_or("unknown")
        }),
    );

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
            "      <p class=\"eyebrow\">agent-exporter integration reports</p>\n",
            "      <h1>{title}</h1>\n",
            "      <p class=\"hero-copy\">这是一张 integration evidence 的本地 front door。它只组织已经保存下来的 onboarding/doctor 报告，不会重新执行诊断，也不会进入 transcript/search corpus。</p>\n",
            "      <dl class=\"meta-grid\">\n",
            "        <div><dt>Generated</dt><dd><code>{generated_at}</code></dd></div>\n",
            "        <div><dt>Saved reports</dt><dd><code>{report_count}</code></dd></div>\n",
            "      </dl>\n",
            "    </header>\n",
            "    <section class=\"search-bar\" aria-label=\"integration reports search\">\n",
            "      <label class=\"search-label\" for=\"integration-report-search\">Report search</label>\n",
            "      <input id=\"integration-report-search\" class=\"search-input\" type=\"search\" placeholder=\"Search title, platform, readiness, target...\" autocomplete=\"off\">\n",
            "      <div class=\"facet-grid\">\n",
            "{platform_facets}\n",
            "{readiness_facets}\n",
            "      </div>\n",
            "      <p id=\"integration-report-status\" class=\"search-status\">Showing <strong>{report_count}</strong> reports.</p>\n",
            "    </section>\n",
            "    <section class=\"card-grid\">\n",
            "{body}\n",
            "    </section>\n",
            "    <p id=\"integration-report-empty\" class=\"empty-result\" hidden>No integration reports matched the current search.</p>\n",
            "  </main>\n",
            "  <script>\n{script}\n  </script>\n",
            "</body>\n",
            "</html>\n"
        ),
        title = escape_html(title),
        generated_at = escape_html(generated_at),
        report_count = reports.len(),
        platform_facets = platform_facets,
        readiness_facets = readiness_facets,
        body = body,
        style = integration_report_style(),
        script = integration_report_script(),
    )
}

fn render_check(check: &IntegrationDoctorCheck) -> String {
    format!(
        concat!(
            "<article class=\"check-card\">",
            "<p class=\"eyebrow\">{label}</p>",
            "<p class=\"chip-row\"><span class=\"chip\">{readiness}</span></p>",
            "<p class=\"check-detail\">{detail}</p>",
            "</article>"
        ),
        label = escape_html(check.label),
        readiness = escape_html(check.readiness.as_str()),
        detail = escape_html(&check.detail),
    )
}

fn render_index_card(entry: &IntegrationReportEntry) -> String {
    let platform = entry.platform.as_deref().unwrap_or("unknown");
    let readiness = entry.readiness.as_deref().unwrap_or("unknown");
    let searchable_text = [
        entry.title.as_str(),
        entry.report_kind.as_deref().unwrap_or(""),
        platform,
        readiness,
        entry.target_root.as_deref().unwrap_or(""),
        entry.generated_at.as_deref().unwrap_or(""),
    ]
    .join(" ")
    .to_lowercase();

    let target_line = entry
        .target_root
        .as_deref()
        .map(|target| {
            format!(
                "<p class=\"mono-inline\">target: <code>{}</code></p>",
                escape_html(target)
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
            "<article class=\"entry-card\" data-search-text=\"{searchable_text}\" data-platform=\"{platform}\" data-readiness=\"{readiness}\">",
            "<p class=\"eyebrow\">Saved integration report</p>",
            "<h2>{title}</h2>",
            "<div class=\"chip-row\">",
            "<span class=\"chip\">{kind}</span>",
            "<span class=\"chip\">{platform}</span>",
            "<span class=\"chip\">{readiness}</span>",
            "</div>",
            "{target_line}",
            "{generated_line}",
            "<p><a class=\"open-link\" href=\"{href}\">Open report</a></p>",
            "</article>"
        ),
        searchable_text = escape_html(&searchable_text),
        platform = escape_html(platform),
        readiness = escape_html(readiness),
        title = escape_html(&entry.title),
        kind = escape_html(entry.report_kind.as_deref().unwrap_or("unknown")),
        target_line = target_line,
        generated_line = generated_line,
        href = escape_html(entry.relative_href.trim_start_matches("./")),
    )
}

fn summarize_reports_by<F>(entries: &[IntegrationReportEntry], label: F) -> Vec<(String, usize)>
where
    F: Fn(&IntegrationReportEntry) -> &str,
{
    let mut counts = BTreeMap::new();
    for entry in entries {
        *counts.entry(label(entry).to_string()).or_insert(0usize) += 1;
    }
    counts.into_iter().collect()
}

fn render_filter_buttons(group: &str, label: &str, items: Vec<(String, usize)>) -> String {
    let mut buttons = vec![format!(
        "<div class=\"facet-group\"><span class=\"facet-label\">{}</span><button type=\"button\" class=\"facet-button active\" data-filter-group=\"{}\" data-filter-value=\"all\">All <span>{}</span></button>",
        escape_html(label),
        escape_html(group),
        items.iter().map(|(_, count)| count).sum::<usize>()
    )];

    buttons.extend(items.into_iter().map(|(value, count)| {
        format!(
            "<button type=\"button\" class=\"facet-button\" data-filter-group=\"{}\" data-filter-value=\"{}\">{} <span>{}</span></button>",
            escape_html(group),
            escape_html(&value),
            escape_html(&value),
            count
        )
    }));
    buttons.push("</div>".to_string());
    buttons.join("")
}

fn integration_report_style() -> &'static str {
    r#"
  :root {
    color-scheme: light;
    --bg: #f4efe6;
    --ink: #1f1a14;
    --muted: #6c6256;
    --card: #fffdf8;
    --line: #d8ccb9;
    --accent: #946b2d;
    --accent-soft: #f1e3c8;
  }
  * { box-sizing: border-box; }
  body {
    margin: 0;
    font-family: "Iowan Old Style", "Palatino Linotype", serif;
    background: linear-gradient(180deg, #f7f0e2 0%, var(--bg) 100%);
    color: var(--ink);
  }
  .page-shell {
    max-width: 1040px;
    margin: 0 auto;
    padding: 32px 20px 56px;
  }
  .hero-card, .section-card, .entry-card {
    background: rgba(255, 253, 248, 0.94);
    border: 1px solid var(--line);
    border-radius: 24px;
    box-shadow: 0 18px 48px rgba(61, 47, 26, 0.08);
  }
  .hero-card, .section-card {
    padding: 24px;
    margin-bottom: 24px;
  }
  .eyebrow {
    margin: 0 0 8px;
    font-size: 12px;
    letter-spacing: 0.14em;
    text-transform: uppercase;
    color: var(--muted);
  }
  h1, h2 { margin: 0 0 12px; }
  .hero-copy, .summary-card, .check-detail, .search-status, .empty-inline, .empty-state p, .empty-result {
    color: var(--muted);
    line-height: 1.6;
  }
  .summary-card {
    padding: 14px 16px;
    background: var(--accent-soft);
    border-radius: 16px;
    color: var(--ink);
    margin: 12px 0 16px;
  }
  .meta-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
    gap: 12px 18px;
    margin: 0 0 16px;
  }
  .meta-grid dt {
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--muted);
    margin-bottom: 4px;
  }
  .meta-grid dd, .mono-inline, code {
    margin: 0;
    font-family: "SFMono-Regular", "Menlo", monospace;
    font-size: 13px;
  }
  .link-row, .chip-row {
    display: flex;
    flex-wrap: wrap;
    gap: 10px;
  }
  .open-link {
    display: inline-flex;
    align-items: center;
    padding: 10px 14px;
    border-radius: 999px;
    text-decoration: none;
    border: 1px solid var(--line);
    color: var(--ink);
    background: #fff;
  }
  .chip {
    display: inline-flex;
    padding: 6px 10px;
    border-radius: 999px;
    background: var(--accent-soft);
    color: var(--ink);
    font-size: 12px;
  }
  .check-grid, .card-grid, .two-col {
    display: grid;
    gap: 16px;
  }
  .check-grid {
    grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
  }
  .card-grid {
    grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
  }
  .two-col {
    grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
  }
  .check-card, .entry-card {
    padding: 18px;
  }
  .mono-list, .step-list {
    margin: 0;
    padding-left: 20px;
  }
  .search-bar {
    padding: 20px 24px;
    margin-bottom: 24px;
    background: rgba(255, 253, 248, 0.92);
    border: 1px solid var(--line);
    border-radius: 22px;
  }
  .search-label, .facet-label {
    display: block;
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--muted);
    margin-bottom: 8px;
  }
  .search-input {
    width: 100%;
    padding: 12px 14px;
    border-radius: 14px;
    border: 1px solid var(--line);
    background: #fff;
    margin-bottom: 16px;
  }
  .facet-grid {
    display: grid;
    gap: 12px;
    margin-bottom: 10px;
  }
  .facet-group {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    align-items: center;
  }
  .facet-button {
    border: 1px solid var(--line);
    background: #fff;
    border-radius: 999px;
    padding: 8px 12px;
    cursor: pointer;
  }
  .facet-button.active {
    background: var(--accent-soft);
    border-color: var(--accent);
  }
  .empty-state, .empty-result {
    padding: 24px;
    text-align: center;
  }
  [hidden] { display: none !important; }
  @media (max-width: 700px) {
    .page-shell { padding-inline: 14px; }
    .hero-card, .section-card, .search-bar, .entry-card { border-radius: 18px; }
  }
"#
}

fn integration_report_script() -> &'static str {
    r#"
const searchInput = document.getElementById('integration-report-search');
const status = document.getElementById('integration-report-status');
const empty = document.getElementById('integration-report-empty');
const cards = Array.from(document.querySelectorAll('.entry-card'));
const groups = ['platform', 'readiness'];
const active = Object.fromEntries(groups.map((group) => [group, 'all']));

document.querySelectorAll('.facet-button').forEach((button) => {
  button.addEventListener('click', () => {
    const group = button.dataset.filterGroup;
    active[group] = button.dataset.filterValue || 'all';
    document.querySelectorAll(`.facet-button[data-filter-group="${group}"]`).forEach((candidate) => {
      candidate.classList.toggle('active', candidate === button);
    });
    update();
  });
});

searchInput?.addEventListener('input', update);

function update() {
  const query = (searchInput?.value || '').trim().toLowerCase();
  let visible = 0;

  cards.forEach((card) => {
    const searchableText = card.dataset.searchText || '';
    const matchesQuery = !query || searchableText.includes(query);
    const matchesFilters = groups.every((group) => active[group] === 'all' || card.dataset[group] === active[group]);
    const show = matchesQuery && matchesFilters;
    card.hidden = !show;
    if (show) visible += 1;
  });

  if (status) {
    status.innerHTML = `Showing <strong>${visible}</strong> reports.`;
  }
  if (empty) {
    empty.hidden = visible !== 0;
  }
}

#[cfg(test)]
mod tests {
    use crate::core::integration_report::IntegrationReportEntry;
    use crate::integrations::{IntegrationDoctorCheck, IntegrationReadiness};

    use super::{
        IntegrationReportDocument, IntegrationReportKind, render_integration_report_document,
        render_integration_reports_index_document,
    };

    #[test]
    fn render_integration_report_document_includes_meta_tags() {
        let html = render_integration_report_document(&IntegrationReportDocument {
            kind: IntegrationReportKind::Doctor,
            platform: "codex".to_string(),
            target_root: "/tmp/codex-pack".to_string(),
            generated_at: "2026-04-06T12:00:00Z".to_string(),
            readiness: "ready".to_string(),
            summary: "codex pack looks ready".to_string(),
            launcher_kind: "repo-local-debug".to_string(),
            launcher_command: "/tmp/agent-exporter".to_string(),
            written_files: Vec::new(),
            unchanged_files: Vec::new(),
            checks: vec![IntegrationDoctorCheck {
                label: "target_root",
                readiness: IntegrationReadiness::Ready,
                detail: "/tmp/codex-pack".to_string(),
            }],
            next_steps: vec!["Review the generated pack".to_string()],
        });

        assert!(html.contains("agent-exporter:report-kind\" content=\"doctor"));
        assert!(html.contains("agent-exporter:integration-platform\" content=\"codex"));
        assert!(html.contains("agent-exporter:integration-readiness\" content=\"ready"));
        assert!(html.contains("agent-exporter:integration-target\" content=\"/tmp/codex-pack"));
        assert!(html.contains("Open integration reports"));
    }

    #[test]
    fn render_integration_reports_index_document_includes_search_and_facets() {
        let html = render_integration_reports_index_document(
            "integration reports",
            "2026-04-06T12:00:00Z",
            &[
                IntegrationReportEntry {
                    file_name: "integration-report-doctor-codex.html".to_string(),
                    relative_href: "integration-report-doctor-codex.html".to_string(),
                    title: "Codex doctor".to_string(),
                    report_kind: Some("doctor".to_string()),
                    platform: Some("codex".to_string()),
                    readiness: Some("ready".to_string()),
                    target_root: Some("/tmp/codex-pack".to_string()),
                    generated_at: Some("2026-04-06T12:00:00Z".to_string()),
                },
                IntegrationReportEntry {
                    file_name: "integration-report-onboard-claude-code.html".to_string(),
                    relative_href: "integration-report-onboard-claude-code.html".to_string(),
                    title: "Claude onboard".to_string(),
                    report_kind: Some("onboard".to_string()),
                    platform: Some("claude-code".to_string()),
                    readiness: Some("partial".to_string()),
                    target_root: Some("/tmp/claude-pack".to_string()),
                    generated_at: Some("2026-04-06T12:05:00Z".to_string()),
                },
            ],
        );

        assert!(html.contains("integration-report-search"));
        assert!(html.contains("data-filter-group=\"platform\""));
        assert!(html.contains("data-filter-group=\"readiness\""));
        assert!(html.contains("data-platform=\"codex\""));
        assert!(html.contains("data-platform=\"claude-code\""));
        assert!(html.contains("data-readiness=\"ready\""));
        assert!(html.contains("data-readiness=\"partial\""));
        assert!(html.contains("Search title, platform, readiness, target"));
    }
}
"#
}
