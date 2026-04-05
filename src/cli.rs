use std::path::PathBuf;

use anyhow::{Result, bail};
use chrono::Utc;
use clap::{Parser, Subcommand, ValueEnum};

use crate::connectors;
use crate::core::archive::{
    AppServerLaunchConfig, ExportRequest, ExportSelector, ExportSource, OutputTarget,
};
use crate::model::{ConnectorKind, OutputFormat, SupportStage};
use crate::output::{
    json as json_output,
    markdown::{self, DEFAULT_MAX_LINES_PER_PART},
};

#[derive(Debug, Parser)]
#[command(
    name = "agent-exporter",
    version,
    about = "Local-first Rust CLI for exporting Codex transcripts and thread archives.",
    long_about = "Local-first Rust CLI for exporting Codex transcripts and thread archives.\
\n\nCurrent delivery: Codex dual-source, a minimal Claude Code second-connector proof, and shared Markdown/JSON export surfaces.\
\nCodex keeps the default canonical app-server front door, while local archival inputs and Claude session-path imports stay explicitly degraded."
)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Show connector support and planned expansion order.
    Connectors,
    /// Print the current repository scaffold status and next implementation slices.
    Scaffold,
    /// Export transcripts and thread archives.
    Export {
        #[command(subcommand)]
        command: ExportCommands,
    },
}

#[derive(Debug, Subcommand)]
enum ExportCommands {
    /// Export a Codex thread through the canonical app-server path or the local archival path.
    Codex(CodexExportArgs),
    /// Export a Claude Code session file through the shared archival transcript contract.
    ClaudeCode(ClaudeCodeExportArgs),
}

#[derive(Debug, clap::Args)]
struct CodexExportArgs {
    /// Source contract to use. `app-server` stays the default canonical path.
    #[arg(long, value_enum, default_value_t = SourceArg::AppServer)]
    source: SourceArg,
    /// Stable Codex thread identifier.
    #[arg(long)]
    thread_id: Option<String>,
    /// Direct rollout jsonl path. Only valid with `--source local`.
    #[arg(long)]
    rollout_path: Option<PathBuf>,
    /// Override `CODEX_HOME` for local direct-read mode.
    #[arg(long)]
    codex_home: Option<PathBuf>,
    /// Output destination contract.
    #[arg(long, value_enum, default_value_t = DestinationArg::Downloads)]
    destination: DestinationArg,
    /// Output format. `markdown` stays the default current path.
    #[arg(long, value_enum, default_value_t = FormatArg::Markdown)]
    format: FormatArg,
    /// Workspace root required when destination is workspace-conversations.
    #[arg(long)]
    workspace_root: Option<PathBuf>,
    /// Override the direct executable used to launch the local Codex app-server.
    /// Host-control utilities and shell-style launchers are rejected.
    #[arg(long, default_value = "codex")]
    app_server_command: String,
    /// Additional args passed to the app-server command. When omitted and the command is the
    /// default `codex`, the CLI automatically uses `codex app-server`. Inline-eval interpreter
    /// flags such as `python -c` are rejected.
    #[arg(long = "app-server-arg")]
    app_server_args: Vec<String>,
}

#[derive(Debug, clap::Args)]
struct ClaudeCodeExportArgs {
    /// Direct Claude Code session path (`.jsonl` or compatible JSON export).
    #[arg(long)]
    session_path: PathBuf,
    /// Output destination contract.
    #[arg(long, value_enum, default_value_t = DestinationArg::Downloads)]
    destination: DestinationArg,
    /// Output format. `markdown` stays the default current path.
    #[arg(long, value_enum, default_value_t = FormatArg::Markdown)]
    format: FormatArg,
    /// Workspace root required when destination is workspace-conversations.
    #[arg(long)]
    workspace_root: Option<PathBuf>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
enum DestinationArg {
    Downloads,
    WorkspaceConversations,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
enum SourceArg {
    AppServer,
    Local,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
enum FormatArg {
    Markdown,
    Json,
}

impl DestinationArg {
    fn into_output_target(self, workspace_root: Option<PathBuf>) -> Result<OutputTarget> {
        match self {
            DestinationArg::Downloads => Ok(OutputTarget::Downloads),
            DestinationArg::WorkspaceConversations => {
                let Some(workspace_root) = workspace_root else {
                    bail!("destination `workspace-conversations` requires --workspace-root <path>");
                };
                Ok(OutputTarget::WorkspaceConversations { workspace_root })
            }
        }
    }
}

impl FormatArg {
    fn into_output_format(self) -> OutputFormat {
        match self {
            Self::Markdown => OutputFormat::Markdown,
            Self::Json => OutputFormat::Json,
        }
    }
}

impl CodexExportArgs {
    fn into_request(self) -> Result<ExportRequest> {
        let source = match self.source {
            SourceArg::AppServer => ExportSource::AppServer,
            SourceArg::Local => ExportSource::Local,
        };

        let selector = match source {
            ExportSource::AppServer => match (self.thread_id, self.rollout_path) {
                (Some(thread_id), None) => ExportSelector::ThreadId(thread_id),
                (None, None) => bail!("app-server source requires --thread-id <THREAD_ID>"),
                (_, Some(_)) => bail!(
                    "`--rollout-path` is only valid with `--source local`; app-server source accepts `--thread-id` only"
                ),
            },
            ExportSource::Local => match (self.thread_id, self.rollout_path) {
                (Some(thread_id), None) => ExportSelector::ThreadId(thread_id),
                (None, Some(rollout_path)) => ExportSelector::RolloutPath(rollout_path),
                (None, None) => bail!(
                    "local source requires either --thread-id <THREAD_ID> or --rollout-path <PATH>"
                ),
                (Some(_), Some(_)) => {
                    bail!("local source accepts either --thread-id or --rollout-path, not both")
                }
            },
            ExportSource::SessionPath => {
                unreachable!("codex export args cannot construct a session-path source")
            }
        };

        let app_server = AppServerLaunchConfig {
            command: self.app_server_command,
            args: self.app_server_args,
        };
        app_server.validate_host_safety()?;

        Ok(ExportRequest {
            connector: ConnectorKind::Codex,
            source,
            selector,
            format: self.format.into_output_format(),
            output_target: self.destination.into_output_target(self.workspace_root)?,
            app_server,
            codex_home: self.codex_home,
        })
    }
}

impl ClaudeCodeExportArgs {
    fn into_request(self) -> Result<ExportRequest> {
        Ok(ExportRequest {
            connector: ConnectorKind::ClaudeCode,
            source: ExportSource::SessionPath,
            selector: ExportSelector::SessionPath(self.session_path),
            format: self.format.into_output_format(),
            output_target: self.destination.into_output_target(self.workspace_root)?,
            app_server: AppServerLaunchConfig::default(),
            codex_home: None,
        })
    }
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Connectors => print_connectors(),
        Commands::Scaffold => print_scaffold_status(),
        Commands::Export { command } => match command {
            ExportCommands::Codex(args) => export_codex(args)?,
            ExportCommands::ClaudeCode(args) => export_claude_code(args)?,
        },
    }
    Ok(())
}

fn print_connectors() {
    println!("Connector roadmap:");
    for connector in connectors::catalog() {
        let stage = match connector.stage {
            SupportStage::Current => "current",
            SupportStage::Planned => "planned",
        };
        println!(
            "- {:<12} | {:<7} | {}",
            connector.kind.as_str(),
            stage,
            connector.summary
        );
    }
}

fn print_scaffold_status() {
    println!("agent-exporter scaffold status");
    println!(
        "- Current scope: Codex dual-source + Claude session-path second connector + shared Markdown/JSON export."
    );
    println!("- Repository shape: source/core/output split with room for future connectors.");
    println!("- Real Codex export path: `agent-exporter export codex --thread-id <id>`.");
    println!(
        "- Real Claude export path: `agent-exporter export claude-code --session-path <path>`."
    );
    println!("- Real JSON export path: add `--format json` to the existing export commands.");
    println!("- Next step: minimal HTML renderer without changing transcript semantics.");
}

fn export_codex(args: CodexExportArgs) -> Result<()> {
    let request = args.into_request()?;
    export_request(request)
}

fn export_claude_code(args: ClaudeCodeExportArgs) -> Result<()> {
    let request = args.into_request()?;
    export_request(request)
}

fn export_request(request: ExportRequest) -> Result<()> {
    let transcript = connectors::export(&request)?;
    let exported_at = Utc::now().to_rfc3339();
    let archive_title = transcript.archive_title(&request.output_target);
    let outcome = match request.format {
        OutputFormat::Markdown => {
            let parts = markdown::render_markdown_parts(
                &transcript,
                &archive_title,
                &exported_at,
                DEFAULT_MAX_LINES_PER_PART,
            );
            if parts.is_empty() {
                bail!(
                    "No exportable Markdown content found for thread `{}`.",
                    transcript.thread_id
                );
            }
            crate::core::archive::write_markdown_parts(&transcript, &request.output_target, &parts)?
        }
        OutputFormat::Json => {
            let document =
                json_output::render_json_document(&transcript, &archive_title, &exported_at);
            crate::core::archive::write_json_document(
                &transcript,
                &request.output_target,
                &document,
            )?
        }
        OutputFormat::Html => bail!("HTML export is planned but not implemented yet"),
    };

    println!("Export completed");
    println!("- Connector    : {}", request.connector.as_str());
    println!("- Selection    : {}", request.source.as_str());
    println!("- Format       : {}", request.format.as_str());
    println!("- Thread ID    : {}", transcript.thread_id);
    println!("- Completeness : {}", outcome.completeness.as_str());
    println!("- Source       : {}", transcript.source_kind.as_str());
    println!("- Files        : {}", outcome.output_paths.len());
    println!("- Rounds       : {}", outcome.exported_turn_count);
    println!("- Items        : {}", outcome.exported_item_count);
    println!("- Output");
    for path in &outcome.output_paths {
        println!("  - {}", path.display());
    }

    Ok(())
}
