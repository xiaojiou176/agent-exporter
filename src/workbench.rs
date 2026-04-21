use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use anyhow::Result;
use chrono::Utc;

use crate::core::archive_index;
use crate::core::integration_report::{
    collect_integration_report_entries, collect_integration_report_json_documents,
    effective_gate_policy_for_platform, find_integration_baseline_for_identity,
    gate_integration_reports, latest_integration_decision_for_candidate,
    read_integration_baseline_registry_document, read_integration_decision_history_document,
    read_integration_report_json_document, resolve_integration_evidence_policy_pack,
    write_integration_reports_index_document, write_integration_reports_index_json_document,
};
use crate::core::search_report::{
    collect_search_report_entries, write_search_reports_index_document,
};
use crate::core::semantic_search::FastEmbedSemanticEmbedder;
use crate::core::workbench::{
    OfficialAnswerPinRecord, OfficialAnswerPinRequest, ShareSafePacketVariant,
    WorkbenchRefreshManifestDocument, build_action_packs, build_official_answer_pin,
    build_team_memory_lane_document, build_workbench_index_document,
    collect_structured_summary_documents, read_official_answer_pin_registry_document,
    read_refresh_manifest_document, remove_official_answer_pin, render_share_safe_packet_markdown,
    render_share_safe_packet_markdown_for_variant, resolve_action_packs_index_html_path,
    resolve_archive_index_json_path, resolve_fleet_view_html_path, resolve_fleet_view_json_path,
    resolve_memory_lane_html_path, resolve_memory_lane_json_path, resolve_official_answer_pin,
    resolve_refresh_manifest_path, upsert_official_answer_pin, write_action_pack_documents,
    write_action_packs_index_json_document, write_archive_index_json_document,
    write_fleet_view_json_document, write_memory_lane_json_document,
    write_refresh_manifest_document, write_share_safe_packet_document,
    write_share_safe_packet_variant_document,
};
use crate::output::archive_index::{self as archive_index_output, render_archive_index_document};
use crate::output::integration_report::{
    build_integration_reports_index_json_document, render_integration_reports_index_document,
};
use crate::output::search_report::render_search_reports_index_document;
use crate::output::workbench::{
    render_action_packs_index_document, render_fleet_view_document, render_memory_lane_document,
};

#[derive(Clone, Debug)]
pub struct WorkbenchRefreshOutcome {
    pub archive_index_path: PathBuf,
    pub archive_index_json_path: PathBuf,
    pub reports_index_path: PathBuf,
    pub integration_index_path: PathBuf,
    pub integration_index_json_path: PathBuf,
    pub share_safe_packet_path: PathBuf,
    pub refresh_manifest_path: PathBuf,
    pub refresh_mode: String,
    pub changed_families: usize,
    pub fleet_view_path: PathBuf,
    pub fleet_view_json_path: PathBuf,
    pub action_packs_index_path: PathBuf,
    pub action_packs_index_json_path: PathBuf,
    pub memory_lane_path: PathBuf,
    pub memory_lane_json_path: PathBuf,
}

pub fn refresh_workspace_workbench(workspace_root: &Path) -> Result<WorkbenchRefreshOutcome> {
    let archive_title = workspace_root
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| format!("{name} archive index"))
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "agent-exporter archive index".to_string());
    let generated_at = Utc::now().to_rfc3339();
    let input_fingerprint = compute_refresh_input_fingerprint(workspace_root)?;
    let previous_manifest = read_refresh_manifest_document(workspace_root)?;
    let previous_index = read_previous_workbench_index(workspace_root)?;
    if previous_manifest
        .as_ref()
        .is_some_and(|manifest| manifest.input_fingerprint == input_fingerprint)
        && all_refresh_outputs_exist(workspace_root)?
    {
        let manifest = WorkbenchRefreshManifestDocument {
            schema_version: crate::core::workbench::REFRESH_MANIFEST_SCHEMA_VERSION,
            generated_at: generated_at.clone(),
            refresh_mode: "reused".to_string(),
            input_fingerprint,
            changed_families: Vec::new(),
            changed_fleet_relations: Vec::new(),
            stale_pin_labels: previous_index
                .as_ref()
                .map(|index| {
                    index
                        .official_answers
                        .iter()
                        .filter(|pin| pin.stale)
                        .map(|pin| pin.label.clone())
                        .collect()
                })
                .unwrap_or_default(),
        };
        let refresh_manifest_path = write_refresh_manifest_document(workspace_root, &manifest)?;
        return Ok(WorkbenchRefreshOutcome {
            archive_index_path: resolve_workspace_archive_index_path(workspace_root)?,
            archive_index_json_path: resolve_archive_index_json_path(workspace_root)?,
            reports_index_path: crate::core::search_report::resolve_search_reports_dir(
                workspace_root,
            )
            .join("index.html"),
            integration_index_path:
                crate::core::integration_report::resolve_integration_reports_dir(workspace_root)
                    .join("index.html"),
            integration_index_json_path:
                crate::core::integration_report::resolve_integration_reports_dir(workspace_root)
                    .join("index.json"),
            share_safe_packet_path: crate::core::workbench::resolve_share_safe_packet_path(
                workspace_root,
            )?,
            refresh_manifest_path,
            refresh_mode: "reused".to_string(),
            changed_families: 0,
            fleet_view_path: resolve_fleet_view_html_path(workspace_root)?,
            fleet_view_json_path: resolve_fleet_view_json_path(workspace_root)?,
            action_packs_index_path: resolve_action_packs_index_html_path(workspace_root)?,
            action_packs_index_json_path:
                crate::core::workbench::resolve_action_packs_index_json_path(workspace_root)?,
            memory_lane_path: resolve_memory_lane_html_path(workspace_root)?,
            memory_lane_json_path: resolve_memory_lane_json_path(workspace_root)?,
        });
    }

    let entries = archive_index::collect_html_archive_entries(workspace_root)?;
    let reports = collect_search_report_entries(workspace_root)?;
    let integration_reports = collect_integration_report_entries(workspace_root)?;
    let integration_json_reports = collect_integration_report_json_documents(workspace_root)?;
    let summaries = collect_structured_summary_documents(workspace_root)?;
    let pins = read_official_answer_pin_registry_document(workspace_root)?;

    let workbench_index = build_workbench_index_document(
        &archive_title,
        &generated_at,
        &entries,
        &summaries,
        &reports,
        &integration_json_reports,
        &pins,
    );
    let archive_document = render_archive_index_document(
        &archive_title,
        &generated_at,
        &entries,
        &reports,
        &integration_reports,
        build_decision_desk_summary(workspace_root, &integration_json_reports).as_ref(),
        Some(&workbench_index),
    );
    let reports_document = render_search_reports_index_document(
        &format!("{archive_title} search reports"),
        &generated_at,
        &reports,
    );
    let integration_index_document = render_integration_reports_index_document(
        &format!("{archive_title} integration reports"),
        &generated_at,
        &integration_reports,
    );
    let integration_index_json_document = build_integration_reports_index_json_document(
        &format!("{archive_title} integration reports"),
        &generated_at,
        &integration_json_reports,
    );
    let action_packs = build_action_packs(
        &workbench_index.families,
        &workbench_index.official_answers,
        &workbench_index.fleet_view,
        &generated_at,
    );
    let memory_lane = build_team_memory_lane_document(
        workspace_root,
        &archive_title,
        &generated_at,
        &workbench_index,
        &action_packs,
    );

    let archive_index_path =
        archive_index::write_archive_index_document(workspace_root, &archive_document)?;
    let archive_index_json_path =
        write_archive_index_json_document(workspace_root, &workbench_index)?;
    let reports_index_path =
        write_search_reports_index_document(workspace_root, &reports_document)?;
    let integration_index_path =
        write_integration_reports_index_document(workspace_root, &integration_index_document)?;
    let integration_index_json_path = write_integration_reports_index_json_document(
        workspace_root,
        &integration_index_json_document,
    )?;
    let share_safe_packet_path = write_share_safe_packet_document(
        workspace_root,
        &render_share_safe_packet_markdown(&archive_title, &generated_at, &workbench_index),
    )?;
    for variant in [
        ShareSafePacketVariant::Teammate,
        ShareSafePacketVariant::Reviewer,
        ShareSafePacketVariant::Public,
    ] {
        write_share_safe_packet_variant_document(
            workspace_root,
            variant,
            &render_share_safe_packet_markdown_for_variant(
                &archive_title,
                &generated_at,
                &workbench_index,
                variant,
            ),
        )?;
    }
    let fleet_view_path = resolve_fleet_view_html_path(workspace_root)?;
    fs::write(
        &fleet_view_path,
        render_fleet_view_document(&archive_title, &generated_at, &workbench_index.fleet_view),
    )?;
    let fleet_view_json_path =
        write_fleet_view_json_document(workspace_root, &workbench_index.fleet_view)?;
    write_action_pack_documents(workspace_root, &action_packs.packs)?;
    let action_packs_index_path = resolve_action_packs_index_html_path(workspace_root)?;
    fs::write(
        &action_packs_index_path,
        render_action_packs_index_document(&action_packs),
    )?;
    let action_packs_index_json_path =
        write_action_packs_index_json_document(workspace_root, &action_packs)?;
    let memory_lane_path = resolve_memory_lane_html_path(workspace_root)?;
    fs::write(&memory_lane_path, render_memory_lane_document(&memory_lane))?;
    let memory_lane_json_path = write_memory_lane_json_document(workspace_root, &memory_lane)?;

    let changed_families = changed_family_keys(previous_index.as_ref(), &workbench_index);
    let changed_fleet_relations =
        changed_fleet_relations(previous_index.as_ref(), &workbench_index);
    let refresh_mode = if previous_manifest.is_some() || previous_index.is_some() {
        "delta".to_string()
    } else {
        "full".to_string()
    };
    let refresh_manifest_path = write_refresh_manifest_document(
        workspace_root,
        &WorkbenchRefreshManifestDocument {
            schema_version: crate::core::workbench::REFRESH_MANIFEST_SCHEMA_VERSION,
            generated_at: generated_at.clone(),
            refresh_mode: refresh_mode.clone(),
            input_fingerprint,
            changed_families: changed_families.clone(),
            changed_fleet_relations,
            stale_pin_labels: workbench_index
                .official_answers
                .iter()
                .filter(|pin| pin.stale)
                .map(|pin| pin.label.clone())
                .collect(),
        },
    )?;

    let _ = maybe_refresh_semantic_index(workspace_root);

    Ok(WorkbenchRefreshOutcome {
        archive_index_path,
        archive_index_json_path,
        reports_index_path,
        integration_index_path,
        integration_index_json_path,
        share_safe_packet_path,
        refresh_manifest_path,
        refresh_mode,
        changed_families: changed_families.len(),
        fleet_view_path,
        fleet_view_json_path,
        action_packs_index_path,
        action_packs_index_json_path,
        memory_lane_path,
        memory_lane_json_path,
    })
}

fn resolve_workspace_archive_index_path(workspace_root: &Path) -> Result<PathBuf> {
    Ok(archive_index::resolve_workspace_conversations_dir(workspace_root)?.join("index.html"))
}

fn workspace_like_name(path: &Path) -> bool {
    path.file_name()
        .and_then(|value| value.to_str())
        .is_some_and(|name| !name.starts_with(".tmp"))
}

pub fn pin_official_answer(
    workspace_root: &Path,
    artifact_path: &Path,
    label: &str,
    note: Option<String>,
    supersedes: Option<&str>,
) -> Result<OfficialAnswerPinRecord> {
    let pinned_at = Utc::now().to_rfc3339();
    let record = build_official_answer_pin(&OfficialAnswerPinRequest {
        workspace_root,
        artifact_path,
        label,
        note,
        pinned_at: &pinned_at,
    })?;
    upsert_official_answer_pin(workspace_root, record.clone(), &pinned_at, supersedes)?;
    Ok(record)
}

pub fn resolve_pinned_official_answer(
    workspace_root: &Path,
    label: &str,
    note: Option<String>,
) -> Result<OfficialAnswerPinRecord> {
    let resolved_at = Utc::now().to_rfc3339();
    resolve_official_answer_pin(workspace_root, label, note, &resolved_at)
}

pub fn unpin_official_answer(workspace_root: &Path, label: &str) -> Result<bool> {
    remove_official_answer_pin(workspace_root, label)
}

fn compute_refresh_input_fingerprint(workspace_root: &Path) -> Result<String> {
    let mut hasher = DefaultHasher::new();
    for path in tracked_refresh_inputs(workspace_root)? {
        path.display().to_string().hash(&mut hasher);
        let metadata = fs::metadata(&path)?;
        metadata.len().hash(&mut hasher);
        metadata
            .modified()
            .ok()
            .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|duration| duration.as_secs())
            .unwrap_or_default()
            .hash(&mut hasher);
    }
    Ok(format!("{:016x}", hasher.finish()))
}

fn tracked_refresh_inputs(workspace_root: &Path) -> Result<Vec<PathBuf>> {
    let conversations_dir = archive_index::resolve_workspace_conversations_dir(workspace_root)?;
    let mut paths = if conversations_dir.is_dir() {
        fs::read_dir(&conversations_dir)?
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| path.is_file())
            .filter(|path| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| {
                        name != "index.html"
                            && name != "index.json"
                            && name != "refresh-manifest.json"
                            && !name.starts_with("share-safe-packet")
                            && name != "fleet-view.html"
                            && name != "fleet-view.json"
                            && name != "memory-lane.html"
                            && name != "memory-lane.json"
                            && name != "action-packs"
                    })
            })
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    let search_dir = crate::core::search_report::resolve_search_reports_dir(workspace_root);
    if search_dir.is_dir() {
        paths.extend(
            fs::read_dir(&search_dir)?
                .filter_map(|entry| entry.ok())
                .map(|entry| entry.path())
                .filter(|path| {
                    path.file_name()
                        .and_then(|name| name.to_str())
                        .is_some_and(|name| name != "index.html")
                }),
        );
    }

    let integration_dir =
        crate::core::integration_report::resolve_integration_reports_dir(workspace_root);
    if integration_dir.is_dir() {
        paths.extend(
            fs::read_dir(&integration_dir)?
                .filter_map(|entry| entry.ok())
                .map(|entry| entry.path())
                .filter(|path| {
                    path.file_name()
                        .and_then(|name| name.to_str())
                        .is_some_and(|name| name != "index.html" && name != "index.json")
                }),
        );
    }

    let parent = workspace_root.parent().map(Path::to_path_buf);
    if let Some(parent) = parent {
        paths.extend(
            fs::read_dir(parent)?
                .filter_map(|entry| entry.ok())
                .map(|entry| entry.path())
                .filter(|path| workspace_like_name(path))
                .filter(|path| path != workspace_root)
                .map(|path| {
                    path.join(".agents")
                        .join("Conversations")
                        .join("index.json")
                })
                .filter(|path| path.is_file()),
        );
    }

    paths.sort();
    Ok(paths)
}

fn read_previous_workbench_index(
    workspace_root: &Path,
) -> Result<Option<crate::core::workbench::WorkbenchIndexJsonDocument>> {
    let path = resolve_archive_index_json_path(workspace_root)?;
    if !path.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(&path)?;
    let document = serde_json::from_str(&content)?;
    Ok(Some(document))
}

fn all_refresh_outputs_exist(workspace_root: &Path) -> Result<bool> {
    Ok([
        resolve_workspace_archive_index_path(workspace_root)?,
        resolve_archive_index_json_path(workspace_root)?,
        crate::core::search_report::resolve_search_reports_dir(workspace_root).join("index.html"),
        crate::core::integration_report::resolve_integration_reports_dir(workspace_root)
            .join("index.html"),
        crate::core::integration_report::resolve_integration_reports_dir(workspace_root)
            .join("index.json"),
        crate::core::workbench::resolve_share_safe_packet_path(workspace_root)?,
        resolve_refresh_manifest_path(workspace_root)?,
        resolve_fleet_view_html_path(workspace_root)?,
        resolve_fleet_view_json_path(workspace_root)?,
        resolve_action_packs_index_html_path(workspace_root)?,
        crate::core::workbench::resolve_action_packs_index_json_path(workspace_root)?,
        resolve_memory_lane_html_path(workspace_root)?,
        resolve_memory_lane_json_path(workspace_root)?,
    ]
    .iter()
    .all(|path| path.exists()))
}

fn changed_family_keys(
    previous_index: Option<&crate::core::workbench::WorkbenchIndexJsonDocument>,
    current_index: &crate::core::workbench::WorkbenchIndexJsonDocument,
) -> Vec<String> {
    let previous = previous_index
        .map(|index| {
            index
                .families
                .iter()
                .map(|family| {
                    (
                        family.family_key.clone(),
                        serde_json::to_string(family).unwrap_or_default(),
                    )
                })
                .collect::<std::collections::BTreeMap<_, _>>()
        })
        .unwrap_or_default();

    current_index
        .families
        .iter()
        .filter(|family| {
            previous
                .get(&family.family_key)
                .is_none_or(|value| value != &serde_json::to_string(family).unwrap_or_default())
        })
        .map(|family| family.family_key.clone())
        .collect()
}

fn changed_fleet_relations(
    previous_index: Option<&crate::core::workbench::WorkbenchIndexJsonDocument>,
    current_index: &crate::core::workbench::WorkbenchIndexJsonDocument,
) -> Vec<String> {
    let previous = previous_index
        .map(|index| {
            index
                .fleet_view
                .iter()
                .map(|entry| {
                    (
                        entry.relation_key.clone(),
                        serde_json::to_string(entry).unwrap_or_default(),
                    )
                })
                .collect::<std::collections::BTreeMap<_, _>>()
        })
        .unwrap_or_default();

    current_index
        .fleet_view
        .iter()
        .filter(|entry| {
            previous
                .get(&entry.relation_key)
                .is_none_or(|value| value != &serde_json::to_string(entry).unwrap_or_default())
        })
        .map(|entry| entry.relation_key.clone())
        .collect()
}

fn maybe_refresh_semantic_index(workspace_root: &Path) -> Result<Option<PathBuf>> {
    let default_model_dir = FastEmbedSemanticEmbedder::default_model_dir()?;
    if !default_model_dir.is_dir() {
        return Ok(None);
    }
    let embedder = FastEmbedSemanticEmbedder::load_from_dir(&default_model_dir)?;
    let execution = crate::core::semantic_search::semantic_search_with_persistent_index(
        &embedder,
        workspace_root,
        "agent exporter",
        1,
    )?;
    Ok(Some(execution.index_path))
}

pub fn build_decision_desk_summary(
    workspace_root: &Path,
    reports: &[crate::core::integration_report::IntegrationReportJsonDocument],
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
                let assessment = crate::core::integration_report::assess_promotion_eligibility(
                    outcome,
                    candidate,
                    &policy_pack,
                );
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
        baseline: baseline.as_ref().map(decision_desk_snapshot_from_report),
        candidate: Some(decision_desk_snapshot_from_report(candidate)),
        active_policy: archive_index_output::DecisionDeskPolicySummary {
            name: policy_pack.name.clone(),
            version: policy_pack.version.clone(),
        },
        promotion,
        history: recent_history,
        remediation_bundle: Some(
            crate::core::integration_report::build_integration_evidence_remediation_bundle(
                candidate,
            ),
        ),
        gate,
    })
}

fn decision_desk_report_json_path(
    workspace_root: &Path,
    report: &crate::core::integration_report::IntegrationReportJsonDocument,
) -> PathBuf {
    crate::core::integration_report::resolve_integration_reports_dir(workspace_root)
        .join(&report.artifact_links.json_report)
}

fn decision_desk_snapshot_from_report(
    report: &crate::core::integration_report::IntegrationReportJsonDocument,
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
