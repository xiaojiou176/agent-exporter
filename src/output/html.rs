use serde_json::Value;

use crate::core::archive::{
    ArchiveToolCall, ArchiveTranscript, ArchiveTurnItem, CommandExecutionRecord,
    DynamicToolCallRecord, FileChangeRecord, McpToolCallRecord,
};

#[derive(Debug, Clone, PartialEq, Eq)]
struct RenderedRound {
    round_number: usize,
    content: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceHtmlNavigation {
    pub archive_shell_href: String,
    pub reports_shell_href: String,
}

pub fn render_html_document(
    transcript: &ArchiveTranscript,
    archive_title: &str,
    exported_at: &str,
    workspace_navigation: Option<&WorkspaceHtmlNavigation>,
) -> String {
    let rounds = render_rounds(transcript);
    let title = format!("{archive_title} 对话归档");
    let workspace_meta = workspace_navigation
        .map(|navigation| {
            format!(
                concat!(
                    "  <meta name=\"agent-exporter:workspace-shell-href\" content=\"{shell}\">\n",
                    "  <meta name=\"agent-exporter:workspace-reports-shell-href\" content=\"{reports}\">\n"
                ),
                shell = escape_html(&navigation.archive_shell_href),
                reports = escape_html(&navigation.reports_shell_href),
            )
        })
        .unwrap_or_default();
    let workspace_nav = workspace_navigation
        .map(render_workspace_navigation)
        .unwrap_or_default();
    let body = if rounds.is_empty() {
        String::new()
    } else {
        rounds
            .iter()
            .map(|round| round.content.as_str())
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
            "  <meta name=\"agent-exporter:archive-title\" content=\"{archive_title_meta}\">\n",
            "  <meta name=\"agent-exporter:thread-display-name\" content=\"{thread_display_name_meta}\">\n",
            "  <meta name=\"agent-exporter:connector\" content=\"{connector_meta}\">\n",
            "  <meta name=\"agent-exporter:thread-id\" content=\"{thread_id_meta}\">\n",
            "  <meta name=\"agent-exporter:exported-at\" content=\"{exported_at_meta}\">\n",
            "  <meta name=\"agent-exporter:completeness\" content=\"{completeness_meta}\">\n",
            "  <meta name=\"agent-exporter:source-kind\" content=\"{source_kind_meta}\">\n",
            "{workspace_meta}",
            "  <style>\n{style}\n  </style>\n",
            "</head>\n",
            "<body>\n",
            "  <main class=\"page-shell\">\n",
            "    <header class=\"hero-card\">\n",
            "      <p class=\"eyebrow\">agent-exporter transcript export</p>\n",
            "      <h1>{title_html}</h1>\n",
            "      <p class=\"hero-copy\">单文件 HTML transcript export。它像一份可直接打开阅读的网页打印稿，继续复用同一份 typed archive core，而不是另造一套浏览平台。</p>\n",
            "      <dl class=\"meta-grid\">\n",
            "        <div><dt>连接器</dt><dd><code>{connector}</code></dd></div>\n",
            "        <div><dt>线程 ID</dt><dd><code>{thread_id}</code></dd></div>\n",
            "        <div><dt>导出时间</dt><dd><code>{exported_at_html}</code></dd></div>\n",
            "        <div><dt>完整性</dt><dd><code>{completeness}</code></dd></div>\n",
            "        <div><dt>来源</dt><dd><code>{source_kind}</code></dd></div>\n",
            "        <div><dt>线程状态</dt><dd><code>{thread_status}</code></dd></div>\n",
            "        <div><dt>轮次</dt><dd><code>{round_count}</code></dd></div>\n",
            "        <div><dt>条目数</dt><dd><code>{item_count}</code></dd></div>\n",
            "      </dl>\n",
            "{workspace_nav}",
            "    </header>\n",
            "    <section class=\"transcript-stack\">\n",
            "{body}\n",
            "    </section>\n",
            "  </main>\n",
            "</body>\n",
            "</html>\n"
        ),
        title = escape_html(&title),
        title_html = escape_html(&title),
        archive_title_meta = escape_html(archive_title),
        thread_display_name_meta = escape_html(
            transcript
                .thread_display_name()
                .unwrap_or(&transcript.thread_id)
        ),
        style = html_style(),
        connector_meta = escape_html(transcript.connector.as_str()),
        connector = escape_html(transcript.connector.as_str()),
        thread_id_meta = escape_html(&transcript.thread_id),
        thread_id = escape_html(&transcript.thread_id),
        exported_at_meta = escape_html(exported_at),
        exported_at_html = escape_html(exported_at),
        completeness_meta = escape_html(transcript.completeness.as_str()),
        completeness = escape_html(transcript.completeness.as_str()),
        source_kind_meta = escape_html(transcript.source_kind.as_str()),
        source_kind = escape_html(transcript.source_kind.as_str()),
        thread_status = escape_html(transcript.thread_status.as_str()),
        round_count = transcript.round_count(),
        item_count = transcript.item_count(),
        workspace_meta = workspace_meta,
        workspace_nav = workspace_nav,
        body = body,
    )
}

fn render_workspace_navigation(navigation: &WorkspaceHtmlNavigation) -> String {
    format!(
        concat!(
            "<section class=\"workspace-nav\">",
            "<p class=\"eyebrow\">Workspace navigation</p>",
            "<p class=\"hero-copy\">这份 transcript 当前位于 workspace-local archive 里。你可以把它理解成“从单张打印稿回到前厅”的返回线：阅读完之后，直接回 archive shell；saved retrieval reports 也仍然留在本地 report 目录里。</p>",
            "<div class=\"link-row\">",
            "<a class=\"open-link\" href=\"{shell_href}\">Open archive shell</a>",
            "<a class=\"open-link\" href=\"{reports_href}\">Open retrieval reports</a>",
            "</div>",
            "</section>"
        ),
        shell_href = escape_html(&navigation.archive_shell_href),
        reports_href = escape_html(&navigation.reports_shell_href),
    )
}

fn render_rounds(transcript: &ArchiveTranscript) -> Vec<RenderedRound> {
    let synthetic_opening_user_message = if transcript.has_user_messages() {
        None
    } else {
        transcript
            .preview
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
    };

    let mut rendered = Vec::new();
    let mut emitted_synthetic_opening = false;

    for (index, round) in transcript.rounds.iter().enumerate() {
        let mut user_sections = Vec::new();
        let mut assistant_sections = Vec::new();
        let mut tool_sections = Vec::new();

        for item in &round.items {
            match item {
                ArchiveTurnItem::UserMessage { text, images, .. } => {
                    let rendered = render_user_message(text, images);
                    if !rendered.trim().is_empty() {
                        user_sections.push(rendered);
                    }
                }
                ArchiveTurnItem::HookPrompt { fragments, .. } => {
                    if !fragments.is_empty() {
                        assistant_sections
                            .push(render_block_section("Hook Prompt", &fragments.join("\n")));
                    }
                }
                ArchiveTurnItem::AssistantMessage { text, phase, .. } => {
                    if !text.trim().is_empty() {
                        let mut body = String::new();
                        if let Some(phase) =
                            phase.as_deref().filter(|value| !value.trim().is_empty())
                        {
                            body.push_str(&format!(
                                "<p class=\"phase-badge\">阶段: <code>{}</code></p>",
                                escape_html(phase)
                            ));
                        }
                        body.push_str(&render_text_content(text));
                        assistant_sections.push(body);
                    }
                }
                ArchiveTurnItem::Plan { text, .. } => {
                    if !text.trim().is_empty() {
                        assistant_sections.push(render_block_section("Plan", text));
                    }
                }
                ArchiveTurnItem::Reasoning {
                    summary, content, ..
                } => {
                    let rendered_reasoning = render_reasoning(summary, content);
                    if !rendered_reasoning.trim().is_empty() {
                        assistant_sections.push(rendered_reasoning);
                    }
                }
                ArchiveTurnItem::ToolCall(tool_call) => {
                    let rendered_tool = render_tool_call(tool_call);
                    if !rendered_tool.trim().is_empty() {
                        tool_sections.push(rendered_tool);
                    }
                }
            }
        }

        if user_sections.is_empty() && assistant_sections.is_empty() && tool_sections.is_empty() {
            continue;
        }

        if !emitted_synthetic_opening {
            if let Some(opening) = synthetic_opening_user_message.as_ref() {
                user_sections.insert(0, render_text_content(opening));
                emitted_synthetic_opening = true;
            }
        }

        let mut sections = vec![format!(
            concat!(
                "<article class=\"round-card\">",
                "<header class=\"round-header\">",
                "<div>",
                "<p class=\"round-kicker\">Round</p>",
                "<h2>第{round_number}轮</h2>",
                "</div>",
                "<dl class=\"round-meta\">",
                "<div><dt>Turn ID</dt><dd><code>{turn_id}</code></dd></div>",
                "<div><dt>状态</dt><dd><code>{status}</code></dd></div>",
                "</dl>",
                "</header>"
            ),
            round_number = index + 1,
            turn_id = escape_html(&round.turn_id),
            status = escape_html(round.status.as_str()),
        )];

        if let Some(error) = round.error.as_ref() {
            sections.push(format!(
                "<p class=\"error-note\">错误: {}</p>",
                escape_html(&error.message)
            ));
        }
        if !user_sections.is_empty() {
            sections.push(render_role_section("用户", "user", &user_sections));
        }
        if !assistant_sections.is_empty() {
            sections.push(render_role_section(
                "助手",
                "assistant",
                &assistant_sections,
            ));
        }
        if !tool_sections.is_empty() {
            sections.push(render_role_section("工具", "tools", &tool_sections));
        }
        sections.push("</article>".to_string());

        rendered.push(RenderedRound {
            round_number: index + 1,
            content: sections.join("\n"),
        });
    }

    if rendered.is_empty() {
        if let Some(opening) = synthetic_opening_user_message {
            rendered.push(RenderedRound {
                round_number: 1,
                content: [
                    "<article class=\"round-card\">".to_string(),
                    "<header class=\"round-header\"><div><p class=\"round-kicker\">Round</p><h2>第1轮</h2></div></header>".to_string(),
                    render_role_section("用户", "user", &[render_text_content(&opening)]),
                    "</article>".to_string(),
                ]
                .join("\n"),
            });
        }
    }

    rendered
}

fn render_role_section(title: &str, class_name: &str, sections: &[String]) -> String {
    format!(
        concat!(
            "<section class=\"role-card role-{class_name}\">",
            "<h3>{title}</h3>",
            "<div class=\"role-body\">{body}</div>",
            "</section>"
        ),
        class_name = class_name,
        title = escape_html(title),
        body = sections.join("\n"),
    )
}

fn render_user_message(text: &str, images: &[String]) -> String {
    let mut parts = Vec::new();
    if !text.trim().is_empty() {
        parts.push(render_text_content(text));
    }
    if !images.is_empty() {
        let items = images
            .iter()
            .map(|image| format!("<li><code>{}</code></li>", escape_html(image)))
            .collect::<Vec<_>>()
            .join("");
        parts.push(format!(
            "<div class=\"inline-group\"><p class=\"section-label\">图片</p><ul class=\"bullet-list\">{items}</ul></div>"
        ));
    }
    parts.join("\n")
}

fn render_reasoning(summary: &[String], content: &[String]) -> String {
    let mut blocks = Vec::new();
    if !summary.is_empty() {
        blocks.push(render_text_content(&summary.join("\n")));
    }
    if !content.is_empty() {
        blocks.push(render_text_content(&content.join("\n")));
    }
    if blocks.is_empty() {
        String::new()
    } else {
        format!(
            "<details class=\"reasoning-card\"><summary>💭 推理过程</summary>{}</details>",
            blocks.join("\n")
        )
    }
}

fn render_tool_call(tool_call: &ArchiveToolCall) -> String {
    match tool_call {
        ArchiveToolCall::CommandExecution(record) => render_command_execution(record),
        ArchiveToolCall::FileChange {
            changes, status, ..
        } => render_file_change(changes, status),
        ArchiveToolCall::McpToolCall(record) => render_mcp_tool_call(record),
        ArchiveToolCall::DynamicToolCall(record) => render_dynamic_tool_call(record),
        ArchiveToolCall::CollabAgentToolCall {
            tool,
            status,
            prompt,
            receiver_thread_ids,
            ..
        } => {
            let mut parts = vec![render_tool_shell("Collab", tool, status.as_deref(), "")];
            if let Some(prompt) = prompt.as_deref().filter(|value| !value.trim().is_empty()) {
                parts.push(render_text_content(prompt));
            }
            if !receiver_thread_ids.is_empty() {
                let targets = receiver_thread_ids
                    .iter()
                    .map(|id| format!("<li><code>{}</code></li>", escape_html(id)))
                    .collect::<Vec<_>>()
                    .join("");
                parts.push(format!(
                    "<div class=\"inline-group\"><p class=\"section-label\">Targets</p><ul class=\"bullet-list\">{targets}</ul></div>"
                ));
            }
            wrap_tool_card(&parts.join("\n"))
        }
        ArchiveToolCall::WebSearch { query, action, .. } => {
            let mut parts = vec![render_tool_shell("Web search", "", None, "")];
            if !query.trim().is_empty() {
                parts.push(format!(
                    "<p class=\"mono-inline\">Query: <code>{}</code></p>",
                    escape_html(query)
                ));
            }
            if let Some(action) = action.as_deref().filter(|value| !value.trim().is_empty()) {
                parts.push(render_code_block("json", action));
            }
            wrap_tool_card(&parts.join("\n"))
        }
        ArchiveToolCall::ImageView { path, .. } => wrap_tool_card(
            &[
                render_tool_shell("Image view", "", None, ""),
                format!(
                    "<p class=\"mono-inline\"><code>{}</code></p>",
                    escape_html(path)
                ),
            ]
            .join("\n"),
        ),
        ArchiveToolCall::LifecycleNote { label, .. } => {
            wrap_tool_card(&render_tool_shell(label, "", None, ""))
        }
        ArchiveToolCall::Unsupported { kind, payload, .. } => wrap_tool_card(
            &[
                render_tool_shell("Unsupported item", kind, None, ""),
                render_code_block("json", &pretty_json(payload)),
            ]
            .join("\n"),
        ),
    }
}

fn render_command_execution(record: &CommandExecutionRecord) -> String {
    let title = if record.command.trim().is_empty() {
        "Command".to_string()
    } else {
        format!("Command: {}", record.command)
    };
    let mut parts = vec![render_tool_shell(&title, "", record.status.as_deref(), "")];
    if let Some(cwd) = record.cwd.as_ref() {
        parts.push(format!(
            "<p class=\"mono-inline\">cwd: <code>{}</code></p>",
            escape_html(&cwd.display().to_string())
        ));
    }
    if let Some(exit_code) = record.exit_code {
        parts.push(format!(
            "<p class=\"mono-inline\">exit code: <code>{exit_code}</code></p>"
        ));
    }
    if let Some(output) = record
        .aggregated_output
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        parts.push(render_code_block("text", output));
    }
    wrap_tool_card(&parts.join("\n"))
}

fn render_file_change(changes: &[FileChangeRecord], status: &Option<String>) -> String {
    let mut parts = vec![render_tool_shell("File changes", "", status.as_deref(), "")];
    if !changes.is_empty() {
        let items = changes
            .iter()
            .map(|change| {
                let mut row = format!(
                    "<li><p class=\"mono-inline\"><code>{}</code>{}</p>",
                    escape_html(&change.path),
                    change
                        .kind
                        .as_deref()
                        .filter(|value| !value.trim().is_empty())
                        .map(|value| format!(" <span class=\"chip\">{}</span>", escape_html(value)))
                        .unwrap_or_default()
                );
                if let Some(diff) = change
                    .diff
                    .as_deref()
                    .filter(|value| !value.trim().is_empty())
                {
                    row.push_str(&render_code_block("diff", diff));
                }
                row.push_str("</li>");
                row
            })
            .collect::<Vec<_>>()
            .join("");
        parts.push(format!("<ul class=\"tool-list\">{items}</ul>"));
    }
    wrap_tool_card(&parts.join("\n"))
}

fn render_mcp_tool_call(record: &McpToolCallRecord) -> String {
    let mut parts = vec![render_tool_shell(
        &format!("MCP: {} / {}", record.server, record.tool),
        "",
        record.status.as_deref(),
        "",
    )];
    if let Some(result) = record
        .result
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        parts.push(render_code_block("json", result));
    }
    if let Some(error) = record
        .error
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        parts.push(render_code_block("json", error));
    }
    wrap_tool_card(&parts.join("\n"))
}

fn render_dynamic_tool_call(record: &DynamicToolCallRecord) -> String {
    let mut parts = vec![render_tool_shell(
        "Dynamic tool",
        &record.tool,
        record.status.as_deref(),
        "",
    )];
    if let Some(success) = record.success {
        parts.push(format!(
            "<p class=\"mono-inline\">success: <code>{success}</code></p>"
        ));
    }
    if !record.content_items.is_empty() {
        parts.push(render_text_content(&record.content_items.join("\n")));
    }
    wrap_tool_card(&parts.join("\n"))
}

fn render_tool_shell(label: &str, detail: &str, status: Option<&str>, extra_class: &str) -> String {
    let detail_html = if detail.trim().is_empty() {
        String::new()
    } else {
        format!(
            " <span class=\"tool-detail\">{}</span>",
            escape_html(detail)
        )
    };
    let status_html = status
        .filter(|value| !value.trim().is_empty())
        .map(|value| format!(" <span class=\"chip\">{}</span>", escape_html(value)))
        .unwrap_or_default();
    format!(
        "<h4 class=\"tool-title {extra_class}\">{label}{detail_html}{status_html}</h4>",
        label = escape_html(label),
        extra_class = extra_class
    )
}

fn wrap_tool_card(body: &str) -> String {
    format!("<article class=\"tool-card\">{body}</article>")
}

fn render_block_section(title: &str, text: &str) -> String {
    format!(
        "<div class=\"inline-group\"><p class=\"section-label\">{}</p>{}</div>",
        escape_html(title),
        render_text_content(text)
    )
}

fn render_text_content(text: &str) -> String {
    let blocks = text
        .trim()
        .split("\n\n")
        .filter(|block| !block.trim().is_empty())
        .map(|block| {
            format!(
                "<p>{}</p>",
                block
                    .lines()
                    .map(escape_html)
                    .collect::<Vec<_>>()
                    .join("<br>")
            )
        })
        .collect::<Vec<_>>();
    if blocks.is_empty() {
        String::new()
    } else {
        format!("<div class=\"text-block\">{}</div>", blocks.join("\n"))
    }
}

fn render_code_block(language: &str, body: &str) -> String {
    format!(
        "<pre class=\"code-block\"><code class=\"language-{}\">{}</code></pre>",
        escape_html(language),
        escape_html(body.trim_end())
    )
}

fn pretty_json(value: &Value) -> String {
    serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string())
}

pub(crate) fn escape_html(value: &str) -> String {
    value
        .chars()
        .map(|ch| match ch {
            '&' => "&amp;".to_string(),
            '<' => "&lt;".to_string(),
            '>' => "&gt;".to_string(),
            '"' => "&quot;".to_string(),
            '\'' => "&#39;".to_string(),
            _ => ch.to_string(),
        })
        .collect()
}

fn html_style() -> &'static str {
    r#"    :root {
      --page-bg: linear-gradient(180deg, #f3ede2 0%, #ece4d6 100%);
      --panel: rgba(255, 250, 242, 0.92);
      --panel-strong: #fffdf8;
      --ink: #1f2933;
      --muted: #5f6b76;
      --border: #d8cbb8;
      --accent: #a86423;
      --user: #fff3dd;
      --assistant: #edf5ff;
      --tools: #eff3ea;
      --danger: #8e2432;
      --shadow: 0 18px 42px rgba(53, 41, 28, 0.12);
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
    .round-card {
      background: var(--panel);
      border: 1px solid var(--border);
      border-radius: 24px;
      box-shadow: var(--shadow);
      backdrop-filter: blur(14px);
    }

    .hero-card {
      padding: 28px;
      margin-bottom: 24px;
    }

    .eyebrow,
    .round-kicker,
    .section-label {
      margin: 0 0 10px;
      text-transform: uppercase;
      letter-spacing: 0.12em;
      font-family: var(--mono);
      font-size: 12px;
      color: var(--accent);
    }

    h1, h2, h3, h4 {
      margin: 0;
      line-height: 1.2;
      font-weight: 700;
    }

    h1 { font-size: clamp(32px, 4vw, 48px); margin-bottom: 12px; }
    h2 { font-size: clamp(24px, 3vw, 32px); }
    h3 { font-size: 20px; margin-bottom: 14px; }
    h4 { font-size: 17px; margin-bottom: 10px; }

    .hero-copy {
      max-width: 72ch;
      margin: 0 0 18px;
      color: var(--muted);
      line-height: 1.7;
    }

    .meta-grid,
    .round-meta {
      display: grid;
      gap: 12px;
      grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
      margin: 0;
    }

    .meta-grid div,
    .round-meta div {
      padding: 12px 14px;
      background: var(--panel-strong);
      border: 1px solid rgba(216, 203, 184, 0.85);
      border-radius: 16px;
    }

    .workspace-nav {
      margin-top: 18px;
      padding: 16px 18px;
      border-radius: 18px;
      border: 1px solid rgba(168, 100, 35, 0.18);
      background: rgba(255, 255, 255, 0.7);
    }

    dt {
      margin-bottom: 6px;
      font-size: 12px;
      letter-spacing: 0.08em;
      text-transform: uppercase;
      color: var(--muted);
      font-family: var(--mono);
    }

    dd {
      margin: 0;
      word-break: break-word;
    }

    code,
    .mono-inline {
      font-family: var(--mono);
      font-size: 0.95em;
    }

    .mono-note {
      margin-top: 12px;
      color: var(--muted);
      font-family: var(--mono);
      font-size: 13px;
    }

    .transcript-stack {
      display: grid;
      gap: 18px;
    }

    .round-card {
      padding: 22px;
    }

    .round-header {
      display: flex;
      justify-content: space-between;
      align-items: flex-start;
      gap: 16px;
      margin-bottom: 18px;
    }

    .error-note {
      margin: 0 0 18px;
      padding: 12px 14px;
      border-radius: 14px;
      background: rgba(142, 36, 50, 0.08);
      color: var(--danger);
      border: 1px solid rgba(142, 36, 50, 0.18);
    }

    .role-card {
      border: 1px solid rgba(216, 203, 184, 0.9);
      border-radius: 18px;
      padding: 18px;
      margin-top: 16px;
    }

    .role-user { background: var(--user); }
    .role-assistant { background: var(--assistant); }
    .role-tools { background: var(--tools); }

    .role-body,
    .text-block,
    .inline-group {
      display: grid;
      gap: 12px;
    }

    p {
      margin: 0;
      line-height: 1.75;
      word-break: break-word;
    }

    .phase-badge {
      color: var(--muted);
      font-family: var(--mono);
      font-size: 13px;
    }

    .tool-card {
      padding: 14px;
      border-radius: 16px;
      background: rgba(255, 255, 255, 0.68);
      border: 1px solid rgba(216, 203, 184, 0.95);
    }

    .tool-title {
      display: flex;
      flex-wrap: wrap;
      align-items: center;
      gap: 8px;
    }

    .tool-detail {
      font-weight: 500;
      color: var(--muted);
    }

    .chip {
      display: inline-flex;
      align-items: center;
      border-radius: 999px;
      padding: 4px 10px;
      background: rgba(168, 100, 35, 0.12);
      color: var(--accent);
      font-family: var(--mono);
      font-size: 12px;
    }

    .link-row {
      margin-top: 14px;
    }

    .bullet-list,
    .tool-list {
      margin: 0;
      padding-left: 22px;
      display: grid;
      gap: 10px;
    }

    .code-block {
      margin: 0;
      overflow-x: auto;
      padding: 14px 16px;
      border-radius: 16px;
      background: #18202a;
      color: #f6f7f9;
      border: 1px solid rgba(24, 32, 42, 0.95);
      font-family: var(--mono);
      font-size: 13px;
      line-height: 1.65;
    }

    .reasoning-card {
      border-radius: 16px;
      border: 1px dashed rgba(168, 100, 35, 0.35);
      padding: 14px 16px;
      background: rgba(255, 253, 248, 0.8);
    }

    .reasoning-card summary {
      cursor: pointer;
      font-weight: 700;
      margin-bottom: 12px;
    }

    @media (max-width: 720px) {
      .page-shell {
        width: min(100vw - 20px, 1080px);
        padding: 16px 0 28px;
      }

      .hero-card,
      .round-card {
        border-radius: 20px;
      }

      .hero-card,
      .round-card,
      .role-card {
        padding: 18px;
      }

      .round-header {
        flex-direction: column;
      }
    }"#
}

#[cfg(test)]
mod tests {
    use super::{WorkspaceHtmlNavigation, render_html_document};
    use crate::core::archive::{
        ArchiveCompleteness, ArchiveRound, ArchiveThreadStatus, ArchiveToolCall, ArchiveTranscript,
        ArchiveTurnItem, ArchiveTurnStatus, CommandExecutionRecord, ConnectorSourceKind,
    };
    use crate::model::ConnectorKind;

    fn sample_transcript() -> ArchiveTranscript {
        ArchiveTranscript {
            connector: ConnectorKind::ClaudeCode,
            thread_id: "session-1".to_string(),
            thread_name: Some("demo".to_string()),
            preview: Some("hello".to_string()),
            completeness: ArchiveCompleteness::Degraded,
            source_kind: ConnectorSourceKind::ClaudeSessionPath,
            thread_status: ArchiveThreadStatus::Unknown("archival-only".to_string()),
            cwd: None,
            path: None,
            model_provider: None,
            created_at: Some(1_744_000_000),
            updated_at: Some(1_744_000_010),
            rounds: vec![ArchiveRound {
                turn_id: "turn-1".to_string(),
                status: ArchiveTurnStatus::Completed,
                error: None,
                items: vec![
                    ArchiveTurnItem::UserMessage {
                        id: "user-1".to_string(),
                        text: "hello <world>".to_string(),
                        images: Vec::new(),
                    },
                    ArchiveTurnItem::ToolCall(ArchiveToolCall::CommandExecution(
                        CommandExecutionRecord {
                            id: "cmd-1".to_string(),
                            command: "pwd".to_string(),
                            cwd: None,
                            status: Some("completed".to_string()),
                            aggregated_output: Some("/tmp/demo\n".to_string()),
                            exit_code: Some(0),
                        },
                    )),
                ],
            }],
        }
    }

    #[test]
    fn render_html_document_wraps_transcript_in_single_page() {
        let document = render_html_document(
            &sample_transcript(),
            "agent-exporter",
            "2026-04-05T00:00:00Z",
            None,
        );

        assert!(document.contains("<!DOCTYPE html>"));
        assert!(document.contains("agent-exporter 对话归档"));
        assert!(document.contains("agent-exporter:archive-title"));
        assert!(document.contains("第1轮"));
        assert!(document.contains("用户"));
        assert!(document.contains("助手") || document.contains("工具"));
        assert!(document.contains("claude-session-path"));
        assert!(!document.contains("Open archive shell"));
    }

    #[test]
    fn render_html_document_escapes_transcript_text() {
        let document = render_html_document(
            &sample_transcript(),
            "agent-exporter",
            "2026-04-05T00:00:00Z",
            None,
        );

        assert!(document.contains("hello &lt;world&gt;"));
        assert!(!document.contains("hello <world>"));
    }

    #[test]
    fn render_html_document_inserts_preview_when_no_user_message_exists() {
        let mut transcript = sample_transcript();
        transcript.rounds[0].items = vec![ArchiveTurnItem::AssistantMessage {
            id: "assistant-1".to_string(),
            text: "continuing".to_string(),
            phase: None,
        }];
        transcript.preview = Some("preview fallback".to_string());

        let document =
            render_html_document(&transcript, "agent-exporter", "2026-04-05T00:00:00Z", None);

        assert!(document.contains("preview fallback"));
        assert!(document.contains("第1轮"));
    }

    #[test]
    fn render_html_document_can_embed_workspace_navigation() {
        let navigation = WorkspaceHtmlNavigation {
            archive_shell_href: "index.html".to_string(),
            reports_shell_href: "../Search/Reports/index.html".to_string(),
        };
        let document = render_html_document(
            &sample_transcript(),
            "agent-exporter",
            "2026-04-05T00:00:00Z",
            Some(&navigation),
        );

        assert!(document.contains("Open archive shell"));
        assert!(document.contains("Open retrieval reports"));
        assert!(document.contains("agent-exporter:workspace-shell-href"));
        assert!(document.contains("agent-exporter:workspace-reports-shell-href"));
        assert!(document.contains("../Search/Reports/index.html"));
    }
}
