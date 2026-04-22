use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread::sleep;
use std::time::{Duration, Instant};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail};
use serde_json::{Map, Value};

use crate::core::archive::{
    AiSummaryOptions, ArchiveTranscript, ExportSource, OutputTarget,
    allocate_ai_summary_document_path,
};
use crate::core::workbench::{
    StructuredSummaryDocument, StructuredSummaryOutputFiles, summary_family_key,
    write_structured_summary_document,
};
use crate::model::OutputFormat;

const DEFAULT_AI_SUMMARY_COMMAND: &str = "codex";
const DEFAULT_AI_SUMMARY_TIMEOUT: Duration = Duration::from_secs(300);

pub struct AiSummaryRequest<'a> {
    pub transcript: &'a ArchiveTranscript,
    pub output_target: &'a OutputTarget,
    pub export_source: ExportSource,
    pub export_format: OutputFormat,
    pub exported_at: &'a str,
    pub exported_paths: &'a [PathBuf],
    pub extra_instructions: Option<&'a str>,
    pub timeout_seconds: Option<u64>,
}

pub struct AiSummaryOutcome {
    pub markdown_output_path: PathBuf,
    pub html_output_path: PathBuf,
    pub json_output_path: PathBuf,
}

pub fn generate_ai_summary(request: &AiSummaryRequest<'_>) -> Result<AiSummaryOutcome> {
    generate_ai_summary_with_options(request, &AiSummaryOptions::default())
}

pub fn generate_ai_summary_with_options(
    request: &AiSummaryRequest<'_>,
    options: &AiSummaryOptions,
) -> Result<AiSummaryOutcome> {
    if request.exported_paths.is_empty() {
        bail!("AI summary requires at least one exported transcript file");
    }

    let output_path = allocate_ai_summary_document_path(request.transcript, request.output_target)?;
    let working_root = summary_working_root(request.output_target)?;
    let prompt = build_default_summary_prompt(request, options);
    let timeout = effective_ai_summary_timeout(request.timeout_seconds, options.timeout_seconds);
    let (stdout_capture_path, stderr_capture_path) =
        ai_summary_capture_paths(&request.transcript.thread_id);
    let stdout_capture = create_ai_summary_capture_file(&stdout_capture_path, "stdout")
        .with_context(|| {
            format!(
                "failed to prepare AI summary stdout capture for transcript `{}`",
                request.transcript.thread_id
            )
        })?;
    let stderr_capture = create_ai_summary_capture_file(&stderr_capture_path, "stderr")
        .with_context(|| {
            format!(
                "failed to prepare AI summary stderr capture for transcript `{}`",
                request.transcript.thread_id
            )
        })?;
    let mut command = Command::new(DEFAULT_AI_SUMMARY_COMMAND);
    command.args(build_ai_summary_exec_args(
        &working_root,
        &output_path,
        options,
    ));
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::from(stdout_capture))
        .stderr(Stdio::from(stderr_capture))
        .spawn()
        .with_context(|| {
            format!(
                "failed to launch AI summary agent `{DEFAULT_AI_SUMMARY_COMMAND}` for transcript `{}`",
                request.transcript.thread_id
            )
        })?;

    {
        let mut stdin = child
            .stdin
            .take()
            .context("AI summary agent stdin unavailable")?;
        stdin
            .write_all(prompt.as_bytes())
            .context("failed to send AI summary prompt")?;
    }

    let result = (|| {
        let start = Instant::now();
        let status = loop {
            if let Some(status) = child
                .try_wait()
                .context("failed while polling AI summary agent")?
            {
                break status;
            }

            if start.elapsed() >= timeout {
                child.kill().ok();
                child.wait().ok();
                if output_path.exists() {
                    return finalize_ai_summary_outputs(&output_path, request, options);
                }
                let stderr = summarize_ai_summary_capture(&stderr_capture_path);
                bail!(
                    "AI summary generation timed out after {} seconds; raw transcript export remains on disk, but the AI summary sidecar was not completed{}.{}",
                    timeout.as_secs(),
                    if stderr.is_empty() {
                        String::new()
                    } else {
                        " Child stderr captured before timeout:".to_string()
                    },
                    stderr
                );
            }

            sleep(Duration::from_millis(200));
        };

        if !status.success() {
            let stdout = read_ai_summary_capture(&stdout_capture_path);
            let stderr = read_ai_summary_capture(&stderr_capture_path);
            bail!(
                "AI summary generation failed with status {}.\nstdout:\n{}\nstderr:\n{}",
                status,
                stdout,
                stderr
            );
        }

        if !output_path.exists() {
            bail!(
                "AI summary agent reported success, but output file was not written: {}",
                output_path.display()
            );
        }

        finalize_ai_summary_outputs(&output_path, request, options)
    })();

    let _ = std::fs::remove_file(&stdout_capture_path);
    let _ = std::fs::remove_file(&stderr_capture_path);

    result
}

fn finalize_ai_summary_outputs(
    output_path: &Path,
    request: &AiSummaryRequest<'_>,
    options: &AiSummaryOptions,
) -> Result<AiSummaryOutcome> {
    let markdown = std::fs::read_to_string(output_path).with_context(|| {
        format!(
            "failed to read AI summary markdown `{}` before generating companions",
            output_path.display()
        )
    })?;
    if markdown.trim().is_empty() {
        bail!(
            "AI summary markdown was written but empty: {}",
            output_path.display()
        );
    }

    let html_output_path = write_html_companion(
        output_path,
        request.transcript,
        request.exported_at,
        &markdown,
    )?;
    let json_output_path = output_path.with_extension("json");
    let structured_summary = build_structured_summary_document(
        request,
        options,
        output_path,
        &html_output_path,
        &json_output_path,
        &markdown,
    );
    write_structured_summary_document(&json_output_path, &structured_summary)?;

    Ok(AiSummaryOutcome {
        markdown_output_path: output_path.to_path_buf(),
        html_output_path,
        json_output_path,
    })
}

fn effective_ai_summary_timeout(
    request_timeout_seconds: Option<u64>,
    options_timeout_seconds: Option<u64>,
) -> Duration {
    Duration::from_secs(
        request_timeout_seconds
            .or(options_timeout_seconds)
            .unwrap_or(DEFAULT_AI_SUMMARY_TIMEOUT.as_secs()),
    )
}

fn build_ai_summary_exec_args(
    working_root: &std::path::Path,
    output_path: &std::path::Path,
    options: &AiSummaryOptions,
) -> Vec<String> {
    let mut args = Vec::new();
    args.push("exec".to_string());

    if let Some(profile) = normalize_option(options.profile.as_deref()) {
        args.push("--profile".to_string());
        args.push(profile.to_string());
    }

    if let Some(model) = normalize_option(options.model.as_deref()) {
        args.push("--model".to_string());
        args.push(model.to_string());
    }

    if let Some(provider) = normalize_option(options.provider.as_deref()) {
        args.push("-c".to_string());
        args.push(format!("model_provider={}", quote_toml_string(provider)));
    }

    if options.profile.is_none() {
        args.push("-c".to_string());
        args.push("model_reasoning_effort=\"low\"".to_string());
        args.push("-c".to_string());
        args.push("model_verbosity=\"low\"".to_string());
    }

    args.extend([
        "--skip-git-repo-check".to_string(),
        "--sandbox".to_string(),
        "read-only".to_string(),
        "-C".to_string(),
        working_root.display().to_string(),
        "-o".to_string(),
        output_path.display().to_string(),
    ]);
    args
}

fn normalize_option(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|value| !value.is_empty())
}

fn ai_summary_capture_paths(thread_id: &str) -> (PathBuf, PathBuf) {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let slug = thread_id
        .chars()
        .map(|ch| match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' => ch,
            _ => '-',
        })
        .collect::<String>();
    let root = std::env::temp_dir();
    (
        root.join(format!(
            "agent-exporter-ai-summary-{slug}-{stamp}.stdout.log"
        )),
        root.join(format!(
            "agent-exporter-ai-summary-{slug}-{stamp}.stderr.log"
        )),
    )
}

fn create_ai_summary_capture_file(path: &Path, label: &str) -> Result<std::fs::File> {
    std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path)
        .with_context(|| {
            format!(
                "failed to open AI summary {label} capture `{}`",
                path.display()
            )
        })
}

fn read_ai_summary_capture(path: &Path) -> String {
    std::fs::read_to_string(path)
        .unwrap_or_else(|_| format!("(capture unavailable: {})", path.display()))
}

fn summarize_ai_summary_capture(path: &Path) -> String {
    let capture = read_ai_summary_capture(path);
    let trimmed = capture.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    let tail = trimmed
        .lines()
        .rev()
        .take(12)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>()
        .join("\n");
    format!("\n{tail}")
}

fn quote_toml_string(value: &str) -> String {
    format!("{value:?}")
}

fn summary_working_root(output_target: &OutputTarget) -> Result<PathBuf> {
    match output_target {
        OutputTarget::WorkspaceConversations { workspace_root } => Ok(workspace_root.clone()),
        OutputTarget::Downloads => {
            let downloads_dir = output_target.resolve_output_dir()?;
            Ok(downloads_dir)
        }
    }
}

fn write_html_companion(
    markdown_output_path: &Path,
    transcript: &ArchiveTranscript,
    exported_at: &str,
    markdown: &str,
) -> Result<PathBuf> {
    let html_output_path = markdown_output_path.with_extension("html");
    let title = transcript
        .thread_display_name()
        .unwrap_or(transcript.thread_id.as_str());
    let document =
        render_summary_html_document(title, &transcript.thread_id, exported_at, markdown);

    std::fs::write(&html_output_path, document).with_context(|| {
        format!(
            "failed to write AI summary HTML companion `{}`",
            html_output_path.display()
        )
    })?;

    Ok(html_output_path)
}

fn render_summary_html_document(
    thread_title: &str,
    thread_id: &str,
    exported_at: &str,
    markdown: &str,
) -> String {
    format!(
        concat!(
            "<!DOCTYPE html>\n",
            "<html lang=\"zh-CN\">\n",
            "<head>\n",
            "  <meta charset=\"utf-8\">\n",
            "  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n",
            "  <title>AI summary - {title}</title>\n",
            "  <meta name=\"agent-exporter:summary-kind\" content=\"ai-summary\">\n",
            "  <meta name=\"agent-exporter:thread-id\" content=\"{thread_id}\">\n",
            "  <meta name=\"agent-exporter:thread-display-name\" content=\"{title}\">\n",
            "  <meta name=\"agent-exporter:exported-at\" content=\"{exported_at}\">\n",
            "  <style>\n",
            "    body {{ margin: 0; font-family: \"IBM Plex Sans\", -apple-system, BlinkMacSystemFont, \"Segoe UI\", sans-serif; background: #f7fafc; color: #0f172a; }}\n",
            "    main {{ width: min(960px, calc(100vw - 32px)); margin: 28px auto 64px; display: grid; gap: 20px; }}\n",
            "    .card {{ background: white; border: 1px solid rgba(15,23,42,0.08); border-radius: 20px; box-shadow: 0 18px 40px rgba(15,23,42,0.08); padding: 24px; }}\n",
            "    .eyebrow {{ margin: 0 0 10px; color: #2563eb; font-family: \"JetBrains Mono\", monospace; font-size: 12px; letter-spacing: 0.12em; text-transform: uppercase; }}\n",
            "    h1 {{ margin: 0 0 12px; font-size: clamp(28px, 4vw, 44px); line-height: 1.05; }}\n",
            "    .meta {{ color: #475569; line-height: 1.7; margin: 0; }}\n",
            "    pre {{ margin: 0; white-space: pre-wrap; word-break: break-word; background: #f8fafc; border: 1px solid rgba(15,23,42,0.08); border-radius: 16px; padding: 18px; font-family: \"JetBrains Mono\", monospace; line-height: 1.65; }}\n",
            "  </style>\n",
            "</head>\n",
            "<body>\n",
            "  <main>\n",
            "    <section class=\"card\">\n",
            "      <p class=\"eyebrow\">AI Summary</p>\n",
            "      <h1>{title}</h1>\n",
            "      <p class=\"meta\">thread: <code>{thread_id}</code><br>generated: <code>{exported_at}</code></p>\n",
            "    </section>\n",
            "    <section class=\"card\">\n",
            "      <pre>{markdown}</pre>\n",
            "    </section>\n",
            "  </main>\n",
            "</body>\n",
            "</html>\n"
        ),
        title = escape_html(thread_title),
        thread_id = escape_html(thread_id),
        exported_at = escape_html(exported_at),
        markdown = escape_html(markdown.trim_end()),
    )
}

fn escape_html(value: &str) -> String {
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

fn build_default_summary_prompt(
    request: &AiSummaryRequest<'_>,
    options: &AiSummaryOptions,
) -> String {
    let display_name = request
        .transcript
        .thread_display_name()
        .unwrap_or(request.transcript.thread_id.as_str());
    let file_list = request
        .exported_paths
        .iter()
        .map(|path| format!("- {}", path.display()))
        .collect::<Vec<_>>()
        .join("\n");

    let preset = resolve_summary_preset(options.preset.as_deref());
    let preset_prompt_label = summary_preset_prompt_label(options.preset.as_deref(), &preset);

    let mut sections = vec![
        "你现在是一个只读 AI 梳理 Agent。".to_string(),
        "请阅读下面这些刚导出的对话归档文件，并输出一份最终 Markdown 梳理文档。".to_string(),
        "只输出最终 Markdown，不要解释过程，不要输出额外前言，不要修改任何文件。".to_string(),
        String::new(),
        "文档必须使用中文为主，并严格区分“事实 / 推断 / 未确认”。".to_string(),
        "如果证据不足，请明确写“未确认”，不要脑补。".to_string(),
        "如果对话里已经省略了过长工具结果，请基于现有导出内容继续梳理，不要猜被省略的内容。"
            .to_string(),
        format!("本次 summary preset：{}。", preset_prompt_label),
        format!("请特别强调：{}。", preset.focus),
        "最终 Markdown 的最前面必须先给出一个 ```json fenced block。".to_string(),
        "这个 JSON block 必须至少包含这些字段：summary_title, overview, share_safe_summary, goals, files_touched, tests_run, risks, blockers, next_steps, citations。".to_string(),
        "在 JSON block 之后，再继续写面向人阅读的 Markdown 梳理正文。".to_string(),
        String::new(),
        format!("线程标题：{}", display_name),
        format!("线程 ID：{}", request.transcript.thread_id),
        format!("连接器：{}", request.transcript.connector.as_str()),
        format!("来源：{}", request.export_source.as_str()),
        format!("导出格式：{}", request.export_format.as_str()),
        format!("导出时间：{}", request.exported_at),
        String::new(),
        "需要阅读的导出文件：".to_string(),
        file_list,
        String::new(),
        "请按下面结构输出：".to_string(),
        "```json".to_string(),
        "{".to_string(),
        "  \"summary_title\": \"一句话标题\",".to_string(),
        "  \"overview\": \"一句话总览\",".to_string(),
        "  \"share_safe_summary\": \"适合安全分享的简短总结\",".to_string(),
        "  \"goals\": [\"...\"],".to_string(),
        "  \"files_touched\": [\"...\"],".to_string(),
        "  \"tests_run\": [\"...\"],".to_string(),
        "  \"risks\": [\"...\"],".to_string(),
        "  \"blockers\": [\"...\"],".to_string(),
        "  \"next_steps\": [\"...\"],".to_string(),
        "  \"citations\": [\"...\"]".to_string(),
        "}".to_string(),
        "```".to_string(),
        "# AI 梳理".to_string(),
        "## 1. 对话概览".to_string(),
        "## 2. 用户核心诉求".to_string(),
        "## 3. 关键决定 / 关键实现".to_string(),
        "## 4. 当前状态".to_string(),
        "## 5. 阻碍与风险".to_string(),
        "## 6. 后续动作".to_string(),
    ];

    if let Some(extra) = request
        .extra_instructions
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        sections.push(String::new());
        sections.push("额外要求：".to_string());
        sections.push(extra.to_string());
    }

    sections.join("\n")
}

struct SummaryPreset {
    id: &'static str,
    focus: &'static str,
}

fn summary_preset_prompt_label(value: Option<&str>, preset: &SummaryPreset) -> String {
    match normalize_option(value) {
        Some("release-note") => "release-note（release family）".to_string(),
        Some(
            "handoff"
            | "bug-rca"
            | "release"
            | "decision"
            | "review-digest"
            | "summary-fast"
            | "incident-brief"
            | "team-update"
            | "share-safe-issue-draft",
        ) => preset.id.to_string(),
        _ => preset.id.to_string(),
    }
}

fn resolve_summary_preset(value: Option<&str>) -> SummaryPreset {
    match normalize_option(value).unwrap_or("workbench") {
        "handoff" => SummaryPreset {
            id: "handoff",
            focus: "交接背景、当前状态、后续动作和 blocker",
        },
        "bug-rca" => SummaryPreset {
            id: "bug-rca",
            focus: "问题症状、根因、修复方式、验证与残余风险",
        },
        "release" => SummaryPreset {
            id: "release",
            focus: "面向发布的用户可见变化、验证与已知风险",
        },
        "release-note" => SummaryPreset {
            id: "release",
            focus: "面向发布的用户可见变化、验证与已知风险",
        },
        "decision" => SummaryPreset {
            id: "decision",
            focus: "关键判断、证据基础、已拍板事项与待确认事项",
        },
        "review-digest" => SummaryPreset {
            id: "review-digest",
            focus: "发现、风险级别、影响范围与建议动作",
        },
        "summary-fast" => SummaryPreset {
            id: "summary-fast",
            focus: "高密度摘要、最小废话、只保留最关键行动信息",
        },
        "incident-brief" => SummaryPreset {
            id: "incident-brief",
            focus: "影响范围、时间线、缓解动作、当前风险和待确认项",
        },
        "team-update" => SummaryPreset {
            id: "team-update",
            focus: "进展、风险、下一步和需要同步的变化",
        },
        "share-safe-issue-draft" => SummaryPreset {
            id: "share-safe-issue-draft",
            focus: "适合外发的问题描述、复现、风险和脱敏后的证据",
        },
        _ => SummaryPreset {
            id: "workbench",
            focus: "适合 archive workbench 的可追溯摘要、行动项与证据",
        },
    }
}

fn build_structured_summary_document(
    request: &AiSummaryRequest<'_>,
    options: &AiSummaryOptions,
    markdown_output_path: &std::path::Path,
    html_output_path: &std::path::Path,
    json_output_path: &std::path::Path,
    markdown: &str,
) -> StructuredSummaryDocument {
    let preset = resolve_summary_preset(options.preset.as_deref());
    let extracted = extract_summary_payload(markdown);
    let title = extracted
        .as_ref()
        .and_then(|payload| string_field(payload, "summary_title"))
        .unwrap_or_else(|| {
            request
                .transcript
                .thread_display_name()
                .unwrap_or(request.transcript.thread_id.as_str())
                .to_string()
        });
    let overview = extracted
        .as_ref()
        .and_then(|payload| string_field(payload, "overview"))
        .unwrap_or_else(|| fallback_overview(markdown));
    let share_safe_summary = extracted
        .as_ref()
        .and_then(|payload| string_field(payload, "share_safe_summary"))
        .unwrap_or_else(|| overview.clone());

    StructuredSummaryDocument {
        schema_version: 1,
        thread_id: request.transcript.thread_id.clone(),
        connector: request.transcript.connector.as_str().to_string(),
        source_kind: request.transcript.source_kind.as_str().to_string(),
        completeness: request.transcript.completeness.as_str().to_string(),
        generated_at: request.exported_at.to_string(),
        profile_id: preset.id.to_string(),
        runtime_profile: options.profile.clone(),
        runtime_model: options.model.clone(),
        runtime_provider: options.provider.clone(),
        family_key: summary_family_key(&request.transcript.thread_id),
        title,
        overview,
        share_safe_summary,
        goals: extracted
            .as_ref()
            .and_then(|payload| string_list_field(payload, "goals"))
            .unwrap_or_default(),
        files_touched: extracted
            .as_ref()
            .and_then(|payload| string_list_field(payload, "files_touched"))
            .unwrap_or_default(),
        tests_run: extracted
            .as_ref()
            .and_then(|payload| string_list_field(payload, "tests_run"))
            .unwrap_or_default(),
        risks: extracted
            .as_ref()
            .and_then(|payload| string_list_field(payload, "risks"))
            .unwrap_or_default(),
        blockers: extracted
            .as_ref()
            .and_then(|payload| string_list_field(payload, "blockers"))
            .unwrap_or_default(),
        next_steps: extracted
            .as_ref()
            .and_then(|payload| string_list_field(payload, "next_steps"))
            .unwrap_or_default(),
        citations: extracted
            .as_ref()
            .and_then(|payload| string_list_field(payload, "citations"))
            .unwrap_or_default(),
        extraction_mode: if extracted.is_some() {
            "json-block".to_string()
        } else {
            "fallback".to_string()
        },
        output_files: StructuredSummaryOutputFiles {
            markdown: markdown_output_path
                .file_name()
                .map(|value| value.to_string_lossy().to_string())
                .unwrap_or_else(|| markdown_output_path.display().to_string()),
            html: html_output_path
                .file_name()
                .map(|value| value.to_string_lossy().to_string())
                .unwrap_or_else(|| html_output_path.display().to_string()),
            json: json_output_path
                .file_name()
                .map(|value| value.to_string_lossy().to_string())
                .unwrap_or_else(|| json_output_path.display().to_string()),
        },
    }
}

fn fallback_overview(markdown: &str) -> String {
    markdown
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty() && !line.starts_with('#') && !line.starts_with("```"))
        .unwrap_or("AI summary generated without a structured overview.")
        .to_string()
}

fn extract_summary_payload(markdown: &str) -> Option<Map<String, Value>> {
    let start = markdown.find("```json")?;
    let rest = &markdown[start + "```json".len()..];
    let end = rest.find("```")?;
    let candidate = rest[..end].trim();
    serde_json::from_str::<Value>(candidate)
        .ok()?
        .as_object()
        .cloned()
}

fn string_field(payload: &Map<String, Value>, key: &str) -> Option<String> {
    payload
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn string_list_field(payload: &Map<String, Value>, key: &str) -> Option<Vec<String>> {
    let values = payload.get(key)?.as_array()?;
    let items = values
        .iter()
        .filter_map(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    Some(items)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::core::archive::{
        AiSummaryOptions, ArchiveCompleteness, ArchiveRound, ArchiveThreadStatus,
        ArchiveTranscript, ArchiveTurnStatus, ConnectorSourceKind, OutputTarget,
    };
    use crate::model::{ConnectorKind, OutputFormat};

    use super::{
        AiSummaryRequest, build_ai_summary_exec_args, build_default_summary_prompt,
        build_structured_summary_document, quote_toml_string,
    };

    fn sample_transcript() -> ArchiveTranscript {
        ArchiveTranscript {
            connector: ConnectorKind::Codex,
            thread_id: "thread-12345678".to_string(),
            thread_name: Some("Demo Thread".to_string()),
            preview: Some("hello".to_string()),
            completeness: ArchiveCompleteness::Complete,
            source_kind: ConnectorSourceKind::AppServerThreadRead,
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
                items: Vec::new(),
            }],
        }
    }

    #[test]
    fn default_summary_prompt_mentions_exported_paths_and_structure() {
        let transcript = sample_transcript();
        let request = AiSummaryRequest {
            transcript: &transcript,
            output_target: &OutputTarget::Downloads,
            export_source: crate::core::archive::ExportSource::AppServer,
            export_format: OutputFormat::Markdown,
            exported_at: "2026-04-21T00:00:00Z",
            exported_paths: &[
                PathBuf::from("/tmp/export-part-01.md"),
                PathBuf::from("/tmp/export-part-02.md"),
            ],
            extra_instructions: Some("请特别关注 blocker。"),
            timeout_seconds: Some(30),
        };

        let prompt = build_default_summary_prompt(
            &request,
            &AiSummaryOptions {
                enabled: true,
                instructions: None,
                timeout_seconds: Some(30),
                profile: Some("summary-fast".to_string()),
                preset: Some("bug-rca".to_string()),
                model: Some("o3".to_string()),
                provider: Some("cliproxyapi".to_string()),
            },
        );

        assert!(prompt.contains("/tmp/export-part-01.md"));
        assert!(prompt.contains("/tmp/export-part-02.md"));
        assert!(prompt.contains("# AI 梳理"));
        assert!(prompt.contains("## 5. 阻碍与风险"));
        assert!(prompt.contains("请特别关注 blocker。"));
        assert!(prompt.contains("summary preset：bug-rca"));
        assert!(prompt.contains("\"summary_title\""));
        assert!(prompt.contains("根因"));
    }

    #[test]
    fn ai_summary_exec_args_include_profile_model_and_provider_controls() {
        let args = build_ai_summary_exec_args(
            std::path::Path::new("/tmp/workspace"),
            std::path::Path::new("/tmp/out.md"),
            &AiSummaryOptions {
                enabled: true,
                instructions: None,
                timeout_seconds: Some(30),
                profile: Some("summary-fast".to_string()),
                preset: Some("handoff".to_string()),
                model: Some("o3".to_string()),
                provider: Some("cliproxyapi".to_string()),
            },
        );

        assert_eq!(args[0], "exec");
        assert!(
            args.windows(2)
                .any(|pair| pair == ["--profile", "summary-fast"])
        );
        assert!(args.windows(2).any(|pair| pair == ["--model", "o3"]));
        assert!(
            args.windows(2)
                .any(|pair| pair == ["-c", "model_provider=\"cliproxyapi\""])
        );
    }

    #[test]
    fn ai_summary_exec_args_default_to_low_reasoning_and_verbosity_without_profile() {
        let args = build_ai_summary_exec_args(
            std::path::Path::new("/tmp/workspace"),
            std::path::Path::new("/tmp/out.md"),
            &AiSummaryOptions {
                enabled: true,
                instructions: None,
                timeout_seconds: Some(30),
                profile: None,
                preset: Some("handoff".to_string()),
                model: None,
                provider: None,
            },
        );

        assert!(
            args.windows(2)
                .any(|pair| pair == ["-c", "model_reasoning_effort=\"low\""])
        );
        assert!(
            args.windows(2)
                .any(|pair| pair == ["-c", "model_verbosity=\"low\""])
        );
    }

    #[test]
    fn effective_ai_summary_timeout_prefers_request_then_options_then_default() {
        assert_eq!(
            super::effective_ai_summary_timeout(Some(15), Some(45)).as_secs(),
            15
        );
        assert_eq!(
            super::effective_ai_summary_timeout(None, Some(45)).as_secs(),
            45
        );
        assert_eq!(
            super::effective_ai_summary_timeout(None, None).as_secs(),
            300
        );
    }

    #[test]
    fn quote_toml_string_wraps_provider_as_toml_basic_string() {
        assert_eq!(quote_toml_string("openai"), "\"openai\"");
        assert_eq!(
            quote_toml_string("provider\"with-quote"),
            "\"provider\\\"with-quote\""
        );
    }

    #[test]
    fn summary_presets_cover_incident_team_update_and_issue_draft_flows() {
        let transcript = sample_transcript();
        let request = AiSummaryRequest {
            transcript: &transcript,
            output_target: &OutputTarget::Downloads,
            export_source: crate::core::archive::ExportSource::AppServer,
            export_format: OutputFormat::Markdown,
            exported_at: "2026-04-21T00:00:00Z",
            exported_paths: &[PathBuf::from("/tmp/export-part-01.md")],
            extra_instructions: None,
            timeout_seconds: Some(30),
        };

        for (preset, expected_focus) in [
            ("incident-brief", "影响范围、时间线、缓解动作"),
            ("team-update", "进展、风险、下一步和需要同步的变化"),
            (
                "share-safe-issue-draft",
                "适合外发的问题描述、复现、风险和脱敏后的证据",
            ),
        ] {
            let prompt = build_default_summary_prompt(
                &request,
                &AiSummaryOptions {
                    enabled: true,
                    instructions: None,
                    timeout_seconds: Some(30),
                    profile: None,
                    preset: Some(preset.to_string()),
                    model: None,
                    provider: None,
                },
            );

            assert!(prompt.contains(&format!("summary preset：{preset}")));
            assert!(prompt.contains(expected_focus));
        }
    }

    #[test]
    fn release_note_preset_aliases_release_family_without_hiding_requested_name() {
        let transcript = sample_transcript();
        let request = AiSummaryRequest {
            transcript: &transcript,
            output_target: &OutputTarget::Downloads,
            export_source: crate::core::archive::ExportSource::AppServer,
            export_format: OutputFormat::Markdown,
            exported_at: "2026-04-21T00:00:00Z",
            exported_paths: &[PathBuf::from("/tmp/export-part-01.md")],
            extra_instructions: None,
            timeout_seconds: Some(30),
        };
        let options = AiSummaryOptions {
            enabled: true,
            instructions: None,
            timeout_seconds: Some(30),
            profile: None,
            preset: Some("release-note".to_string()),
            model: None,
            provider: None,
        };

        let prompt = build_default_summary_prompt(&request, &options);
        assert!(prompt.contains("summary preset：release-note（release family）"));
        assert!(prompt.contains("面向发布的用户可见变化、验证与已知风险"));

        let structured = build_structured_summary_document(
            &request,
            &options,
            std::path::Path::new("/tmp/release-note.md"),
            std::path::Path::new("/tmp/release-note.html"),
            std::path::Path::new("/tmp/release-note.json"),
            r#"```json
{
  "summary_title": "Release digest",
  "overview": "Release overview",
  "share_safe_summary": "Share-safe release summary",
  "goals": ["Ship"],
  "files_touched": ["src/core/ai_summary.rs"],
  "tests_run": ["cargo test release_note_preset_aliases_release_family_without_hiding_requested_name -- --exact"],
  "risks": ["docs drift"],
  "blockers": [],
  "next_steps": ["publish"],
  "citations": ["summary-v1"]
}
```"#,
        );

        assert_eq!(structured.profile_id, "release");
    }
}
