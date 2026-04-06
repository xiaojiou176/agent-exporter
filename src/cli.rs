use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use chrono::Utc;
use clap::{Parser, Subcommand, ValueEnum};

use crate::connectors;
use crate::core::archive::{
    AppServerLaunchConfig, ExportRequest, ExportSelector, ExportSource, OutputTarget,
};
use crate::core::archive_index;
use crate::core::integration_report::{
    collect_integration_report_entries, resolve_integration_reports_dir,
    write_integration_report_document, write_integration_reports_index_document,
};
use crate::core::search_report::{
    collect_search_report_entries, write_search_report_document,
    write_search_reports_index_document,
};
use crate::core::semantic_search::{
    FastEmbedSemanticEmbedder, SemanticEmbedder, hybrid_search_with_persistent_index,
    semantic_search_with_persistent_index,
};
use crate::integrations::{
    IntegrationDoctorCheck, IntegrationDoctorRequest, IntegrationMaterializeRequest,
    IntegrationOnboardRequest, IntegrationPlatform, doctor_integration, doctor_next_steps,
    doctor_summary, materialize_integration, onboard_integration,
};
use crate::model::{ConnectorKind, OutputFormat, SupportStage};
use crate::output::{
    archive_index as archive_index_output, html as html_output,
    integration_report::{
        IntegrationReportDocument, IntegrationReportKind, render_integration_report_document,
        render_integration_reports_index_document,
    },
    json as json_output,
    markdown::{self, DEFAULT_MAX_LINES_PER_PART},
    search_report::{
        SearchReportDocument, SearchReportHit, SearchReportKind, render_search_report_document,
        render_search_reports_index_document,
    },
};

#[derive(Debug, Parser)]
#[command(
    name = "agent-exporter",
    version,
    about = "Local-first Rust CLI for exporting Codex transcripts and thread archives.",
    long_about = "Local-first Rust CLI for exporting Codex transcripts and thread archives.\
\n\nCurrent delivery: Codex dual-source, a minimal Claude Code second-connector proof, and shared Markdown/JSON/HTML export surfaces.\
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
    /// Generate a local archive index for existing transcript exports.
    Publish {
        #[command(subcommand)]
        command: PublishCommands,
    },
    /// Query the local archive corpus with semantic retrieval.
    Search {
        #[command(subcommand)]
        command: SearchCommands,
    },
    /// Materialize repo-owned integration assets into an explicit staging target directory.
    Integrate {
        #[command(subcommand)]
        command: IntegrateCommands,
    },
    /// Check integration readiness for a target directory without mutating it.
    Doctor {
        #[command(subcommand)]
        command: DoctorCommands,
    },
    /// Materialize a platform pack and immediately explain the resulting readiness and next steps.
    Onboard {
        #[command(subcommand)]
        command: OnboardCommands,
    },
}

#[derive(Debug, Subcommand)]
enum ExportCommands {
    /// Export a Codex thread through the canonical app-server path or the local archival path.
    Codex(CodexExportArgs),
    /// Export a Claude Code session file through the shared archival transcript contract.
    ClaudeCode(ClaudeCodeExportArgs),
}

#[derive(Debug, Subcommand)]
enum PublishCommands {
    /// Generate a static index for HTML transcript exports inside workspace conversations.
    ArchiveIndex(PublishArchiveIndexArgs),
}

#[derive(Debug, Subcommand)]
enum SearchCommands {
    /// Run embedding-based semantic retrieval over local HTML transcript exports.
    Semantic(SearchSemanticArgs),
    /// Run hybrid retrieval that blends semantic ranking with lexical metadata signals.
    Hybrid(SearchHybridArgs),
}

#[derive(Debug, Subcommand)]
enum IntegrateCommands {
    /// Materialize Codex integration assets into an explicit staging target directory.
    Codex(IntegrateArgs),
    /// Materialize Claude Code integration assets into an explicit staging target directory.
    ClaudeCode(IntegrateArgs),
    /// Materialize OpenClaw bundle/plugin assets into an explicit staging target directory.
    #[command(name = "openclaw")]
    OpenClaw(IntegrateArgs),
}

#[derive(Debug, Subcommand)]
enum DoctorCommands {
    /// Check integration readiness for one platform and one explicit target directory.
    Integrations(DoctorIntegrationsArgs),
}

#[derive(Debug, Subcommand)]
enum OnboardCommands {
    /// Materialize a Codex onboarding pack into a staging target and explain the next steps.
    Codex(OnboardArgs),
    /// Materialize a Claude Code onboarding pack into a staging target and explain the next steps.
    ClaudeCode(OnboardArgs),
    /// Materialize an OpenClaw onboarding pack into a staging target and explain the next steps.
    #[command(name = "openclaw")]
    OpenClaw(OnboardArgs),
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

#[derive(Debug, clap::Args)]
struct PublishArchiveIndexArgs {
    /// Workspace root whose `.agents/Conversations` directory should be indexed.
    #[arg(long)]
    workspace_root: PathBuf,
}

#[derive(Debug, clap::Args)]
struct SearchSemanticArgs {
    /// Workspace root whose `.agents/Conversations` directory should be searched.
    #[arg(long)]
    workspace_root: PathBuf,
    /// Natural-language semantic query.
    #[arg(long)]
    query: String,
    /// Maximum number of hits to return.
    #[arg(long, default_value_t = 5)]
    top_k: usize,
    /// Override the local embedding model directory.
    #[arg(long)]
    model_dir: Option<PathBuf>,
    /// Save this retrieval run as a local HTML report artifact.
    #[arg(long, default_value_t = false)]
    save_report: bool,
}

#[derive(Debug, clap::Args)]
struct SearchHybridArgs {
    /// Workspace root whose `.agents/Conversations` directory should be searched.
    #[arg(long)]
    workspace_root: PathBuf,
    /// Natural-language query blended across semantic and lexical metadata signals.
    #[arg(long)]
    query: String,
    /// Maximum number of hits to return.
    #[arg(long, default_value_t = 5)]
    top_k: usize,
    /// Override the local embedding model directory.
    #[arg(long)]
    model_dir: Option<PathBuf>,
    /// Save this retrieval run as a local HTML report artifact.
    #[arg(long, default_value_t = false)]
    save_report: bool,
}

#[derive(Debug, clap::Args)]
struct IntegrateArgs {
    /// Explicit staging target directory where integration assets should be materialized.
    /// Live host/global roots such as `~/.codex`, `~/.claude*`, and direct OpenClaw bundle/plugin roots are rejected.
    #[arg(long)]
    target: PathBuf,
}

#[derive(Debug, clap::Args)]
struct DoctorIntegrationsArgs {
    /// Platform whose integration target should be checked.
    #[arg(long, value_enum)]
    platform: PlatformArg,
    /// Explicit target directory to inspect. The doctor never mutates this path.
    #[arg(long)]
    target: PathBuf,
    /// Save a static integration evidence report under the current working directory.
    #[arg(long, default_value_t = false)]
    save_report: bool,
}

#[derive(Debug, clap::Args)]
struct OnboardArgs {
    /// Explicit staging target directory where integration assets should be materialized.
    /// Live host/global roots such as `~/.codex`, `~/.claude*`, and direct OpenClaw bundle/plugin roots are rejected.
    #[arg(long)]
    target: PathBuf,
    /// Save a static integration evidence report under the current working directory.
    #[arg(long, default_value_t = false)]
    save_report: bool,
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
    Html,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
enum PlatformArg {
    Codex,
    ClaudeCode,
    #[value(name = "openclaw")]
    OpenClaw,
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
            Self::Html => OutputFormat::Html,
        }
    }
}

impl PlatformArg {
    fn into_integration_platform(self) -> IntegrationPlatform {
        match self {
            Self::Codex => IntegrationPlatform::Codex,
            Self::ClaudeCode => IntegrationPlatform::ClaudeCode,
            Self::OpenClaw => IntegrationPlatform::OpenClaw,
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
        Commands::Publish { command } => match command {
            PublishCommands::ArchiveIndex(args) => publish_archive_index(args)?,
        },
        Commands::Search { command } => match command {
            SearchCommands::Semantic(args) => search_semantic(args)?,
            SearchCommands::Hybrid(args) => search_hybrid(args)?,
        },
        Commands::Integrate { command } => match command {
            IntegrateCommands::Codex(args) => integrate_platform(IntegrationPlatform::Codex, args)?,
            IntegrateCommands::ClaudeCode(args) => {
                integrate_platform(IntegrationPlatform::ClaudeCode, args)?
            }
            IntegrateCommands::OpenClaw(args) => {
                integrate_platform(IntegrationPlatform::OpenClaw, args)?
            }
        },
        Commands::Doctor { command } => match command {
            DoctorCommands::Integrations(args) => doctor_integrations(args)?,
        },
        Commands::Onboard { command } => match command {
            OnboardCommands::Codex(args) => onboard_platform(IntegrationPlatform::Codex, args)?,
            OnboardCommands::ClaudeCode(args) => {
                onboard_platform(IntegrationPlatform::ClaudeCode, args)?
            }
            OnboardCommands::OpenClaw(args) => {
                onboard_platform(IntegrationPlatform::OpenClaw, args)?
            }
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
        "- Current scope: Codex dual-source + Claude session-path second connector + shared Markdown/JSON/HTML export + local archive index + semantic retrieval + persistent local semantic index + hybrid retrieval + local multi-agent archive shell + retrieval report artifacts + workspace-local transcript backlinks + local reports shell + reports-shell metadata search + integration pack + minimal stdio MCP bridge + repo-owned integration materializer + integration doctor + platform-aware integration doctor diagnostics + integration pack-shape hardening + integration onboarding experience + forbidden-target onboarding guardrails + integration evidence pack reports."
    );
    println!("- Repository shape: source/core/output split with room for future connectors.");
    println!("- Real Codex export path: `agent-exporter export codex --thread-id <id>`.");
    println!(
        "- Real Claude export path: `agent-exporter export claude-code --session-path <path>`."
    );
    println!("- Real JSON export path: add `--format json` to the existing export commands.");
    println!("- Real HTML export path: add `--format html` to the existing export commands.");
    println!(
        "- Real archive index path: `agent-exporter publish archive-index --workspace-root <repo>`."
    );
    println!(
        "- Real reports shell path: generated by `agent-exporter publish archive-index --workspace-root <repo>` into `.agents/Search/Reports/index.html`."
    );
    println!(
        "- Real semantic retrieval path: `agent-exporter search semantic --workspace-root <repo> --query <text>`."
    );
    println!(
        "- Real hybrid retrieval path: `agent-exporter search hybrid --workspace-root <repo> --query <text>`."
    );
    println!(
        "- Real MCP bridge path: `python3 scripts/agent_exporter_mcp.py` with local stdio tool exposure for publish/search workflows."
    );
    println!(
        "- Real integration materializer path: `agent-exporter integrate <platform> --target <dir>`."
    );
    println!(
        "- Real integration doctor path: `agent-exporter doctor integrations --platform <platform> --target <dir>`."
    );
    println!("- Real onboarding path: `agent-exporter onboard <platform> --target <dir>`.");
    println!(
        "- Real integration evidence path: `agent-exporter doctor integrations --platform <platform> --target <dir> --save-report` or `agent-exporter onboard <platform> --target <dir> --save-report`."
    );
    println!(
        "- Next step: a new post-Phase-26 product decision, while staying local-first and non-hosted."
    );
}

fn export_codex(args: CodexExportArgs) -> Result<()> {
    let request = args.into_request()?;
    export_request(request)
}

fn export_claude_code(args: ClaudeCodeExportArgs) -> Result<()> {
    let request = args.into_request()?;
    export_request(request)
}

fn publish_archive_index(args: PublishArchiveIndexArgs) -> Result<()> {
    let entries = archive_index::collect_html_archive_entries(&args.workspace_root)?;
    let reports = collect_search_report_entries(&args.workspace_root)?;
    let archive_title = args
        .workspace_root
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| format!("{name} archive index"))
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "agent-exporter archive index".to_string());
    let generated_at = Utc::now().to_rfc3339();
    let document = archive_index_output::render_archive_index_document(
        &archive_title,
        &generated_at,
        &entries,
        &reports,
    );
    let reports_document = render_search_reports_index_document(
        &format!("{archive_title} search reports"),
        &generated_at,
        &reports,
    );
    let archive_dir = archive_index::resolve_workspace_conversations_dir(&args.workspace_root)?;
    let index_path = archive_index::write_archive_index_document(&args.workspace_root, &document)?;
    let reports_index_path =
        write_search_reports_index_document(&args.workspace_root, &reports_document)?;

    println!("Archive index published");
    println!("- Workspace    : {}", args.workspace_root.display());
    println!("- Archive Dir  : {}", archive_dir.display());
    println!("- Entries      : {}", entries.len());
    println!("- Reports      : {}", reports.len());
    println!("- Index        : {}", index_path.display());
    println!("- Reports Index: {}", reports_index_path.display());

    Ok(())
}

fn search_semantic(args: SearchSemanticArgs) -> Result<()> {
    let model_dir = match args.model_dir {
        Some(path) => path,
        None => FastEmbedSemanticEmbedder::default_model_dir()?,
    };
    let embedder = FastEmbedSemanticEmbedder::load_from_dir(&model_dir)?;
    let execution = semantic_search_with_persistent_index(
        &embedder,
        &args.workspace_root,
        &args.query,
        args.top_k,
    )?;

    println!("Semantic search completed");
    println!("- Workspace    : {}", args.workspace_root.display());
    println!("- Query        : {}", args.query);
    println!("- Model Dir    : {}", model_dir.display());
    println!("- True Semantic: {}", embedder.is_true_semantic());
    println!("- Index Path   : {}", execution.index_path.display());
    println!("- Documents    : {}", execution.total_documents);
    println!("- Reused       : {}", execution.reused_documents);
    println!("- Embedded     : {}", execution.embedded_documents);
    println!("- Hits         : {}", execution.hits.len());
    for (index, hit) in execution.hits.iter().enumerate() {
        println!(
            "  {}. {:.4} | {} | {}",
            index + 1,
            hit.score,
            hit.entry.title,
            hit.entry.relative_href
        );
        if let Some(connector) = hit.entry.connector.as_deref() {
            println!("     connector    : {connector}");
        }
        if let Some(thread_id) = hit.entry.thread_id.as_deref() {
            println!("     thread       : {thread_id}");
        }
        if let Some(completeness) = hit.entry.completeness.as_deref() {
            println!("     completeness : {completeness}");
        }
    }

    if args.save_report {
        let report_path = write_semantic_report(
            &args.workspace_root,
            &args.query,
            &model_dir,
            &execution,
            &Utc::now().to_rfc3339(),
        )?;
        println!("- Report       : {}", report_path.display());
    }

    Ok(())
}

fn search_hybrid(args: SearchHybridArgs) -> Result<()> {
    let model_dir = match args.model_dir {
        Some(path) => path,
        None => FastEmbedSemanticEmbedder::default_model_dir()?,
    };
    let embedder = FastEmbedSemanticEmbedder::load_from_dir(&model_dir)?;
    let execution = hybrid_search_with_persistent_index(
        &embedder,
        &args.workspace_root,
        &args.query,
        args.top_k,
    )?;

    println!("Hybrid search completed");
    println!("- Workspace    : {}", args.workspace_root.display());
    println!("- Query        : {}", args.query);
    println!("- Model Dir    : {}", model_dir.display());
    println!("- True Semantic: {}", embedder.is_true_semantic());
    println!("- Index Path   : {}", execution.index_path.display());
    println!("- Documents    : {}", execution.total_documents);
    println!("- Reused       : {}", execution.reused_documents);
    println!("- Embedded     : {}", execution.embedded_documents);
    println!("- Hits         : {}", execution.hits.len());
    for (index, hit) in execution.hits.iter().enumerate() {
        println!(
            "  {}. {:.4} | semantic {:.4} | lexical {:.4} | {} | {}",
            index + 1,
            hit.hybrid_score,
            hit.semantic_score,
            hit.lexical_score,
            hit.entry.title,
            hit.entry.relative_href
        );
        if let Some(connector) = hit.entry.connector.as_deref() {
            println!("     connector    : {connector}");
        }
        if let Some(thread_id) = hit.entry.thread_id.as_deref() {
            println!("     thread       : {thread_id}");
        }
        if let Some(completeness) = hit.entry.completeness.as_deref() {
            println!("     completeness : {completeness}");
        }
    }

    if args.save_report {
        let report_path = write_hybrid_report(
            &args.workspace_root,
            &args.query,
            &model_dir,
            &execution,
            &Utc::now().to_rfc3339(),
        )?;
        println!("- Report       : {}", report_path.display());
    }

    Ok(())
}

fn write_semantic_report(
    workspace_root: &std::path::Path,
    query: &str,
    model_dir: &std::path::Path,
    execution: &crate::core::semantic_search::SemanticSearchExecution,
    generated_at: &str,
) -> Result<PathBuf> {
    let report = SearchReportDocument {
        kind: SearchReportKind::Semantic,
        query: query.to_string(),
        generated_at: generated_at.to_string(),
        workspace_root: workspace_root.display().to_string(),
        model_dir: model_dir.display().to_string(),
        index_path: execution.index_path.display().to_string(),
        total_documents: execution.total_documents,
        reused_documents: execution.reused_documents,
        embedded_documents: execution.embedded_documents,
        hits: execution
            .hits
            .iter()
            .map(|hit| SearchReportHit {
                entry: hit.entry.clone(),
                primary_score: hit.score,
                semantic_score: None,
                lexical_score: None,
            })
            .collect(),
    };
    let rendered = render_search_report_document(&report);
    write_search_report_document(
        workspace_root,
        report.kind.as_str(),
        &report.query,
        generated_at,
        &rendered,
    )
}

fn write_hybrid_report(
    workspace_root: &std::path::Path,
    query: &str,
    model_dir: &std::path::Path,
    execution: &crate::core::semantic_search::HybridSearchExecution,
    generated_at: &str,
) -> Result<PathBuf> {
    let report = SearchReportDocument {
        kind: SearchReportKind::Hybrid,
        query: query.to_string(),
        generated_at: generated_at.to_string(),
        workspace_root: workspace_root.display().to_string(),
        model_dir: model_dir.display().to_string(),
        index_path: execution.index_path.display().to_string(),
        total_documents: execution.total_documents,
        reused_documents: execution.reused_documents,
        embedded_documents: execution.embedded_documents,
        hits: execution
            .hits
            .iter()
            .map(|hit| SearchReportHit {
                entry: hit.entry.clone(),
                primary_score: hit.hybrid_score,
                semantic_score: Some(hit.semantic_score),
                lexical_score: Some(hit.lexical_score),
            })
            .collect(),
    };
    let rendered = render_search_report_document(&report);
    write_search_report_document(
        workspace_root,
        report.kind.as_str(),
        &report.query,
        generated_at,
        &rendered,
    )
}

fn integrate_platform(platform: IntegrationPlatform, args: IntegrateArgs) -> Result<()> {
    let outcome = materialize_integration(&IntegrationMaterializeRequest {
        platform,
        target_root: args.target,
    })?;

    println!("Integration materialized");
    println!("- Platform     : {}", outcome.platform.as_str());
    println!("- Target       : {}", outcome.target_root.display());
    println!("- Launcher     : {}", outcome.launcher.kind);
    println!("- Command      : {}", outcome.launcher.shell_command());
    println!("- Written      : {}", outcome.written_files.len());
    println!("- Unchanged    : {}", outcome.unchanged_files.len());
    if !outcome.written_files.is_empty() {
        println!("- Files Written");
        for path in &outcome.written_files {
            println!("  - {}", path.display());
        }
    }
    if !outcome.unchanged_files.is_empty() {
        println!("- Files Unchanged");
        for path in &outcome.unchanged_files {
            println!("  - {}", path.display());
        }
    }

    Ok(())
}

fn doctor_integrations(args: DoctorIntegrationsArgs) -> Result<()> {
    let outcome = doctor_integration(&IntegrationDoctorRequest {
        platform: args.platform.into_integration_platform(),
        target_root: args.target,
    })?;
    let saved_report = if args.save_report {
        let workspace_root = integration_reports_workspace_root()?;
        Some(write_doctor_integration_report(
            &workspace_root,
            &outcome,
            &doctor_next_steps(&outcome),
        )?)
    } else {
        None
    };

    println!("Integration doctor completed");
    println!("- Platform     : {}", outcome.platform.as_str());
    println!("- Target       : {}", outcome.target_root.display());
    println!("- Readiness    : {}", outcome.overall_readiness.as_str());
    println!("- Summary      : {}", doctor_summary(&outcome));
    println!("- Launcher     : {}", outcome.launcher.kind);
    println!("- Command      : {}", outcome.launcher.shell_command());
    let next_steps = doctor_next_steps(&outcome);
    if !next_steps.is_empty() {
        println!("- Next Steps");
        for (index, step) in next_steps.iter().enumerate() {
            println!("  {}. {}", index + 1, step);
        }
    }
    if let Some((report_path, index_path)) = &saved_report {
        println!("- Report       : {}", report_path.display());
        println!("- Reports Index: {}", index_path.display());
        println!(
            "- Reports Root : {}",
            resolve_integration_reports_dir(&integration_reports_workspace_root()?).display()
        );
    }
    println!("- Checks");
    for check in &outcome.checks {
        print_doctor_check(check);
    }

    Ok(())
}

fn onboard_platform(platform: IntegrationPlatform, args: OnboardArgs) -> Result<()> {
    let outcome = onboard_integration(&IntegrationOnboardRequest {
        platform,
        target_root: args.target,
    })?;
    let saved_report = if args.save_report {
        let workspace_root = integration_reports_workspace_root()?;
        Some(write_onboard_integration_report(&workspace_root, &outcome)?)
    } else {
        None
    };

    println!("Integration onboarding completed");
    println!("- Platform     : {}", outcome.platform.as_str());
    println!("- Target       : {}", outcome.target_root.display());
    println!(
        "- Readiness    : {}",
        outcome.doctor.overall_readiness.as_str()
    );
    println!("- Summary      : {}", doctor_summary(&outcome.doctor));
    println!("- Launcher     : {}", outcome.doctor.launcher.kind);
    println!(
        "- Command      : {}",
        outcome.doctor.launcher.shell_command()
    );
    println!(
        "- Written      : {}",
        outcome.materialized.written_files.len()
    );
    println!(
        "- Unchanged    : {}",
        outcome.materialized.unchanged_files.len()
    );
    if !outcome.materialized.written_files.is_empty() {
        println!("- Files Written");
        for path in &outcome.materialized.written_files {
            println!("  - {}", path.display());
        }
    }
    if !outcome.next_steps.is_empty() {
        println!("- Next Steps");
        for (index, step) in outcome.next_steps.iter().enumerate() {
            println!("  {}. {}", index + 1, step);
        }
    }
    if let Some((report_path, index_path)) = &saved_report {
        println!("- Report       : {}", report_path.display());
        println!("- Reports Index: {}", index_path.display());
        println!(
            "- Reports Root : {}",
            resolve_integration_reports_dir(&integration_reports_workspace_root()?).display()
        );
    }

    Ok(())
}

fn print_doctor_check(check: &IntegrationDoctorCheck) {
    println!(
        "  - {} [{}] {}",
        check.label,
        check.readiness.as_str(),
        check.detail
    );
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
        OutputFormat::Html => {
            let document = html_output::render_html_document(
                &transcript,
                &archive_title,
                &exported_at,
                workspace_html_navigation(&request.output_target).as_ref(),
            );
            crate::core::archive::write_html_document(
                &transcript,
                &request.output_target,
                &document,
            )?
        }
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

fn workspace_html_navigation(
    output_target: &OutputTarget,
) -> Option<html_output::WorkspaceHtmlNavigation> {
    match output_target {
        OutputTarget::Downloads => None,
        OutputTarget::WorkspaceConversations { .. } => Some(html_output::WorkspaceHtmlNavigation {
            archive_shell_href: "index.html".to_string(),
            reports_shell_href: "../Search/Reports/index.html".to_string(),
        }),
    }
}

fn integration_reports_workspace_root() -> Result<PathBuf> {
    std::env::current_dir()
        .context("failed to resolve current working directory for integration reports")
}

fn integration_reports_title(workspace_root: &std::path::Path) -> String {
    workspace_root
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| format!("{name} integration reports"))
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "agent-exporter integration reports".to_string())
}

fn write_doctor_integration_report(
    workspace_root: &std::path::Path,
    outcome: &crate::integrations::IntegrationDoctorOutcome,
    next_steps: &[String],
) -> Result<(PathBuf, PathBuf)> {
    let generated_at = Utc::now().to_rfc3339();
    let report = IntegrationReportDocument {
        kind: IntegrationReportKind::Doctor,
        platform: outcome.platform.as_str().to_string(),
        target_root: outcome.target_root.display().to_string(),
        generated_at: generated_at.clone(),
        readiness: outcome.overall_readiness.as_str().to_string(),
        summary: doctor_summary(outcome),
        launcher_kind: outcome.launcher.kind.to_string(),
        launcher_command: outcome.launcher.shell_command(),
        written_files: Vec::new(),
        unchanged_files: Vec::new(),
        checks: outcome.checks.clone(),
        next_steps: next_steps.to_vec(),
    };
    write_integration_report_bundle(workspace_root, &report)
}

fn write_onboard_integration_report(
    workspace_root: &std::path::Path,
    outcome: &crate::integrations::IntegrationOnboardOutcome,
) -> Result<(PathBuf, PathBuf)> {
    let generated_at = Utc::now().to_rfc3339();
    let report = IntegrationReportDocument {
        kind: IntegrationReportKind::Onboard,
        platform: outcome.platform.as_str().to_string(),
        target_root: outcome.target_root.display().to_string(),
        generated_at: generated_at.clone(),
        readiness: outcome.doctor.overall_readiness.as_str().to_string(),
        summary: doctor_summary(&outcome.doctor),
        launcher_kind: outcome.doctor.launcher.kind.to_string(),
        launcher_command: outcome.doctor.launcher.shell_command(),
        written_files: outcome
            .materialized
            .written_files
            .iter()
            .map(|path| path.display().to_string())
            .collect(),
        unchanged_files: outcome
            .materialized
            .unchanged_files
            .iter()
            .map(|path| path.display().to_string())
            .collect(),
        checks: outcome.doctor.checks.clone(),
        next_steps: outcome.next_steps.clone(),
    };
    write_integration_report_bundle(workspace_root, &report)
}

fn write_integration_report_bundle(
    workspace_root: &std::path::Path,
    report: &IntegrationReportDocument,
) -> Result<(PathBuf, PathBuf)> {
    let document = render_integration_report_document(report);
    let report_path = write_integration_report_document(
        workspace_root,
        report.kind.as_str(),
        &report.platform,
        &report.generated_at,
        &document,
    )?;
    let entries = collect_integration_report_entries(workspace_root)?;
    let index_document = render_integration_reports_index_document(
        &integration_reports_title(workspace_root),
        &report.generated_at,
        &entries,
    );
    let index_path = write_integration_reports_index_document(workspace_root, &index_document)?;
    Ok((report_path, index_path))
}
