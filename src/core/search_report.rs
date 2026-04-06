use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SearchReportEntry {
    pub file_name: String,
    pub relative_href: String,
    pub title: String,
    pub report_kind: Option<String>,
    pub query: Option<String>,
    pub generated_at: Option<String>,
}

pub fn resolve_search_reports_dir(workspace_root: &Path) -> PathBuf {
    workspace_root
        .join(".agents")
        .join("Search")
        .join("Reports")
}

pub fn collect_search_report_entries(workspace_root: &Path) -> Result<Vec<SearchReportEntry>> {
    let reports_dir = resolve_search_reports_dir(workspace_root);
    if !reports_dir.exists() {
        return Ok(Vec::new());
    }

    let mut entries = fs::read_dir(&reports_dir)
        .with_context(|| {
            format!(
                "failed to read search report directory `{}`",
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
        .map(read_search_report_entry)
        .collect::<Result<Vec<_>>>()?;

    entries.sort_by(|left, right| {
        right
            .generated_at
            .cmp(&left.generated_at)
            .then_with(|| left.file_name.cmp(&right.file_name))
    });

    Ok(entries)
}

pub fn write_search_reports_index_document(
    workspace_root: &Path,
    document: &str,
) -> Result<PathBuf> {
    let reports_dir = resolve_search_reports_dir(workspace_root);
    fs::create_dir_all(&reports_dir).with_context(|| {
        format!(
            "failed to prepare search report directory `{}`",
            reports_dir.display()
        )
    })?;

    let index_path = reports_dir.join("index.html");
    fs::write(&index_path, format!("{}\n", document.trim_end())).with_context(|| {
        format!(
            "failed to write search report index `{}`",
            index_path.display()
        )
    })?;
    Ok(index_path)
}

pub fn write_search_report_document(
    workspace_root: &Path,
    kind: &str,
    query: &str,
    generated_at: &str,
    document: &str,
) -> Result<PathBuf> {
    let reports_dir = resolve_search_reports_dir(workspace_root);
    fs::create_dir_all(&reports_dir).with_context(|| {
        format!(
            "failed to prepare search report directory `{}`",
            reports_dir.display()
        )
    })?;

    let file_name = format!(
        "search-report-{kind}-{timestamp}-{query}.html",
        kind = slugify(kind),
        timestamp = slugify(generated_at),
        query = slugify(query),
    );
    let report_path = reports_dir.join(file_name);
    fs::write(&report_path, format!("{}\n", document.trim_end()))
        .with_context(|| format!("failed to write search report `{}`", report_path.display()))?;
    Ok(report_path)
}

fn slugify(value: &str) -> String {
    let mut slug = String::new();
    let mut last_was_dash = false;

    for ch in value.chars() {
        let normalized = if ch.is_ascii_alphanumeric() {
            Some(ch.to_ascii_lowercase())
        } else {
            None
        };

        if let Some(ch) = normalized {
            slug.push(ch);
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

fn read_search_report_entry(path: PathBuf) -> Result<SearchReportEntry> {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .map(str::to_string)
        .unwrap_or_else(|| path.display().to_string());
    let content = fs::read_to_string(&path)
        .with_context(|| format!("failed to read search report `{}`", path.display()))?;

    Ok(SearchReportEntry {
        file_name: file_name.clone(),
        relative_href: file_name,
        title: extract_meta_value(&content, "report-title")
            .or_else(|| extract_title(&content))
            .unwrap_or_else(|| "Retrieval report".to_string()),
        report_kind: extract_meta_value(&content, "report-kind"),
        query: extract_meta_value(&content, "search-query"),
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
        collect_search_report_entries, resolve_search_reports_dir, write_search_report_document,
        write_search_reports_index_document,
    };

    #[test]
    fn write_search_report_document_writes_under_reports_dir() {
        let workspace = tempdir().expect("workspace");
        let path = write_search_report_document(
            workspace.path(),
            "semantic",
            "login issue",
            "2026-04-05T12:00:00Z",
            "<!DOCTYPE html>",
        )
        .expect("write report");

        assert!(path.exists());
        assert!(path.starts_with(resolve_search_reports_dir(workspace.path())));
        assert!(
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.contains("search-report-semantic"))
        );
    }

    #[test]
    fn collect_search_report_entries_reads_report_meta_tags() {
        let workspace = tempdir().expect("workspace");
        let reports_dir = resolve_search_reports_dir(workspace.path());
        std::fs::create_dir_all(&reports_dir).expect("mkdirs");
        std::fs::write(
            reports_dir.join("search-report-semantic-demo.html"),
            concat!(
                "<!DOCTYPE html><html><head>",
                "<title>Semantic Retrieval Report</title>",
                "<meta name=\"agent-exporter:report-title\" content=\"Semantic Retrieval Report\">",
                "<meta name=\"agent-exporter:report-kind\" content=\"semantic\">",
                "<meta name=\"agent-exporter:search-query\" content=\"login issue\">",
                "<meta name=\"agent-exporter:generated-at\" content=\"2026-04-05T12:00:00Z\">",
                "</head><body></body></html>"
            ),
        )
        .expect("write report");

        let entries = collect_search_report_entries(workspace.path()).expect("collect reports");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].report_kind.as_deref(), Some("semantic"));
        assert_eq!(entries[0].query.as_deref(), Some("login issue"));
    }

    #[test]
    fn collect_search_report_entries_ignores_reports_index_page() {
        let workspace = tempdir().expect("workspace");
        let reports_dir = resolve_search_reports_dir(workspace.path());
        std::fs::create_dir_all(&reports_dir).expect("mkdirs");
        std::fs::write(
            reports_dir.join("index.html"),
            "<!DOCTYPE html><html><head><title>Reports shell</title></head><body></body></html>",
        )
        .expect("write index");
        std::fs::write(
            reports_dir.join("search-report-semantic-demo.html"),
            concat!(
                "<!DOCTYPE html><html><head>",
                "<title>Semantic Retrieval Report</title>",
                "<meta name=\"agent-exporter:report-title\" content=\"Semantic Retrieval Report\">",
                "<meta name=\"agent-exporter:report-kind\" content=\"semantic\">",
                "<meta name=\"agent-exporter:search-query\" content=\"login issue\">",
                "<meta name=\"agent-exporter:generated-at\" content=\"2026-04-05T12:00:00Z\">",
                "</head><body></body></html>"
            ),
        )
        .expect("write report");

        let entries = collect_search_report_entries(workspace.path()).expect("collect reports");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].file_name, "search-report-semantic-demo.html");
    }

    #[test]
    fn write_search_reports_index_document_writes_index_file() {
        let workspace = tempdir().expect("workspace");
        let path = write_search_reports_index_document(workspace.path(), "<!DOCTYPE html>")
            .expect("write report index");

        assert!(path.exists());
        assert_eq!(
            path.file_name().and_then(|name| name.to_str()),
            Some("index.html")
        );
        assert!(path.starts_with(resolve_search_reports_dir(workspace.path())));
    }
}
