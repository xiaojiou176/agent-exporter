use std::path::PathBuf;

use anyhow::{Result, bail};
use chrono::Utc;
use clap::{Parser, Subcommand, ValueEnum};

use crate::connectors;
use crate::core::archive::{AppServerLaunchConfig, ExportRequest, ExportSelector, OutputTarget};
use crate::model::{ConnectorKind, OutputFormat, SupportStage};
use crate::output::markdown::{self, DEFAULT_MAX_LINES_PER_PART};

#[derive(Debug, Parser)]
#[command(
    name = "agent-exporter",
    version,
    about = "Local-first Rust CLI for exporting Codex transcripts and thread archives.",
    long_about = "Local-first Rust CLI for exporting Codex transcripts and thread archives.\
\n\nCurrent delivery: Codex-only v1 via the canonical app-server path.\
\nPlanned expansion: Claude Code and other local agent CLIs."
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
    /// Export a Codex thread through the canonical app-server path.
    Codex(CodexExportArgs),
}

#[derive(Debug, clap::Args)]
struct CodexExportArgs {
    /// Stable Codex thread identifier.
    #[arg(long)]
    thread_id: String,
    /// Output destination contract.
    #[arg(long, value_enum, default_value_t = DestinationArg::Downloads)]
    destination: DestinationArg,
    /// Workspace root required when destination is workspace-conversations.
    #[arg(long)]
    workspace_root: Option<PathBuf>,
    /// Override the command used to launch the local Codex app-server.
    #[arg(long, default_value = "codex")]
    app_server_command: String,
    /// Additional args passed to the app-server command. When omitted and the command is the
    /// default `codex`, the CLI automatically uses `codex app-server`.
    #[arg(long = "app-server-arg")]
    app_server_args: Vec<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
enum DestinationArg {
    Downloads,
    WorkspaceConversations,
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

impl CodexExportArgs {
    fn into_request(self) -> Result<ExportRequest> {
        Ok(ExportRequest {
            connector: ConnectorKind::Codex,
            selector: ExportSelector::ThreadId(self.thread_id),
            format: OutputFormat::Markdown,
            output_target: self.destination.into_output_target(self.workspace_root)?,
            app_server: AppServerLaunchConfig {
                command: self.app_server_command,
                args: self.app_server_args,
            },
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
    println!("- Current scope: Codex-only v1 via the canonical app-server path.");
    println!("- Repository shape: source/core/output split with room for future connectors.");
    println!("- Real export path: `agent-exporter export codex --thread-id <id>`.");
    println!("- Next expansion slice: local direct-read after canonical parity stays green.");
}

fn export_codex(args: CodexExportArgs) -> Result<()> {
    let request = args.into_request()?;

    let transcript = connectors::export(&request)?;
    let exported_at = Utc::now().to_rfc3339();
    let archive_title = transcript.archive_title(&request.output_target);
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

    let outcome =
        crate::core::archive::write_markdown_parts(&transcript, &request.output_target, &parts)?;

    println!("Codex export completed");
    println!("- Connector    : {}", request.connector.as_str());
    println!("- Thread ID    : {}", transcript.thread_id);
    println!("- Completeness : {}", outcome.completeness.as_str());
    println!("- Source       : {}", transcript.source_kind.as_str());
    println!("- Parts        : {}", outcome.exported_part_count);
    println!("- Rounds       : {}", outcome.exported_turn_count);
    println!("- Items        : {}", outcome.exported_item_count);
    println!("- Markdown");
    for path in &outcome.markdown_paths {
        println!("  - {}", path.display());
    }

    Ok(())
}
