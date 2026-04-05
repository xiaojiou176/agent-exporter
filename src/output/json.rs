use serde_json::{Value, json};

use crate::core::archive::{
    ArchiveRound, ArchiveToolCall, ArchiveTranscript, ArchiveTurnItem, CommandExecutionRecord,
    DynamicToolCallRecord, FileChangeRecord, McpToolCallRecord,
};

pub fn render_json_document(
    transcript: &ArchiveTranscript,
    archive_title: &str,
    exported_at: &str,
) -> Value {
    json!({
        "schema_version": 1,
        "format": "json",
        "archive_title": archive_title,
        "exported_at": exported_at,
        "transcript": render_transcript(transcript),
    })
}

fn render_transcript(transcript: &ArchiveTranscript) -> Value {
    json!({
        "connector": transcript.connector.as_str(),
        "thread_id": transcript.thread_id,
        "thread_name": transcript.thread_name,
        "preview": transcript.preview,
        "completeness": transcript.completeness.as_str(),
        "source_kind": transcript.source_kind.as_str(),
        "thread_status": transcript.thread_status.as_str(),
        "cwd": transcript.cwd.as_ref().map(|path| path.display().to_string()),
        "path": transcript.path.as_ref().map(|path| path.display().to_string()),
        "model_provider": transcript.model_provider,
        "created_at": transcript.created_at,
        "updated_at": transcript.updated_at,
        "round_count": transcript.round_count(),
        "item_count": transcript.item_count(),
        "rounds": transcript
            .rounds
            .iter()
            .enumerate()
            .map(|(index, round)| render_round(index + 1, round))
            .collect::<Vec<_>>(),
    })
}

fn render_round(round_number: usize, round: &ArchiveRound) -> Value {
    json!({
        "round_number": round_number,
        "turn_id": round.turn_id,
        "status": round.status.as_str(),
        "error": round.error.as_ref().map(|error| {
            json!({
                "message": error.message,
            })
        }),
        "items": round.items.iter().map(render_item).collect::<Vec<_>>(),
    })
}

fn render_item(item: &ArchiveTurnItem) -> Value {
    match item {
        ArchiveTurnItem::UserMessage { id, text, images } => json!({
            "kind": "user_message",
            "id": id,
            "text": text,
            "images": images,
        }),
        ArchiveTurnItem::HookPrompt { id, fragments } => json!({
            "kind": "hook_prompt",
            "id": id,
            "fragments": fragments,
        }),
        ArchiveTurnItem::AssistantMessage { id, text, phase } => json!({
            "kind": "assistant_message",
            "id": id,
            "text": text,
            "phase": phase,
        }),
        ArchiveTurnItem::Plan { id, text } => json!({
            "kind": "plan",
            "id": id,
            "text": text,
        }),
        ArchiveTurnItem::Reasoning {
            id,
            summary,
            content,
        } => json!({
            "kind": "reasoning",
            "id": id,
            "summary": summary,
            "content": content,
        }),
        ArchiveTurnItem::ToolCall(tool_call) => json!({
            "kind": "tool_call",
            "tool_call": render_tool_call(tool_call),
        }),
    }
}

fn render_tool_call(tool_call: &ArchiveToolCall) -> Value {
    match tool_call {
        ArchiveToolCall::CommandExecution(record) => render_command_execution(record),
        ArchiveToolCall::FileChange {
            id,
            status,
            changes,
        } => json!({
            "kind": "file_change",
            "id": id,
            "status": status,
            "changes": changes.iter().map(render_file_change).collect::<Vec<_>>(),
        }),
        ArchiveToolCall::McpToolCall(record) => render_mcp_tool_call(record),
        ArchiveToolCall::DynamicToolCall(record) => render_dynamic_tool_call(record),
        ArchiveToolCall::CollabAgentToolCall {
            id,
            tool,
            status,
            prompt,
            receiver_thread_ids,
        } => json!({
            "kind": "collab_agent_tool_call",
            "id": id,
            "tool": tool,
            "status": status,
            "prompt": prompt,
            "receiver_thread_ids": receiver_thread_ids,
        }),
        ArchiveToolCall::WebSearch { id, query, action } => json!({
            "kind": "web_search",
            "id": id,
            "query": query,
            "action": action,
        }),
        ArchiveToolCall::ImageView { id, path } => json!({
            "kind": "image_view",
            "id": id,
            "path": path,
        }),
        ArchiveToolCall::LifecycleNote { id, label } => json!({
            "kind": "lifecycle_note",
            "id": id,
            "label": label,
        }),
        ArchiveToolCall::Unsupported { id, kind, payload } => json!({
            "kind": "unsupported",
            "id": id,
            "unsupported_kind": kind,
            "payload": payload,
        }),
    }
}

fn render_command_execution(record: &CommandExecutionRecord) -> Value {
    json!({
        "kind": "command_execution",
        "id": record.id,
        "command": record.command,
        "cwd": record.cwd.as_ref().map(|path| path.display().to_string()),
        "status": record.status,
        "aggregated_output": record.aggregated_output,
        "exit_code": record.exit_code,
    })
}

fn render_file_change(change: &FileChangeRecord) -> Value {
    json!({
        "path": change.path,
        "kind": change.kind,
        "diff": change.diff,
    })
}

fn render_mcp_tool_call(record: &McpToolCallRecord) -> Value {
    json!({
        "kind": "mcp_tool_call",
        "id": record.id,
        "server": record.server,
        "tool": record.tool,
        "status": record.status,
        "result": record.result,
        "error": record.error,
    })
}

fn render_dynamic_tool_call(record: &DynamicToolCallRecord) -> Value {
    json!({
        "kind": "dynamic_tool_call",
        "id": record.id,
        "tool": record.tool,
        "status": record.status,
        "content_items": record.content_items,
        "success": record.success,
    })
}

#[cfg(test)]
mod tests {
    use super::render_json_document;
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
                        text: "hello".to_string(),
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
    fn render_json_document_wraps_transcript_with_export_metadata() {
        let document = render_json_document(
            &sample_transcript(),
            "agent-exporter",
            "2026-04-05T00:00:00Z",
        );

        assert_eq!(document["schema_version"], 1);
        assert_eq!(document["format"], "json");
        assert_eq!(document["archive_title"], "agent-exporter");
        assert_eq!(document["transcript"]["connector"], "claude-code");
        assert_eq!(document["transcript"]["completeness"], "degraded");
        assert_eq!(document["transcript"]["round_count"], 1);
        assert_eq!(document["transcript"]["item_count"], 2);
    }

    #[test]
    fn render_json_document_keeps_tool_call_kind_tags() {
        let document = render_json_document(
            &sample_transcript(),
            "agent-exporter",
            "2026-04-05T00:00:00Z",
        );

        assert_eq!(
            document["transcript"]["rounds"][0]["items"][1]["kind"],
            "tool_call"
        );
        assert_eq!(
            document["transcript"]["rounds"][0]["items"][1]["tool_call"]["kind"],
            "command_execution"
        );
        assert_eq!(
            document["transcript"]["rounds"][0]["items"][1]["tool_call"]["command"],
            "pwd"
        );
    }
}
