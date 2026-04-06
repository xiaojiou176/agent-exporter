use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

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

pub fn resolve_integration_reports_dir(workspace_root: &Path) -> PathBuf {
    workspace_root
        .join(".agents")
        .join("Integration")
        .join("Reports")
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
        "integration-report-{kind}-{platform}-{timestamp}.html",
        kind = slugify(kind),
        platform = slugify(platform),
        timestamp = slugify(generated_at),
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
        collect_integration_report_entries, resolve_integration_reports_dir,
        write_integration_report_document, write_integration_reports_index_document,
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
}
