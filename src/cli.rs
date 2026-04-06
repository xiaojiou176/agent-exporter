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
    IntegrationBaselineRecord, IntegrationDecisionRecord, IntegrationEvidenceDiff,
    IntegrationEvidenceGateOutcome, IntegrationEvidenceGatePolicy, IntegrationEvidencePolicyPack,
    IntegrationReportCheckRecord, IntegrationReportJsonDocument,
    append_integration_decision_record, assess_promotion_eligibility, build_baseline_record,
    build_integration_evidence_explain, canonical_report_json_path,
    collect_integration_report_entries, collect_integration_report_json_documents,
    collect_repo_owned_integration_policy_packs, diff_integration_reports,
    effective_gate_policy_for_platform, find_integration_baseline_for_identity,
    find_integration_baseline_record, gate_integration_reports, integration_target_identity,
    latest_integration_decision_for_candidate, read_integration_baseline_registry_document,
    read_integration_decision_history_document, read_integration_report_json_document,
    repo_owned_integration_policy_dir, resolve_integration_baseline_registry_path,
    resolve_integration_decision_history_path, resolve_integration_evidence_policy_pack,
    resolve_integration_reports_dir, upsert_integration_baseline_record,
    write_integration_baseline_registry_document, write_integration_decision_history_document,
    write_integration_report_document, write_integration_report_json_document,
    write_integration_reports_index_document, write_integration_reports_index_json_document,
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
        IntegrationReportDocument, IntegrationReportKind, build_integration_report_json_document,
        build_integration_reports_index_json_document, render_integration_report_document,
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
    /// Compare saved integration evidence snapshots without rerunning doctor/onboard.
    Evidence {
        #[command(subcommand)]
        command: EvidenceCommands,
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

#[derive(Debug, Subcommand)]
enum EvidenceCommands {
    /// Diff two saved integration evidence reports (`.json` or sibling `.html` paths).
    Diff(EvidenceDiffArgs),
    /// Gate a candidate evidence snapshot against a baseline snapshot.
    Gate(EvidenceGateArgs),
    /// Explain the ordered remediation sequence for one saved integration evidence report.
    Explain(EvidenceExplainArgs),
    /// Manage official baselines for saved integration evidence.
    Baseline {
        #[command(subcommand)]
        command: EvidenceBaselineCommands,
    },
    /// Inspect repo-owned policy packs for integration governance.
    Policy {
        #[command(subcommand)]
        command: EvidencePolicyCommands,
    },
    /// Compare a candidate against the official baseline, record decision history, and promote when eligible.
    Promote(EvidencePromoteArgs),
    /// Show recent decision history.
    History(EvidenceHistoryArgs),
}

#[derive(Debug, Subcommand)]
enum EvidenceBaselineCommands {
    /// List registered official baselines under the current workspace.
    List,
    /// Show one registered baseline by name.
    Show(EvidenceBaselineShowArgs),
    /// Seed or update an official baseline directly from one saved report.
    Promote(EvidenceBaselinePromoteArgs),
}

#[derive(Debug, Subcommand)]
enum EvidencePolicyCommands {
    /// List repo-owned governance policy packs.
    List,
    /// Show one repo-owned governance policy pack by name.
    Show(EvidencePolicyShowArgs),
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
    /// Print an ordered remediation plan with why-first and recheck guidance.
    #[arg(long, default_value_t = false)]
    explain: bool,
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

#[derive(Debug, clap::Args)]
struct EvidenceDiffArgs {
    /// Left report path (`.json` preferred; `.html` resolves to sibling `.json`).
    #[arg(long)]
    left: PathBuf,
    /// Right report path (`.json` preferred; `.html` resolves to sibling `.json`).
    #[arg(long)]
    right: PathBuf,
}

#[derive(Debug, clap::Args)]
struct EvidenceGateArgs {
    /// Baseline report path or registered baseline name.
    #[arg(long)]
    baseline: String,
    /// Candidate report path (`.json` preferred; `.html` resolves to sibling `.json`).
    #[arg(long)]
    candidate: PathBuf,
    /// Optional policy pack name or local JSON policy path.
    #[arg(long)]
    policy: Option<String>,
}

#[derive(Debug, clap::Args)]
struct EvidenceExplainArgs {
    /// Saved report path (`.json` preferred; `.html` resolves to sibling `.json`).
    #[arg(long)]
    report: PathBuf,
}

#[derive(Debug, clap::Args)]
struct EvidenceBaselineShowArgs {
    /// Registered baseline name.
    #[arg(long)]
    name: String,
}

#[derive(Debug, clap::Args)]
struct EvidenceBaselinePromoteArgs {
    /// Saved report path (`.json` preferred; `.html` resolves to sibling `.json`).
    #[arg(long)]
    report: PathBuf,
    /// Official baseline name.
    #[arg(long)]
    name: String,
    /// Optional policy pack name or local JSON policy path.
    #[arg(long)]
    policy: Option<String>,
    /// Optional human note or rationale.
    #[arg(long)]
    note: Option<String>,
    /// Source verdict recorded into the baseline registry entry.
    #[arg(long, default_value = "manual")]
    verdict: String,
}

#[derive(Debug, clap::Args)]
struct EvidencePolicyShowArgs {
    /// Repo-owned policy pack name.
    #[arg(long)]
    name: String,
}

#[derive(Debug, clap::Args)]
struct EvidencePromoteArgs {
    /// Candidate report path (`.json` preferred; `.html` resolves to sibling `.json`).
    #[arg(long)]
    candidate: PathBuf,
    /// Baseline registry name whose official report should be used.
    #[arg(long)]
    baseline_name: String,
    /// Optional policy pack name or local JSON policy path.
    #[arg(long)]
    policy: Option<String>,
    /// Optional human note or rationale.
    #[arg(long)]
    note: Option<String>,
}

#[derive(Debug, clap::Args)]
struct EvidenceHistoryArgs {
    /// Optional baseline name filter.
    #[arg(long)]
    baseline_name: Option<String>,
    /// Maximum entries to print.
    #[arg(long, default_value_t = 10)]
    limit: usize,
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
        Commands::Evidence { command } => match command {
            EvidenceCommands::Diff(args) => evidence_diff(args)?,
            EvidenceCommands::Gate(args) => evidence_gate(args)?,
            EvidenceCommands::Explain(args) => evidence_explain(args)?,
            EvidenceCommands::Baseline { command } => match command {
                EvidenceBaselineCommands::List => evidence_baseline_list()?,
                EvidenceBaselineCommands::Show(args) => evidence_baseline_show(args)?,
                EvidenceBaselineCommands::Promote(args) => evidence_baseline_promote(args)?,
            },
            EvidenceCommands::Policy { command } => match command {
                EvidencePolicyCommands::List => evidence_policy_list()?,
                EvidencePolicyCommands::Show(args) => evidence_policy_show(args)?,
            },
            EvidenceCommands::Promote(args) => evidence_promote(args)?,
            EvidenceCommands::History(args) => evidence_history(args)?,
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
        "- Current scope: Codex dual-source + Claude session-path second connector + shared Markdown/JSON/HTML export + local archive index + semantic retrieval + persistent local semantic index + hybrid retrieval + local multi-agent archive shell + retrieval report artifacts + workspace-local transcript backlinks + local reports shell + reports-shell metadata search + integration pack + minimal stdio MCP bridge + repo-owned integration materializer + integration doctor + platform-aware integration doctor diagnostics + integration pack-shape hardening + integration onboarding experience + forbidden-target onboarding guardrails + integration evidence pack reports + integration evidence shell search + machine-readable integration evidence + integration evidence timeline/diff + evidence gate/explain + baseline registry + policy packs + decision promotion/history + read-only governance MCP surface + local decision governance desk."
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
        "- Real decision desk path: generated by `agent-exporter publish archive-index --workspace-root <repo>` into `.agents/Conversations/index.html`, using saved integration evidence plus baseline/policy/history governance artifacts as a read-only decision panel."
    );
    println!(
        "- Real semantic retrieval path: `agent-exporter search semantic --workspace-root <repo> --query <text>`."
    );
    println!(
        "- Real hybrid retrieval path: `agent-exporter search hybrid --workspace-root <repo> --query <text>`."
    );
    println!(
        "- Real MCP bridge path: `python3 scripts/agent_exporter_mcp.py` with local stdio tool exposure for publish/search/evidence/governance read-only workflows."
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
        "- Real evidence diff path: `agent-exporter evidence diff --left <report.json|html> --right <report.json|html>`."
    );
    println!(
        "- Real evidence gate path: `agent-exporter evidence gate --baseline <report.json|html> --candidate <report.json|html>`."
    );
    println!(
        "- Real evidence explain path: `agent-exporter evidence explain --report <report.json|html>`."
    );
    println!(
        "- Real baseline registry path: `agent-exporter evidence baseline list|show|promote` backed by `.agents/Integration/Reports/baseline-registry.json`."
    );
    println!(
        "- Real policy pack path: `agent-exporter evidence policy list|show` backed by `policies/integration-evidence/*.json`."
    );
    println!(
        "- Real decision promotion path: `agent-exporter evidence promote --candidate <report> --baseline-name <name>`."
    );
    println!(
        "- Real decision history path: `agent-exporter evidence history` backed by `.agents/Integration/Reports/decision-history.json`."
    );
    println!(
        "- Next step: a new post-Phase-31 product decision, while staying local-first and non-hosted."
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
    let integration_reports = collect_integration_report_entries(&args.workspace_root)?;
    let integration_json_reports = collect_integration_report_json_documents(&args.workspace_root)?;
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
        &integration_reports,
        build_decision_desk_summary(&args.workspace_root, &integration_json_reports).as_ref(),
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
    println!("- Evidence     : {}", integration_reports.len());
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
    if args.explain {
        let explain_steps = build_integration_evidence_explain(
            outcome.platform.as_str(),
            &outcome.target_root.display().to_string(),
            &doctor_check_records(&outcome.checks),
            &next_steps,
        );
        print_integration_explain_steps(&explain_steps);
    }
    if let Some(bundle) = &saved_report {
        println!("- Report       : {}", bundle.html_report.display());
        println!("- Report JSON  : {}", bundle.json_report.display());
        println!("- Reports Index: {}", bundle.index_html.display());
        println!("- Reports JSON : {}", bundle.index_json.display());
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
    if let Some(bundle) = &saved_report {
        println!("- Report       : {}", bundle.html_report.display());
        println!("- Report JSON  : {}", bundle.json_report.display());
        println!("- Reports Index: {}", bundle.index_html.display());
        println!("- Reports JSON : {}", bundle.index_json.display());
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
            integration_shell_href: "../Integration/Reports/index.html".to_string(),
        }),
    }
}

fn integration_reports_workspace_root() -> Result<PathBuf> {
    std::env::current_dir()
        .context("failed to resolve current working directory for integration reports")
}

fn resolve_baseline_report_reference(
    workspace_root: &std::path::Path,
    reference: &str,
) -> Result<(PathBuf, Option<IntegrationBaselineRecord>)> {
    let candidate_path = PathBuf::from(reference);
    if candidate_path.exists()
        || candidate_path.is_absolute()
        || reference.contains(std::path::MAIN_SEPARATOR)
        || reference.ends_with(".json")
        || reference.ends_with(".html")
    {
        return Ok((canonical_report_json_path(&candidate_path)?, None));
    }

    let registry = read_integration_baseline_registry_document(workspace_root)?;
    let record = find_integration_baseline_record(&registry, reference)
        .cloned()
        .with_context(|| {
            format!(
                "failed to resolve baseline `{reference}` from `{}`",
                resolve_integration_baseline_registry_path(workspace_root).display()
            )
        })?;

    Ok((PathBuf::from(&record.report_json_path), Some(record)))
}

fn build_baseline_registry_record(
    workspace_root: &std::path::Path,
    name: &str,
    report: &IntegrationReportJsonDocument,
    report_json_path: &std::path::Path,
    policy: &IntegrationEvidencePolicyPack,
    promoted_at: &str,
    promoted_from_verdict: &str,
    note: Option<String>,
) -> IntegrationBaselineRecord {
    build_baseline_record(
        workspace_root,
        name,
        report_json_path,
        report,
        promoted_at,
        promoted_from_verdict,
        &policy.gate,
        note,
    )
}

fn integration_reports_title(workspace_root: &std::path::Path) -> String {
    workspace_root
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| format!("{name} integration reports"))
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "agent-exporter integration reports".to_string())
}

fn decision_desk_report_json_path(
    workspace_root: &std::path::Path,
    report: &IntegrationReportJsonDocument,
) -> PathBuf {
    resolve_integration_reports_dir(workspace_root).join(&report.artifact_links.json_report)
}

fn build_decision_desk_summary(
    workspace_root: &std::path::Path,
    reports: &[IntegrationReportJsonDocument],
) -> Option<archive_index_output::DecisionDeskSummary> {
    let candidate = reports.first()?;
    let registry = read_integration_baseline_registry_document(workspace_root).ok()?;
    let history = read_integration_decision_history_document(workspace_root).ok()?;
    let baseline_record =
        find_integration_baseline_for_identity(&registry, &candidate.platform, &candidate.target);
    let baseline = baseline_record.and_then(|record| {
        read_integration_report_json_document(PathBuf::from(&record.report_json_path).as_path())
            .ok()
    });
    let policy_reference = baseline_record.map(|record| record.policy_name.clone());
    let (_, policy_pack) =
        resolve_integration_evidence_policy_pack(policy_reference.as_deref()).ok()?;
    let gate_policy = effective_gate_policy_for_platform(&policy_pack, &candidate.platform);
    let gate = baseline
        .as_ref()
        .and_then(|baseline| gate_integration_reports(baseline, candidate, &gate_policy).ok());
    let promotion = if let Some(outcome) = gate.as_ref() {
        let candidate_path = decision_desk_report_json_path(workspace_root, candidate)
            .canonicalize()
            .ok()
            .map(|path| path.display().to_string());
        if let Some(candidate_path) = candidate_path.as_deref() {
            if let Some(record) =
                latest_integration_decision_for_candidate(&history, candidate_path)
            {
                archive_index_output::DecisionDeskPromotionSummary {
                    state: if record.promoted {
                        "promoted".to_string()
                    } else {
                        "not-promoted".to_string()
                    },
                    summary: record.summary.clone(),
                }
            } else {
                let assessment = assess_promotion_eligibility(outcome, candidate, &policy_pack);
                archive_index_output::DecisionDeskPromotionSummary {
                    state: if assessment.eligible {
                        "not-promoted".to_string()
                    } else {
                        "ineligible".to_string()
                    },
                    summary: assessment.summary,
                }
            }
        } else {
            archive_index_output::DecisionDeskPromotionSummary {
                state: "insufficient".to_string(),
                summary:
                    "candidate report path could not be resolved for decision-history matching"
                        .to_string(),
            }
        }
    } else {
        archive_index_output::DecisionDeskPromotionSummary {
            state: "insufficient".to_string(),
            summary: "save an official baseline for this platform/target before expecting promotion status".to_string(),
        }
    };
    let recent_history = history
        .decisions
        .iter()
        .filter(|entry| entry.platform == candidate.platform && entry.target == candidate.target)
        .take(5)
        .map(|entry| archive_index_output::DecisionDeskHistoryEntry {
            decided_at: entry.decided_at.clone(),
            baseline_name: entry.baseline_name.clone(),
            verdict: entry.verdict.clone(),
            promoted: entry.promoted,
            summary: entry.summary.clone(),
        })
        .collect::<Vec<_>>();

    Some(archive_index_output::DecisionDeskSummary {
        evidence_report_count: reports.len(),
        evidence_shell_href: "../Integration/Reports/index.html".to_string(),
        baseline_name: baseline_record.map(|record| record.name.clone()),
        baseline: baseline
            .as_ref()
            .map(|report| decision_desk_snapshot_from_report(report)),
        candidate: Some(decision_desk_snapshot_from_report(candidate)),
        active_policy: archive_index_output::DecisionDeskPolicySummary {
            name: policy_pack.name.clone(),
            version: policy_pack.version.clone(),
        },
        promotion,
        history: recent_history,
        gate,
    })
}

fn decision_desk_snapshot_from_report(
    report: &IntegrationReportJsonDocument,
) -> archive_index_output::DecisionDeskSnapshot {
    archive_index_output::DecisionDeskSnapshot {
        title: report.title.clone(),
        kind: report.kind.clone(),
        platform: report.platform.clone(),
        readiness: report.readiness.clone(),
        target: report.target.clone(),
        generated_at: report.generated_at.clone(),
        html_href: format!(
            "../Integration/Reports/{}",
            report.artifact_links.html_report
        ),
    }
}

fn write_doctor_integration_report(
    workspace_root: &std::path::Path,
    outcome: &crate::integrations::IntegrationDoctorOutcome,
    next_steps: &[String],
) -> Result<IntegrationReportBundlePaths> {
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
) -> Result<IntegrationReportBundlePaths> {
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
) -> Result<IntegrationReportBundlePaths> {
    let document = render_integration_report_document(report);
    let html_report = write_integration_report_document(
        workspace_root,
        report.kind.as_str(),
        &report.platform,
        &report.generated_at,
        &document,
    )?;
    let html_report_name = html_report
        .file_name()
        .and_then(|name| name.to_str())
        .map(str::to_string)
        .context("failed to derive integration report html file name")?;
    let json_report_name = html_report_name.replace(".html", ".json");
    let json_report_document =
        build_integration_report_json_document(report, &html_report_name, &json_report_name);
    let json_report = write_integration_report_json_document(
        workspace_root,
        report.kind.as_str(),
        &report.platform,
        &report.generated_at,
        &json_report_document,
    )?;
    let html_entries = collect_integration_report_entries(workspace_root)?;
    let index_document = render_integration_reports_index_document(
        &integration_reports_title(workspace_root),
        &report.generated_at,
        &html_entries,
    );
    let index_html = write_integration_reports_index_document(workspace_root, &index_document)?;
    let json_entries = collect_integration_report_json_documents(workspace_root)?;
    let index_json_document = build_integration_reports_index_json_document(
        &integration_reports_title(workspace_root),
        &report.generated_at,
        &json_entries,
    );
    let index_json =
        write_integration_reports_index_json_document(workspace_root, &index_json_document)?;
    Ok(IntegrationReportBundlePaths {
        html_report,
        json_report,
        index_html,
        index_json,
    })
}

#[derive(Debug)]
struct IntegrationReportBundlePaths {
    html_report: PathBuf,
    json_report: PathBuf,
    index_html: PathBuf,
    index_json: PathBuf,
}

fn evidence_diff(args: EvidenceDiffArgs) -> Result<()> {
    let left = read_integration_report_json_document(&args.left)?;
    let right = read_integration_report_json_document(&args.right)?;
    let diff = diff_integration_reports(&left, &right);
    print_integration_evidence_diff(&diff, &args.left, &args.right);
    Ok(())
}

fn evidence_gate(args: EvidenceGateArgs) -> Result<()> {
    let workspace_root = integration_reports_workspace_root()?;
    let (baseline_path, baseline_record) =
        resolve_baseline_report_reference(&workspace_root, &args.baseline)?;
    let candidate_path = canonical_report_json_path(&args.candidate)?;
    let baseline = read_integration_report_json_document(&baseline_path)?;
    let candidate = read_integration_report_json_document(&candidate_path)?;
    let policy_reference = args
        .policy
        .clone()
        .or_else(|| baseline_record.map(|record| record.policy_name));
    let (policy_path, policy_pack) =
        resolve_integration_evidence_policy_pack(policy_reference.as_deref())?;
    let gate_policy = effective_gate_policy_for_platform(&policy_pack, &candidate.platform);
    let outcome = gate_integration_reports(&baseline, &candidate, &gate_policy)?;
    print_integration_evidence_gate(
        &outcome,
        &baseline_path.display().to_string(),
        &candidate_path.display().to_string(),
        &policy_path.display().to_string(),
        &policy_pack,
        &gate_policy,
    );
    Ok(())
}

fn evidence_explain(args: EvidenceExplainArgs) -> Result<()> {
    let report = read_integration_report_json_document(&args.report)?;
    let explain_steps = build_integration_evidence_explain(
        &report.platform,
        &report.target,
        &combined_report_checks(&report),
        &report.next_steps,
    );
    println!("Integration evidence explain");
    println!("- Report       : {}", args.report.display());
    println!("- Platform     : {}", report.platform);
    println!("- Target       : {}", report.target);
    println!("- Readiness    : {}", report.readiness);
    print_integration_explain_steps(&explain_steps);
    Ok(())
}

fn evidence_baseline_list() -> Result<()> {
    let workspace_root = integration_reports_workspace_root()?;
    let registry = read_integration_baseline_registry_document(&workspace_root)?;
    println!("Integration baseline registry");
    println!(
        "- Registry     : {}",
        resolve_integration_baseline_registry_path(&workspace_root).display()
    );
    println!("- Baselines    : {}", registry.baselines.len());
    if registry.baselines.is_empty() {
        println!("- Entries      : none");
        return Ok(());
    }

    println!("- Entries");
    for record in &registry.baselines {
        println!(
            "  - {} | {} | {} | {}",
            record.name, record.platform, record.policy_name, record.promoted_at
        );
    }
    Ok(())
}

fn evidence_baseline_show(args: EvidenceBaselineShowArgs) -> Result<()> {
    let workspace_root = integration_reports_workspace_root()?;
    let registry = read_integration_baseline_registry_document(&workspace_root)?;
    let record = find_integration_baseline_record(&registry, &args.name)
        .with_context(|| format!("baseline `{}` is not registered", args.name))?;

    println!("Integration baseline");
    println!("- Name         : {}", record.name);
    println!("- Platform     : {}", record.platform);
    println!("- Target       : {}", record.target);
    println!("- Identity     : {}", record.target_identity);
    println!("- Report       : {}", record.report_json_path);
    println!("- Promoted At  : {}", record.promoted_at);
    println!("- Verdict      : {}", record.promoted_from_verdict);
    println!(
        "- Policy       : {} v{}",
        record.policy_name, record.policy_version
    );
    println!(
        "- Note         : {}",
        record.note.as_deref().unwrap_or("none")
    );
    Ok(())
}

fn evidence_baseline_promote(args: EvidenceBaselinePromoteArgs) -> Result<()> {
    let workspace_root = integration_reports_workspace_root()?;
    let report_path = canonical_report_json_path(&args.report)?;
    let report = read_integration_report_json_document(&report_path)?;
    let promoted_at = Utc::now().to_rfc3339();
    let (policy_path, policy_pack) =
        resolve_integration_evidence_policy_pack(args.policy.as_deref())?;

    let mut registry = read_integration_baseline_registry_document(&workspace_root)?;
    registry.generated_at = promoted_at.clone();
    let record = build_baseline_registry_record(
        &workspace_root,
        &args.name,
        &report,
        &report_path,
        &policy_pack,
        &promoted_at,
        &args.verdict,
        args.note.clone(),
    );
    upsert_integration_baseline_record(&mut registry, record.clone());
    write_integration_baseline_registry_document(&workspace_root, &registry)?;

    let mut history = read_integration_decision_history_document(&workspace_root)?;
    history.generated_at = promoted_at.clone();
    append_integration_decision_record(
        &mut history,
        IntegrationDecisionRecord {
            baseline_name: record.name.clone(),
            platform: record.platform.clone(),
            target: record.target.clone(),
            target_identity: record.target_identity.clone(),
            baseline_report_json_path: None,
            baseline_report_title: None,
            candidate_report_json_path: record.report_json_path.clone(),
            candidate_report_title: record.report_title.clone(),
            policy_name: record.policy_name.clone(),
            policy_version: record.policy_version.clone(),
            verdict: args.verdict.clone(),
            promoted: true,
            decided_at: promoted_at.clone(),
            summary: args.note.clone().unwrap_or_else(|| {
                "official baseline seeded directly from saved report".to_string()
            }),
        },
    );
    write_integration_decision_history_document(&workspace_root, &history)?;

    println!("Integration baseline promoted");
    println!("- Name         : {}", record.name);
    println!("- Platform     : {}", record.platform);
    println!("- Target       : {}", record.target);
    println!("- Report       : {}", record.report_json_path);
    println!("- Verdict      : {}", record.promoted_from_verdict);
    println!(
        "- Policy       : {} v{}",
        policy_pack.name, policy_pack.version
    );
    println!("- Policy Path  : {}", policy_path.display());
    println!("- Promoted At  : {}", promoted_at);
    Ok(())
}

fn evidence_policy_list() -> Result<()> {
    let policies = collect_repo_owned_integration_policy_packs()?;
    println!("Integration governance policies");
    println!(
        "- Directory    : {}",
        repo_owned_integration_policy_dir().display()
    );
    println!("- Policies     : {}", policies.len());
    if policies.is_empty() {
        println!("- Entries      : none");
        return Ok(());
    }

    println!("- Entries");
    for (path, policy) in policies {
        println!(
            "  - {} v{} | {}",
            policy.name,
            policy.version,
            path.display()
        );
    }
    Ok(())
}

fn evidence_policy_show(args: EvidencePolicyShowArgs) -> Result<()> {
    let path = repo_owned_integration_policy_dir().join(format!("{}.json", args.name));
    let policy = resolve_integration_evidence_policy_pack(Some(&args.name))?.1;
    let raw_json = std::fs::read_to_string(&path)
        .with_context(|| format!("failed to read `{}`", path.display()))?;
    println!("Integration governance policy");
    println!("- Name         : {}", policy.name);
    println!("- Version      : {}", policy.version);
    println!("- Path         : {}", path.display());
    println!("- Description  : {}", policy.description);
    println!(
        "- Gate         : {} blocking / {} warning labels",
        policy.gate.blocking_check_labels.len(),
        policy.gate.warning_check_labels.len()
    );
    println!(
        "- Promotion    : verdicts [{}], readiness [{}], non-regression {}",
        policy.promotion.allowed_verdicts.join(", "),
        policy.promotion.allowed_candidate_readiness.join(", "),
        if policy.promotion.require_non_regression {
            "required"
        } else {
            "optional"
        }
    );
    println!("- JSON");
    println!("{}", raw_json.trim_end());
    Ok(())
}

fn evidence_promote(args: EvidencePromoteArgs) -> Result<()> {
    let workspace_root = integration_reports_workspace_root()?;
    let candidate_path = canonical_report_json_path(&args.candidate)?;
    let candidate = read_integration_report_json_document(&candidate_path)?;
    let mut registry = read_integration_baseline_registry_document(&workspace_root)?;
    let baseline_record = find_integration_baseline_record(&registry, &args.baseline_name)
        .cloned()
        .with_context(|| format!("baseline `{}` is not registered", args.baseline_name))?;
    let baseline = read_integration_report_json_document(
        PathBuf::from(&baseline_record.report_json_path).as_path(),
    )?;
    let policy_reference = args
        .policy
        .clone()
        .or_else(|| Some(baseline_record.policy_name.clone()));
    let (policy_path, policy_pack) =
        resolve_integration_evidence_policy_pack(policy_reference.as_deref())?;
    let gate_policy = effective_gate_policy_for_platform(&policy_pack, &candidate.platform);
    let outcome = gate_integration_reports(&baseline, &candidate, &gate_policy)?;
    let assessment = assess_promotion_eligibility(&outcome, &candidate, &policy_pack);
    let decided_at = Utc::now().to_rfc3339();

    let summary = args
        .note
        .clone()
        .unwrap_or_else(|| assessment.summary.clone());
    let mut history = read_integration_decision_history_document(&workspace_root)?;
    history.generated_at = decided_at.clone();
    append_integration_decision_record(
        &mut history,
        IntegrationDecisionRecord {
            baseline_name: baseline_record.name.clone(),
            platform: candidate.platform.clone(),
            target: candidate.target.clone(),
            target_identity: integration_target_identity(&candidate.platform, &candidate.target),
            baseline_report_json_path: Some(baseline_record.report_json_path.clone()),
            baseline_report_title: Some(baseline_record.report_title.clone()),
            candidate_report_json_path: candidate_path.display().to_string(),
            candidate_report_title: candidate.title.clone(),
            policy_name: policy_pack.name.clone(),
            policy_version: policy_pack.version.clone(),
            verdict: outcome.verdict.as_str().to_string(),
            promoted: assessment.eligible,
            decided_at: decided_at.clone(),
            summary: summary.clone(),
        },
    );
    write_integration_decision_history_document(&workspace_root, &history)?;

    if assessment.eligible {
        registry.generated_at = decided_at.clone();
        upsert_integration_baseline_record(
            &mut registry,
            build_baseline_registry_record(
                &workspace_root,
                &baseline_record.name,
                &candidate,
                &candidate_path,
                &policy_pack,
                &decided_at,
                outcome.verdict.as_str(),
                args.note.clone(),
            ),
        );
        write_integration_baseline_registry_document(&workspace_root, &registry)?;
    }

    println!("Integration evidence promote");
    println!("- Baseline     : {}", baseline_record.name);
    println!("- Candidate    : {}", candidate_path.display());
    println!("- Verdict      : {}", outcome.verdict.as_str());
    println!(
        "- Promoted     : {}",
        if assessment.eligible { "yes" } else { "no" }
    );
    println!(
        "- Policy       : {} v{}",
        policy_pack.name, policy_pack.version
    );
    println!("- Policy Path  : {}", policy_path.display());
    println!("- Summary      : {}", summary);
    if !assessment.reasons.is_empty() {
        println!("- Reasons");
        for reason in &assessment.reasons {
            println!("  - {}", reason);
        }
    }
    print_integration_evidence_gate(
        &outcome,
        &baseline_record.report_json_path,
        &candidate_path.display().to_string(),
        &policy_path.display().to_string(),
        &policy_pack,
        &gate_policy,
    );
    Ok(())
}

fn evidence_history(args: EvidenceHistoryArgs) -> Result<()> {
    let workspace_root = integration_reports_workspace_root()?;
    let history = read_integration_decision_history_document(&workspace_root)?;
    println!("Integration decision history");
    println!(
        "- History      : {}",
        resolve_integration_decision_history_path(&workspace_root).display()
    );

    let entries = history
        .decisions
        .iter()
        .filter(|entry| {
            args.baseline_name
                .as_deref()
                .is_none_or(|name| entry.baseline_name == name)
        })
        .take(args.limit)
        .collect::<Vec<_>>();

    println!("- Decisions    : {}", entries.len());
    if entries.is_empty() {
        println!("- Entries      : none");
        return Ok(());
    }

    println!("- Entries");
    for entry in entries {
        println!(
            "  - {} | {} | verdict {} | promoted {} | {}",
            entry.decided_at,
            entry.baseline_name,
            entry.verdict,
            if entry.promoted { "yes" } else { "no" },
            entry.summary
        );
    }
    Ok(())
}

fn print_integration_evidence_diff(
    diff: &IntegrationEvidenceDiff,
    left_path: &std::path::Path,
    right_path: &std::path::Path,
) {
    println!("Integration evidence diff");
    println!("- Platform     : {}", diff.platform);
    println!("- Target       : {}", diff.target);
    println!("- Left         : {}", left_path.display());
    println!("- Right        : {}", right_path.display());
    println!(
        "- Readiness    : {} -> {}",
        diff.left_readiness, diff.right_readiness
    );
    println!("- Left Stamp   : {}", diff.left_generated_at);
    println!("- Right Stamp  : {}", diff.right_generated_at);

    if diff.changed_checks.is_empty() {
        println!("- Changed Checks: none");
    } else {
        println!("- Changed Checks");
        for check in &diff.changed_checks {
            println!(
                "  - {}: {} -> {}",
                check.label,
                check.left_readiness.as_deref().unwrap_or("missing"),
                check.right_readiness.as_deref().unwrap_or("missing")
            );
        }
    }

    if !diff.added_next_steps.is_empty() {
        println!("- Added Next Steps");
        for step in &diff.added_next_steps {
            println!("  - {}", step);
        }
    }
    if !diff.removed_next_steps.is_empty() {
        println!("- Removed Next Steps");
        for step in &diff.removed_next_steps {
            println!("  - {}", step);
        }
    }
}

fn print_integration_evidence_gate(
    outcome: &IntegrationEvidenceGateOutcome,
    baseline: &str,
    candidate: &str,
    policy_path: &str,
    policy_pack: &IntegrationEvidencePolicyPack,
    policy: &IntegrationEvidenceGatePolicy,
) {
    println!("Integration evidence gate");
    println!("- Baseline     : {}", baseline);
    println!("- Candidate    : {}", candidate);
    println!("- Verdict      : {}", outcome.verdict.as_str());
    println!(
        "- Readiness    : {} -> {}",
        outcome.baseline_readiness, outcome.candidate_readiness
    );
    println!(
        "- Regression   : {}",
        if outcome.regression { "yes" } else { "no" }
    );
    println!(
        "- Policy       : {} v{}",
        policy_pack.name, policy_pack.version
    );
    println!("- Policy Path  : {}", policy_path);
    println!(
        "- Policy Rules : {} blocking labels / {} warning labels / {} ignorable labels",
        policy.blocking_check_labels.len(),
        policy.warning_check_labels.len(),
        policy.ignorable_check_labels.len()
    );

    if !outcome.blocking_changes.is_empty() {
        println!("- Blocking Changes");
        for change in &outcome.blocking_changes {
            println!(
                "  - {}: {} -> {}",
                change.label,
                change.left_readiness.as_deref().unwrap_or("missing"),
                change.right_readiness.as_deref().unwrap_or("missing")
            );
        }
    }
    if !outcome.warning_changes.is_empty() {
        println!("- Warning Changes");
        for change in &outcome.warning_changes {
            println!(
                "  - {}: {} -> {}",
                change.label,
                change.left_readiness.as_deref().unwrap_or("missing"),
                change.right_readiness.as_deref().unwrap_or("missing")
            );
        }
    }
    if !outcome.ignored_changes.is_empty() {
        println!("- Ignorable Changes");
        for change in &outcome.ignored_changes {
            println!("  - {}", change.label);
        }
    }
    print_integration_explain_steps(&outcome.remediation_steps);
}

fn print_integration_explain_steps(
    steps: &[crate::core::integration_report::IntegrationEvidenceExplainStep],
) {
    if steps.is_empty() {
        println!("- Remediation  : none");
        return;
    }

    println!("- Remediation");
    for (index, step) in steps.iter().enumerate() {
        println!("  {}. {}", index + 1, step.title);
        println!("     why     : {}", step.why);
        println!("     recheck : {}", step.recheck);
    }
}

fn doctor_check_records(
    checks: &[crate::integrations::IntegrationDoctorCheck],
) -> Vec<IntegrationReportCheckRecord> {
    checks
        .iter()
        .map(|check| IntegrationReportCheckRecord {
            label: check.label.to_string(),
            readiness: check.readiness.as_str().to_string(),
            detail: check.detail.clone(),
        })
        .collect()
}

fn combined_report_checks(
    report: &crate::core::integration_report::IntegrationReportJsonDocument,
) -> Vec<IntegrationReportCheckRecord> {
    let mut checks = report.pack_shape_checks.clone();
    checks.extend(report.checks.clone());
    checks
}
