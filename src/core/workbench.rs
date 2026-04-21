use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::core::archive_index::{ArchiveIndexEntry, resolve_workspace_conversations_dir};
use crate::core::integration_report::{IntegrationReportJsonDocument, integration_target_identity};
use crate::core::search_report::SearchReportEntry;
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

const WORKBENCH_SCHEMA_VERSION: u32 = 1;
const PIN_REGISTRY_SCHEMA_VERSION: u32 = 1;
pub(crate) const REFRESH_MANIFEST_SCHEMA_VERSION: u32 = 1;
const FLEET_VIEW_SCHEMA_VERSION: u32 = 1;
const ACTION_PACKS_SCHEMA_VERSION: u32 = 1;
const MEMORY_LANE_SCHEMA_VERSION: u32 = 1;

fn default_pin_status() -> String {
    "active".to_string()
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StructuredSummaryOutputFiles {
    pub markdown: String,
    pub html: String,
    pub json: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StructuredSummaryDocument {
    pub schema_version: u32,
    pub thread_id: String,
    pub connector: String,
    pub source_kind: String,
    pub completeness: String,
    pub generated_at: String,
    pub profile_id: String,
    pub runtime_profile: Option<String>,
    pub runtime_model: Option<String>,
    pub runtime_provider: Option<String>,
    pub family_key: String,
    pub title: String,
    pub overview: String,
    pub share_safe_summary: String,
    pub goals: Vec<String>,
    pub files_touched: Vec<String>,
    pub tests_run: Vec<String>,
    pub risks: Vec<String>,
    pub blockers: Vec<String>,
    pub next_steps: Vec<String>,
    pub citations: Vec<String>,
    pub extraction_mode: String,
    pub output_files: StructuredSummaryOutputFiles,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StructuredSummaryIndexEntry {
    pub relative_href: String,
    pub artifact_json_path: String,
    pub document: StructuredSummaryDocument,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceTimelineEvent {
    pub id: String,
    pub kind: String,
    pub title: String,
    pub occurred_at: String,
    pub href: Option<String>,
    pub connector: Option<String>,
    pub relation_key: Option<String>,
    pub summary: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreadFamilyLatestSummary {
    pub title: String,
    pub href: String,
    pub json_href: String,
    pub generated_at: String,
    pub profile_id: String,
    pub overview: String,
    pub share_safe_summary: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreadFamilyOfficialAnswer {
    pub label: String,
    pub status: String,
    pub href: String,
    pub summary: String,
    pub pinned_at: String,
    pub stale: bool,
    pub note: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreadFamilyDiffSummary {
    pub status: String,
    pub summary: String,
    pub new_files_touched: Vec<String>,
    pub new_tests_run: Vec<String>,
    pub new_risks: Vec<String>,
    pub new_blockers: Vec<String>,
    pub new_next_steps: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreadFamilySummary {
    pub family_key: String,
    pub label: String,
    pub latest_at: String,
    pub transcript_count: usize,
    pub summary_count: usize,
    pub latest_href: Option<String>,
    pub connectors: Vec<String>,
    pub profiles: Vec<String>,
    pub files_touched: Vec<String>,
    pub tests_run: Vec<String>,
    pub risks: Vec<String>,
    pub blockers: Vec<String>,
    pub next_steps: Vec<String>,
    pub citations: Vec<String>,
    pub latest_summary: Option<ThreadFamilyLatestSummary>,
    pub official_answer: Option<ThreadFamilyOfficialAnswer>,
    pub latest_vs_pinned: Option<ThreadFamilyDiffSummary>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FleetHistoryEntry {
    pub generated_at: String,
    pub readiness: String,
    pub kind: String,
    pub summary: String,
    pub html_href: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FleetViewEntry {
    pub relation_key: String,
    pub platform: String,
    pub target: String,
    pub latest_readiness: String,
    pub latest_generated_at: String,
    pub report_count: usize,
    pub html_href: Option<String>,
    pub latest_summary: String,
    pub next_steps: Vec<String>,
    pub history: Vec<FleetHistoryEntry>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionPackRecord {
    pub id: String,
    pub kind: String,
    pub title: String,
    pub summary: String,
    pub relation_key: Option<String>,
    pub source_kind: String,
    pub markdown_href: String,
    pub json_href: String,
    pub steps: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionPackIndexDocument {
    pub schema_version: u32,
    pub title: String,
    pub generated_at: String,
    pub packs: Vec<ActionPackRecord>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamMemoryWorkspaceEntry {
    pub workspace_name: String,
    pub workspace_root: String,
    pub archive_title: String,
    pub generated_at: String,
    pub family_count: usize,
    pub official_answer_count: usize,
    pub fleet_count: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamMemoryRoleView {
    pub role: String,
    pub summary: String,
    pub items: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamMemoryLaneDocument {
    pub schema_version: u32,
    pub title: String,
    pub generated_at: String,
    pub workspaces: Vec<TeamMemoryWorkspaceEntry>,
    pub role_views: Vec<TeamMemoryRoleView>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkbenchRefreshManifestDocument {
    pub schema_version: u32,
    pub generated_at: String,
    pub refresh_mode: String,
    pub input_fingerprint: String,
    pub changed_families: Vec<String>,
    pub changed_fleet_relations: Vec<String>,
    pub stale_pin_labels: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OfficialAnswerPinRecord {
    pub label: String,
    pub artifact_kind: String,
    pub artifact_json_path: String,
    pub artifact_href: String,
    pub title: String,
    pub summary: String,
    pub generated_at: String,
    pub pinned_at: String,
    pub note: Option<String>,
    pub relation_key: String,
    #[serde(default = "default_pin_status")]
    pub status: String,
    #[serde(default)]
    pub resolved_at: Option<String>,
    #[serde(default)]
    pub superseded_at: Option<String>,
    #[serde(default)]
    pub superseded_by: Option<String>,
    #[serde(default)]
    pub supersedes: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OfficialAnswerPinRegistryDocument {
    pub schema_version: u32,
    pub generated_at: String,
    pub pins: Vec<OfficialAnswerPinRecord>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OfficialAnswerPinView {
    pub label: String,
    pub artifact_kind: String,
    pub artifact_json_path: String,
    pub title: String,
    pub summary: String,
    pub href: String,
    pub generated_at: String,
    pub pinned_at: String,
    pub note: Option<String>,
    pub relation_key: String,
    pub status: String,
    pub resolved_at: Option<String>,
    pub superseded_at: Option<String>,
    pub superseded_by: Option<String>,
    pub supersedes: Option<String>,
    pub stale: bool,
    pub stale_reason: Option<String>,
    pub lifecycle_summary: Option<String>,
    pub latest_related_at: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkbenchIndexJsonDocument {
    pub schema_version: u32,
    pub title: String,
    pub generated_at: String,
    pub timeline: Vec<WorkspaceTimelineEvent>,
    pub families: Vec<ThreadFamilySummary>,
    pub official_answers: Vec<OfficialAnswerPinView>,
    pub fleet_view: Vec<FleetViewEntry>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShareSafePacketVariant {
    Teammate,
    Reviewer,
    Public,
}

impl ShareSafePacketVariant {
    fn file_name(self) -> &'static str {
        match self {
            Self::Teammate => "share-safe-packet.teammate.md",
            Self::Reviewer => "share-safe-packet.reviewer.md",
            Self::Public => "share-safe-packet.public.md",
        }
    }

    fn heading(self) -> &'static str {
        match self {
            Self::Teammate => "teammate-safe workbench packet",
            Self::Reviewer => "Share-safe workbench packet",
            Self::Public => "public-safe workbench packet",
        }
    }
}

pub struct OfficialAnswerPinRequest<'a> {
    pub workspace_root: &'a Path,
    pub artifact_path: &'a Path,
    pub label: &'a str,
    pub note: Option<String>,
    pub pinned_at: &'a str,
}

pub fn summary_family_key(thread_id: &str) -> String {
    format!("thread:{thread_id}")
}

pub fn resolve_archive_index_json_path(workspace_root: &Path) -> Result<PathBuf> {
    Ok(resolve_workspace_conversations_dir(workspace_root)?.join("index.json"))
}

pub fn resolve_share_safe_packet_path(workspace_root: &Path) -> Result<PathBuf> {
    Ok(resolve_workspace_conversations_dir(workspace_root)?.join("share-safe-packet.md"))
}

pub fn resolve_share_safe_packet_variant_path(
    workspace_root: &Path,
    variant: ShareSafePacketVariant,
) -> Result<PathBuf> {
    Ok(resolve_workspace_conversations_dir(workspace_root)?.join(variant.file_name()))
}

pub fn resolve_official_answer_pins_path(workspace_root: &Path) -> Result<PathBuf> {
    Ok(resolve_workspace_conversations_dir(workspace_root)?.join("official-answer-pins.json"))
}

pub fn resolve_refresh_manifest_path(workspace_root: &Path) -> Result<PathBuf> {
    Ok(resolve_workspace_conversations_dir(workspace_root)?.join("refresh-manifest.json"))
}

pub fn resolve_fleet_view_html_path(workspace_root: &Path) -> Result<PathBuf> {
    Ok(resolve_workspace_conversations_dir(workspace_root)?.join("fleet-view.html"))
}

pub fn resolve_fleet_view_json_path(workspace_root: &Path) -> Result<PathBuf> {
    Ok(resolve_workspace_conversations_dir(workspace_root)?.join("fleet-view.json"))
}

pub fn resolve_action_packs_dir(workspace_root: &Path) -> Result<PathBuf> {
    Ok(resolve_workspace_conversations_dir(workspace_root)?.join("action-packs"))
}

pub fn resolve_action_packs_index_html_path(workspace_root: &Path) -> Result<PathBuf> {
    Ok(resolve_action_packs_dir(workspace_root)?.join("index.html"))
}

pub fn resolve_action_packs_index_json_path(workspace_root: &Path) -> Result<PathBuf> {
    Ok(resolve_action_packs_dir(workspace_root)?.join("index.json"))
}

pub fn resolve_memory_lane_html_path(workspace_root: &Path) -> Result<PathBuf> {
    Ok(resolve_workspace_conversations_dir(workspace_root)?.join("memory-lane.html"))
}

pub fn resolve_memory_lane_json_path(workspace_root: &Path) -> Result<PathBuf> {
    Ok(resolve_workspace_conversations_dir(workspace_root)?.join("memory-lane.json"))
}

pub fn write_archive_index_json_document(
    workspace_root: &Path,
    document: &WorkbenchIndexJsonDocument,
) -> Result<PathBuf> {
    let path = resolve_archive_index_json_path(workspace_root)?;
    let rendered =
        serde_json::to_string_pretty(document).context("failed to render archive index json")?;
    fs::write(&path, format!("{rendered}\n"))
        .with_context(|| format!("failed to write archive index json `{}`", path.display()))?;
    Ok(path)
}

pub fn write_refresh_manifest_document(
    workspace_root: &Path,
    document: &WorkbenchRefreshManifestDocument,
) -> Result<PathBuf> {
    let path = resolve_refresh_manifest_path(workspace_root)?;
    let rendered =
        serde_json::to_string_pretty(document).context("failed to render refresh manifest json")?;
    fs::write(&path, format!("{rendered}\n"))
        .with_context(|| format!("failed to write refresh manifest `{}`", path.display()))?;
    Ok(path)
}

pub fn read_refresh_manifest_document(
    workspace_root: &Path,
) -> Result<Option<WorkbenchRefreshManifestDocument>> {
    let path = resolve_refresh_manifest_path(workspace_root)?;
    if !path.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(&path)
        .with_context(|| format!("failed to read refresh manifest `{}`", path.display()))?;
    let document = serde_json::from_str(&content)
        .with_context(|| format!("failed to parse refresh manifest `{}`", path.display()))?;
    Ok(Some(document))
}

pub fn write_fleet_view_json_document(
    workspace_root: &Path,
    entries: &[FleetViewEntry],
) -> Result<PathBuf> {
    let path = resolve_fleet_view_json_path(workspace_root)?;
    let rendered = serde_json::to_string_pretty(&serde_json::json!({
        "schemaVersion": FLEET_VIEW_SCHEMA_VERSION,
        "entries": entries,
    }))
    .context("failed to render fleet view json")?;
    fs::write(&path, format!("{rendered}\n"))
        .with_context(|| format!("failed to write fleet view json `{}`", path.display()))?;
    Ok(path)
}

pub fn write_action_packs_index_json_document(
    workspace_root: &Path,
    document: &ActionPackIndexDocument,
) -> Result<PathBuf> {
    let path = resolve_action_packs_index_json_path(workspace_root)?;
    let rendered =
        serde_json::to_string_pretty(document).context("failed to render action pack json")?;
    fs::create_dir_all(
        path.parent()
            .with_context(|| format!("missing parent for `{}`", path.display()))?,
    )
    .with_context(|| format!("failed to prepare action pack dir for `{}`", path.display()))?;
    fs::write(&path, format!("{rendered}\n"))
        .with_context(|| format!("failed to write action pack index `{}`", path.display()))?;
    Ok(path)
}

pub fn write_action_pack_documents(
    workspace_root: &Path,
    packs: &[ActionPackRecord],
) -> Result<()> {
    let dir = resolve_action_packs_dir(workspace_root)?;
    fs::create_dir_all(&dir)
        .with_context(|| format!("failed to prepare action pack dir `{}`", dir.display()))?;
    for pack in packs {
        let markdown_path = dir.join(&pack.markdown_href);
        let json_path = dir.join(&pack.json_href);
        let markdown = format!(
            "# {}\n\n- Kind: `{}`\n- Source: `{}`\n- Relation: `{}`\n\n{}\n\n## Next actions\n{}\n",
            pack.title,
            pack.kind,
            pack.source_kind,
            pack.relation_key.as_deref().unwrap_or("n/a"),
            pack.summary,
            if pack.steps.is_empty() {
                "- No explicit next actions.\n".to_string()
            } else {
                pack.steps
                    .iter()
                    .map(|step| format!("- {step}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        );
        fs::write(&markdown_path, markdown).with_context(|| {
            format!(
                "failed to write action pack markdown `{}`",
                markdown_path.display()
            )
        })?;
        let rendered = serde_json::to_string_pretty(pack)
            .context("failed to render action pack document json")?;
        fs::write(&json_path, format!("{rendered}\n")).with_context(|| {
            format!("failed to write action pack json `{}`", json_path.display())
        })?;
    }
    Ok(())
}

pub fn write_memory_lane_json_document(
    workspace_root: &Path,
    document: &TeamMemoryLaneDocument,
) -> Result<PathBuf> {
    let path = resolve_memory_lane_json_path(workspace_root)?;
    let rendered =
        serde_json::to_string_pretty(document).context("failed to render memory lane json")?;
    fs::write(&path, format!("{rendered}\n"))
        .with_context(|| format!("failed to write memory lane json `{}`", path.display()))?;
    Ok(path)
}

pub fn write_share_safe_packet_document(workspace_root: &Path, document: &str) -> Result<PathBuf> {
    let path = resolve_share_safe_packet_path(workspace_root)?;
    fs::write(&path, format!("{}\n", document.trim_end()))
        .with_context(|| format!("failed to write share-safe packet `{}`", path.display()))?;
    Ok(path)
}

pub fn write_share_safe_packet_variant_document(
    workspace_root: &Path,
    variant: ShareSafePacketVariant,
    document: &str,
) -> Result<PathBuf> {
    let path = resolve_share_safe_packet_variant_path(workspace_root, variant)?;
    fs::write(&path, format!("{}\n", document.trim_end()))
        .with_context(|| format!("failed to write share-safe packet `{}`", path.display()))?;
    Ok(path)
}

pub fn read_official_answer_pin_registry_document(
    workspace_root: &Path,
) -> Result<OfficialAnswerPinRegistryDocument> {
    let path = resolve_official_answer_pins_path(workspace_root)?;
    if !path.exists() {
        return Ok(OfficialAnswerPinRegistryDocument {
            schema_version: PIN_REGISTRY_SCHEMA_VERSION,
            generated_at: String::new(),
            pins: Vec::new(),
        });
    }

    let content = fs::read_to_string(&path)
        .with_context(|| format!("failed to read official pins `{}`", path.display()))?;
    serde_json::from_str(&content)
        .with_context(|| format!("failed to parse official pins `{}`", path.display()))
}

pub fn write_official_answer_pin_registry_document(
    workspace_root: &Path,
    document: &OfficialAnswerPinRegistryDocument,
) -> Result<PathBuf> {
    let path = resolve_official_answer_pins_path(workspace_root)?;
    let rendered =
        serde_json::to_string_pretty(document).context("failed to render official pins json")?;
    fs::write(&path, format!("{rendered}\n"))
        .with_context(|| format!("failed to write official pins `{}`", path.display()))?;
    Ok(path)
}

pub fn upsert_official_answer_pin(
    workspace_root: &Path,
    mut pin: OfficialAnswerPinRecord,
    generated_at: &str,
    supersedes: Option<&str>,
) -> Result<PathBuf> {
    let mut registry = read_official_answer_pin_registry_document(workspace_root)?;
    registry.generated_at = generated_at.to_string();
    if let Some(previous_label) = supersedes {
        if let Some(previous) = registry
            .pins
            .iter_mut()
            .find(|existing| existing.label == previous_label)
        {
            previous.status = "superseded".to_string();
            previous.superseded_by = Some(pin.label.clone());
            previous.superseded_at = Some(generated_at.to_string());
        }
        pin.supersedes = Some(previous_label.to_string());
    }
    registry.pins.retain(|existing| existing.label != pin.label);
    registry.pins.push(pin);
    registry.pins.sort_by(|left, right| {
        right
            .pinned_at
            .cmp(&left.pinned_at)
            .then_with(|| left.label.cmp(&right.label))
    });
    write_official_answer_pin_registry_document(workspace_root, &registry)
}

pub fn resolve_official_answer_pin(
    workspace_root: &Path,
    label: &str,
    note: Option<String>,
    resolved_at: &str,
) -> Result<OfficialAnswerPinRecord> {
    let mut registry = read_official_answer_pin_registry_document(workspace_root)?;
    registry.generated_at = resolved_at.to_string();
    let pin = registry
        .pins
        .iter_mut()
        .find(|existing| existing.label == label)
        .with_context(|| format!("official answer `{label}` was not found"))?;
    pin.status = "resolved".to_string();
    pin.resolved_at = Some(resolved_at.to_string());
    if let Some(note) = note {
        pin.note = Some(note);
    }
    let snapshot = pin.clone();
    write_official_answer_pin_registry_document(workspace_root, &registry)?;
    Ok(snapshot)
}

pub fn remove_official_answer_pin(workspace_root: &Path, label: &str) -> Result<bool> {
    let mut registry = read_official_answer_pin_registry_document(workspace_root)?;
    let original_len = registry.pins.len();
    registry.pins.retain(|pin| pin.label != label);
    if registry.pins.len() == original_len {
        return Ok(false);
    }
    write_official_answer_pin_registry_document(workspace_root, &registry)?;
    Ok(true)
}

pub fn build_official_answer_pin(
    request: &OfficialAnswerPinRequest<'_>,
) -> Result<OfficialAnswerPinRecord> {
    if !request.artifact_path.exists() {
        bail!(
            "pin artifact does not exist: {}",
            request.artifact_path.display()
        );
    }

    if let Ok(summary) = read_structured_summary_document(request.artifact_path) {
        return Ok(OfficialAnswerPinRecord {
            label: request.label.to_string(),
            artifact_kind: "structured-summary".to_string(),
            artifact_json_path: canonicalize_or_display(request.artifact_path)?,
            artifact_href: summary.output_files.html,
            title: summary.title,
            summary: summary.share_safe_summary,
            generated_at: summary.generated_at,
            pinned_at: request.pinned_at.to_string(),
            note: request.note.clone(),
            relation_key: summary.family_key,
            status: default_pin_status(),
            resolved_at: None,
            superseded_at: None,
            superseded_by: None,
            supersedes: None,
        });
    }

    if let Ok(report) = crate::core::integration_report::read_integration_report_json_document(
        request.artifact_path,
    ) {
        return Ok(OfficialAnswerPinRecord {
            label: request.label.to_string(),
            artifact_kind: "integration-report".to_string(),
            artifact_json_path: canonicalize_or_display(request.artifact_path)?,
            artifact_href: format!(
                "../Integration/Reports/{}",
                report.artifact_links.html_report
            ),
            title: report.title,
            summary: report.summary,
            generated_at: report.generated_at,
            pinned_at: request.pinned_at.to_string(),
            note: request.note.clone(),
            relation_key: integration_target_identity(&report.platform, &report.target),
            status: default_pin_status(),
            resolved_at: None,
            superseded_at: None,
            superseded_by: None,
            supersedes: None,
        });
    }

    bail!(
        "pin artifact `{}` is not a supported structured summary or integration report json",
        request.artifact_path.display()
    );
}

pub fn read_structured_summary_document(path: &Path) -> Result<StructuredSummaryDocument> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("failed to read structured summary `{}`", path.display()))?;
    serde_json::from_str(&content)
        .with_context(|| format!("failed to parse structured summary `{}`", path.display()))
}

pub fn write_structured_summary_document(
    path: &Path,
    document: &StructuredSummaryDocument,
) -> Result<()> {
    let rendered = serde_json::to_string_pretty(document)
        .context("failed to render structured summary json")?;
    fs::write(path, format!("{rendered}\n"))
        .with_context(|| format!("failed to write structured summary `{}`", path.display()))
}

pub fn collect_structured_summary_documents(
    workspace_root: &Path,
) -> Result<Vec<StructuredSummaryIndexEntry>> {
    let conversations_dir = resolve_workspace_conversations_dir(workspace_root)?;
    if !conversations_dir.exists() {
        return Ok(Vec::new());
    }

    let mut entries = fs::read_dir(&conversations_dir)
        .with_context(|| {
            format!(
                "failed to read conversations directory `{}`",
                conversations_dir.display()
            )
        })?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .and_then(|value| value.to_str())
                .is_some_and(|value| value.eq_ignore_ascii_case("json"))
                && path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.contains("-ai-summary-rounds-"))
        })
        .filter_map(|path| {
            let relative_href = path.file_name()?.to_string_lossy().to_string();
            let document = read_structured_summary_document(&path).ok()?;
            Some(StructuredSummaryIndexEntry {
                relative_href,
                artifact_json_path: canonicalize_or_display(&path).ok()?,
                document,
            })
        })
        .collect::<Vec<_>>();

    entries.sort_by(|left, right| {
        right
            .document
            .generated_at
            .cmp(&left.document.generated_at)
            .then_with(|| left.relative_href.cmp(&right.relative_href))
    });
    Ok(entries)
}

pub fn build_workbench_index_document(
    title: &str,
    generated_at: &str,
    archive_entries: &[ArchiveIndexEntry],
    summaries: &[StructuredSummaryIndexEntry],
    search_reports: &[SearchReportEntry],
    integration_reports: &[IntegrationReportJsonDocument],
    pin_registry: &OfficialAnswerPinRegistryDocument,
) -> WorkbenchIndexJsonDocument {
    let mut families = build_thread_families(archive_entries, summaries);
    let fleet_view = build_fleet_view(integration_reports);
    let official_answers =
        build_official_answer_views(pin_registry, &families, integration_reports);
    enrich_thread_families_with_official_answers(&mut families, summaries, &official_answers);
    let timeline = build_workspace_timeline(
        archive_entries,
        summaries,
        search_reports,
        integration_reports,
        &official_answers,
    );

    WorkbenchIndexJsonDocument {
        schema_version: WORKBENCH_SCHEMA_VERSION,
        title: title.to_string(),
        generated_at: generated_at.to_string(),
        timeline,
        families,
        official_answers,
        fleet_view,
    }
}

pub fn render_share_safe_packet_markdown(
    title: &str,
    generated_at: &str,
    document: &WorkbenchIndexJsonDocument,
) -> String {
    render_share_safe_packet_markdown_for_variant(
        title,
        generated_at,
        document,
        ShareSafePacketVariant::Reviewer,
    )
}

pub fn render_share_safe_packet_markdown_for_variant(
    title: &str,
    generated_at: &str,
    document: &WorkbenchIndexJsonDocument,
    variant: ShareSafePacketVariant,
) -> String {
    let pinned = if document.official_answers.is_empty() {
        "- No official answers pinned yet.".to_string()
    } else {
        document
            .official_answers
            .iter()
            .take(5)
            .map(|pin| {
                let stale = if pin.stale { "stale" } else { "fresh" };
                match variant {
                    ShareSafePacketVariant::Public => {
                        format!("- **{}** `{}`: {}", pin.label, stale, pin.summary)
                    }
                    _ => format!(
                        "- **{}** `{}` `{}`: {}",
                        pin.label, pin.status, stale, pin.summary
                    ),
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    };
    let timeline = if document.timeline.is_empty() {
        "- No recent workbench activity.".to_string()
    } else {
        document
            .timeline
            .iter()
            .take(8)
            .map(|event| {
                let relation = match variant {
                    ShareSafePacketVariant::Public => String::new(),
                    _ => event
                        .relation_key
                        .as_deref()
                        .map(|key| format!(" ({key})"))
                        .unwrap_or_default(),
                };
                format!(
                    "- `{}` {}: {}{}",
                    event.occurred_at, event.kind, event.title, relation
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };
    let families = if document.families.is_empty() {
        "- No stitched families yet.".to_string()
    } else {
        document
            .families
            .iter()
            .take(8)
            .map(|family| match variant {
                ShareSafePacketVariant::Public => format!(
                    "- **{}**: {} transcript(s), {} summary sidecar(s)",
                    family.label, family.transcript_count, family.summary_count
                ),
                _ => format!(
                    "- **{}**: {} transcript(s), {} summary sidecar(s), tests: {}, blockers: {}",
                    family.label,
                    family.transcript_count,
                    family.summary_count,
                    family.tests_run.len(),
                    family.blockers.len()
                ),
            })
            .collect::<Vec<_>>()
            .join("\n")
    };
    let citations = if matches!(variant, ShareSafePacketVariant::Public) {
        String::new()
    } else {
        let citation_lines = document
            .families
            .iter()
            .flat_map(|family| family.citations.iter().take(3))
            .take(8)
            .cloned()
            .collect::<Vec<_>>();
        if citation_lines.is_empty() {
            String::new()
        } else {
            format!(
                "\n## Citations\n{}\n",
                citation_lines
                    .iter()
                    .map(|citation| format!("- `{citation}`"))
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        }
    };

    format!(
        concat!(
            "# {heading}\n\n",
            "- Workspace: `{title}`\n",
            "- Generated: `{generated_at}`\n",
            "- Archive shell: `./index.html`\n",
            "- Reports shell: `../Search/Reports/index.html`\n",
            "- Evidence shell: `../Integration/Reports/index.html`\n\n",
            "## Current official answers\n",
            "{pinned}\n\n",
            "## Recent activity\n",
            "{timeline}\n\n",
            "## Active thread families\n",
            "{families}\n",
            "{citations}"
        ),
        heading = variant.heading(),
        title = title,
        generated_at = generated_at,
        pinned = pinned,
        timeline = timeline,
        families = families,
        citations = citations,
    )
}

fn build_thread_families(
    archive_entries: &[ArchiveIndexEntry],
    summaries: &[StructuredSummaryIndexEntry],
) -> Vec<ThreadFamilySummary> {
    #[derive(Default)]
    struct FamilyAccumulator {
        label: Option<String>,
        latest_at: Option<String>,
        transcript_count: usize,
        summary_count: usize,
        latest_href: Option<String>,
        connectors: BTreeSet<String>,
        profiles: BTreeSet<String>,
        files_touched: BTreeSet<String>,
        tests_run: BTreeSet<String>,
        risks: BTreeSet<String>,
        blockers: BTreeSet<String>,
        next_steps: BTreeSet<String>,
        citations: BTreeSet<String>,
        latest_summary: Option<ThreadFamilyLatestSummary>,
    }

    let mut families = BTreeMap::<String, FamilyAccumulator>::new();
    for entry in archive_entries {
        let Some(thread_id) = entry.thread_id.as_deref() else {
            continue;
        };
        let key = summary_family_key(thread_id);
        let accumulator = families.entry(key).or_default();
        accumulator.transcript_count += 1;
        if let Some(exported_at) = entry.exported_at.as_ref()
            && accumulator
                .latest_at
                .as_ref()
                .is_none_or(|current| exported_at > current)
        {
            accumulator.latest_at = Some(exported_at.clone());
            accumulator.latest_href = Some(entry.relative_href.clone());
            accumulator.label = Some(entry.title.clone());
        }
        if let Some(connector) = entry.connector.as_ref() {
            accumulator.connectors.insert(connector.clone());
        }
    }

    for summary in summaries {
        let accumulator = families
            .entry(summary.document.family_key.clone())
            .or_default();
        accumulator.summary_count += 1;
        if accumulator.label.is_none() {
            accumulator.label = Some(summary.document.title.clone());
        }
        if accumulator
            .latest_at
            .as_ref()
            .is_none_or(|current| summary.document.generated_at > *current)
        {
            accumulator.latest_at = Some(summary.document.generated_at.clone());
            accumulator.latest_href = Some(summary.document.output_files.html.clone());
        }
        accumulator
            .connectors
            .insert(summary.document.connector.clone());
        accumulator
            .profiles
            .insert(summary.document.profile_id.clone());
        accumulator
            .files_touched
            .extend(summary.document.files_touched.iter().cloned());
        accumulator
            .tests_run
            .extend(summary.document.tests_run.iter().cloned());
        accumulator
            .risks
            .extend(summary.document.risks.iter().cloned());
        accumulator
            .blockers
            .extend(summary.document.blockers.iter().cloned());
        accumulator
            .next_steps
            .extend(summary.document.next_steps.iter().cloned());
        accumulator
            .citations
            .extend(summary.document.citations.iter().cloned());
        if accumulator
            .latest_summary
            .as_ref()
            .is_none_or(|current| summary.document.generated_at > current.generated_at)
        {
            accumulator.latest_summary = Some(ThreadFamilyLatestSummary {
                title: summary.document.title.clone(),
                href: summary.document.output_files.html.clone(),
                json_href: summary.document.output_files.json.clone(),
                generated_at: summary.document.generated_at.clone(),
                profile_id: summary.document.profile_id.clone(),
                overview: summary.document.overview.clone(),
                share_safe_summary: summary.document.share_safe_summary.clone(),
            });
        }
    }

    let mut results = families
        .into_iter()
        .map(|(family_key, accumulator)| ThreadFamilySummary {
            family_key,
            label: accumulator
                .label
                .unwrap_or_else(|| "Untitled family".to_string()),
            latest_at: accumulator.latest_at.unwrap_or_default(),
            transcript_count: accumulator.transcript_count,
            summary_count: accumulator.summary_count,
            latest_href: accumulator.latest_href,
            connectors: accumulator.connectors.into_iter().collect(),
            profiles: accumulator.profiles.into_iter().collect(),
            files_touched: accumulator.files_touched.into_iter().collect(),
            tests_run: accumulator.tests_run.into_iter().collect(),
            risks: accumulator.risks.into_iter().collect(),
            blockers: accumulator.blockers.into_iter().collect(),
            next_steps: accumulator.next_steps.into_iter().collect(),
            citations: accumulator.citations.into_iter().collect(),
            latest_summary: accumulator.latest_summary,
            official_answer: None,
            latest_vs_pinned: None,
        })
        .collect::<Vec<_>>();

    results.sort_by(|left, right| {
        right
            .latest_at
            .cmp(&left.latest_at)
            .then_with(|| left.label.cmp(&right.label))
    });
    results
}

fn enrich_thread_families_with_official_answers(
    families: &mut [ThreadFamilySummary],
    summaries: &[StructuredSummaryIndexEntry],
    official_answers: &[OfficialAnswerPinView],
) {
    let latest_summary_by_family = summaries.iter().fold(
        BTreeMap::<String, StructuredSummaryDocument>::new(),
        |mut map, summary| {
            map.entry(summary.document.family_key.clone())
                .and_modify(|current| {
                    if summary.document.generated_at > current.generated_at {
                        *current = summary.document.clone();
                    }
                })
                .or_insert_with(|| summary.document.clone());
            map
        },
    );
    let summaries_by_path = summaries
        .iter()
        .map(|summary| (summary.artifact_json_path.clone(), summary.document.clone()))
        .collect::<BTreeMap<_, _>>();
    let pins_by_relation = official_answers.iter().fold(
        BTreeMap::<String, OfficialAnswerPinView>::new(),
        |mut map, pin| {
            map.entry(pin.relation_key.clone())
                .or_insert_with(|| pin.clone());
            map
        },
    );

    for family in families {
        let Some(pin) = pins_by_relation.get(&family.family_key) else {
            continue;
        };
        family.official_answer = Some(ThreadFamilyOfficialAnswer {
            label: pin.label.clone(),
            status: pin.status.clone(),
            href: pin.href.clone(),
            summary: pin.summary.clone(),
            pinned_at: pin.pinned_at.clone(),
            stale: pin.stale,
            note: pin.note.clone(),
        });
        family.latest_vs_pinned = build_latest_vs_pinned_summary(
            latest_summary_by_family.get(&family.family_key),
            summaries_by_path.get(&pin.artifact_json_path),
        );
    }
}

fn build_latest_vs_pinned_summary(
    latest_summary: Option<&StructuredSummaryDocument>,
    pinned_summary: Option<&StructuredSummaryDocument>,
) -> Option<ThreadFamilyDiffSummary> {
    let latest_summary = latest_summary?;
    let Some(pinned_summary) = pinned_summary else {
        return Some(ThreadFamilyDiffSummary {
            status: "missing-pinned-artifact".to_string(),
            summary: "The pinned answer no longer has a readable structured summary json, so this case view cannot compute a field-level diff.".to_string(),
            new_files_touched: Vec::new(),
            new_tests_run: Vec::new(),
            new_risks: Vec::new(),
            new_blockers: Vec::new(),
            new_next_steps: Vec::new(),
        });
    };

    if latest_summary.generated_at <= pinned_summary.generated_at {
        return Some(ThreadFamilyDiffSummary {
            status: "matched".to_string(),
            summary: "The latest family summary matches the currently pinned answer.".to_string(),
            new_files_touched: Vec::new(),
            new_tests_run: Vec::new(),
            new_risks: Vec::new(),
            new_blockers: Vec::new(),
            new_next_steps: Vec::new(),
        });
    }

    let latest_document = summary_fields_from_doc(latest_summary);
    let pinned_document = summary_fields_from_doc(pinned_summary);
    Some(ThreadFamilyDiffSummary {
        status: "ahead".to_string(),
        summary: format!(
            "The family has a newer summary at {} than the pinned answer at {}.",
            latest_summary.generated_at, pinned_summary.generated_at
        ),
        new_files_touched: difference(
            &latest_document.files_touched,
            &pinned_document.files_touched,
        ),
        new_tests_run: difference(&latest_document.tests_run, &pinned_document.tests_run),
        new_risks: difference(&latest_document.risks, &pinned_document.risks),
        new_blockers: difference(&latest_document.blockers, &pinned_document.blockers),
        new_next_steps: difference(&latest_document.next_steps, &pinned_document.next_steps),
    })
}

struct ThreadFamilySummaryFields {
    files_touched: Vec<String>,
    tests_run: Vec<String>,
    risks: Vec<String>,
    blockers: Vec<String>,
    next_steps: Vec<String>,
}

fn summary_fields_from_doc(document: &StructuredSummaryDocument) -> ThreadFamilySummaryFields {
    ThreadFamilySummaryFields {
        files_touched: document.files_touched.clone(),
        tests_run: document.tests_run.clone(),
        risks: document.risks.clone(),
        blockers: document.blockers.clone(),
        next_steps: document.next_steps.clone(),
    }
}

fn difference(left: &[String], right: &[String]) -> Vec<String> {
    let right = right.iter().cloned().collect::<BTreeSet<_>>();
    left.iter()
        .filter(|value| !right.contains(*value))
        .cloned()
        .collect()
}

fn build_fleet_view(reports: &[IntegrationReportJsonDocument]) -> Vec<FleetViewEntry> {
    #[derive(Clone)]
    struct FleetAccumulator {
        latest: IntegrationReportJsonDocument,
        report_count: usize,
        history: Vec<FleetHistoryEntry>,
    }

    let mut by_relation = BTreeMap::<String, FleetAccumulator>::new();
    for report in reports {
        let relation_key = integration_target_identity(&report.platform, &report.target);
        by_relation
            .entry(relation_key)
            .and_modify(|accumulator| {
                accumulator.report_count += 1;
                accumulator.history.push(FleetHistoryEntry {
                    generated_at: report.generated_at.clone(),
                    readiness: report.readiness.clone(),
                    kind: report.kind.clone(),
                    summary: report.summary.clone(),
                    html_href: format!(
                        "../Integration/Reports/{}",
                        report.artifact_links.html_report
                    ),
                });
                if report.generated_at > accumulator.latest.generated_at {
                    accumulator.latest = report.clone();
                }
            })
            .or_insert_with(|| FleetAccumulator {
                latest: report.clone(),
                report_count: 1,
                history: vec![FleetHistoryEntry {
                    generated_at: report.generated_at.clone(),
                    readiness: report.readiness.clone(),
                    kind: report.kind.clone(),
                    summary: report.summary.clone(),
                    html_href: format!(
                        "../Integration/Reports/{}",
                        report.artifact_links.html_report
                    ),
                }],
            });
    }

    let mut fleet = by_relation
        .into_iter()
        .map(|(relation_key, mut accumulator)| {
            accumulator
                .history
                .sort_by(|left, right| right.generated_at.cmp(&left.generated_at));
            FleetViewEntry {
                relation_key,
                platform: accumulator.latest.platform.clone(),
                target: accumulator.latest.target.clone(),
                latest_readiness: accumulator.latest.readiness.clone(),
                latest_generated_at: accumulator.latest.generated_at.clone(),
                report_count: accumulator.report_count,
                html_href: Some(format!(
                    "../Integration/Reports/{}",
                    accumulator.latest.artifact_links.html_report
                )),
                latest_summary: accumulator.latest.summary.clone(),
                next_steps: accumulator.latest.next_steps.clone(),
                history: accumulator.history,
            }
        })
        .collect::<Vec<_>>();

    fleet.sort_by(|left, right| {
        right
            .latest_generated_at
            .cmp(&left.latest_generated_at)
            .then_with(|| left.relation_key.cmp(&right.relation_key))
    });
    fleet
}

fn build_official_answer_views(
    registry: &OfficialAnswerPinRegistryDocument,
    families: &[ThreadFamilySummary],
    integration_reports: &[IntegrationReportJsonDocument],
) -> Vec<OfficialAnswerPinView> {
    let family_latest = families
        .iter()
        .map(|family| (family.family_key.clone(), family.latest_at.clone()))
        .collect::<BTreeMap<_, _>>();
    let integration_latest =
        integration_reports
            .iter()
            .fold(BTreeMap::<String, String>::new(), |mut map, report| {
                let key = integration_target_identity(&report.platform, &report.target);
                let entry = map.entry(key).or_default();
                if report.generated_at > *entry {
                    *entry = report.generated_at.clone();
                }
                map
            });

    let mut pins = registry
        .pins
        .iter()
        .map(|pin| {
            let latest_related_at = if pin.artifact_kind == "structured-summary" {
                family_latest.get(&pin.relation_key).cloned()
            } else {
                integration_latest.get(&pin.relation_key).cloned()
            };
            let stale = latest_related_at
                .as_ref()
                .is_some_and(|latest| latest > &pin.generated_at);
            let stale_reason = if stale {
                latest_related_at.as_ref().map(|latest| {
                    if pin.artifact_kind == "structured-summary" {
                        format!(
                            "A newer summary in this case was generated at {latest}. Re-review before treating this pinned answer as current."
                        )
                    } else {
                        format!(
                            "A newer integration report for this target was generated at {latest}. Re-review before treating this pinned answer as current."
                        )
                    }
                })
            } else {
                None
            };
            let lifecycle_summary = match pin.status.as_str() {
                "resolved" => pin
                    .resolved_at
                    .as_ref()
                    .map(|resolved_at| format!("Marked resolved at {resolved_at}.")),
                "superseded" => Some(match (&pin.superseded_by, &pin.superseded_at) {
                    (Some(label), Some(at)) => {
                        format!("Superseded by {label} at {at}.")
                    }
                    (Some(label), None) => format!("Superseded by {label}."),
                    _ => "Superseded by a newer official answer.".to_string(),
                }),
                _ => None,
            };
            OfficialAnswerPinView {
                label: pin.label.clone(),
                artifact_kind: pin.artifact_kind.clone(),
                artifact_json_path: pin.artifact_json_path.clone(),
                title: pin.title.clone(),
                summary: pin.summary.clone(),
                href: pin.artifact_href.clone(),
                generated_at: pin.generated_at.clone(),
                pinned_at: pin.pinned_at.clone(),
                note: pin.note.clone(),
                relation_key: pin.relation_key.clone(),
                status: pin.status.clone(),
                resolved_at: pin.resolved_at.clone(),
                superseded_at: pin.superseded_at.clone(),
                superseded_by: pin.superseded_by.clone(),
                supersedes: pin.supersedes.clone(),
                stale,
                stale_reason,
                lifecycle_summary,
                latest_related_at,
            }
        })
        .collect::<Vec<_>>();
    pins.sort_by(|left, right| {
        right
            .pinned_at
            .cmp(&left.pinned_at)
            .then_with(|| left.label.cmp(&right.label))
    });
    pins
}

fn build_workspace_timeline(
    archive_entries: &[ArchiveIndexEntry],
    summaries: &[StructuredSummaryIndexEntry],
    search_reports: &[SearchReportEntry],
    integration_reports: &[IntegrationReportJsonDocument],
    pins: &[OfficialAnswerPinView],
) -> Vec<WorkspaceTimelineEvent> {
    let mut events = Vec::new();

    for entry in archive_entries {
        if let Some(occurred_at) = entry.exported_at.as_ref() {
            events.push(WorkspaceTimelineEvent {
                id: format!("transcript:{}", entry.file_name),
                kind: "transcript-export".to_string(),
                title: entry.title.clone(),
                occurred_at: occurred_at.clone(),
                href: Some(entry.relative_href.clone()),
                connector: entry.connector.clone(),
                relation_key: entry.thread_id.as_deref().map(summary_family_key),
                summary: None,
            });
        }
    }

    for summary in summaries {
        events.push(WorkspaceTimelineEvent {
            id: format!("summary:{}", summary.relative_href),
            kind: "ai-summary".to_string(),
            title: summary.document.title.clone(),
            occurred_at: summary.document.generated_at.clone(),
            href: Some(summary.document.output_files.html.clone()),
            connector: Some(summary.document.connector.clone()),
            relation_key: Some(summary.document.family_key.clone()),
            summary: Some(summary.document.share_safe_summary.clone()),
        });
    }

    for report in search_reports {
        if let Some(occurred_at) = report.generated_at.as_ref() {
            events.push(WorkspaceTimelineEvent {
                id: format!("search:{}", report.file_name),
                kind: "search-report".to_string(),
                title: report.title.clone(),
                occurred_at: occurred_at.clone(),
                href: Some(format!("../Search/Reports/{}", report.relative_href)),
                connector: None,
                relation_key: report.query.clone(),
                summary: report.query.clone(),
            });
        }
    }

    for report in integration_reports {
        events.push(WorkspaceTimelineEvent {
            id: format!("integration:{}", report.artifact_links.json_report),
            kind: "integration-report".to_string(),
            title: report.title.clone(),
            occurred_at: report.generated_at.clone(),
            href: Some(format!(
                "../Integration/Reports/{}",
                report.artifact_links.html_report
            )),
            connector: None,
            relation_key: Some(integration_target_identity(
                &report.platform,
                &report.target,
            )),
            summary: Some(report.summary.clone()),
        });
    }

    for pin in pins {
        events.push(WorkspaceTimelineEvent {
            id: format!("pin:{}", pin.label),
            kind: "official-answer-pin".to_string(),
            title: pin.label.clone(),
            occurred_at: pin.pinned_at.clone(),
            href: Some(pin.href.clone()),
            connector: None,
            relation_key: Some(pin.relation_key.clone()),
            summary: Some(pin.summary.clone()),
        });
    }

    events.sort_by(|left, right| {
        right
            .occurred_at
            .cmp(&left.occurred_at)
            .then_with(|| left.id.cmp(&right.id))
    });
    events
}

pub fn build_action_packs(
    families: &[ThreadFamilySummary],
    official_answers: &[OfficialAnswerPinView],
    fleet_view: &[FleetViewEntry],
    generated_at: &str,
) -> ActionPackIndexDocument {
    let mut packs = Vec::new();

    for pin in official_answers.iter().filter(|pin| pin.stale) {
        let id = slugify(&format!("re-review-{}", pin.label));
        packs.push(ActionPackRecord {
            id: id.clone(),
            kind: "re-review pack".to_string(),
            title: format!("Re-review {}", pin.label),
            summary: pin
                .stale_reason
                .clone()
                .unwrap_or_else(|| "A newer related artifact exists.".to_string()),
            relation_key: Some(pin.relation_key.clone()),
            source_kind: "official-answer".to_string(),
            markdown_href: format!("{id}.md"),
            json_href: format!("{id}.json"),
            steps: vec![
                "Open the current pinned answer and the latest related artifact side-by-side."
                    .to_string(),
                "Decide whether the old answer should be superseded, resolved, or kept."
                    .to_string(),
            ],
        });
    }

    for family in families.iter().filter(|family| {
        family
            .latest_vs_pinned
            .as_ref()
            .is_some_and(|diff| diff.status == "ahead")
    }) {
        let id = slugify(&format!("what-changed-{}", family.label));
        let diff = family.latest_vs_pinned.as_ref().expect("checked above");
        packs.push(ActionPackRecord {
            id: id.clone(),
            kind: "what changed note".to_string(),
            title: format!("What changed in {}", family.label),
            summary: diff.summary.clone(),
            relation_key: Some(family.family_key.clone()),
            source_kind: "thread-family".to_string(),
            markdown_href: format!("{id}.md"),
            json_href: format!("{id}.json"),
            steps: diff
                .new_files_touched
                .iter()
                .map(|file| format!("Review the new file touch: {file}"))
                .chain(
                    diff.new_tests_run
                        .iter()
                        .map(|test| format!("Confirm the new test/run: {test}")),
                )
                .take(6)
                .collect(),
        });
    }

    for entry in fleet_view
        .iter()
        .filter(|entry| entry.latest_readiness != "ready" || !entry.next_steps.is_empty())
    {
        let id = slugify(&format!("fleet-{}-{}", entry.platform, entry.target));
        packs.push(ActionPackRecord {
            id: id.clone(),
            kind: "issue draft".to_string(),
            title: format!("Action draft for {} {}", entry.platform, entry.target),
            summary: entry.latest_summary.clone(),
            relation_key: Some(entry.relation_key.clone()),
            source_kind: "fleet-view".to_string(),
            markdown_href: format!("{id}.md"),
            json_href: format!("{id}.json"),
            steps: if entry.next_steps.is_empty() {
                vec!["Inspect the latest fleet report and convert the observed drift into an actionable issue.".to_string()]
            } else {
                entry.next_steps.clone()
            },
        });
    }

    packs.sort_by(|left, right| left.title.cmp(&right.title));
    ActionPackIndexDocument {
        schema_version: ACTION_PACKS_SCHEMA_VERSION,
        title: "report-to-action bridge".to_string(),
        generated_at: generated_at.to_string(),
        packs,
    }
}

pub fn build_team_memory_lane_document(
    workspace_root: &Path,
    archive_title: &str,
    generated_at: &str,
    current_index: &WorkbenchIndexJsonDocument,
    action_packs: &ActionPackIndexDocument,
) -> TeamMemoryLaneDocument {
    let mut workspaces = discover_workspace_snapshots(workspace_root);
    if !workspaces
        .iter()
        .any(|entry| entry.workspace_root == workspace_root.display().to_string())
    {
        workspaces.push(TeamMemoryWorkspaceEntry {
            workspace_name: workspace_root
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or("workspace")
                .to_string(),
            workspace_root: workspace_root.display().to_string(),
            archive_title: archive_title.to_string(),
            generated_at: generated_at.to_string(),
            family_count: current_index.families.len(),
            official_answer_count: current_index.official_answers.len(),
            fleet_count: current_index.fleet_view.len(),
        });
    }
    workspaces.sort_by(|left, right| left.workspace_name.cmp(&right.workspace_name));

    let stale_pin_count = current_index
        .official_answers
        .iter()
        .filter(|pin| pin.stale)
        .count();
    let maintainer_items = current_index
        .families
        .iter()
        .take(5)
        .map(|family| {
            format!(
                "{}: {} family summaries tracked",
                family.label, family.summary_count
            )
        })
        .collect::<Vec<_>>();
    let reviewer_items = current_index
        .official_answers
        .iter()
        .take(5)
        .map(|pin| format!("{} ({})", pin.label, pin.status))
        .collect::<Vec<_>>();
    let operator_items = current_index
        .fleet_view
        .iter()
        .take(5)
        .map(|entry| {
            format!(
                "{} {} -> {}",
                entry.platform, entry.target, entry.latest_readiness
            )
        })
        .collect::<Vec<_>>();

    TeamMemoryLaneDocument {
        schema_version: MEMORY_LANE_SCHEMA_VERSION,
        title: "team and org memory lane".to_string(),
        generated_at: generated_at.to_string(),
        workspaces,
        role_views: vec![
            TeamMemoryRoleView {
                role: "maintainer".to_string(),
                summary: format!(
                    "{} action pack(s) currently translate repo state into next-step work.",
                    action_packs.packs.len()
                ),
                items: maintainer_items,
            },
            TeamMemoryRoleView {
                role: "reviewer".to_string(),
                summary: format!(
                    "{stale_pin_count} stale official answer(s) currently need re-review."
                ),
                items: reviewer_items,
            },
            TeamMemoryRoleView {
                role: "operator".to_string(),
                summary: format!(
                    "{} fleet relation(s) are visible in the latest local workbench snapshot.",
                    current_index.fleet_view.len()
                ),
                items: operator_items,
            },
        ],
    }
}

fn discover_workspace_snapshots(workspace_root: &Path) -> Vec<TeamMemoryWorkspaceEntry> {
    let Some(parent) = workspace_root.parent() else {
        return Vec::new();
    };
    let Ok(entries) = fs::read_dir(parent) else {
        return Vec::new();
    };

    let mut workspaces = Vec::new();
    for entry in entries.filter_map(|entry| entry.ok()) {
        let path = entry.path();
        let candidate = path
            .join(".agents")
            .join("Conversations")
            .join("index.json");
        if !candidate.is_file() {
            continue;
        }
        let Ok(content) = fs::read_to_string(&candidate) else {
            continue;
        };
        let Ok(document) = serde_json::from_str::<WorkbenchIndexJsonDocument>(&content) else {
            continue;
        };
        workspaces.push(TeamMemoryWorkspaceEntry {
            workspace_name: path
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or("workspace")
                .to_string(),
            workspace_root: path.display().to_string(),
            archive_title: document.title,
            generated_at: document.generated_at,
            family_count: document.families.len(),
            official_answer_count: document.official_answers.len(),
            fleet_count: document.fleet_view.len(),
        });
    }
    workspaces
}

fn slugify(value: &str) -> String {
    let mut slug = value
        .chars()
        .map(|ch| match ch {
            'a'..='z' | '0'..='9' => ch,
            'A'..='Z' => ch.to_ascii_lowercase(),
            _ => '-',
        })
        .collect::<String>();
    while slug.contains("--") {
        slug = slug.replace("--", "-");
    }
    slug.trim_matches('-').to_string()
}

fn canonicalize_or_display(path: &Path) -> Result<String> {
    Ok(path
        .canonicalize()
        .unwrap_or_else(|_| path.to_path_buf())
        .display()
        .to_string())
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::{
        OfficialAnswerPinRecord, OfficialAnswerPinRegistryDocument, StructuredSummaryDocument,
        StructuredSummaryOutputFiles, build_workbench_index_document,
        render_share_safe_packet_markdown, summary_family_key,
        write_official_answer_pin_registry_document, write_structured_summary_document,
    };
    use crate::core::archive_index::ArchiveIndexEntry;
    use crate::core::integration_report::{
        IntegrationArtifactLinks, IntegrationReportJsonDocument,
    };
    use crate::core::search_report::SearchReportEntry;

    #[test]
    fn build_workbench_index_document_marks_stale_official_answers() {
        let family_key = summary_family_key("thread-1");
        let archive_entries = vec![ArchiveIndexEntry {
            file_name: "thread-1.html".to_string(),
            relative_href: "thread-1.html".to_string(),
            title: "Thread 1".to_string(),
            connector: Some("codex".to_string()),
            thread_id: Some("thread-1".to_string()),
            completeness: Some("complete".to_string()),
            source_kind: Some("app-server-thread-read".to_string()),
            exported_at: Some("2026-04-21T12:00:00Z".to_string()),
            ai_summary_href: None,
        }];
        let summaries = Vec::new();
        let reports = Vec::<SearchReportEntry>::new();
        let integration = vec![IntegrationReportJsonDocument {
            schema_version: 1,
            title: "codex doctor".to_string(),
            kind: "doctor".to_string(),
            platform: "codex".to_string(),
            target: "/tmp/project".to_string(),
            generated_at: "2026-04-21T12:30:00Z".to_string(),
            readiness: "ready".to_string(),
            summary: "ready".to_string(),
            launcher_status: "ready".to_string(),
            launcher_kind: "binary".to_string(),
            launcher_command: "agent-exporter".to_string(),
            bridge_status: "ready".to_string(),
            pack_shape_checks: Vec::new(),
            checks: Vec::new(),
            next_steps: Vec::new(),
            written_files: Vec::new(),
            unchanged_files: Vec::new(),
            artifact_links: IntegrationArtifactLinks {
                html_report: "integration-report.html".to_string(),
                json_report: "integration-report.json".to_string(),
                index_html: "index.html".to_string(),
                index_json: "index.json".to_string(),
            },
        }];
        let pins = OfficialAnswerPinRegistryDocument {
            schema_version: 1,
            generated_at: "2026-04-21T12:31:00Z".to_string(),
            pins: vec![
                OfficialAnswerPinRecord {
                    label: "Pinned summary".to_string(),
                    artifact_kind: "structured-summary".to_string(),
                    artifact_json_path: "/tmp/summary.json".to_string(),
                    artifact_href: "summary.html".to_string(),
                    title: "Pinned".to_string(),
                    summary: "summary".to_string(),
                    generated_at: "2026-04-21T11:00:00Z".to_string(),
                    pinned_at: "2026-04-21T11:01:00Z".to_string(),
                    note: None,
                    relation_key: family_key,
                    status: "active".to_string(),
                    resolved_at: None,
                    superseded_at: None,
                    superseded_by: None,
                    supersedes: None,
                },
                OfficialAnswerPinRecord {
                    label: "Pinned report".to_string(),
                    artifact_kind: "integration-report".to_string(),
                    artifact_json_path: "/tmp/report.json".to_string(),
                    artifact_href: "../Integration/Reports/integration-report.html".to_string(),
                    title: "Pinned report".to_string(),
                    summary: "report".to_string(),
                    generated_at: "2026-04-21T11:30:00Z".to_string(),
                    pinned_at: "2026-04-21T11:31:00Z".to_string(),
                    note: None,
                    relation_key: "codex::/tmp/project".to_string(),
                    status: "active".to_string(),
                    resolved_at: None,
                    superseded_at: None,
                    superseded_by: None,
                    supersedes: None,
                },
            ],
        };

        let document = build_workbench_index_document(
            "demo archive",
            "2026-04-21T13:00:00Z",
            &archive_entries,
            &summaries,
            &reports,
            &integration,
            &pins,
        );

        assert_eq!(document.timeline.len(), 4);
        assert_eq!(document.families.len(), 1);
        assert_eq!(document.official_answers.len(), 2);
        assert!(document.official_answers.iter().all(|pin| pin.stale));
    }

    #[test]
    fn render_share_safe_packet_markdown_avoids_absolute_workspace_paths() {
        let workspace = tempdir().expect("workspace");
        let conversations = workspace.path().join(".agents").join("Conversations");
        std::fs::create_dir_all(&conversations).expect("conversations dir");

        let summary_path = conversations.join("demo-ai-summary.json");
        write_structured_summary_document(
            &summary_path,
            &StructuredSummaryDocument {
                schema_version: 1,
                thread_id: "thread-1".to_string(),
                connector: "codex".to_string(),
                source_kind: "app-server-thread-read".to_string(),
                completeness: "complete".to_string(),
                generated_at: "2026-04-21T12:00:00Z".to_string(),
                profile_id: "handoff".to_string(),
                runtime_profile: None,
                runtime_model: None,
                runtime_provider: None,
                family_key: summary_family_key("thread-1"),
                title: "handoff".to_string(),
                overview: "overview".to_string(),
                share_safe_summary: "share safe".to_string(),
                goals: vec!["ship".to_string()],
                files_touched: Vec::new(),
                tests_run: Vec::new(),
                risks: Vec::new(),
                blockers: Vec::new(),
                next_steps: Vec::new(),
                citations: Vec::new(),
                extraction_mode: "fallback".to_string(),
                output_files: StructuredSummaryOutputFiles {
                    markdown: "demo.md".to_string(),
                    html: "demo.html".to_string(),
                    json: "demo-ai-summary.json".to_string(),
                },
            },
        )
        .expect("write summary");
        write_official_answer_pin_registry_document(
            workspace.path(),
            &OfficialAnswerPinRegistryDocument {
                schema_version: super::PIN_REGISTRY_SCHEMA_VERSION,
                generated_at: "2026-04-21T12:30:00Z".to_string(),
                pins: Vec::new(),
            },
        )
        .expect("write pin registry");

        let document = build_workbench_index_document(
            "demo archive",
            "2026-04-21T12:30:00Z",
            &Vec::new(),
            &Vec::new(),
            &Vec::new(),
            &Vec::new(),
            &OfficialAnswerPinRegistryDocument {
                schema_version: super::PIN_REGISTRY_SCHEMA_VERSION,
                generated_at: "2026-04-21T12:30:00Z".to_string(),
                pins: Vec::new(),
            },
        );
        let packet =
            render_share_safe_packet_markdown("demo archive", "2026-04-21T12:30:00Z", &document);
        assert!(packet.contains("Share-safe workbench packet"));
        assert!(!packet.contains(&workspace.path().display().to_string()));
    }
}
