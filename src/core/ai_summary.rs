use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread::sleep;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail};

use crate::core::archive::{
    AiSummaryOptions, ArchiveTranscript, ExportSource, OutputTarget,
    allocate_ai_summary_document_path,
};
use crate::model::OutputFormat;

const DEFAULT_AI_SUMMARY_COMMAND: &str = "codex";
const DEFAULT_AI_SUMMARY_TIMEOUT: Duration = Duration::from_secs(120);

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
    let prompt = build_default_summary_prompt(request);
    let mut command = Command::new(DEFAULT_AI_SUMMARY_COMMAND);
    command.args(build_ai_summary_exec_args(
        &working_root,
        &output_path,
        options,
    ));
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
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

    let timeout = Duration::from_secs(
        request
            .timeout_seconds
            .unwrap_or(DEFAULT_AI_SUMMARY_TIMEOUT.as_secs()),
    );
    let start = Instant::now();
    let output = loop {
        if let Some(status) = child
            .try_wait()
            .context("failed while polling AI summary agent")?
        {
            let stdout = child
                .stdout
                .take()
                .context("AI summary agent stdout unavailable")?;
            let stderr = child
                .stderr
                .take()
                .context("AI summary agent stderr unavailable")?;
            let output = std::process::Output {
                status,
                stdout: read_all(stdout).context("failed to read AI summary stdout")?,
                stderr: read_all(stderr).context("failed to read AI summary stderr")?,
            };
            break output;
        }

        if start.elapsed() >= timeout {
            child.kill().ok();
            child.wait().ok();
            bail!(
                "AI summary generation timed out after {} seconds; raw transcript export remains on disk, but the AI summary sidecar was not completed",
                timeout.as_secs()
            );
        }

        sleep(Duration::from_millis(200));
    };
    if !output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!(
            "AI summary generation failed with status {}.\nstdout:\n{}\nstderr:\n{}",
            output.status,
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

    let html_output_path =
        write_html_companion(&output_path, request.transcript, request.exported_at)?;

    Ok(AiSummaryOutcome {
        markdown_output_path: output_path,
        html_output_path,
    })
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

fn read_all<R: std::io::Read>(mut reader: R) -> Result<Vec<u8>> {
    let mut buffer = Vec::new();
    std::io::Read::read_to_end(&mut reader, &mut buffer)?;
    Ok(buffer)
}

fn write_html_companion(
    markdown_output_path: &PathBuf,
    transcript: &ArchiveTranscript,
    exported_at: &str,
) -> Result<PathBuf> {
    let markdown = std::fs::read_to_string(markdown_output_path).with_context(|| {
        format!(
            "failed to read AI summary markdown `{}` before generating HTML companion",
            markdown_output_path.display()
        )
    })?;

    let html_output_path = markdown_output_path.with_extension("html");
    let title = transcript
        .thread_display_name()
        .unwrap_or(transcript.thread_id.as_str());
    let document =
        render_summary_html_document(title, &transcript.thread_id, exported_at, &markdown);

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

fn build_default_summary_prompt(request: &AiSummaryRequest<'_>) -> String {
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

    let mut sections = vec![
        "你现在是一个只读 AI 梳理 Agent。".to_string(),
        "请阅读下面这些刚导出的对话归档文件，并输出一份最终 Markdown 梳理文档。".to_string(),
        "只输出最终 Markdown，不要解释过程，不要输出额外前言，不要修改任何文件。".to_string(),
        String::new(),
        "文档必须使用中文为主，并严格区分“事实 / 推断 / 未确认”。".to_string(),
        "如果证据不足，请明确写“未确认”，不要脑补。".to_string(),
        "如果对话里已经省略了过长工具结果，请基于现有导出内容继续梳理，不要猜被省略的内容。"
            .to_string(),
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
        quote_toml_string,
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

        let prompt = build_default_summary_prompt(&request);

        assert!(prompt.contains("/tmp/export-part-01.md"));
        assert!(prompt.contains("/tmp/export-part-02.md"));
        assert!(prompt.contains("# AI 梳理"));
        assert!(prompt.contains("## 5. 阻碍与风险"));
        assert!(prompt.contains("请特别关注 blocker。"));
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
    fn quote_toml_string_wraps_provider_as_toml_basic_string() {
        assert_eq!(quote_toml_string("openai"), "\"openai\"");
        assert_eq!(
            quote_toml_string("provider\"with-quote"),
            "\"provider\\\"with-quote\""
        );
    }
}
