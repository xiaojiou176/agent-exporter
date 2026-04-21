use crate::core::workbench::{ActionPackIndexDocument, FleetViewEntry, TeamMemoryLaneDocument};
use crate::output::html::escape_html;

pub fn render_fleet_view_document(
    archive_title: &str,
    generated_at: &str,
    entries: &[FleetViewEntry],
) -> String {
    let body = if entries.is_empty() {
        "<p class=\"empty-inline\">No fleet relations are visible yet.</p>".to_string()
    } else {
        entries
            .iter()
            .map(|entry| {
                let history = if entry.history.is_empty() {
                    String::new()
                } else {
                    format!(
                        "<ol class=\"step-list\">{}</ol>",
                        entry.history
                            .iter()
                            .take(6)
                            .map(|item| {
                                format!(
                                    "<li><strong>{}</strong><p class=\"mono-inline\"><code>{}</code> · {}</p><p>{}</p><p><a class=\"open-link\" href=\"{}\">Open report</a></p></li>",
                                    escape_html(&item.kind),
                                    escape_html(&item.generated_at),
                                    escape_html(&item.readiness),
                                    escape_html(&item.summary),
                                    escape_html(&item.html_href),
                                )
                            })
                            .collect::<Vec<_>>()
                            .join("")
                    )
                };
                let next_steps = if entry.next_steps.is_empty() {
                    String::new()
                } else {
                    format!(
                        "<p>next steps: {}</p>",
                        escape_html(&entry.next_steps.join(", "))
                    )
                };
                format!(
                    concat!(
                        "<article class=\"summary-card\">",
                        "<p class=\"eyebrow\">Fleet relation</p>",
                        "<h2>{platform}</h2>",
                        "<div class=\"chip-row\">",
                        "<span class=\"chip\">{readiness}</span>",
                        "<span class=\"chip\">reports <span>{report_count}</span></span>",
                        "</div>",
                        "<p class=\"mono-inline\">target: <code>{target}</code></p>",
                        "<p>{summary}</p>",
                        "{next_steps}",
                        "<p class=\"eyebrow\">Readiness drift timeline</p>",
                        "{history}",
                        "</article>"
                    ),
                    platform = escape_html(&entry.platform),
                    readiness = escape_html(&entry.latest_readiness),
                    report_count = entry.report_count,
                    target = escape_html(&entry.target),
                    summary = escape_html(&entry.latest_summary),
                    next_steps = next_steps,
                    history = history,
                )
            })
            .collect::<Vec<_>>()
            .join("")
    };

    render_page(
        "Fleet readiness board",
        archive_title,
        generated_at,
        "Track readiness across integration targets without leaving the local workbench.",
        &body,
    )
}

pub fn render_action_packs_index_document(document: &ActionPackIndexDocument) -> String {
    let body = if document.packs.is_empty() {
        "<p class=\"empty-inline\">No action packs were generated from the current workbench snapshot.</p>".to_string()
    } else {
        document
            .packs
            .iter()
            .map(|pack| {
                let steps = if pack.steps.is_empty() {
                    String::new()
                } else {
                    format!(
                        "<ol class=\"step-list\">{}</ol>",
                        pack.steps
                            .iter()
                            .map(|step| format!("<li>{}</li>", escape_html(step)))
                            .collect::<Vec<_>>()
                            .join("")
                    )
                };
                format!(
                    concat!(
                        "<article class=\"summary-card\">",
                        "<p class=\"eyebrow\">Action pack</p>",
                        "<h2>{title}</h2>",
                        "<div class=\"chip-row\"><span class=\"chip\">{kind}</span></div>",
                        "<p>{summary}</p>",
                        "<p class=\"mono-inline\">source: <code>{source}</code></p>",
                        "{steps}",
                        "<p><a class=\"open-link\" href=\"{markdown_href}\">Open markdown</a></p>",
                        "</article>"
                    ),
                    title = escape_html(&pack.title),
                    kind = escape_html(&pack.kind),
                    summary = escape_html(&pack.summary),
                    source = escape_html(&pack.source_kind),
                    steps = steps,
                    markdown_href = escape_html(&pack.markdown_href),
                )
            })
            .collect::<Vec<_>>()
            .join("")
    };

    render_page(
        "Report-to-action bridge",
        &document.title,
        &document.generated_at,
        "Turn stale pins, family diffs, and fleet drift into concrete next-step packets.",
        &body,
    )
}

pub fn render_memory_lane_document(document: &TeamMemoryLaneDocument) -> String {
    let workspaces = if document.workspaces.is_empty() {
        "<p class=\"empty-inline\">No sibling workspace workbenches were discovered yet.</p>"
            .to_string()
    } else {
        document
            .workspaces
            .iter()
            .map(|workspace| {
                format!(
                    concat!(
                        "<article class=\"summary-card\">",
                        "<p class=\"eyebrow\">Workspace</p>",
                        "<h2>{name}</h2>",
                        "<p class=\"mono-inline\">root: <code>{root}</code></p>",
                        "<p class=\"mono-inline\">generated: <code>{generated_at}</code></p>",
                        "<p>families: {family_count} · official answers: {official_count} · fleet: {fleet_count}</p>",
                        "</article>"
                    ),
                    name = escape_html(&workspace.workspace_name),
                    root = escape_html(&workspace.workspace_root),
                    generated_at = escape_html(&workspace.generated_at),
                    family_count = workspace.family_count,
                    official_count = workspace.official_answer_count,
                    fleet_count = workspace.fleet_count,
                )
            })
            .collect::<Vec<_>>()
            .join("")
    };

    let roles = document
        .role_views
        .iter()
        .map(|role| {
            format!(
                concat!(
                    "<article class=\"summary-card\">",
                    "<p class=\"eyebrow\">Role slice</p>",
                    "<h2>{role}</h2>",
                    "<p>{summary}</p>",
                    "<ol class=\"step-list\">{items}</ol>",
                    "</article>"
                ),
                role = escape_html(&role.role),
                summary = escape_html(&role.summary),
                items = role
                    .items
                    .iter()
                    .map(|item| format!("<li>{}</li>", escape_html(item)))
                    .collect::<Vec<_>>()
                    .join(""),
            )
        })
        .collect::<Vec<_>>()
        .join("");

    render_page(
        "Team and org memory lane",
        &document.title,
        &document.generated_at,
        "Project workspace snapshots and role-based recall slices in one local lane.",
        &format!("{workspaces}{roles}"),
    )
}

fn render_page(
    heading: &str,
    title: &str,
    generated_at: &str,
    summary: &str,
    body: &str,
) -> String {
    format!(
        concat!(
            "<!DOCTYPE html><html lang=\"zh-CN\"><head><meta charset=\"utf-8\">",
            "<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">",
            "<title>{heading}</title><style>{style}</style></head><body><main class=\"page-shell\">",
            "<header class=\"hero-card\"><p class=\"eyebrow\">workbench lane</p><h1>{heading}</h1>",
            "<p>{summary}</p><p class=\"mono-inline\">title: <code>{title}</code> · generated: <code>{generated_at}</code></p>",
            "<div class=\"link-row\"><a class=\"open-link\" href=\"index.html\">Open archive shell</a><a class=\"open-link\" href=\"../Integration/Reports/index.html\">Open integration reports</a></div>",
            "</header><section class=\"card-grid\">{body}</section></main></body></html>"
        ),
        heading = escape_html(heading),
        summary = escape_html(summary),
        title = escape_html(title),
        generated_at = escape_html(generated_at),
        body = body,
        style = shared_style(),
    )
}

fn shared_style() -> &'static str {
    "
    body { margin: 0; font-family: \"IBM Plex Sans\", -apple-system, BlinkMacSystemFont, \"Segoe UI\", sans-serif; background: #f5f7fb; color: #0f172a; }
    .page-shell { width: min(1180px, calc(100vw - 32px)); margin: 32px auto 80px; display: grid; gap: 24px; }
    .hero-card, .summary-card { background: white; border: 1px solid rgba(15,23,42,0.08); border-radius: 20px; box-shadow: 0 16px 40px rgba(15,23,42,0.08); padding: 24px; }
    .card-grid { display: grid; gap: 18px; grid-template-columns: repeat(auto-fit, minmax(280px, 1fr)); }
    .chip-row, .link-row { display: flex; gap: 10px; flex-wrap: wrap; }
    .chip { display: inline-flex; gap: 6px; border-radius: 999px; padding: 6px 12px; background: #e2e8f0; font-size: 13px; }
    .eyebrow { margin: 0 0 10px; color: #2563eb; font-family: \"JetBrains Mono\", monospace; font-size: 12px; letter-spacing: 0.12em; text-transform: uppercase; }
    .mono-inline { font-family: \"JetBrains Mono\", monospace; font-size: 13px; color: #334155; }
    .open-link { color: #0f172a; font-weight: 600; }
    .step-list { margin: 0; padding-left: 20px; display: grid; gap: 10px; }
    .empty-inline { color: #64748b; }
    "
}
