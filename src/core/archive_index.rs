use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use super::archive::OutputTarget;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ArchiveIndexEntry {
    pub file_name: String,
    pub relative_href: String,
    pub title: String,
    pub connector: Option<String>,
    pub thread_id: Option<String>,
    pub completeness: Option<String>,
    pub source_kind: Option<String>,
    pub exported_at: Option<String>,
}

pub fn collect_html_archive_entries(workspace_root: &Path) -> Result<Vec<ArchiveIndexEntry>> {
    let archive_dir = resolve_workspace_conversations_dir(workspace_root)?;
    if !archive_dir.exists() {
        return Ok(Vec::new());
    }

    let mut entries = fs::read_dir(&archive_dir)
        .with_context(|| {
            format!(
                "failed to read archive directory `{}`",
                archive_dir.display()
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
        .map(read_html_archive_entry)
        .collect::<Result<Vec<_>>>()?;

    entries.sort_by(|left, right| {
        right
            .exported_at
            .cmp(&left.exported_at)
            .then_with(|| left.file_name.cmp(&right.file_name))
    });

    Ok(entries)
}

pub fn write_archive_index_document(workspace_root: &Path, document: &str) -> Result<PathBuf> {
    let archive_dir = resolve_workspace_conversations_dir(workspace_root)?;
    fs::create_dir_all(&archive_dir).with_context(|| {
        format!(
            "failed to prepare archive directory `{}`",
            archive_dir.display()
        )
    })?;

    let index_path = archive_dir.join("index.html");
    fs::write(&index_path, format!("{}\n", document.trim_end()))
        .with_context(|| format!("failed to write archive index `{}`", index_path.display()))?;
    Ok(index_path)
}

pub fn resolve_workspace_conversations_dir(workspace_root: &Path) -> Result<PathBuf> {
    OutputTarget::WorkspaceConversations {
        workspace_root: workspace_root.to_path_buf(),
    }
    .resolve_output_dir()
}

fn read_html_archive_entry(path: PathBuf) -> Result<ArchiveIndexEntry> {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .map(str::to_string)
        .unwrap_or_else(|| path.display().to_string());
    let content = fs::read_to_string(&path)
        .with_context(|| format!("failed to read archive file `{}`", path.display()))?;

    Ok(ArchiveIndexEntry {
        file_name: file_name.clone(),
        relative_href: file_name,
        title: extract_meta_value(&content, "thread-display-name")
            .or_else(|| extract_meta_value(&content, "archive-title"))
            .or_else(|| extract_title(&content))
            .unwrap_or_else(|| "Untitled transcript".to_string()),
        connector: extract_meta_value(&content, "connector"),
        thread_id: extract_meta_value(&content, "thread-id"),
        completeness: extract_meta_value(&content, "completeness"),
        source_kind: extract_meta_value(&content, "source-kind"),
        exported_at: extract_meta_value(&content, "exported-at"),
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

    use super::{ArchiveIndexEntry, collect_html_archive_entries, write_archive_index_document};

    #[test]
    fn collect_html_archive_entries_reads_agent_exporter_meta_tags() {
        let workspace = tempdir().expect("workspace");
        let archive_dir = workspace.path().join(".agents").join("Conversations");
        std::fs::create_dir_all(&archive_dir).expect("mkdirs");
        std::fs::write(
            archive_dir.join("demo.html"),
            concat!(
                "<!DOCTYPE html><html><head>",
                "<title>demo transcript</title>",
                "<meta name=\"agent-exporter:archive-title\" content=\"Demo &amp; Archive\">",
                "<meta name=\"agent-exporter:connector\" content=\"codex\">",
                "<meta name=\"agent-exporter:thread-id\" content=\"thread-1\">",
                "<meta name=\"agent-exporter:completeness\" content=\"complete\">",
                "<meta name=\"agent-exporter:source-kind\" content=\"app-server-thread-read\">",
                "<meta name=\"agent-exporter:exported-at\" content=\"2026-04-05T00:00:00Z\">",
                "</head><body></body></html>"
            ),
        )
        .expect("write transcript");

        let entries = collect_html_archive_entries(workspace.path()).expect("collect entries");
        assert_eq!(
            entries,
            vec![ArchiveIndexEntry {
                file_name: "demo.html".to_string(),
                relative_href: "demo.html".to_string(),
                title: "Demo & Archive".to_string(),
                connector: Some("codex".to_string()),
                thread_id: Some("thread-1".to_string()),
                completeness: Some("complete".to_string()),
                source_kind: Some("app-server-thread-read".to_string()),
                exported_at: Some("2026-04-05T00:00:00Z".to_string()),
            }]
        );
    }

    #[test]
    fn write_archive_index_document_writes_index_html() {
        let workspace = tempdir().expect("workspace");
        let path =
            write_archive_index_document(workspace.path(), "<!DOCTYPE html>").expect("write index");
        assert!(path.ends_with("index.html"));
        assert!(path.exists());
    }

    #[test]
    fn collect_html_archive_entries_ignores_search_reports_directory() {
        let workspace = tempdir().expect("workspace");
        let archive_dir = workspace.path().join(".agents").join("Conversations");
        let reports_dir = workspace
            .path()
            .join(".agents")
            .join("Search")
            .join("Reports");
        std::fs::create_dir_all(&archive_dir).expect("archive mkdirs");
        std::fs::create_dir_all(&reports_dir).expect("reports mkdirs");

        std::fs::write(
            archive_dir.join("demo.html"),
            concat!(
                "<!DOCTYPE html><html><head>",
                "<meta name=\"agent-exporter:thread-display-name\" content=\"Demo transcript\">",
                "<meta name=\"agent-exporter:connector\" content=\"codex\">",
                "<meta name=\"agent-exporter:thread-id\" content=\"thread-1\">",
                "<meta name=\"agent-exporter:completeness\" content=\"complete\">",
                "<meta name=\"agent-exporter:source-kind\" content=\"app-server-thread-read\">",
                "<meta name=\"agent-exporter:exported-at\" content=\"2026-04-05T00:00:00Z\">",
                "</head><body></body></html>"
            ),
        )
        .expect("write transcript");
        std::fs::write(
            reports_dir.join("search-report-semantic.html"),
            "<!DOCTYPE html><html><head><title>report</title></head><body></body></html>",
        )
        .expect("write report");

        let entries = collect_html_archive_entries(workspace.path()).expect("collect entries");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].file_name, "demo.html");
    }

    #[test]
    fn collect_html_archive_entries_ignores_integration_reports_directory() {
        let workspace = tempdir().expect("workspace");
        let archive_dir = workspace.path().join(".agents").join("Conversations");
        let integration_reports_dir = workspace
            .path()
            .join(".agents")
            .join("Integration")
            .join("Reports");
        std::fs::create_dir_all(&archive_dir).expect("archive mkdirs");
        std::fs::create_dir_all(&integration_reports_dir).expect("integration mkdirs");

        std::fs::write(
            archive_dir.join("demo.html"),
            concat!(
                "<!DOCTYPE html><html><head>",
                "<meta name=\"agent-exporter:thread-display-name\" content=\"Demo transcript\">",
                "<meta name=\"agent-exporter:connector\" content=\"codex\">",
                "<meta name=\"agent-exporter:thread-id\" content=\"thread-1\">",
                "<meta name=\"agent-exporter:completeness\" content=\"complete\">",
                "<meta name=\"agent-exporter:source-kind\" content=\"app-server-thread-read\">",
                "<meta name=\"agent-exporter:exported-at\" content=\"2026-04-05T00:00:00Z\">",
                "</head><body></body></html>"
            ),
        )
        .expect("write transcript");
        std::fs::write(
            integration_reports_dir.join("integration-report-doctor-codex.html"),
            "<!DOCTYPE html><html><head><title>integration report</title></head><body></body></html>",
        )
        .expect("write integration report");

        let entries = collect_html_archive_entries(workspace.path()).expect("collect entries");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].file_name, "demo.html");
    }
}
