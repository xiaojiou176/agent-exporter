use std::collections::BTreeMap;

use crate::core::integration_report::{
    IntegrationArtifactLinks, IntegrationReportCheckRecord, IntegrationReportEntry,
    IntegrationReportJsonDocument, IntegrationReportTimelineEntry,
    IntegrationReportsIndexJsonDocument,
};
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

pub fn build_integration_report_json_document(
    report: &IntegrationReportDocument,
    html_report_file_name: &str,
    json_report_file_name: &str,
) -> IntegrationReportJsonDocument {
    IntegrationReportJsonDocument {
        schema_version: 1,
        title: format!("{} - {}", report.platform, report.kind.title()),
        kind: report.kind.as_str().to_string(),
        platform: report.platform.clone(),
        target: report.target_root.clone(),
        generated_at: report.generated_at.clone(),
        readiness: report.readiness.clone(),
        summary: report.summary.clone(),
        launcher_status: launcher_status(report),
        launcher_kind: report.launcher_kind.clone(),
        launcher_command: report.launcher_command.clone(),
        bridge_status: bridge_status(report),
        pack_shape_checks: pack_shape_checks(report),
        checks: report.checks.iter().map(check_record).collect::<Vec<_>>(),
        next_steps: report.next_steps.clone(),
        written_files: report.written_files.clone(),
        unchanged_files: report.unchanged_files.clone(),
        artifact_links: IntegrationArtifactLinks {
            html_report: html_report_file_name.to_string(),
            json_report: json_report_file_name.to_string(),
            index_html: "index.html".to_string(),
            index_json: "index.json".to_string(),
        },
    }
}

pub fn build_integration_reports_index_json_document(
    title: &str,
    generated_at: &str,
    reports: &[IntegrationReportJsonDocument],
) -> IntegrationReportsIndexJsonDocument {
    IntegrationReportsIndexJsonDocument {
        schema_version: 1,
        title: title.to_string(),
        generated_at: generated_at.to_string(),
        report_count: reports.len(),
        timeline: reports
            .iter()
            .map(|report| IntegrationReportTimelineEntry {
                title: report.title.clone(),
                kind: report.kind.clone(),
                platform: report.platform.clone(),
                readiness: report.readiness.clone(),
                target: report.target.clone(),
                generated_at: report.generated_at.clone(),
                html_href: report.artifact_links.html_report.clone(),
                json_href: report.artifact_links.json_report.clone(),
            })
            .collect(),
    }
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
            "      <p class=\"hero-copy\">{description} 这页属于 repo-owned integration pack 这条 secondary surface：它是接线结果单，不是整个产品的主门。真正的 primary front door 仍然是 CLI quickstart，archive shell proof 是第一层可浏览证明，transcript/archive 则继续是最先可浏览的工作线。</p>\n",
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
            "        <a class=\"open-link\" href=\"../../Conversations/index.html\">Open archive shell</a>\n",
            "        <a class=\"open-link\" href=\"../../Search/Reports/index.html\">Open retrieval reports</a>\n",
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
    let lane_glance = render_integration_lane_glance(reports);

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
            "      <p class=\"eyebrow\">integration evidence lane</p>\n",
            "      <p class=\"hero-kicker\">{title}</p>\n",
            "      <h1>Review integration evidence before you compare or promote.</h1>\n",
            "      <p class=\"hero-copy\">这页回答的是：当前这套接线更像 ready 还是 partial、最近一次 doctor/onboard 留下了什么 receipt、以及你现在值不值得进入 baseline、policy、promotion 或 remediation 的更深 lane。它是 archive workbench 的侧门，不是主门，也不会在浏览器里重新执行诊断。</p>\n",
            "      <dl class=\"meta-grid\">\n",
            "        <div><dt>Generated</dt><dd><code>{generated_at}</code></dd></div>\n",
            "        <div><dt>Saved reports</dt><dd><code>{report_count}</code></dd></div>\n",
            "      </dl>\n",
            "      <div class=\"link-row\">\n",
            "        <a class=\"open-link\" href=\"../../Conversations/index.html\">Open archive shell</a>\n",
            "        <a class=\"open-link\" href=\"../../Search/Reports/index.html\">Open retrieval reports</a>\n",
            "      </div>\n",
            "    </header>\n",
            "    <section class=\"route-grid\" aria-label=\"integration lane framing\">\n",
            "{lane_glance}\n",
            "    </section>\n",
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
        lane_glance = lane_glance,
        platform_facets = platform_facets,
        readiness_facets = readiness_facets,
        body = body,
        style = integration_report_style(),
        script = integration_report_script(),
    )
}

fn render_integration_lane_glance(reports: &[IntegrationReportEntry]) -> String {
    let latest = reports
        .iter()
        .max_by_key(|entry| entry.generated_at.as_deref().unwrap_or(""));
    let latest_platform = latest
        .and_then(|entry| entry.platform.as_deref())
        .unwrap_or("unknown");
    let latest_readiness = latest
        .and_then(|entry| entry.readiness.as_deref())
        .unwrap_or("unknown");
    let latest_target = latest
        .and_then(|entry| entry.target_root.as_deref())
        .unwrap_or("no saved target yet");
    let latest_generated = latest
        .and_then(|entry| entry.generated_at.as_deref())
        .unwrap_or("unknown");

    format!(
        concat!(
            "<article class=\"route-card\">",
            "<p class=\"eyebrow\">Use this page when</p>",
            "<h2>You need onboarding or doctor receipts, not transcript browsing.</h2>",
            "<p>这里是 integration evidence 的收据架。先看现在有哪些 saved reports、ready/partial 怎么分布，再决定要不要继续进入治理层。</p>",
            "</article>",
            "<article class=\"route-card\">",
            "<p class=\"eyebrow\">Do not use this page for</p>",
            "<h2>First contact with the product.</h2>",
            "<p>如果你还没决定从哪开始，先回 archive shell；如果你只是想回看 semantic/hybrid 查询，先去 search reports，而不是在这里绕进接线票据。</p>",
            "</article>",
            "<article class=\"route-card\">",
            "<p class=\"eyebrow\">Current integration picture</p>",
            "<h2>Latest saved report at a glance</h2>",
            "<div class=\"chip-row\">",
            "<span class=\"chip\">{latest_platform}</span>",
            "<span class=\"chip\">{latest_readiness}</span>",
            "</div>",
            "<p class=\"mono-inline\">target: <code>{latest_target}</code></p>",
            "<p class=\"mono-inline\">generated: <code>{latest_generated}</code></p>",
            "<p>先确认最近一次 receipt 是什么，再决定要不要下钻历史列表。</p>",
            "</article>"
        ),
        latest_platform = escape_html(latest_platform),
        latest_readiness = escape_html(latest_readiness),
        latest_target = escape_html(latest_target),
        latest_generated = escape_html(latest_generated),
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

fn check_record(check: &IntegrationDoctorCheck) -> IntegrationReportCheckRecord {
    IntegrationReportCheckRecord {
        label: check.label.to_string(),
        readiness: check.readiness.as_str().to_string(),
        detail: check.detail.clone(),
    }
}

fn pack_shape_checks(report: &IntegrationReportDocument) -> Vec<IntegrationReportCheckRecord> {
    report
        .checks
        .iter()
        .filter(|check| {
            matches!(
                check.label,
                "repo_templates"
                    | "target_files"
                    | "target_content_sync"
                    | "codex_config_shape"
                    | "claude_project_shape"
                    | "claude_pack_shape"
                    | "openclaw_bundle_shape"
            ) || check.label.contains("shape")
                || check.label.contains("sync")
        })
        .map(check_record)
        .collect()
}

fn launcher_status(report: &IntegrationReportDocument) -> String {
    report
        .checks
        .iter()
        .find(|check| check.label == "launcher_probe")
        .map(|check| check.readiness.as_str().to_string())
        .unwrap_or_else(|| report.readiness.clone())
}

fn bridge_status(report: &IntegrationReportDocument) -> String {
    let mut statuses = report
        .checks
        .iter()
        .filter(|check| matches!(check.label, "bridge_script" | "python3"))
        .map(|check| check.readiness.as_str());

    let Some(first) = statuses.next() else {
        return "unknown".to_string();
    };
    let mut aggregate = first;
    for status in statuses {
        aggregate = match (aggregate, status) {
            ("missing", _) | (_, "missing") => "missing",
            ("partial", _) | (_, "partial") => "partial",
            _ => "ready",
        };
    }
    aggregate.to_string()
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
            "<div class=\"entry-card-head\">",
            "<p class=\"eyebrow\">Saved integration report</p>",
            "<h2>{title}</h2>",
            "<div class=\"chip-row\">",
            "<span class=\"chip\">{kind}</span>",
            "<span class=\"chip\">{platform}</span>",
            "<span class=\"chip\">{readiness}</span>",
            "</div>",
            "</div>",
            "<div class=\"entry-card-body\">",
            "{generated_line}",
            "{target_line}",
            "<p><a class=\"open-link\" href=\"{href}\">Open report</a></p>",
            "</div>",
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
    background: var(--bg);
    color: var(--ink);
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
    max-width: 1180px;
    margin: 0 auto;
    padding: 28px 0 72px;
    position: relative;
    z-index: 1;
  }
  .hero-card, .section-card, .entry-card {
    position: relative;
    overflow: hidden;
    background: var(--surface);
    border: 1px solid var(--line);
    border-radius: 28px;
    box-shadow: var(--shadow-panel);
    backdrop-filter: blur(14px);
  }
  .hero-card::before, .section-card::before, .entry-card::before {
    content: "";
    position: absolute;
    inset: 0;
    pointer-events: none;
    background: linear-gradient(180deg, rgba(255, 255, 255, 0.65), transparent 28%);
  }
  .hero-card, .section-card {
    padding: 24px;
    margin-bottom: 24px;
  }
  .hero-card {
    padding: 32px;
    border-radius: 32px;
    box-shadow: var(--shadow-hero);
    background:
      radial-gradient(circle at top left, rgba(37, 99, 235, 0.14), transparent 34%),
      linear-gradient(135deg, rgba(255, 255, 255, 0.92), rgba(248, 250, 252, 0.84)),
      var(--surface);
  }
  .eyebrow {
    margin: 0 0 8px;
    font-size: 11px;
    letter-spacing: 0.14em;
    text-transform: uppercase;
    color: var(--accent);
    font-family: var(--mono);
  }
  h1, h2 {
    margin: 0 0 12px;
    line-height: 1.04;
    letter-spacing: -0.03em;
    font-family: var(--display);
    color: var(--ink);
  }
  h1 { font-size: clamp(32px, 4.2vw, 52px); }
  h2 { font-size: clamp(22px, 2.6vw, 30px); }
  .hero-copy, .summary-card, .check-detail, .search-status, .empty-inline, .empty-state p, .empty-result {
    color: var(--ink-soft);
    line-height: 1.72;
  }
  .hero-kicker {
    margin: 0 0 12px;
    color: var(--muted);
    font-family: var(--mono);
    font-size: 13px;
    letter-spacing: 0.04em;
  }
  .summary-card {
    padding: 16px 18px;
    background: var(--accent-soft);
    border: 1px solid var(--line-strong);
    border-radius: 18px;
    color: var(--ink);
    margin: 12px 0 16px;
  }
  .meta-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 12px 18px;
    margin: 0 0 16px;
  }
  .meta-grid div {
    padding: 14px 16px;
    background: rgba(255, 255, 255, 0.76);
    border-radius: 18px;
    box-shadow: var(--shadow-border);
  }
  .meta-grid dt {
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--muted);
    margin-bottom: 4px;
    font-family: var(--mono);
  }
  .meta-grid dd {
    margin: 0;
    font-size: 15px;
    color: var(--ink);
    font-weight: 600;
    line-height: 1.5;
  }
  .mono-inline, code {
    margin: 0;
    font-family: var(--mono);
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
    min-height: 42px;
    padding: 10px 14px;
    border-radius: 999px;
    text-decoration: none;
    border: 1px solid var(--line);
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
  .chip {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 6px 10px;
    border-radius: 999px;
    background: var(--accent-soft);
    color: var(--accent-strong);
    border: 1px solid var(--line-strong);
    font-family: var(--mono);
    font-size: 12px;
  }
  .check-grid, .card-grid, .two-col, .route-grid {
    display: grid;
    gap: 16px;
  }
  .check-grid {
    grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
  }
  .card-grid {
    grid-template-columns: 1fr;
  }
  .two-col, .route-grid {
    grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
  }
  .route-grid {
    margin-bottom: 18px;
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
  .check-card, .entry-card {
    padding: 18px;
    background: rgba(255, 255, 255, 0.84);
    border-radius: 22px;
  }
  .entry-card {
    display: grid;
    grid-template-columns: minmax(0, 1.2fr) minmax(280px, 0.8fr);
    gap: 18px;
    align-items: start;
  }
  .entry-card-head, .entry-card-body {
    display: grid;
    gap: 10px;
  }
  .mono-list, .step-list {
    margin: 0;
    padding-left: 20px;
  }
  .search-bar {
    padding: 20px 24px;
    margin-bottom: 24px;
    background: var(--surface-muted);
    border: 1px solid var(--line);
    border-radius: 24px;
    box-shadow: var(--shadow-panel);
  }
  .search-label, .facet-label {
    display: block;
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.12em;
    color: var(--accent);
    margin-bottom: 8px;
    font-family: var(--mono);
  }
  .search-input {
    width: 100%;
    padding: 13px 15px;
    border-radius: 16px;
    border: 1px solid var(--line);
    background: rgba(255, 255, 255, 0.96);
    margin-bottom: 16px;
    font-family: var(--mono);
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
    background: rgba(255, 255, 255, 0.86);
    border-radius: 999px;
    padding: 8px 12px;
    cursor: pointer;
    font-family: var(--mono);
    transition: transform 140ms ease, border-color 140ms ease, background 140ms ease;
  }
  .facet-button:hover {
    transform: translateY(-1px);
    border-color: var(--line-strong);
  }
  .facet-button.active {
    background: var(--accent-soft);
    border-color: var(--line-strong);
    color: var(--accent-strong);
  }
  .empty-state, .empty-result {
    padding: 24px;
    text-align: center;
  }
  a:focus-visible,
  button:focus-visible,
  input:focus-visible {
    outline: 3px solid rgba(37, 99, 235, 0.28);
    outline-offset: 4px;
  }
  [hidden] { display: none !important; }
  @media (max-width: 700px) {
    .page-shell { padding-inline: 14px; }
    .hero-card, .section-card, .search-bar, .entry-card, .route-card { border-radius: 20px; }
    .card-grid, .two-col, .route-grid { grid-template-columns: 1fr; }
    .entry-card { grid-template-columns: 1fr; }
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
    use crate::core::integration_report::{IntegrationReportEntry, IntegrationReportJsonDocument};
    use crate::integrations::{IntegrationDoctorCheck, IntegrationReadiness};

    use super::{
        IntegrationReportDocument, IntegrationReportKind, build_integration_report_json_document,
        build_integration_reports_index_json_document, render_integration_report_document,
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
        assert!(html.contains("Open archive shell"));
        assert!(html.contains("Open retrieval reports"));
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
        assert!(html.contains("Open archive shell"));
        assert!(html.contains("Open retrieval reports"));
    }

    #[test]
    fn build_integration_report_json_document_promotes_machine_readable_fields() {
        let report = IntegrationReportDocument {
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
            checks: vec![
                IntegrationDoctorCheck {
                    label: "bridge_script",
                    readiness: IntegrationReadiness::Ready,
                    detail: "ok".to_string(),
                },
                IntegrationDoctorCheck {
                    label: "python3",
                    readiness: IntegrationReadiness::Ready,
                    detail: "python3".to_string(),
                },
                IntegrationDoctorCheck {
                    label: "launcher_probe",
                    readiness: IntegrationReadiness::Ready,
                    detail: "repo-local binary available".to_string(),
                },
                IntegrationDoctorCheck {
                    label: "codex_config_shape",
                    readiness: IntegrationReadiness::Ready,
                    detail: "command + args present".to_string(),
                },
            ],
            next_steps: vec!["Review the generated pack".to_string()],
        };

        let json = build_integration_report_json_document(
            &report,
            "integration-report-doctor-codex.html",
            "integration-report-doctor-codex.json",
        );

        assert_eq!(json.bridge_status, "ready");
        assert_eq!(json.launcher_status, "ready");
        assert_eq!(json.pack_shape_checks.len(), 1);
        assert_eq!(json.artifact_links.index_json, "index.json");
    }

    #[test]
    fn build_integration_reports_index_json_document_exposes_timeline() {
        let report = IntegrationReportJsonDocument {
            schema_version: 1,
            title: "Codex doctor".to_string(),
            kind: "doctor".to_string(),
            platform: "codex".to_string(),
            target: "/tmp/codex-pack".to_string(),
            generated_at: "2026-04-06T12:00:00Z".to_string(),
            readiness: "ready".to_string(),
            summary: "ready".to_string(),
            launcher_status: "ready".to_string(),
            launcher_kind: "repo-local-debug".to_string(),
            launcher_command: "/tmp/agent-exporter".to_string(),
            bridge_status: "ready".to_string(),
            pack_shape_checks: Vec::new(),
            checks: Vec::new(),
            next_steps: Vec::new(),
            written_files: Vec::new(),
            unchanged_files: Vec::new(),
            artifact_links: IntegrationArtifactLinks {
                html_report: "integration-report-doctor-codex.html".to_string(),
                json_report: "integration-report-doctor-codex.json".to_string(),
                index_html: "index.html".to_string(),
                index_json: "index.json".to_string(),
            },
        };

        let index =
            build_integration_reports_index_json_document("integration reports", "now", &[report]);

        assert_eq!(index.report_count, 1);
        assert_eq!(index.timeline[0].json_href, "integration-report-doctor-codex.json");
    }
}
"#
}
