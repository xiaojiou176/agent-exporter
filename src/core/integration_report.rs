use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntegrationReportEntry {
    pub file_name: String,
    pub relative_href: String,
    pub title: String,
    pub report_kind: Option<String>,
    pub platform: Option<String>,
    pub readiness: Option<String>,
    pub target_root: Option<String>,
    pub generated_at: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct IntegrationArtifactLinks {
    pub html_report: String,
    pub json_report: String,
    pub index_html: String,
    pub index_json: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct IntegrationReportCheckRecord {
    pub label: String,
    pub readiness: String,
    pub detail: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct IntegrationReportJsonDocument {
    pub schema_version: u32,
    pub title: String,
    pub kind: String,
    pub platform: String,
    pub target: String,
    pub generated_at: String,
    pub readiness: String,
    pub summary: String,
    pub launcher_status: String,
    pub launcher_kind: String,
    pub launcher_command: String,
    pub bridge_status: String,
    pub pack_shape_checks: Vec<IntegrationReportCheckRecord>,
    pub checks: Vec<IntegrationReportCheckRecord>,
    pub next_steps: Vec<String>,
    pub written_files: Vec<String>,
    pub unchanged_files: Vec<String>,
    pub artifact_links: IntegrationArtifactLinks,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct IntegrationReportTimelineEntry {
    pub title: String,
    pub kind: String,
    pub platform: String,
    pub readiness: String,
    pub target: String,
    pub generated_at: String,
    pub html_href: String,
    pub json_href: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct IntegrationReportsIndexJsonDocument {
    pub schema_version: u32,
    pub title: String,
    pub generated_at: String,
    pub report_count: usize,
    pub timeline: Vec<IntegrationReportTimelineEntry>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct IntegrationBaselineRegistryDocument {
    pub schema_version: u32,
    pub generated_at: String,
    pub baselines: Vec<IntegrationBaselineRecord>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct IntegrationBaselineRecord {
    pub name: String,
    pub platform: String,
    pub target: String,
    pub target_identity: String,
    pub report_title: String,
    pub report_json_path: String,
    pub report_html_path: String,
    pub promoted_at: String,
    pub promoted_from_verdict: String,
    pub policy_name: String,
    pub policy_version: String,
    pub note: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct IntegrationDecisionHistoryDocument {
    pub schema_version: u32,
    pub generated_at: String,
    pub decisions: Vec<IntegrationDecisionRecord>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct IntegrationDecisionRecord {
    pub baseline_name: String,
    pub platform: String,
    pub target: String,
    pub target_identity: String,
    pub baseline_report_json_path: Option<String>,
    pub baseline_report_title: Option<String>,
    pub candidate_report_json_path: String,
    pub candidate_report_title: String,
    pub policy_name: String,
    pub policy_version: String,
    pub verdict: String,
    pub promoted: bool,
    pub decided_at: String,
    pub summary: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct IntegrationEvidenceCheckDiff {
    pub label: String,
    pub left_readiness: Option<String>,
    pub right_readiness: Option<String>,
    pub left_detail: Option<String>,
    pub right_detail: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct IntegrationEvidenceDiff {
    pub platform: String,
    pub target: String,
    pub left_title: String,
    pub right_title: String,
    pub left_generated_at: String,
    pub right_generated_at: String,
    pub left_readiness: String,
    pub right_readiness: String,
    pub changed_checks: Vec<IntegrationEvidenceCheckDiff>,
    pub added_next_steps: Vec<String>,
    pub removed_next_steps: Vec<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum IntegrationEvidenceGateVerdict {
    Pass,
    Warn,
    Fail,
}

impl IntegrationEvidenceGateVerdict {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::Warn => "warn",
            Self::Fail => "fail",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct IntegrationEvidenceExplainStep {
    pub title: String,
    pub why: String,
    pub recheck: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct IntegrationEvidenceGatePolicy {
    #[serde(default = "default_policy_name")]
    pub name: String,
    #[serde(default = "default_policy_version")]
    pub version: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_all_platforms")]
    pub platforms: Vec<String>,
    pub fail_on_readiness_regression: bool,
    pub blocking_check_labels: Vec<String>,
    pub warning_check_labels: Vec<String>,
    #[serde(default)]
    pub ignorable_check_labels: Vec<String>,
    #[serde(default = "default_allowed_verdicts")]
    pub allowed_verdicts: Vec<String>,
    #[serde(default = "default_promotable_readinesses")]
    pub allowed_candidate_readiness: Vec<String>,
    #[serde(default = "default_true")]
    pub require_no_regression: bool,
    #[serde(default = "default_true")]
    pub require_no_blocking_changes: bool,
    #[serde(default = "default_true")]
    pub require_no_next_steps: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntegrationPromotionAssessment {
    pub eligible: bool,
    pub reasons: Vec<String>,
    pub summary: String,
}

pub type IntegrationEvidencePromotionAssessment = IntegrationPromotionAssessment;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct IntegrationEvidencePromotionPolicy {
    pub allowed_verdicts: Vec<String>,
    #[serde(default = "default_promotable_readinesses")]
    pub allowed_candidate_readiness: Vec<String>,
    #[serde(default = "default_true")]
    pub require_non_regression: bool,
    #[serde(default = "default_true")]
    pub require_no_blocking_changes: bool,
    #[serde(default)]
    pub require_no_next_steps: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct IntegrationEvidencePolicyPack {
    pub schema_version: u32,
    pub name: String,
    pub version: String,
    pub description: String,
    #[serde(default = "default_all_platforms")]
    pub platforms: Vec<String>,
    #[serde(flatten)]
    pub gate: IntegrationEvidenceGatePolicy,
    #[serde(flatten)]
    pub promotion: IntegrationEvidencePromotionPolicy,
}

impl std::ops::Deref for IntegrationEvidencePolicyPack {
    type Target = IntegrationEvidenceGatePolicy;

    fn deref(&self) -> &Self::Target {
        &self.gate
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedIntegrationEvidencePolicy {
    pub source: String,
    pub policy: IntegrationEvidencePolicyPack,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct IntegrationEvidenceGateOutcome {
    pub verdict: IntegrationEvidenceGateVerdict,
    pub baseline_title: String,
    pub candidate_title: String,
    pub baseline_generated_at: String,
    pub candidate_generated_at: String,
    pub baseline_readiness: String,
    pub candidate_readiness: String,
    pub regression: bool,
    pub blocking_changes: Vec<IntegrationEvidenceCheckDiff>,
    pub warning_changes: Vec<IntegrationEvidenceCheckDiff>,
    pub ignored_changes: Vec<IntegrationEvidenceCheckDiff>,
    pub remediation_steps: Vec<IntegrationEvidenceExplainStep>,
}

fn default_true() -> bool {
    true
}

fn default_policy_name() -> String {
    "default".to_string()
}

fn default_policy_version() -> String {
    "1.0.0".to_string()
}

fn default_all_platforms() -> Vec<String> {
    vec!["*".to_string()]
}

fn default_allowed_verdicts() -> Vec<String> {
    vec![IntegrationEvidenceGateVerdict::Pass.as_str().to_string()]
}

fn default_promotable_readinesses() -> Vec<String> {
    vec!["ready".to_string()]
}

impl IntegrationEvidenceGatePolicy {
    pub fn label(&self) -> String {
        format!("{}@{}", self.name, self.version)
    }

    pub fn allows_promotion_verdict(&self, verdict: IntegrationEvidenceGateVerdict) -> bool {
        self.allowed_verdicts
            .iter()
            .any(|value| value == verdict.as_str())
    }
}

pub fn resolve_integration_reports_dir(workspace_root: &Path) -> PathBuf {
    workspace_root
        .join(".agents")
        .join("Integration")
        .join("Reports")
}

pub fn resolve_integration_baseline_registry_path(workspace_root: &Path) -> PathBuf {
    resolve_integration_reports_dir(workspace_root).join("baseline-registry.json")
}

pub fn resolve_integration_decision_history_path(workspace_root: &Path) -> PathBuf {
    resolve_integration_reports_dir(workspace_root).join("decision-history.json")
}

pub fn latest_official_baseline_record(
    document: &IntegrationBaselineRegistryDocument,
) -> Option<&IntegrationBaselineRecord> {
    document.baselines.iter().max_by(|left, right| {
        left.promoted_at
            .cmp(&right.promoted_at)
            .then_with(|| left.name.cmp(&right.name))
    })
}

pub fn integration_report_base_name(kind: &str, platform: &str, generated_at: &str) -> String {
    format!(
        "integration-report-{kind}-{platform}-{timestamp}",
        kind = slugify(kind),
        platform = slugify(platform),
        timestamp = slugify(generated_at),
    )
}

pub fn integration_target_identity(platform: &str, target: &str) -> String {
    format!("{platform}::{target}")
}

pub fn canonical_report_json_path(path: &Path) -> Result<PathBuf> {
    let candidate = match path.extension().and_then(|value| value.to_str()) {
        Some("html") => path.with_extension("json"),
        _ => path.to_path_buf(),
    };

    fs::canonicalize(&candidate).with_context(|| {
        format!(
            "failed to resolve saved integration report `{}`",
            candidate.display()
        )
    })
}

pub fn repo_owned_integration_policy_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("policies")
        .join("integration-evidence")
}

pub fn collect_integration_report_entries(
    workspace_root: &Path,
) -> Result<Vec<IntegrationReportEntry>> {
    let reports_dir = resolve_integration_reports_dir(workspace_root);
    if !reports_dir.exists() {
        return Ok(Vec::new());
    }

    let mut entries = fs::read_dir(&reports_dir)
        .with_context(|| {
            format!(
                "failed to read integration report directory `{}`",
                reports_dir.display()
            )
        })?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .and_then(|value| value.to_str())
                .is_some_and(|value| value.eq_ignore_ascii_case("html"))
        })
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name != "index.html")
        })
        .map(read_integration_report_entry)
        .collect::<Result<Vec<_>>>()?;

    entries.sort_by(|left, right| {
        right
            .generated_at
            .cmp(&left.generated_at)
            .then_with(|| left.file_name.cmp(&right.file_name))
    });

    Ok(entries)
}

pub fn write_integration_reports_index_document(
    workspace_root: &Path,
    document: &str,
) -> Result<PathBuf> {
    let reports_dir = resolve_integration_reports_dir(workspace_root);
    fs::create_dir_all(&reports_dir).with_context(|| {
        format!(
            "failed to prepare integration report directory `{}`",
            reports_dir.display()
        )
    })?;

    let index_path = reports_dir.join("index.html");
    fs::write(&index_path, format!("{}\n", document.trim_end())).with_context(|| {
        format!(
            "failed to write integration report index `{}`",
            index_path.display()
        )
    })?;
    Ok(index_path)
}

pub fn write_integration_report_document(
    workspace_root: &Path,
    kind: &str,
    platform: &str,
    generated_at: &str,
    document: &str,
) -> Result<PathBuf> {
    let reports_dir = resolve_integration_reports_dir(workspace_root);
    fs::create_dir_all(&reports_dir).with_context(|| {
        format!(
            "failed to prepare integration report directory `{}`",
            reports_dir.display()
        )
    })?;

    let report_path = reports_dir.join(format!(
        "{}.html",
        integration_report_base_name(kind, platform, generated_at)
    ));
    fs::write(&report_path, format!("{}\n", document.trim_end())).with_context(|| {
        format!(
            "failed to write integration report `{}`",
            report_path.display()
        )
    })?;
    Ok(report_path)
}

pub fn write_integration_report_json_document(
    workspace_root: &Path,
    kind: &str,
    platform: &str,
    generated_at: &str,
    document: &IntegrationReportJsonDocument,
) -> Result<PathBuf> {
    let reports_dir = resolve_integration_reports_dir(workspace_root);
    fs::create_dir_all(&reports_dir).with_context(|| {
        format!(
            "failed to prepare integration report directory `{}`",
            reports_dir.display()
        )
    })?;

    let report_path = reports_dir.join(format!(
        "{}.json",
        integration_report_base_name(kind, platform, generated_at)
    ));
    let payload = serde_json::to_string_pretty(document)
        .context("failed to render integration report json")?;
    fs::write(&report_path, format!("{payload}\n")).with_context(|| {
        format!(
            "failed to write integration report json `{}`",
            report_path.display()
        )
    })?;
    Ok(report_path)
}

pub fn write_integration_reports_index_json_document(
    workspace_root: &Path,
    document: &IntegrationReportsIndexJsonDocument,
) -> Result<PathBuf> {
    let reports_dir = resolve_integration_reports_dir(workspace_root);
    fs::create_dir_all(&reports_dir).with_context(|| {
        format!(
            "failed to prepare integration report directory `{}`",
            reports_dir.display()
        )
    })?;

    let index_path = reports_dir.join("index.json");
    let payload = serde_json::to_string_pretty(document)
        .context("failed to render integration index json")?;
    fs::write(&index_path, format!("{payload}\n")).with_context(|| {
        format!(
            "failed to write integration report index json `{}`",
            index_path.display()
        )
    })?;
    Ok(index_path)
}

pub fn collect_integration_report_json_documents(
    workspace_root: &Path,
) -> Result<Vec<IntegrationReportJsonDocument>> {
    let reports_dir = resolve_integration_reports_dir(workspace_root);
    if !reports_dir.exists() {
        return Ok(Vec::new());
    }

    let mut entries = fs::read_dir(&reports_dir)
        .with_context(|| {
            format!(
                "failed to read integration report directory `{}`",
                reports_dir.display()
            )
        })?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .and_then(|value| value.to_str())
                .is_some_and(|value| value.eq_ignore_ascii_case("json"))
        })
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name != "index.json" && name.starts_with("integration-report-"))
        })
        .map(|path| read_integration_report_json_document(&path))
        .collect::<Result<Vec<_>>>()?;

    entries.sort_by(|left, right| {
        right
            .generated_at
            .cmp(&left.generated_at)
            .then_with(|| left.title.cmp(&right.title))
    });

    Ok(entries)
}

pub fn read_integration_report_json_document(path: &Path) -> Result<IntegrationReportJsonDocument> {
    let resolved_path = match path.extension().and_then(|value| value.to_str()) {
        Some("json") => path.to_path_buf(),
        Some("html") => path.with_extension("json"),
        _ => path.to_path_buf(),
    };
    let content = fs::read_to_string(&resolved_path).with_context(|| {
        format!(
            "failed to read integration report json `{}`",
            resolved_path.display()
        )
    })?;
    serde_json::from_str(&content).with_context(|| {
        format!(
            "failed to parse integration report json `{}`",
            resolved_path.display()
        )
    })
}

pub fn read_integration_baseline_registry_document(
    workspace_root: &Path,
) -> Result<IntegrationBaselineRegistryDocument> {
    let path = resolve_integration_baseline_registry_path(workspace_root);
    if !path.exists() {
        return Ok(IntegrationBaselineRegistryDocument {
            schema_version: 1,
            generated_at: String::new(),
            baselines: Vec::new(),
        });
    }
    let content = fs::read_to_string(&path)
        .with_context(|| format!("failed to read baseline registry `{}`", path.display()))?;
    serde_json::from_str(&content)
        .with_context(|| format!("failed to parse baseline registry `{}`", path.display()))
}

pub fn write_integration_baseline_registry_document(
    workspace_root: &Path,
    document: &IntegrationBaselineRegistryDocument,
) -> Result<PathBuf> {
    let reports_dir = resolve_integration_reports_dir(workspace_root);
    fs::create_dir_all(&reports_dir).with_context(|| {
        format!(
            "failed to prepare integration report directory `{}`",
            reports_dir.display()
        )
    })?;

    let path = resolve_integration_baseline_registry_path(workspace_root);
    let payload = serde_json::to_string_pretty(document)
        .context("failed to render baseline registry json")?;
    fs::write(&path, format!("{payload}\n"))
        .with_context(|| format!("failed to write baseline registry `{}`", path.display()))?;
    Ok(path)
}

pub fn read_integration_decision_history_document(
    workspace_root: &Path,
) -> Result<IntegrationDecisionHistoryDocument> {
    let path = resolve_integration_decision_history_path(workspace_root);
    if !path.exists() {
        return Ok(IntegrationDecisionHistoryDocument {
            schema_version: 1,
            generated_at: String::new(),
            decisions: Vec::new(),
        });
    }
    let content = fs::read_to_string(&path)
        .with_context(|| format!("failed to read decision history `{}`", path.display()))?;
    serde_json::from_str(&content)
        .with_context(|| format!("failed to parse decision history `{}`", path.display()))
}

pub fn write_integration_decision_history_document(
    workspace_root: &Path,
    document: &IntegrationDecisionHistoryDocument,
) -> Result<PathBuf> {
    let reports_dir = resolve_integration_reports_dir(workspace_root);
    fs::create_dir_all(&reports_dir).with_context(|| {
        format!(
            "failed to prepare integration report directory `{}`",
            reports_dir.display()
        )
    })?;

    let path = resolve_integration_decision_history_path(workspace_root);
    let payload =
        serde_json::to_string_pretty(document).context("failed to render decision history json")?;
    fs::write(&path, format!("{payload}\n"))
        .with_context(|| format!("failed to write decision history `{}`", path.display()))?;
    Ok(path)
}

pub fn read_integration_evidence_policy_pack(path: &Path) -> Result<IntegrationEvidencePolicyPack> {
    let gate = read_integration_evidence_gate_policy(path)?;
    Ok(IntegrationEvidencePolicyPack {
        schema_version: 1,
        name: gate.name.clone(),
        version: gate.version.clone(),
        description: gate.description.clone(),
        platforms: gate.platforms.clone(),
        gate: gate.clone(),
        promotion: IntegrationEvidencePromotionPolicy {
            allowed_verdicts: gate.allowed_verdicts.clone(),
            allowed_candidate_readiness: gate.allowed_candidate_readiness.clone(),
            require_non_regression: gate.require_no_regression,
            require_no_blocking_changes: gate.require_no_blocking_changes,
            require_no_next_steps: gate.require_no_next_steps,
        },
    })
}

pub fn collect_repo_owned_integration_policy_packs()
-> Result<Vec<(PathBuf, IntegrationEvidencePolicyPack)>> {
    let policy_dir = repo_owned_integration_policy_dir();
    if !policy_dir.exists() {
        return Ok(Vec::new());
    }

    let mut entries = fs::read_dir(&policy_dir)
        .with_context(|| format!("failed to read policy directory `{}`", policy_dir.display()))?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .and_then(|value| value.to_str())
                .is_some_and(|value| value.eq_ignore_ascii_case("json"))
        })
        .map(|path| {
            let pack = read_integration_evidence_policy_pack(&path)?;
            Ok((path, pack))
        })
        .collect::<Result<Vec<_>>>()?;

    entries.sort_by(|left, right| left.0.cmp(&right.0));
    Ok(entries)
}

pub fn resolve_integration_evidence_policy_pack(
    reference: Option<&str>,
) -> Result<(PathBuf, IntegrationEvidencePolicyPack)> {
    let candidate_path = match reference {
        None => repo_owned_integration_policy_dir().join("default.json"),
        Some(reference) => {
            let path = PathBuf::from(reference);
            if path.exists() {
                path
            } else {
                repo_owned_integration_policy_dir().join(format!("{reference}.json"))
            }
        }
    };

    let resolved_path = fs::canonicalize(&candidate_path).with_context(|| {
        format!(
            "failed to resolve evidence policy pack `{}`",
            candidate_path.display()
        )
    })?;
    let pack = read_integration_evidence_policy_pack(&resolved_path)?;
    Ok((resolved_path, pack))
}

pub fn resolve_integration_evidence_policy(
    reference: Option<&str>,
    platform_hint: Option<&str>,
) -> Result<ResolvedIntegrationEvidencePolicy> {
    let (resolved_path, pack) = match reference {
        Some(_) => resolve_integration_evidence_policy_pack(reference)?,
        None => {
            let policy_dir = repo_owned_integration_policy_dir();
            let platform_path = platform_hint
                .map(|platform| policy_dir.join(format!("{platform}.json")))
                .filter(|path| path.exists());
            let path = platform_path.unwrap_or_else(|| policy_dir.join("default.json"));
            resolve_integration_evidence_policy_pack(Some(path.to_string_lossy().as_ref()))?
        }
    };
    Ok(ResolvedIntegrationEvidencePolicy {
        source: resolved_path.display().to_string(),
        policy: pack,
    })
}

pub fn effective_gate_policy_for_platform(
    pack: &IntegrationEvidencePolicyPack,
    platform: &str,
) -> IntegrationEvidenceGatePolicy {
    if pack.platforms.is_empty()
        || pack
            .platforms
            .iter()
            .any(|value| value == "*" || value == platform)
    {
        pack.gate.clone()
    } else {
        default_integration_evidence_gate_policy()
    }
}

pub fn default_integration_evidence_gate_policy() -> IntegrationEvidenceGatePolicy {
    IntegrationEvidenceGatePolicy {
        name: default_policy_name(),
        version: default_policy_version(),
        description: "Default local governance policy for integration evidence.".to_string(),
        platforms: default_all_platforms(),
        fail_on_readiness_regression: true,
        blocking_check_labels: vec![
            "target_root".to_string(),
            "target_files".to_string(),
            "target_content_sync".to_string(),
            "materialized_paths".to_string(),
            "launcher_probe".to_string(),
            "codex_config_shape".to_string(),
            "claude_project_shape".to_string(),
            "claude_pack_shape".to_string(),
            "openclaw_bundle_shape".to_string(),
        ],
        warning_check_labels: vec![
            "repo_templates".to_string(),
            "bridge_script".to_string(),
            "python3".to_string(),
        ],
        ignorable_check_labels: Vec::new(),
        allowed_verdicts: default_allowed_verdicts(),
        allowed_candidate_readiness: default_promotable_readinesses(),
        require_no_regression: default_true(),
        require_no_blocking_changes: default_true(),
        require_no_next_steps: default_true(),
    }
}

pub fn read_integration_evidence_gate_policy(path: &Path) -> Result<IntegrationEvidenceGatePolicy> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("failed to read evidence gate policy `{}`", path.display()))?;
    serde_json::from_str(&content)
        .with_context(|| format!("failed to parse evidence gate policy `{}`", path.display()))
}

pub fn build_integration_evidence_explain(
    platform: &str,
    target: &str,
    checks: &[IntegrationReportCheckRecord],
    next_steps: &[String],
) -> Vec<IntegrationEvidenceExplainStep> {
    let failing_checks = checks
        .iter()
        .filter(|check| check.readiness != "ready")
        .cloned()
        .collect::<Vec<_>>();
    let recheck = format!(
        "Rerun `agent-exporter doctor integrations --platform {platform} --target {target}` and, if needed, save a fresh report before comparing again."
    );

    if !next_steps.is_empty() {
        return next_steps
            .iter()
            .enumerate()
            .map(|(index, step)| {
                let related_check = failing_checks
                    .get(index)
                    .or_else(|| failing_checks.first())
                    .cloned();
                let why = related_check
                    .map(|check| {
                        format!(
                            "因为 `{}` 当前是 `{}`：{}。",
                            check.label, check.readiness, check.detail
                        )
                    })
                    .unwrap_or_else(|| "因为这是当前 evidence 已记录的下一步。".to_string());
                IntegrationEvidenceExplainStep {
                    title: step.clone(),
                    why,
                    recheck: recheck.clone(),
                }
            })
            .collect();
    }

    failing_checks
        .into_iter()
        .map(|check| IntegrationEvidenceExplainStep {
            title: default_step_for_check(&check.label),
            why: format!(
                "因为 `{}` 当前是 `{}`：{}。",
                check.label, check.readiness, check.detail
            ),
            recheck: recheck.clone(),
        })
        .collect()
}

pub fn diff_integration_reports(
    left: &IntegrationReportJsonDocument,
    right: &IntegrationReportJsonDocument,
) -> IntegrationEvidenceDiff {
    let left_checks = combined_check_map(left);
    let right_checks = combined_check_map(right);

    let mut labels = left_checks
        .keys()
        .chain(right_checks.keys())
        .cloned()
        .collect::<Vec<_>>();
    labels.sort();
    labels.dedup();

    let changed_checks = labels
        .into_iter()
        .filter_map(|label| {
            let left_check = left_checks.get(&label);
            let right_check = right_checks.get(&label);
            let left_readiness = left_check.map(|check| check.readiness.clone());
            let right_readiness = right_check.map(|check| check.readiness.clone());
            let left_detail = left_check.map(|check| check.detail.clone());
            let right_detail = right_check.map(|check| check.detail.clone());

            if left_readiness == right_readiness && left_detail == right_detail {
                None
            } else {
                Some(IntegrationEvidenceCheckDiff {
                    label,
                    left_readiness,
                    right_readiness,
                    left_detail,
                    right_detail,
                })
            }
        })
        .collect();

    let added_next_steps = right
        .next_steps
        .iter()
        .filter(|step| !left.next_steps.contains(step))
        .cloned()
        .collect();
    let removed_next_steps = left
        .next_steps
        .iter()
        .filter(|step| !right.next_steps.contains(step))
        .cloned()
        .collect();

    IntegrationEvidenceDiff {
        platform: right.platform.clone(),
        target: right.target.clone(),
        left_title: left.title.clone(),
        right_title: right.title.clone(),
        left_generated_at: left.generated_at.clone(),
        right_generated_at: right.generated_at.clone(),
        left_readiness: left.readiness.clone(),
        right_readiness: right.readiness.clone(),
        changed_checks,
        added_next_steps,
        removed_next_steps,
    }
}

pub fn gate_integration_reports(
    baseline: &IntegrationReportJsonDocument,
    candidate: &IntegrationReportJsonDocument,
    policy: &IntegrationEvidenceGatePolicy,
) -> Result<IntegrationEvidenceGateOutcome> {
    if baseline.platform != candidate.platform || baseline.target != candidate.target {
        bail!(
            "baseline and candidate must share the same platform and target before gate can produce a verdict"
        );
    }

    let diff = diff_integration_reports(baseline, candidate);
    let regression = readiness_rank(&candidate.readiness) < readiness_rank(&baseline.readiness);

    let mut blocking_changes = Vec::new();
    let mut warning_changes = Vec::new();
    let mut ignored_changes = Vec::new();

    for change in diff.changed_checks {
        if policy
            .ignorable_check_labels
            .iter()
            .any(|label| label == &change.label)
        {
            ignored_changes.push(change);
            continue;
        }

        let worsened = check_change_worsened(&change);
        if worsened
            && policy
                .blocking_check_labels
                .iter()
                .any(|label| label == &change.label)
        {
            blocking_changes.push(change);
        } else if worsened
            || policy
                .warning_check_labels
                .iter()
                .any(|label| label == &change.label)
        {
            warning_changes.push(change);
        } else {
            ignored_changes.push(change);
        }
    }

    if !diff.added_next_steps.is_empty() && blocking_changes.is_empty() {
        for step in &diff.added_next_steps {
            warning_changes.push(IntegrationEvidenceCheckDiff {
                label: format!("next_step::{step}"),
                left_readiness: None,
                right_readiness: Some("added".to_string()),
                left_detail: None,
                right_detail: Some(step.clone()),
            });
        }
    }

    let verdict =
        if (policy.fail_on_readiness_regression && regression) || !blocking_changes.is_empty() {
            IntegrationEvidenceGateVerdict::Fail
        } else if !warning_changes.is_empty() {
            IntegrationEvidenceGateVerdict::Warn
        } else {
            IntegrationEvidenceGateVerdict::Pass
        };

    Ok(IntegrationEvidenceGateOutcome {
        verdict,
        baseline_title: baseline.title.clone(),
        candidate_title: candidate.title.clone(),
        baseline_generated_at: baseline.generated_at.clone(),
        candidate_generated_at: candidate.generated_at.clone(),
        baseline_readiness: baseline.readiness.clone(),
        candidate_readiness: candidate.readiness.clone(),
        regression,
        blocking_changes,
        warning_changes,
        ignored_changes,
        remediation_steps: build_integration_evidence_explain(
            &candidate.platform,
            &candidate.target,
            &combined_check_records(candidate),
            &candidate.next_steps,
        ),
    })
}

pub fn assess_promotion_eligibility(
    outcome: &IntegrationEvidenceGateOutcome,
    candidate: &IntegrationReportJsonDocument,
    pack: &IntegrationEvidencePolicyPack,
) -> IntegrationPromotionAssessment {
    let mut reasons = Vec::new();

    if !pack
        .promotion
        .allowed_verdicts
        .iter()
        .any(|value| value == outcome.verdict.as_str())
    {
        reasons.push(format!(
            "verdict `{}` is not promotable under policy `{}`",
            outcome.verdict.as_str(),
            pack.name
        ));
    }

    if !pack
        .promotion
        .allowed_candidate_readiness
        .iter()
        .any(|value| value == &candidate.readiness)
    {
        reasons.push(format!(
            "candidate readiness `{}` is not promotable under policy `{}`",
            candidate.readiness, pack.name
        ));
    }

    if pack.promotion.require_non_regression && outcome.regression {
        reasons.push("candidate regressed readiness relative to the official baseline".to_string());
    }

    let eligible = reasons.is_empty();
    let summary = if eligible {
        format!(
            "candidate is eligible for promotion under policy `{}`",
            pack.name
        )
    } else {
        reasons
            .first()
            .cloned()
            .unwrap_or_else(|| "candidate is not eligible for promotion".to_string())
    };

    IntegrationPromotionAssessment {
        eligible,
        reasons,
        summary,
    }
}

pub fn build_baseline_record(
    workspace_root: &Path,
    baseline_name: &str,
    source_report_path: &Path,
    report: &IntegrationReportJsonDocument,
    promoted_at: &str,
    promoted_from_verdict: &str,
    policy: &IntegrationEvidenceGatePolicy,
    note: Option<String>,
) -> IntegrationBaselineRecord {
    let reports_dir = resolve_integration_reports_dir(workspace_root);
    let report_json_path = reports_dir.join(&report.artifact_links.json_report);
    let report_html_path = reports_dir.join(&report.artifact_links.html_report);

    IntegrationBaselineRecord {
        name: baseline_name.to_string(),
        platform: report.platform.clone(),
        target: report.target.clone(),
        target_identity: integration_target_identity(&report.platform, &report.target),
        report_title: report.title.clone(),
        report_json_path: display_relative_or_absolute(workspace_root, &report_json_path),
        report_html_path: display_relative_or_absolute(workspace_root, &report_html_path),
        promoted_at: promoted_at.to_string(),
        promoted_from_verdict: promoted_from_verdict.to_string(),
        policy_name: policy.name.clone(),
        policy_version: policy.version.clone(),
        note: note.or_else(|| {
            Some(format!(
                "source report: {}",
                display_relative_or_absolute(workspace_root, source_report_path)
            ))
        }),
    }
}

pub fn build_decision_record(
    workspace_root: &Path,
    baseline_name: &str,
    baseline_record: Option<&IntegrationBaselineRecord>,
    candidate_report_path: &Path,
    candidate: &IntegrationReportJsonDocument,
    policy: &ResolvedIntegrationEvidencePolicy,
    verdict: &str,
    promoted: bool,
    decided_at: &str,
    summary: &str,
    note: Option<String>,
) -> IntegrationDecisionRecord {
    let reports_dir = resolve_integration_reports_dir(workspace_root);
    let candidate_json_path = reports_dir.join(&candidate.artifact_links.json_report);

    IntegrationDecisionRecord {
        baseline_name: baseline_name.to_string(),
        platform: candidate.platform.clone(),
        target: candidate.target.clone(),
        target_identity: integration_target_identity(&candidate.platform, &candidate.target),
        baseline_report_json_path: baseline_record.map(|entry| entry.report_json_path.clone()),
        baseline_report_title: baseline_record.map(|entry| entry.report_title.clone()),
        candidate_report_json_path: display_relative_or_absolute(
            workspace_root,
            &candidate_json_path,
        ),
        candidate_report_title: candidate.title.clone(),
        policy_name: policy.policy.name.clone(),
        policy_version: policy.policy.version.clone(),
        verdict: verdict.to_string(),
        promoted,
        decided_at: decided_at.to_string(),
        summary: note.unwrap_or_else(|| {
            format!(
                "{summary}; source report: {}",
                display_relative_or_absolute(workspace_root, candidate_report_path)
            )
        }),
    }
}

pub fn upsert_integration_baseline_record(
    document: &mut IntegrationBaselineRegistryDocument,
    record: IntegrationBaselineRecord,
) {
    if let Some(existing) = document
        .baselines
        .iter_mut()
        .find(|baseline| baseline.name == record.name)
    {
        *existing = record;
    } else {
        document.baselines.push(record);
    }
    document.baselines.sort_by(|left, right| {
        right
            .promoted_at
            .cmp(&left.promoted_at)
            .then_with(|| left.name.cmp(&right.name))
    });
}

pub fn append_integration_decision_record(
    document: &mut IntegrationDecisionHistoryDocument,
    record: IntegrationDecisionRecord,
) {
    document.decisions.push(record);
    document.decisions.sort_by(|left, right| {
        right
            .decided_at
            .cmp(&left.decided_at)
            .then_with(|| left.baseline_name.cmp(&right.baseline_name))
    });
}

pub fn find_integration_baseline_record<'a>(
    document: &'a IntegrationBaselineRegistryDocument,
    name: &str,
) -> Option<&'a IntegrationBaselineRecord> {
    document.baselines.iter().find(|record| record.name == name)
}

pub fn find_integration_baseline_for_identity<'a>(
    document: &'a IntegrationBaselineRegistryDocument,
    platform: &str,
    target: &str,
) -> Option<&'a IntegrationBaselineRecord> {
    let identity = integration_target_identity(platform, target);
    document
        .baselines
        .iter()
        .filter(|record| record.target_identity == identity)
        .max_by(|left, right| left.promoted_at.cmp(&right.promoted_at))
}

pub fn latest_integration_decision_for_candidate<'a>(
    document: &'a IntegrationDecisionHistoryDocument,
    candidate_report_json_path: &str,
) -> Option<&'a IntegrationDecisionRecord> {
    document
        .decisions
        .iter()
        .find(|record| record.candidate_report_json_path == candidate_report_json_path)
}

fn combined_check_map<'a>(
    report: &'a IntegrationReportJsonDocument,
) -> BTreeMap<String, &'a IntegrationReportCheckRecord> {
    let mut checks = BTreeMap::new();
    for check in report.pack_shape_checks.iter().chain(report.checks.iter()) {
        checks.insert(check.label.clone(), check);
    }
    checks
}

fn combined_check_records(
    report: &IntegrationReportJsonDocument,
) -> Vec<IntegrationReportCheckRecord> {
    let mut checks = BTreeMap::new();
    for check in report.pack_shape_checks.iter().chain(report.checks.iter()) {
        checks.insert(check.label.clone(), check.clone());
    }
    checks.into_values().collect()
}

fn readiness_rank(value: &str) -> i32 {
    match value {
        "ready" => 2,
        "partial" => 1,
        "missing" => 0,
        _ => -1,
    }
}

fn check_change_worsened(change: &IntegrationEvidenceCheckDiff) -> bool {
    match (
        change.left_readiness.as_deref(),
        change.right_readiness.as_deref(),
    ) {
        (Some(left), Some(right)) => readiness_rank(right) < readiness_rank(left),
        (Some(_), None) => true,
        _ => false,
    }
}

fn default_step_for_check(label: &str) -> String {
    match label {
        "target_root" => "Point the workflow at an explicit staging target.".to_string(),
        "target_files" => "Materialize the platform pack into the staging target.".to_string(),
        "target_content_sync" => {
            "Refresh stale materialized files from the current repo-generated pack.".to_string()
        }
        "materialized_paths" => {
            "Replace placeholder or generic launcher paths with repo-local launcher/script paths."
                .to_string()
        }
        "launcher_probe" => {
            "Build or expose a repo-local binary so launcher probe can reach ready.".to_string()
        }
        "codex_config_shape" => {
            "Repair `.codex/config.toml` so `command` and non-empty `args` are both present."
                .to_string()
        }
        "claude_project_shape" => {
            "Repair `.mcp.json` so the project-scoped MCP config parses and points at the bridge."
                .to_string()
        }
        "claude_pack_shape" => {
            "Repair `CLAUDE.md` and `.claude/commands/*` so the Claude pack shape is complete."
                .to_string()
        }
        "openclaw_bundle_shape" => {
            "Repair the OpenClaw bundle/plugin manifests and `.mcp.json` before calling it ready."
                .to_string()
        }
        "repo_templates" => "Restore missing repo-owned integration template assets.".to_string(),
        "bridge_script" => "Restore `scripts/agent_exporter_mcp.py` in this repo.".to_string(),
        "python3" => "Ensure `python3` is available for the MCP bridge.".to_string(),
        other => format!("Fix the `{other}` check until it reaches ready."),
    }
}

fn display_relative_or_absolute(workspace_root: &Path, path: &Path) -> String {
    match path.strip_prefix(workspace_root) {
        Ok(relative) => relative.display().to_string(),
        Err(_) => path.display().to_string(),
    }
}

fn slugify(value: &str) -> String {
    let mut slug = String::new();
    let mut last_was_dash = false;

    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
            last_was_dash = false;
        } else if !last_was_dash && !slug.is_empty() {
            slug.push('-');
            last_was_dash = true;
        }
    }

    while slug.ends_with('-') {
        slug.pop();
    }

    if slug.is_empty() {
        "report".to_string()
    } else {
        slug
    }
}

fn read_integration_report_entry(path: PathBuf) -> Result<IntegrationReportEntry> {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .map(str::to_string)
        .unwrap_or_else(|| path.display().to_string());
    let content = fs::read_to_string(&path)
        .with_context(|| format!("failed to read integration report `{}`", path.display()))?;

    Ok(IntegrationReportEntry {
        file_name: file_name.clone(),
        relative_href: file_name,
        title: extract_meta_value(&content, "report-title")
            .or_else(|| extract_title(&content))
            .unwrap_or_else(|| "Integration evidence report".to_string()),
        report_kind: extract_meta_value(&content, "report-kind"),
        platform: extract_meta_value(&content, "integration-platform"),
        readiness: extract_meta_value(&content, "integration-readiness"),
        target_root: extract_meta_value(&content, "integration-target"),
        generated_at: extract_meta_value(&content, "generated-at"),
    })
}

fn extract_meta_value(content: &str, key: &str) -> Option<String> {
    let needle = format!("name=\"agent-exporter:{key}\" content=\"");
    let start = content.find(&needle)? + needle.len();
    let tail = &content[start..];
    let end = tail.find('"')?;
    Some(unescape_html(&tail[..end]))
}

fn extract_title(content: &str) -> Option<String> {
    let start_tag = "<title>";
    let end_tag = "</title>";
    let start = content.find(start_tag)? + start_tag.len();
    let tail = &content[start..];
    let end = tail.find(end_tag)?;
    Some(unescape_html(&tail[..end]))
}

fn unescape_html(value: &str) -> String {
    value
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::{
        IntegrationArtifactLinks, IntegrationBaselineRecord, IntegrationDecisionHistoryDocument,
        IntegrationDecisionRecord, IntegrationEvidenceGateVerdict, IntegrationReportCheckRecord,
        IntegrationReportJsonDocument, assess_promotion_eligibility,
        build_integration_evidence_explain, collect_integration_report_entries,
        collect_integration_report_json_documents, collect_repo_owned_integration_policy_packs,
        default_integration_evidence_gate_policy, diff_integration_reports,
        gate_integration_reports, integration_target_identity,
        read_integration_baseline_registry_document, read_integration_decision_history_document,
        read_integration_evidence_policy_pack, resolve_integration_baseline_registry_path,
        resolve_integration_decision_history_path, resolve_integration_reports_dir,
        write_integration_baseline_registry_document, write_integration_decision_history_document,
        write_integration_report_document, write_integration_report_json_document,
    };

    fn sample_report(readiness: &str, label_readiness: &str) -> IntegrationReportJsonDocument {
        IntegrationReportJsonDocument {
            schema_version: 1,
            title: "Codex doctor".to_string(),
            kind: "doctor".to_string(),
            platform: "codex".to_string(),
            target: "/tmp/codex-pack".to_string(),
            generated_at: "2026-04-06T12:00:00Z".to_string(),
            readiness: readiness.to_string(),
            summary: readiness.to_string(),
            launcher_status: "ready".to_string(),
            launcher_kind: "repo-local-debug".to_string(),
            launcher_command: "/tmp/agent-exporter".to_string(),
            bridge_status: "ready".to_string(),
            pack_shape_checks: vec![IntegrationReportCheckRecord {
                label: "codex_config_shape".to_string(),
                readiness: label_readiness.to_string(),
                detail: "shape".to_string(),
            }],
            checks: Vec::new(),
            next_steps: vec!["restore codex args".to_string()],
            written_files: Vec::new(),
            unchanged_files: Vec::new(),
            artifact_links: IntegrationArtifactLinks {
                html_report: "report.html".to_string(),
                json_report: "report.json".to_string(),
                index_html: "index.html".to_string(),
                index_json: "index.json".to_string(),
            },
        }
    }

    #[test]
    fn write_integration_report_document_writes_under_integration_reports_dir() {
        let workspace = tempdir().expect("workspace");
        let path = write_integration_report_document(
            workspace.path(),
            "doctor",
            "codex",
            "2026-04-06T12:00:00Z",
            "<!DOCTYPE html>",
        )
        .expect("write report");

        assert!(path.exists());
        assert!(path.starts_with(resolve_integration_reports_dir(workspace.path())));
    }

    #[test]
    fn collect_integration_report_entries_reads_report_meta_tags() {
        let workspace = tempdir().expect("workspace");
        let reports_dir = resolve_integration_reports_dir(workspace.path());
        std::fs::create_dir_all(&reports_dir).expect("mkdirs");
        std::fs::write(
            reports_dir.join("integration-report-doctor-codex-demo.html"),
            concat!(
                "<!DOCTYPE html><html><head>",
                "<title>Codex doctor report</title>",
                "<meta name=\"agent-exporter:report-title\" content=\"Codex doctor report\">",
                "<meta name=\"agent-exporter:report-kind\" content=\"doctor\">",
                "<meta name=\"agent-exporter:integration-platform\" content=\"codex\">",
                "<meta name=\"agent-exporter:integration-readiness\" content=\"ready\">",
                "<meta name=\"agent-exporter:integration-target\" content=\"/tmp/codex-pack\">",
                "<meta name=\"agent-exporter:generated-at\" content=\"2026-04-06T12:00:00Z\">",
                "</head><body></body></html>"
            ),
        )
        .expect("write report");

        let entries =
            collect_integration_report_entries(workspace.path()).expect("collect reports");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].platform.as_deref(), Some("codex"));
    }

    #[test]
    fn collect_integration_report_json_documents_ignores_governance_manifests() {
        let workspace = tempdir().expect("workspace");
        let reports_dir = resolve_integration_reports_dir(workspace.path());
        std::fs::create_dir_all(&reports_dir).expect("mkdirs");
        write_integration_report_json_document(
            workspace.path(),
            "doctor",
            "codex",
            "2026-04-06T12:00:00Z",
            &sample_report("ready", "ready"),
        )
        .expect("write report");
        std::fs::write(
            resolve_integration_baseline_registry_path(workspace.path()),
            r#"{"schema_version":1,"generated_at":"","baselines":[]}"#,
        )
        .expect("write registry");
        std::fs::write(
            resolve_integration_decision_history_path(workspace.path()),
            r#"{"schema_version":1,"generated_at":"","decisions":[]}"#,
        )
        .expect("write history");

        let entries =
            collect_integration_report_json_documents(workspace.path()).expect("collect reports");
        assert_eq!(entries.len(), 1);
    }

    #[test]
    fn write_and_read_governance_documents() {
        let workspace = tempdir().expect("workspace");
        write_integration_baseline_registry_document(
            workspace.path(),
            &super::IntegrationBaselineRegistryDocument {
                schema_version: 1,
                generated_at: "2026-04-06T12:00:00Z".to_string(),
                baselines: vec![IntegrationBaselineRecord {
                    name: "codex-main".to_string(),
                    platform: "codex".to_string(),
                    target: "/tmp/codex-pack".to_string(),
                    target_identity: integration_target_identity("codex", "/tmp/codex-pack"),
                    report_title: "Codex doctor".to_string(),
                    report_json_path: "/tmp/report.json".to_string(),
                    report_html_path: "/tmp/report.html".to_string(),
                    promoted_at: "2026-04-06T12:00:00Z".to_string(),
                    promoted_from_verdict: "pass".to_string(),
                    policy_name: "codex".to_string(),
                    policy_version: "1.0.0".to_string(),
                    note: Some("seed".to_string()),
                }],
            },
        )
        .expect("write registry");
        write_integration_decision_history_document(
            workspace.path(),
            &IntegrationDecisionHistoryDocument {
                schema_version: 1,
                generated_at: "2026-04-06T12:00:00Z".to_string(),
                decisions: vec![IntegrationDecisionRecord {
                    baseline_name: "codex-main".to_string(),
                    platform: "codex".to_string(),
                    target: "/tmp/codex-pack".to_string(),
                    target_identity: integration_target_identity("codex", "/tmp/codex-pack"),
                    baseline_report_json_path: None,
                    baseline_report_title: None,
                    candidate_report_json_path: "/tmp/report.json".to_string(),
                    candidate_report_title: "Codex doctor".to_string(),
                    policy_name: "codex".to_string(),
                    policy_version: "1.0.0".to_string(),
                    verdict: "pass".to_string(),
                    promoted: true,
                    decided_at: "2026-04-06T12:00:00Z".to_string(),
                    summary: "seed".to_string(),
                }],
            },
        )
        .expect("write history");

        assert_eq!(
            read_integration_baseline_registry_document(workspace.path())
                .expect("read registry")
                .baselines
                .len(),
            1
        );
        assert_eq!(
            read_integration_decision_history_document(workspace.path())
                .expect("read history")
                .decisions
                .len(),
            1
        );
    }

    #[test]
    fn collect_repo_owned_policies_reads_json_files() {
        let policies = collect_repo_owned_integration_policy_packs().expect("policy packs");
        assert!(policies.iter().any(|(_, pack)| pack.name == "default"));
        assert!(policies.iter().any(|(_, pack)| pack.name == "codex"));
    }

    #[test]
    fn read_policy_pack_parses_flattened_gate_and_promotion_fields() {
        let path = super::repo_owned_integration_policy_dir().join("codex.json");
        let pack = read_integration_evidence_policy_pack(&path).expect("read codex policy");
        assert_eq!(pack.name, "codex");
        assert!(
            pack.gate
                .blocking_check_labels
                .contains(&"codex_config_shape".to_string())
        );
        assert_eq!(pack.allowed_verdicts, vec!["pass".to_string()]);
    }

    #[test]
    fn diff_integration_reports_reports_readiness_and_next_step_changes() {
        let mut left = sample_report("ready", "ready");
        left.next_steps = vec!["nothing to do".to_string()];
        let right = sample_report("partial", "partial");

        let diff = diff_integration_reports(&left, &right);
        assert_eq!(diff.left_readiness, "ready");
        assert_eq!(diff.right_readiness, "partial");
        assert_eq!(diff.changed_checks.len(), 1);
        assert_eq!(
            diff.added_next_steps,
            vec!["restore codex args".to_string()]
        );
        assert_eq!(diff.removed_next_steps, vec!["nothing to do".to_string()]);
    }

    #[test]
    fn gate_integration_reports_fails_on_blocking_regression() {
        let baseline = sample_report("ready", "ready");
        let candidate = sample_report("partial", "partial");

        let outcome = gate_integration_reports(
            &baseline,
            &candidate,
            &default_integration_evidence_gate_policy(),
        )
        .expect("comparable gate");
        assert_eq!(outcome.verdict, IntegrationEvidenceGateVerdict::Fail);
        assert!(outcome.regression);
        assert_eq!(outcome.blocking_changes.len(), 1);
    }

    #[test]
    fn gate_integration_reports_rejects_mixed_platform_or_target_pairs() {
        let baseline = sample_report("ready", "ready");
        let mut candidate = sample_report("ready", "ready");
        candidate.platform = "claude-code".to_string();
        candidate.target = "/tmp/claude-pack".to_string();

        let error = gate_integration_reports(
            &baseline,
            &candidate,
            &default_integration_evidence_gate_policy(),
        )
        .expect_err("mixed pair should fail");
        assert!(
            error
                .to_string()
                .contains("must share the same platform and target")
        );
    }

    #[test]
    fn assess_promotion_eligibility_rejects_warn_or_regression() {
        let baseline = sample_report("ready", "ready");
        let candidate = sample_report("partial", "partial");
        let outcome = gate_integration_reports(
            &baseline,
            &candidate,
            &default_integration_evidence_gate_policy(),
        )
        .expect("gate");
        let (_, pack) =
            super::resolve_integration_evidence_policy_pack(Some("codex")).expect("resolve policy");
        let assessment = assess_promotion_eligibility(&outcome, &candidate, &pack);
        assert!(!assessment.eligible);
        assert!(!assessment.reasons.is_empty());
    }

    #[test]
    fn build_integration_evidence_explain_prefers_recorded_next_steps() {
        let steps = build_integration_evidence_explain(
            "codex",
            "/tmp/codex-pack",
            &[IntegrationReportCheckRecord {
                label: "codex_config_shape".to_string(),
                readiness: "partial".to_string(),
                detail: "args missing".to_string(),
            }],
            &["restore codex args".to_string()],
        );
        assert_eq!(steps[0].title, "restore codex args");
    }

    #[test]
    fn upsert_integration_baseline_record_replaces_by_name() {
        let mut document = super::IntegrationBaselineRegistryDocument {
            schema_version: 1,
            generated_at: "2026-04-06T12:00:00Z".to_string(),
            baselines: vec![IntegrationBaselineRecord {
                name: "codex-main".to_string(),
                platform: "codex".to_string(),
                target: "/tmp/codex-pack".to_string(),
                target_identity: integration_target_identity("codex", "/tmp/codex-pack"),
                report_title: "old".to_string(),
                report_json_path: "/tmp/old.json".to_string(),
                report_html_path: "/tmp/old.html".to_string(),
                promoted_at: "2026-04-06T12:00:00Z".to_string(),
                promoted_from_verdict: "pass".to_string(),
                policy_name: "codex".to_string(),
                policy_version: "1.0.0".to_string(),
                note: None,
            }],
        };

        super::upsert_integration_baseline_record(
            &mut document,
            IntegrationBaselineRecord {
                name: "codex-main".to_string(),
                platform: "codex".to_string(),
                target: "/tmp/codex-pack".to_string(),
                target_identity: integration_target_identity("codex", "/tmp/codex-pack"),
                report_title: "new".to_string(),
                report_json_path: "/tmp/new.json".to_string(),
                report_html_path: "/tmp/new.html".to_string(),
                promoted_at: "2026-04-06T12:10:00Z".to_string(),
                promoted_from_verdict: "pass".to_string(),
                policy_name: "codex".to_string(),
                policy_version: "1.0.0".to_string(),
                note: None,
            },
        );

        assert_eq!(document.baselines.len(), 1);
        assert_eq!(document.baselines[0].report_title, "new");
    }
}
