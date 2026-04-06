use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
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

pub fn resolve_integration_reports_dir(workspace_root: &Path) -> PathBuf {
    workspace_root
        .join(".agents")
        .join("Integration")
        .join("Reports")
}

pub fn integration_report_base_name(kind: &str, platform: &str, generated_at: &str) -> String {
    format!(
        "integration-report-{kind}-{platform}-{timestamp}",
        kind = slugify(kind),
        platform = slugify(platform),
        timestamp = slugify(generated_at),
    )
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
                .is_none_or(|name| name != "index.html")
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

    let file_name = format!(
        "{}.html",
        integration_report_base_name(kind, platform, generated_at)
    );
    let report_path = reports_dir.join(file_name);
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
                .is_none_or(|name| name != "index.json")
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

pub fn diff_integration_reports(
    left: &IntegrationReportJsonDocument,
    right: &IntegrationReportJsonDocument,
) -> IntegrationEvidenceDiff {
    let mut left_checks = std::collections::BTreeMap::new();
    let mut right_checks = std::collections::BTreeMap::new();

    for check in &left.checks {
        left_checks.insert(check.label.clone(), check);
    }
    for check in &right.checks {
        right_checks.insert(check.label.clone(), check);
    }

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
        IntegrationArtifactLinks, IntegrationReportCheckRecord, IntegrationReportJsonDocument,
        IntegrationReportsIndexJsonDocument, collect_integration_report_entries,
        collect_integration_report_json_documents, diff_integration_reports,
        read_integration_report_json_document, resolve_integration_reports_dir,
        write_integration_report_document, write_integration_report_json_document,
        write_integration_reports_index_document, write_integration_reports_index_json_document,
    };

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
        assert!(
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.contains("integration-report-doctor-codex"))
        );
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
        assert_eq!(entries[0].readiness.as_deref(), Some("ready"));
    }

    #[test]
    fn collect_integration_report_entries_ignores_index_page() {
        let workspace = tempdir().expect("workspace");
        let reports_dir = resolve_integration_reports_dir(workspace.path());
        std::fs::create_dir_all(&reports_dir).expect("mkdirs");
        std::fs::write(
            reports_dir.join("index.html"),
            "<!DOCTYPE html><html><head><title>Integration reports</title></head><body></body></html>",
        )
        .expect("write index");
        std::fs::write(
            reports_dir.join("integration-report-onboard-claude-code-demo.html"),
            concat!(
                "<!DOCTYPE html><html><head>",
                "<meta name=\"agent-exporter:report-title\" content=\"Claude onboard report\">",
                "<meta name=\"agent-exporter:report-kind\" content=\"onboard\">",
                "<meta name=\"agent-exporter:integration-platform\" content=\"claude-code\">",
                "<meta name=\"agent-exporter:integration-readiness\" content=\"partial\">",
                "</head><body></body></html>"
            ),
        )
        .expect("write report");

        let entries =
            collect_integration_report_entries(workspace.path()).expect("collect reports");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].title, "Claude onboard report");
    }

    #[test]
    fn write_integration_reports_index_document_writes_index_html() {
        let workspace = tempdir().expect("workspace");
        let path = write_integration_reports_index_document(workspace.path(), "<!DOCTYPE html>")
            .expect("write index");

        assert!(path.ends_with("index.html"));
        assert!(path.exists());
    }

    #[test]
    fn write_and_collect_integration_report_json_documents() {
        let workspace = tempdir().expect("workspace");
        let document = IntegrationReportJsonDocument {
            schema_version: 1,
            title: "Codex doctor".to_string(),
            kind: "doctor".to_string(),
            platform: "codex".to_string(),
            target: "/tmp/codex-pack".to_string(),
            generated_at: "2026-04-06T12:00:00Z".to_string(),
            readiness: "ready".to_string(),
            summary: "looks ready".to_string(),
            launcher_status: "ready".to_string(),
            launcher_kind: "repo-local-debug".to_string(),
            launcher_command: "/tmp/agent-exporter".to_string(),
            bridge_status: "ready".to_string(),
            pack_shape_checks: vec![IntegrationReportCheckRecord {
                label: "target_content_sync".to_string(),
                readiness: "ready".to_string(),
                detail: "in sync".to_string(),
            }],
            checks: vec![IntegrationReportCheckRecord {
                label: "bridge_script".to_string(),
                readiness: "ready".to_string(),
                detail: "ok".to_string(),
            }],
            next_steps: vec!["review pack".to_string()],
            written_files: vec![],
            unchanged_files: vec![],
            artifact_links: IntegrationArtifactLinks {
                html_report: "integration-report-doctor-codex-demo.html".to_string(),
                json_report: "integration-report-doctor-codex-demo.json".to_string(),
                index_html: "index.html".to_string(),
                index_json: "index.json".to_string(),
            },
        };

        let path = write_integration_report_json_document(
            workspace.path(),
            "doctor",
            "codex",
            "2026-04-06T12:00:00Z",
            &document,
        )
        .expect("write json report");

        assert!(path.exists());
        let parsed = read_integration_report_json_document(&path).expect("read json");
        assert_eq!(parsed.platform, "codex");

        let entries = collect_integration_report_json_documents(workspace.path())
            .expect("collect json reports");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].artifact_links.index_json, "index.json");
    }

    #[test]
    fn write_integration_reports_index_json_document_writes_index_json() {
        let workspace = tempdir().expect("workspace");
        let path = write_integration_reports_index_json_document(
            workspace.path(),
            &IntegrationReportsIndexJsonDocument {
                schema_version: 1,
                title: "integration reports".to_string(),
                generated_at: "2026-04-06T12:00:00Z".to_string(),
                report_count: 0,
                timeline: Vec::new(),
            },
        )
        .expect("write json index");

        assert!(path.ends_with("index.json"));
        assert!(path.exists());
    }

    #[test]
    fn diff_integration_reports_reports_readiness_and_next_step_changes() {
        let left = IntegrationReportJsonDocument {
            schema_version: 1,
            title: "Codex doctor".to_string(),
            kind: "doctor".to_string(),
            platform: "codex".to_string(),
            target: "/tmp/codex-pack".to_string(),
            generated_at: "2026-04-06T12:00:00Z".to_string(),
            readiness: "ready".to_string(),
            summary: "ready".to_string(),
            launcher_status: "ready".to_string(),
            launcher_kind: "repo-local-debug".to_string(),
            launcher_command: "/tmp/agent-exporter".to_string(),
            bridge_status: "ready".to_string(),
            pack_shape_checks: Vec::new(),
            checks: vec![IntegrationReportCheckRecord {
                label: "codex_config_shape".to_string(),
                readiness: "ready".to_string(),
                detail: "command and args present".to_string(),
            }],
            next_steps: vec!["nothing to do".to_string()],
            written_files: Vec::new(),
            unchanged_files: Vec::new(),
            artifact_links: IntegrationArtifactLinks {
                html_report: "left.html".to_string(),
                json_report: "left.json".to_string(),
                index_html: "index.html".to_string(),
                index_json: "index.json".to_string(),
            },
        };
        let right = IntegrationReportJsonDocument {
            schema_version: 1,
            title: "Codex doctor".to_string(),
            kind: "doctor".to_string(),
            platform: "codex".to_string(),
            target: "/tmp/codex-pack".to_string(),
            generated_at: "2026-04-06T12:05:00Z".to_string(),
            readiness: "partial".to_string(),
            summary: "partial".to_string(),
            launcher_status: "ready".to_string(),
            launcher_kind: "repo-local-debug".to_string(),
            launcher_command: "/tmp/agent-exporter".to_string(),
            bridge_status: "ready".to_string(),
            pack_shape_checks: Vec::new(),
            checks: vec![IntegrationReportCheckRecord {
                label: "codex_config_shape".to_string(),
                readiness: "partial".to_string(),
                detail: "args missing".to_string(),
            }],
            next_steps: vec!["restore codex args".to_string()],
            written_files: Vec::new(),
            unchanged_files: Vec::new(),
            artifact_links: IntegrationArtifactLinks {
                html_report: "right.html".to_string(),
                json_report: "right.json".to_string(),
                index_html: "index.html".to_string(),
                index_json: "index.json".to_string(),
            },
        };

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
}
