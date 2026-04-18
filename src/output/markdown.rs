use serde_json::Value;

use crate::core::archive::{
    ArchiveToolCall, ArchiveTranscript, ArchiveTurnItem, CommandExecutionRecord,
    DynamicToolCallRecord, FileChangeRecord, McpToolCallRecord,
};

pub const DEFAULT_MAX_LINES_PER_PART: usize = 4000;

#[derive(Debug, Clone, PartialEq, Eq)]
struct RenderedRound {
    round_number: usize,
    content: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderedArchivePart {
    pub part_index: usize,
    pub start_round: usize,
    pub end_round: usize,
    pub content: String,
    pub line_count: usize,
}

pub fn render_markdown_parts(
    transcript: &ArchiveTranscript,
    archive_title: &str,
    exported_at: &str,
    max_lines_per_part: usize,
) -> Vec<RenderedArchivePart> {
    let rounds = render_rounds(transcript);
    if rounds.is_empty() {
        return Vec::new();
    }

    let max_lines = max_lines_per_part.max(1);
    let mut parts = Vec::new();
    let mut start_index = 0usize;

    while start_index < rounds.len() {
        let part_index = parts.len() + 1;
        let mut end_index = start_index;
        let mut chosen_end_index = start_index;

        loop {
            let candidate = build_markdown_document(
                transcript,
                archive_title,
                exported_at,
                part_index,
                &rounds[start_index..=end_index],
            );
            let candidate_line_count = count_lines(&candidate);
            if candidate_line_count > max_lines && end_index > start_index {
                break;
            }
            chosen_end_index = end_index;
            if candidate_line_count > max_lines || end_index + 1 >= rounds.len() {
                break;
            }
            end_index += 1;
        }

        let selected = build_markdown_document(
            transcript,
            archive_title,
            exported_at,
            part_index,
            &rounds[start_index..=chosen_end_index],
        );
        parts.push(RenderedArchivePart {
            part_index,
            start_round: rounds[start_index].round_number,
            end_round: rounds[chosen_end_index].round_number,
            line_count: count_lines(&selected),
            content: selected,
        });
        start_index = chosen_end_index + 1;
    }

    parts
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
                        assistant_sections.push(
                            ["#### Hook Prompt".to_string(), fragments.join("\n")].join("\n\n"),
                        );
                    }
                }
                ArchiveTurnItem::AssistantMessage { text, phase, .. } => {
                    if !text.trim().is_empty() {
                        let mut body = Vec::new();
                        if let Some(phase) =
                            phase.as_deref().filter(|value| !value.trim().is_empty())
                        {
                            body.push(format!("> 阶段: `{phase}`"));
                        }
                        body.push(text.clone());
                        assistant_sections.push(body.join("\n\n"));
                    }
                }
                ArchiveTurnItem::Plan { text, .. } => {
                    if !text.trim().is_empty() {
                        assistant_sections
                            .push(["#### Plan".to_string(), text.clone()].join("\n\n"));
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

        if !emitted_synthetic_opening && let Some(opening) = synthetic_opening_user_message.as_ref()
        {
            user_sections.insert(0, opening.clone());
            emitted_synthetic_opening = true;
        }

        let mut parts = vec![
            format!("# 第{}轮", index + 1),
            format!("- Turn ID: `{}`", round.turn_id),
            format!("- 状态: `{}`", round.status.as_str()),
        ];
        if let Some(error) = round.error.as_ref() {
            parts.push(format!("> 错误: {}", error.message));
        }
        if !user_sections.is_empty() {
            parts.push("## 用户".to_string());
            parts.push(user_sections.join("\n\n"));
        }
        if !assistant_sections.is_empty() {
            parts.push("## 助手".to_string());
            parts.push(assistant_sections.join("\n\n"));
        }
        if !tool_sections.is_empty() {
            parts.push("### 工具".to_string());
            parts.push(tool_sections.join("\n\n"));
        }

        rendered.push(RenderedRound {
            round_number: index + 1,
            content: parts.join("\n\n"),
        });
    }

    if rendered.is_empty()
        && let Some(opening) = synthetic_opening_user_message
    {
        rendered.push(RenderedRound {
            round_number: 1,
            content: ["# 第1轮".to_string(), "## 用户".to_string(), opening].join("\n\n"),
        });
    }

    rendered
}

fn render_user_message(text: &str, images: &[String]) -> String {
    let mut parts = Vec::new();
    if !text.trim().is_empty() {
        parts.push(text.trim().to_string());
    }
    if !images.is_empty() {
        let image_lines = images
            .iter()
            .map(|image| format!("- 图片: `{image}`"))
            .collect::<Vec<_>>()
            .join("\n");
        parts.push(image_lines);
    }
    parts.join("\n\n")
}

fn render_reasoning(summary: &[String], content: &[String]) -> String {
    let mut blocks = Vec::new();
    if !summary.is_empty() {
        blocks.push(summary.join("\n"));
    }
    if !content.is_empty() {
        blocks.push(content.join("\n"));
    }
    if blocks.is_empty() {
        String::new()
    } else {
        [
            "<details>".to_string(),
            "<summary>💭 推理过程</summary>".to_string(),
            String::new(),
            blocks.join("\n\n"),
            String::new(),
            "</details>".to_string(),
        ]
        .join("\n")
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
            let mut parts = vec![format!(
                "#### Collab: {}{}",
                if tool.trim().is_empty() { "tool" } else { tool },
                status
                    .as_deref()
                    .filter(|value| !value.trim().is_empty())
                    .map(|value| format!(" ({value})"))
                    .unwrap_or_default()
            )];
            if let Some(prompt) = prompt.as_deref().filter(|value| !value.trim().is_empty()) {
                parts.push(prompt.to_string());
            }
            if !receiver_thread_ids.is_empty() {
                parts.push(format!(
                    "Targets: {}",
                    receiver_thread_ids
                        .iter()
                        .map(|id| format!("`{id}`"))
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
            parts.join("\n\n")
        }
        ArchiveToolCall::WebSearch { query, action, .. } => {
            let mut parts = vec!["#### Web search".to_string()];
            if !query.trim().is_empty() {
                parts.push(format!("Query: `{query}`"));
            }
            if let Some(action) = action.as_deref().filter(|value| !value.trim().is_empty()) {
                parts.push(code_fence("json", action));
            }
            parts.join("\n\n")
        }
        ArchiveToolCall::ImageView { path, .. } => {
            format!("#### Image view\n\n`{path}`")
        }
        ArchiveToolCall::LifecycleNote { label, .. } => format!("#### {label}"),
        ArchiveToolCall::Unsupported { kind, payload, .. } => [
            format!("#### Unsupported item: {kind}"),
            "```json".to_string(),
            pretty_json(payload),
            "```".to_string(),
        ]
        .join("\n\n"),
    }
}

fn render_command_execution(record: &CommandExecutionRecord) -> String {
    let title = if record.command.trim().is_empty() {
        "Command".to_string()
    } else {
        format!("Command: {}", record.command)
    };
    let mut parts = vec![format!(
        "#### {}{}",
        title,
        record
            .status
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .map(|value| format!(" ({value})"))
            .unwrap_or_default()
    )];
    if let Some(cwd) = record.cwd.as_ref() {
        parts.push(format!("cwd: `{}`", cwd.display()));
    }
    if let Some(exit_code) = record.exit_code {
        parts.push(format!("exit code: `{exit_code}`"));
    }
    if let Some(output) = record
        .aggregated_output
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        parts.push(code_fence("text", output));
    }
    parts.join("\n\n")
}

fn render_file_change(changes: &[FileChangeRecord], status: &Option<String>) -> String {
    let mut parts = vec![format!(
        "#### File changes{}",
        status
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .map(|value| format!(" ({value})"))
            .unwrap_or_default()
    )];

    if !changes.is_empty() {
        parts.push(
            changes
                .iter()
                .map(|change| {
                    let mut line = format!(
                        "- `{}`{}",
                        change.path,
                        change
                            .kind
                            .as_deref()
                            .filter(|value| !value.trim().is_empty())
                            .map(|value| format!(" [{value}]"))
                            .unwrap_or_default()
                    );
                    if let Some(diff) = change
                        .diff
                        .as_deref()
                        .filter(|value| !value.trim().is_empty())
                    {
                        line.push('\n');
                        line.push_str(&code_fence("diff", diff));
                    }
                    line
                })
                .collect::<Vec<_>>()
                .join("\n"),
        );
    }

    parts.join("\n\n")
}

fn render_mcp_tool_call(record: &McpToolCallRecord) -> String {
    let mut parts = vec![format!(
        "#### MCP: {} / {}{}",
        record.server,
        record.tool,
        record
            .status
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .map(|value| format!(" ({value})"))
            .unwrap_or_default()
    )];
    if let Some(result) = record
        .result
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        parts.push(code_fence("json", result));
    }
    if let Some(error) = record
        .error
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        parts.push(code_fence("json", error));
    }
    parts.join("\n\n")
}

fn render_dynamic_tool_call(record: &DynamicToolCallRecord) -> String {
    let mut parts = vec![format!(
        "#### Dynamic tool: {}{}",
        record.tool,
        record
            .status
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .map(|value| format!(" ({value})"))
            .unwrap_or_default()
    )];
    if let Some(success) = record.success {
        parts.push(format!("success: `{success}`"));
    }
    if !record.content_items.is_empty() {
        parts.push(record.content_items.join("\n"));
    }
    parts.join("\n\n")
}

fn build_markdown_document(
    transcript: &ArchiveTranscript,
    archive_title: &str,
    exported_at: &str,
    part_index: usize,
    rounds: &[RenderedRound],
) -> String {
    let start_round = rounds.first().map(|round| round.round_number).unwrap_or(0);
    let end_round = rounds.last().map(|round| round.round_number).unwrap_or(0);
    let header = build_part_header(
        transcript,
        archive_title,
        exported_at,
        part_index,
        start_round,
        end_round,
    );
    if rounds.is_empty() {
        header
    } else {
        let body = rounds
            .iter()
            .map(|round| round.content.as_str())
            .collect::<Vec<_>>()
            .join("\n\n");
        format!("{header}\n\n{body}")
    }
}

fn build_part_header(
    transcript: &ArchiveTranscript,
    archive_title: &str,
    exported_at: &str,
    part_index: usize,
    start_round: usize,
    end_round: usize,
) -> String {
    [
        format!("# {archive_title} 对话归档"),
        String::new(),
        format!("- 连接器: `{}`", transcript.connector.as_str()),
        format!("- 线程 ID: `{}`", transcript.thread_id),
        format!("- 导出时间: `{exported_at}`"),
        format!("- 完整性: `{}`", transcript.completeness.as_str()),
        format!("- 来源: `{}`", transcript.source_kind.as_str()),
        format!("- 线程状态: `{}`", transcript.thread_status.as_str()),
        format!("- 条目数: `{}`", transcript.item_count()),
        format!("- 当前分片: `第 {part_index} 部分`"),
        format!("- 包含轮次: `第 {start_round} 轮 - 第 {end_round} 轮`"),
    ]
    .join("\n")
}

fn count_lines(value: &str) -> usize {
    if value.is_empty() {
        0
    } else {
        value.lines().count()
    }
}

fn code_fence(info: &str, body: &str) -> String {
    format!("```{info}\n{}\n```", body.trim_end())
}

fn pretty_json(value: &Value) -> String {
    serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string())
}

#[cfg(test)]
mod tests {
    use super::render_markdown_parts;
    use crate::core::archive::{
        ArchiveCompleteness, ArchiveRound, ArchiveThreadStatus, ArchiveTranscript, ArchiveTurnItem,
        ArchiveTurnStatus, ConnectorSourceKind,
    };
    use crate::model::ConnectorKind;

    fn sample_transcript_with_assistant_line_counts(line_counts: &[usize]) -> ArchiveTranscript {
        let rounds = line_counts
            .iter()
            .enumerate()
            .map(|(index, line_count)| ArchiveRound {
                turn_id: format!("turn-{}", index + 1),
                status: ArchiveTurnStatus::Completed,
                error: None,
                items: vec![
                    ArchiveTurnItem::UserMessage {
                        id: format!("user-{}", index + 1),
                        text: format!("user message {}", index + 1),
                        images: Vec::new(),
                    },
                    ArchiveTurnItem::AssistantMessage {
                        id: format!("assistant-{}", index + 1),
                        text: std::iter::repeat_n("assistant line", *line_count)
                            .collect::<Vec<_>>()
                            .join("\n"),
                        phase: None,
                    },
                ],
            })
            .collect::<Vec<_>>();

        ArchiveTranscript {
            connector: ConnectorKind::Codex,
            thread_id: "thread-1".to_string(),
            thread_name: Some("Demo".to_string()),
            preview: Some("hello".to_string()),
            completeness: ArchiveCompleteness::Complete,
            source_kind: ConnectorSourceKind::AppServerThreadRead,
            thread_status: ArchiveThreadStatus::NotLoaded,
            cwd: None,
            path: None,
            model_provider: None,
            created_at: None,
            updated_at: None,
            rounds,
        }
    }

    #[test]
    fn render_markdown_parts_splits_by_round_without_breaking_rounds() {
        let transcript = sample_transcript_with_assistant_line_counts(&[
            1000, 1000, 1000, 1280, 1280, 1296, 600, 4200, 200, 200,
        ]);
        let parts =
            render_markdown_parts(&transcript, "agent-exporter", "2026-04-04T00:00:00Z", 4000);

        assert_eq!(
            parts
                .iter()
                .map(|part| (part.start_round, part.end_round))
                .collect::<Vec<_>>(),
            vec![(1, 3), (4, 6), (7, 7), (8, 8), (9, 10)]
        );
        assert!(parts[3].line_count > 4000);
        assert!(parts[0].content.contains("第 1 部分"));
        assert!(parts[1].content.contains("第 4 轮 - 第 6 轮"));
    }

    #[test]
    fn render_markdown_keeps_round_sections() {
        let transcript = sample_transcript_with_assistant_line_counts(&[2]);
        let parts =
            render_markdown_parts(&transcript, "agent-exporter", "2026-04-04T00:00:00Z", 4000);

        assert_eq!(parts.len(), 1);
        assert!(parts[0].content.contains("# 第1轮"));
        assert!(parts[0].content.contains("## 用户"));
        assert!(parts[0].content.contains("## 助手"));
    }

    #[test]
    fn render_markdown_inserts_preview_when_no_user_message_exists() {
        let transcript = ArchiveTranscript {
            connector: ConnectorKind::Codex,
            thread_id: "thread-preview".to_string(),
            thread_name: Some("Preview".to_string()),
            preview: Some("synthetic opening".to_string()),
            completeness: ArchiveCompleteness::Incomplete,
            source_kind: ConnectorSourceKind::AppServerResumeFallback,
            thread_status: ArchiveThreadStatus::NotLoaded,
            cwd: None,
            path: None,
            model_provider: None,
            created_at: None,
            updated_at: None,
            rounds: vec![ArchiveRound {
                turn_id: "turn-1".to_string(),
                status: ArchiveTurnStatus::Completed,
                error: None,
                items: vec![ArchiveTurnItem::AssistantMessage {
                    id: "assistant-1".to_string(),
                    text: "fallback assistant".to_string(),
                    phase: None,
                }],
            }],
        };

        let parts =
            render_markdown_parts(&transcript, "agent-exporter", "2026-04-04T00:00:00Z", 4000);

        assert!(parts[0].content.contains("synthetic opening"));
    }
}
