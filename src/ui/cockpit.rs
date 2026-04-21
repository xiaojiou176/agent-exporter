use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};

use anyhow::{Context, Result, bail};
use axum::extract::Path as AxumPath;
use axum::extract::State;
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio::net::TcpListener;
use tokio::task;

use crate::connectors::state_index::{list_primary_thread_metadata, resolve_codex_home};
use crate::connectors::{self, AppServerClient};
use crate::core::ai_summary::{
    AiSummaryOutcome, AiSummaryRequest, generate_ai_summary_with_options,
};
use crate::core::archive::{
    AiSummaryOptions, AppServerLaunchConfig, ExportOutcome, ExportRequest, ExportSelector,
    ExportSource, OutputTarget,
};
use crate::core::integration_report::{
    collect_integration_report_entries, collect_integration_report_json_documents,
    write_integration_reports_index_document, write_integration_reports_index_json_document,
};
use crate::model::{ConnectorKind, OutputFormat};
use crate::output::html as html_output;
use crate::output::integration_report::{
    build_integration_reports_index_json_document, render_integration_reports_index_document,
};
use crate::output::markdown::{self, DEFAULT_MAX_LINES_PER_PART};

const COCKPIT_HTML: &str = include_str!("assets/cockpit.html");
const COCKPIT_CSS: &str = include_str!("assets/cockpit.css");
const COCKPIT_JS: &str = include_str!("assets/cockpit.js");

#[derive(Clone)]
struct CockpitState {
    workspace_root: PathBuf,
    codex_home: Option<PathBuf>,
    jobs: Arc<Mutex<BTreeMap<String, ExportJobSnapshot>>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DiscoveryThread {
    thread_id: String,
    display_name: String,
    updated_at: Option<i64>,
    created_at: Option<i64>,
    model_provider: Option<String>,
    cwd: Option<String>,
    workspace_key: String,
    workspace_label: String,
    workspace_path: String,
    rollout_path: String,
    workspace_match_kind: String,
    source_kind: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DiscoveryResponse {
    workspace_root: String,
    codex_home: String,
    discovery_mode: String,
    threads: Vec<DiscoveryThread>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExportSelectionInput {
    thread_id: String,
    workspace_path: String,
    workspace_label: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExportRequestBody {
    thread_id: Option<String>,
    workspace_path: Option<String>,
    workspace_label: Option<String>,
    selections: Option<Vec<ExportSelectionInput>>,
    app_server_command: Option<String>,
    app_server_args: Option<Vec<String>>,
    ai_summary: Option<bool>,
    ai_summary_instructions: Option<String>,
    ai_summary_profile: Option<String>,
    ai_summary_model: Option<String>,
    ai_summary_provider: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ExportThreadResult {
    thread_id: String,
    display_name: String,
    transcript_paths: Vec<String>,
    ai_summary_paths: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ExportWorkspaceResult {
    workspace_label: String,
    workspace_path: String,
    archive_shell_path: String,
    reports_shell_path: String,
    integration_shell_path: Option<String>,
    copy_bundle_text: String,
    exported_thread_ids: Vec<String>,
    threads: Vec<ExportThreadResult>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ExportResponse {
    status: &'static str,
    job_id: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ExportJobStep {
    id: String,
    label: String,
    status: String,
    started_at: Option<String>,
    finished_at: Option<String>,
    detail: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ExportJobSnapshot {
    job_id: String,
    status: String,
    started_at: String,
    updated_at: String,
    current_phase: Option<String>,
    current_workspace_label: Option<String>,
    current_thread_title: Option<String>,
    current_thread_id: Option<String>,
    exported_count: usize,
    workspace_count: usize,
    warnings: Vec<String>,
    error_message: Option<String>,
    workspaces: Vec<ExportWorkspaceResult>,
    steps: Vec<ExportJobStep>,
}

struct ThreadExportResult {
    thread_id: String,
    display_name: String,
    workspace_label: String,
    transcript: crate::core::archive::ArchiveTranscript,
    exported_at: String,
    output: ExportOutcome,
    ai_summary: Option<AiSummaryOutcome>,
    ai_summary_warning: Option<String>,
}

fn build_workspace_copy_bundle_text(workspace: &ExportWorkspaceResult) -> String {
    let mut sections = vec![format!(
        "# 工作区: {}\n{}",
        workspace.workspace_label, workspace.workspace_path
    )];

    for thread in &workspace.threads {
        let mut paths = thread.transcript_paths.clone();
        paths.extend(thread.ai_summary_paths.iter().cloned());
        if paths.is_empty() {
            continue;
        }
        sections.push(format!(
            "# 会话: {}\n{}",
            thread.display_name,
            paths.join("\n")
        ));
    }

    let mut shell_paths = vec![
        workspace.archive_shell_path.clone(),
        workspace.reports_shell_path.clone(),
    ];
    if let Some(path) = &workspace.integration_shell_path {
        shell_paths.push(path.clone());
    }
    sections.push(format!(
        "# 当前Repo全局工作台 Index：\n{}",
        shell_paths.join("\n")
    ));

    sections.join("\n\n")
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct CockpitPreferences {
    locale: Option<String>,
    workspace_labels: BTreeMap<String, String>,
}

pub fn run(
    workspace_root: Option<PathBuf>,
    codex_home: Option<PathBuf>,
    open_browser: bool,
) -> Result<()> {
    let workspace_root = workspace_root.unwrap_or(env::current_dir()?);
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .context("failed to build cockpit runtime")?;
    runtime.block_on(run_async(workspace_root, codex_home, open_browser))
}

async fn run_async(
    workspace_root: PathBuf,
    codex_home: Option<PathBuf>,
    open_browser: bool,
) -> Result<()> {
    let state = Arc::new(CockpitState {
        workspace_root,
        codex_home,
        jobs: Arc::new(Mutex::new(BTreeMap::new())),
    });

    let app = Router::new()
        .route("/", get(index))
        .route("/assets/cockpit.css", get(styles))
        .route("/assets/cockpit.js", get(script))
        .route(
            "/api/preferences",
            get(read_preferences).post(save_preferences),
        )
        .route("/api/discovery", get(discovery))
        .route("/api/export", post(export))
        .route("/api/export/jobs/{job_id}", get(read_export_job))
        .with_state(state.clone());

    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .context("failed to bind local cockpit server")?;
    let addr = listener
        .local_addr()
        .context("failed to read local cockpit address")?;
    let url = format!("http://{addr}/");

    println!("Export Cockpit running");
    println!("- URL         : {url}");
    println!("- Workspace   : {}", state.workspace_root.display());

    if open_browser {
        let _ = webbrowser::open(&url);
    }

    axum::serve(listener, app)
        .await
        .context("cockpit server terminated unexpectedly")?;

    Ok(())
}

async fn index() -> Html<&'static str> {
    Html(COCKPIT_HTML)
}

async fn styles() -> Response {
    (
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static("text/css; charset=utf-8"),
        )],
        COCKPIT_CSS,
    )
        .into_response()
}

async fn script() -> Response {
    (
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/javascript; charset=utf-8"),
        )],
        COCKPIT_JS,
    )
        .into_response()
}

fn preferences_path(codex_home: &Path) -> PathBuf {
    codex_home.join("agent-exporter-cockpit-preferences.json")
}

fn load_preferences_file(codex_home: &Path) -> Result<CockpitPreferences> {
    let path = preferences_path(codex_home);
    if !path.exists() {
        return Ok(CockpitPreferences::default());
    }

    let content = fs::read_to_string(&path)
        .with_context(|| format!("failed to read cockpit preferences `{}`", path.display()))?;
    let mut preferences: CockpitPreferences = serde_json::from_str(&content)
        .with_context(|| format!("failed to parse cockpit preferences `{}`", path.display()))?;
    preferences
        .workspace_labels
        .retain(|_, value| !value.trim().is_empty());
    Ok(preferences)
}

fn save_preferences_file(codex_home: &Path, preferences: &CockpitPreferences) -> Result<()> {
    let path = preferences_path(codex_home);
    let payload = serde_json::to_string_pretty(preferences)
        .context("failed to serialize cockpit preferences")?;
    fs::write(&path, payload)
        .with_context(|| format!("failed to write cockpit preferences `{}`", path.display()))?;
    Ok(())
}

async fn read_preferences(
    State(state): State<Arc<CockpitState>>,
) -> Result<Json<CockpitPreferences>, ApiError> {
    let codex_home = resolve_codex_home(state.codex_home.as_deref()).map_err(ApiError::from)?;
    let preferences = load_preferences_file(&codex_home).map_err(ApiError::from)?;
    Ok(Json(preferences))
}

async fn save_preferences(
    State(state): State<Arc<CockpitState>>,
    Json(mut body): Json<CockpitPreferences>,
) -> Result<Json<CockpitPreferences>, ApiError> {
    let codex_home = resolve_codex_home(state.codex_home.as_deref()).map_err(ApiError::from)?;
    body.workspace_labels
        .retain(|_, value| !value.trim().is_empty());
    if let Some(locale) = body.locale.as_deref().map(str::trim)
        && locale.is_empty()
    {
        body.locale = None;
    }
    save_preferences_file(&codex_home, &body).map_err(ApiError::from)?;
    Ok(Json(body))
}

fn new_job_id() -> String {
    format!("job-{}", Utc::now().timestamp_millis())
}

fn now_rfc3339() -> String {
    Utc::now().to_rfc3339()
}

fn with_job_mut<F>(state: &CockpitState, job_id: &str, update: F)
where
    F: FnOnce(&mut ExportJobSnapshot),
{
    if let Ok(mut jobs) = state.jobs.lock()
        && let Some(job) = jobs.get_mut(job_id)
    {
        update(job);
        job.updated_at = now_rfc3339();
    }
}

async fn read_export_job(
    State(state): State<Arc<CockpitState>>,
    AxumPath(job_id): AxumPath<String>,
) -> Result<Json<ExportJobSnapshot>, ApiError> {
    let jobs = state
        .jobs
        .lock()
        .map_err(|_| ApiError(anyhow::anyhow!("failed to read export jobs state")))?;
    let job = jobs
        .get(&job_id)
        .cloned()
        .with_context(|| format!("export job `{job_id}` was not found"))
        .map_err(ApiError::from)?;
    Ok(Json(job))
}

async fn discovery(
    State(state): State<Arc<CockpitState>>,
) -> Result<Json<DiscoveryResponse>, ApiError> {
    let codex_home = resolve_codex_home(state.codex_home.as_deref()).map_err(ApiError::from)?;
    let preferences = load_preferences_file(&codex_home).map_err(ApiError::from)?;
    let (discovery_mode, threads) =
        discover_threads(&state, &codex_home, &preferences).map_err(ApiError::from)?;

    Ok(Json(DiscoveryResponse {
        workspace_root: state.workspace_root.display().to_string(),
        codex_home: codex_home.display().to_string(),
        discovery_mode,
        threads,
    }))
}

fn discover_threads(
    state: &CockpitState,
    codex_home: &Path,
    preferences: &CockpitPreferences,
) -> Result<(String, Vec<DiscoveryThread>)> {
    let entries = list_primary_thread_metadata(codex_home)?;
    let local_threads = entries
        .into_iter()
        .filter_map(|entry| build_local_discovery_thread(codex_home, preferences, entry))
        .collect::<Vec<_>>();

    if should_use_live_discovery(state, codex_home)
        && let Ok(live_threads) = discover_live_threads(preferences)
    {
        let mut merged = BTreeMap::new();
        for thread in local_threads {
            merged.insert(thread.thread_id.clone(), thread);
        }
        for thread in live_threads {
            merged.insert(thread.thread_id.clone(), thread);
        }

        let mut threads = merged.into_values().collect::<Vec<_>>();
        threads.sort_by(|left, right| {
            right
                .updated_at
                .cmp(&left.updated_at)
                .then_with(|| left.display_name.cmp(&right.display_name))
                .then_with(|| left.thread_id.cmp(&right.thread_id))
        });
        return Ok((
            "canonical-app-server-thread-list+local-fallback".to_string(),
            threads,
        ));
    }

    Ok((
        "persisted-codex-state-vscode-fallback".to_string(),
        local_threads,
    ))
}

fn should_use_live_discovery(state: &CockpitState, codex_home: &Path) -> bool {
    match state.codex_home.as_deref() {
        None => true,
        Some(explicit) => {
            resolve_codex_home(None)
                .map(|default| normalize_path(explicit) == normalize_path(&default))
                .unwrap_or(false)
                && normalize_path(explicit) == normalize_path(codex_home)
        }
    }
}

fn normalize_path(path: &Path) -> String {
    path.to_string_lossy()
        .trim()
        .trim_end_matches('/')
        .to_string()
}

fn discover_live_threads(preferences: &CockpitPreferences) -> Result<Vec<DiscoveryThread>> {
    let mut client = AppServerClient::spawn(&AppServerLaunchConfig::default())
        .context("failed to launch codex app-server for cockpit discovery")?;
    client
        .initialize()
        .context("failed to initialize codex app-server for cockpit discovery")?;

    let result = client
        .list_threads()
        .context("failed to read thread/list from codex app-server")?;
    let data = result
        .get("data")
        .and_then(Value::as_array)
        .context("thread/list result did not include `data[]`")?;

    let mut threads = data
        .iter()
        .filter_map(|value| build_live_discovery_thread(preferences, value).ok())
        .collect::<Vec<_>>();
    threads.sort_by(|left, right| {
        right
            .updated_at
            .cmp(&left.updated_at)
            .then_with(|| left.display_name.cmp(&right.display_name))
            .then_with(|| left.thread_id.cmp(&right.thread_id))
    });
    Ok(threads)
}

fn build_live_discovery_thread(
    preferences: &CockpitPreferences,
    value: &Value,
) -> Result<DiscoveryThread> {
    let record = value
        .as_object()
        .context("thread/list item was not an object")?;
    let source = record
        .get("source")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if source != "vscode" {
        bail!("thread/list item is not a vscode thread");
    }

    let thread_id = required_string(record, "id")?;
    let cwd_path = record
        .get("cwd")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(PathBuf::from)
        .context("thread/list item missing cwd")?;
    let workspace_path = cwd_path.display().to_string();
    let workspace_label = renamed_workspace_label(preferences, &cwd_path);
    let rollout_path = record
        .get("path")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    if rollout_path.trim().is_empty() || !Path::new(&rollout_path).exists() {
        bail!("thread/list item missing rollout path on disk");
    }
    let preview = record
        .get("preview")
        .and_then(Value::as_str)
        .map(single_line_text)
        .filter(|value| !value.is_empty());
    let display_name = record
        .get("name")
        .and_then(Value::as_str)
        .map(single_line_text)
        .filter(|value| !value.is_empty())
        .or(preview.clone())
        .unwrap_or_else(|| format!("Untitled thread {}", short_thread_id(&thread_id)));

    Ok(DiscoveryThread {
        thread_id,
        display_name,
        updated_at: record.get("updatedAt").and_then(Value::as_i64),
        created_at: record.get("createdAt").and_then(Value::as_i64),
        model_provider: record
            .get("modelProvider")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
        cwd: Some(workspace_path.clone()),
        workspace_key: workspace_path.clone(),
        workspace_label,
        workspace_path,
        rollout_path,
        workspace_match_kind: "live-cwd".to_string(),
        source_kind: "canonical-app-server-thread-list".to_string(),
    })
}

fn build_local_discovery_thread(
    codex_home: &Path,
    preferences: &CockpitPreferences,
    entry: crate::connectors::state_index::CodexThreadMetadata,
) -> Option<DiscoveryThread> {
    let cwd_path = entry
        .cwd
        .clone()
        .unwrap_or_else(|| codex_home.to_path_buf());
    let workspace_path = cwd_path.display().to_string();
    let workspace_label = renamed_workspace_label(preferences, &cwd_path);
    let resolved_rollout_path = if entry.rollout_path.is_absolute() {
        entry.rollout_path.clone()
    } else {
        codex_home.join(&entry.rollout_path)
    };
    if !resolved_rollout_path.exists() {
        return None;
    }
    let fallback_preview = entry
        .first_user_message
        .clone()
        .filter(|value| !value.trim().is_empty())
        .map(|value| single_line_text(&value))
        .or_else(|| read_rollout_first_user_message(codex_home, &entry.rollout_path));
    let display_name = custom_title(&entry)
        .map(single_line_text)
        .filter(|value| !value.is_empty())
        .or(fallback_preview)
        .unwrap_or_else(|| format!("Untitled thread {}", short_thread_id(&entry.thread_id)));

    Some(DiscoveryThread {
        thread_id: entry.thread_id,
        display_name,
        updated_at: entry.updated_at,
        created_at: entry.created_at,
        model_provider: entry.model_provider,
        cwd: Some(workspace_path.clone()),
        workspace_key: workspace_path.clone(),
        workspace_label,
        workspace_path,
        rollout_path: resolved_rollout_path.display().to_string(),
        workspace_match_kind: "persisted-cwd".to_string(),
        source_kind: "persisted-codex-state-vscode".to_string(),
    })
}

fn required_string(record: &serde_json::Map<String, Value>, key: &str) -> Result<String> {
    record
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .with_context(|| format!("missing required string field `{key}`"))
}

fn single_line_text(value: &str) -> String {
    value
        .lines()
        .map(str::trim)
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

fn read_rollout_first_user_message(codex_home: &Path, rollout_path: &Path) -> Option<String> {
    let resolved = if rollout_path.is_absolute() {
        rollout_path.to_path_buf()
    } else {
        codex_home.join(rollout_path)
    };
    let content = fs::read_to_string(resolved).ok()?;
    for line in content.lines() {
        let entry = serde_json::from_str::<Value>(line).ok()?;
        let payload = entry.get("payload")?.as_object()?;
        if entry.get("type")?.as_str()? == "event_msg"
            && payload.get("type")?.as_str()? == "user_message"
        {
            let message = payload.get("message")?.as_str()?;
            let single = single_line_text(message);
            if !single.is_empty() {
                return Some(single);
            }
        }
    }
    None
}

#[cfg(test)]
fn thread_display_name(entry: &crate::connectors::state_index::CodexThreadMetadata) -> String {
    custom_title(entry)
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| format!("Untitled thread {}", short_thread_id(&entry.thread_id)))
}

fn short_thread_id(thread_id: &str) -> &str {
    thread_id.get(..8).unwrap_or(thread_id)
}

fn custom_title<'a>(
    entry: &'a crate::connectors::state_index::CodexThreadMetadata,
) -> Option<&'a str> {
    let title = entry
        .title
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())?;
    let first_user_message = entry
        .first_user_message
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());

    if let Some(first_user_message) = first_user_message {
        if title == first_user_message {
            return None;
        }
        if title.len() >= 32 && first_user_message.starts_with(title) {
            return None;
        }
    }

    Some(title)
}

fn workspace_label(path: &Path) -> String {
    path.file_name()
        .and_then(|value| value.to_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| path.display().to_string())
}

fn renamed_workspace_label(preferences: &CockpitPreferences, cwd_path: &Path) -> String {
    let key = cwd_path.display().to_string();
    preferences
        .workspace_labels
        .get(&key)
        .map(String::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| workspace_label(cwd_path))
}

async fn export(
    State(state): State<Arc<CockpitState>>,
    Json(body): Json<ExportRequestBody>,
) -> Result<Json<ExportResponse>, ApiError> {
    let codex_home = resolve_codex_home(state.codex_home.as_deref()).map_err(ApiError::from)?;
    let app_server_command = body
        .app_server_command
        .clone()
        .unwrap_or_else(|| "codex".to_string());
    let app_server_args = body.app_server_args.clone().unwrap_or_default();
    let ai_summary_enabled = body.ai_summary.unwrap_or(false);
    let ai_summary_instructions = body.ai_summary_instructions.clone();
    let ai_summary_profile = body.ai_summary_profile.clone();
    let ai_summary_model = body.ai_summary_model.clone();
    let ai_summary_provider = body.ai_summary_provider.clone();
    let selections = normalize_export_selections(body)?;
    let job_id = new_job_id();

    let initial = ExportJobSnapshot {
        job_id: job_id.clone(),
        status: "running".to_string(),
        started_at: now_rfc3339(),
        updated_at: now_rfc3339(),
        current_phase: Some("queued".to_string()),
        current_workspace_label: None,
        current_thread_title: None,
        current_thread_id: None,
        exported_count: 0,
        workspace_count: 0,
        warnings: Vec::new(),
        error_message: None,
        workspaces: Vec::new(),
        steps: Vec::new(),
    };
    state
        .jobs
        .lock()
        .map_err(|_| ApiError(anyhow::anyhow!("failed to store export job state")))?
        .insert(job_id.clone(), initial);

    let state_clone = state.clone();
    let job_id_clone = job_id.clone();
    task::spawn_blocking(move || {
        run_export_job(
            state_clone,
            &job_id_clone,
            codex_home,
            selections,
            app_server_command,
            app_server_args,
            ai_summary_enabled,
            ai_summary_instructions,
            ai_summary_profile,
            ai_summary_model,
            ai_summary_provider,
        );
    });

    Ok(Json(ExportResponse {
        status: "started",
        job_id,
    }))
}

fn run_export_job(
    state: Arc<CockpitState>,
    job_id: &str,
    codex_home: PathBuf,
    selections: Vec<ExportSelectionInput>,
    app_server_command: String,
    app_server_args: Vec<String>,
    ai_summary_enabled: bool,
    ai_summary_instructions: Option<String>,
    ai_summary_profile: Option<String>,
    ai_summary_model: Option<String>,
    ai_summary_provider: Option<String>,
) {
    let result = execute_export_job(
        &state,
        job_id,
        &codex_home,
        selections,
        &app_server_command,
        &app_server_args,
        ai_summary_enabled,
        ai_summary_instructions.as_deref(),
        ai_summary_profile.as_deref(),
        ai_summary_model.as_deref(),
        ai_summary_provider.as_deref(),
    );

    match result {
        Ok(mut snapshot) => {
            snapshot.job_id = job_id.to_string();
            snapshot.status = if snapshot.warnings.is_empty() {
                "completed".to_string()
            } else {
                "completed_with_warnings".to_string()
            };
            snapshot.updated_at = now_rfc3339();
            with_job_mut(&state, job_id, |job| *job = snapshot);
        }
        Err(error) => {
            with_job_mut(&state, job_id, |job| {
                job.status = "failed".to_string();
                job.current_phase = Some("failed".to_string());
                job.error_message = Some(error.to_string());
            });
        }
    }
}

fn execute_export_job(
    state: &CockpitState,
    job_id: &str,
    codex_home: &Path,
    selections: Vec<ExportSelectionInput>,
    app_server_command: &str,
    app_server_args: &[String],
    ai_summary_enabled: bool,
    ai_summary_instructions: Option<&str>,
    ai_summary_profile: Option<&str>,
    ai_summary_model: Option<&str>,
    ai_summary_provider: Option<&str>,
) -> Result<ExportJobSnapshot> {
    let total = selections.len();
    let current_exe = env::current_exe().context("failed to resolve current executable")?;
    let mut workspace_order = Vec::<String>::new();
    let mut workspaces = BTreeMap::<String, ExportWorkspaceResult>::new();
    let mut warnings = Vec::<String>::new();
    let started_at = now_rfc3339();
    let mut completed = 0usize;

    for selection in selections {
        let workspace_root = parse_workspace_root(&selection.workspace_path)?;
        let workspace_key = workspace_root.display().to_string();
        if !workspace_order.contains(&workspace_key) {
            workspace_order.push(workspace_key.clone());
        }

        with_job_mut(state, job_id, |job| {
            job.current_phase = Some(format!("exporting_raw_{}/{}", completed + 1, total));
            job.current_workspace_label = selection
                .workspace_label
                .clone()
                .or_else(|| Some(workspace_label(&workspace_root)));
            job.current_thread_title = Some(selection.thread_id.clone());
            job.current_thread_id = Some(selection.thread_id.clone());
            job.steps.push(ExportJobStep {
                id: format!("raw-export-{}", selection.thread_id),
                label: format!("export raw transcript {}", selection.thread_id),
                status: "running".to_string(),
                started_at: Some(now_rfc3339()),
                finished_at: None,
                detail: Some(workspace_key.clone()),
            });
        });

        let thread_result = perform_export_selection(
            codex_home,
            &workspace_root,
            &selection,
            app_server_command,
            app_server_args,
        )?;

        completed += 1;
        with_job_mut(state, job_id, |job| {
            if let Some(step) = job
                .steps
                .iter_mut()
                .find(|step| step.id == format!("raw-export-{}", selection.thread_id))
            {
                step.status = "completed".to_string();
                step.finished_at = Some(now_rfc3339());
            }
            job.exported_count = completed;
        });

        let mut thread_result = thread_result;
        if ai_summary_enabled {
            let ai_step_id = format!("ai-summary-{}", selection.thread_id);
            with_job_mut(state, job_id, |job| {
                job.current_phase = Some(format!("ai_summary_{}/{}", completed, total));
                job.current_workspace_label = Some(thread_result.workspace_label.clone());
                job.current_thread_title = Some(thread_result.display_name.clone());
                job.current_thread_id = Some(thread_result.thread_id.clone());
                job.steps.push(ExportJobStep {
                    id: ai_step_id.clone(),
                    label: format!("ai summary {}", thread_result.display_name),
                    status: "running".to_string(),
                    started_at: Some(now_rfc3339()),
                    finished_at: None,
                    detail: None,
                });
            });

            match generate_ai_summary_with_options(
                &AiSummaryRequest {
                    transcript: &thread_result.transcript,
                    output_target: &OutputTarget::WorkspaceConversations {
                        workspace_root: workspace_root.clone(),
                    },
                    export_source: ExportSource::AppServer,
                    export_format: OutputFormat::Html,
                    exported_at: &thread_result.exported_at,
                    exported_paths: &thread_result.output.output_paths,
                    extra_instructions: ai_summary_instructions,
                    timeout_seconds: None,
                },
                &AiSummaryOptions {
                    enabled: true,
                    instructions: ai_summary_instructions.map(ToOwned::to_owned),
                    timeout_seconds: None,
                    profile: ai_summary_profile.map(ToOwned::to_owned),
                    model: ai_summary_model.map(ToOwned::to_owned),
                    provider: ai_summary_provider.map(ToOwned::to_owned),
                },
            ) {
                Ok(outcome) => {
                    thread_result.ai_summary = Some(outcome);
                    with_job_mut(state, job_id, |job| {
                        if let Some(step) = job.steps.iter_mut().find(|step| step.id == ai_step_id)
                        {
                            step.status = "completed".to_string();
                            step.finished_at = Some(now_rfc3339());
                        }
                    });
                }
                Err(error) => {
                    let warning = error.to_string();
                    thread_result.ai_summary_warning = Some(warning.clone());
                    warnings.push(format!("{}: {}", thread_result.display_name, warning));
                    with_job_mut(state, job_id, |job| {
                        if let Some(step) = job.steps.iter_mut().find(|step| step.id == ai_step_id)
                        {
                            step.status = "warning".to_string();
                            step.finished_at = Some(now_rfc3339());
                            step.detail = Some(warning.clone());
                        }
                    });
                }
            }
        }

        let entry =
            workspaces
                .entry(workspace_key.clone())
                .or_insert_with(|| ExportWorkspaceResult {
                    workspace_label: thread_result.workspace_label.clone(),
                    workspace_path: workspace_key.clone(),
                    archive_shell_path: workspace_root
                        .join(".agents")
                        .join("Conversations")
                        .join("index.html")
                        .display()
                        .to_string(),
                    reports_shell_path: workspace_root
                        .join(".agents")
                        .join("Search")
                        .join("Reports")
                        .join("index.html")
                        .display()
                        .to_string(),
                    integration_shell_path: Some(
                        workspace_root
                            .join(".agents")
                            .join("Integration")
                            .join("Reports")
                            .join("index.html")
                            .display()
                            .to_string(),
                    ),
                    copy_bundle_text: String::new(),
                    exported_thread_ids: Vec::new(),
                    threads: Vec::new(),
                });
        entry
            .exported_thread_ids
            .push(thread_result.thread_id.clone());
        let mut ai_summary_paths = Vec::new();
        if let Some(ai_summary) = thread_result.ai_summary {
            ai_summary_paths.push(ai_summary.markdown_output_path.display().to_string());
            ai_summary_paths.push(ai_summary.html_output_path.display().to_string());
        }
        entry.threads.push(ExportThreadResult {
            thread_id: thread_result.thread_id,
            display_name: thread_result.display_name,
            transcript_paths: thread_result
                .output
                .output_paths
                .iter()
                .map(|path| path.display().to_string())
                .collect(),
            ai_summary_paths,
        });
    }

    for workspace_key in &workspace_order {
        let workspace_root = PathBuf::from(workspace_key);
        let workspace_label = workspaces
            .get(workspace_key)
            .map(|item| item.workspace_label.clone())
            .unwrap_or_else(|| workspace_label(&workspace_root));
        with_job_mut(state, job_id, |job| {
            job.current_phase = Some(format!("publishing {}", workspace_label));
            job.current_workspace_label = Some(workspace_label.clone());
            job.current_thread_title = None;
            job.current_thread_id = None;
            job.steps.push(ExportJobStep {
                id: format!("publish-{}", workspace_key),
                label: format!("publish shells {}", workspace_label),
                status: "running".to_string(),
                started_at: Some(now_rfc3339()),
                finished_at: None,
                detail: Some(workspace_key.clone()),
            });
        });
        run_publish_command(&current_exe, &workspace_root)?;
        ensure_integration_reports_shell(&workspace_root)?;
        with_job_mut(state, job_id, |job| {
            if let Some(step) = job
                .steps
                .iter_mut()
                .find(|step| step.id == format!("publish-{}", workspace_key))
            {
                step.status = "completed".to_string();
                step.finished_at = Some(now_rfc3339());
            }
        });
    }

    let mut workspace_results = workspace_order
        .into_iter()
        .filter_map(|key| workspaces.remove(&key))
        .collect::<Vec<_>>();
    workspace_results.iter_mut().for_each(|entry| {
        for thread in &mut entry.threads {
            sort_paths_markdown_first(&mut thread.transcript_paths);
            sort_paths_markdown_first(&mut thread.ai_summary_paths);
        }
        entry.copy_bundle_text = build_workspace_copy_bundle_text(entry);
    });

    let steps = state
        .jobs
        .lock()
        .ok()
        .and_then(|jobs| jobs.get(job_id).map(|job| job.steps.clone()))
        .unwrap_or_default();

    Ok(ExportJobSnapshot {
        job_id: job_id.to_string(),
        status: "running".to_string(),
        started_at,
        updated_at: now_rfc3339(),
        current_phase: Some("completed".to_string()),
        current_workspace_label: None,
        current_thread_title: None,
        current_thread_id: None,
        exported_count: completed,
        workspace_count: workspace_results.len(),
        warnings,
        error_message: None,
        workspaces: workspace_results,
        steps,
    })
}

fn normalize_export_selections(body: ExportRequestBody) -> Result<Vec<ExportSelectionInput>> {
    let mut selections = body.selections.unwrap_or_default();
    if selections.is_empty()
        && let (Some(thread_id), Some(workspace_path)) = (body.thread_id, body.workspace_path)
    {
        selections.push(ExportSelectionInput {
            thread_id,
            workspace_path,
            workspace_label: body.workspace_label,
        });
    }

    if selections.is_empty() {
        bail!("export requires at least one selected thread");
    }

    let mut seen = BTreeSet::new();
    selections.retain(|selection| {
        let key = format!("{}::{}", selection.thread_id, selection.workspace_path);
        seen.insert(key)
    });
    Ok(selections)
}

fn parse_workspace_root(path: &str) -> Result<PathBuf> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        bail!("selected workspace path cannot be empty");
    }
    let workspace_root = PathBuf::from(trimmed);
    if !workspace_root.is_absolute() {
        bail!("selected workspace path must be absolute: {trimmed}");
    }
    Ok(workspace_root)
}

fn sort_paths_markdown_first(paths: &mut Vec<String>) {
    paths.sort_by(|left, right| {
        path_priority(left)
            .cmp(&path_priority(right))
            .then_with(|| left.cmp(right))
    });
    paths.dedup();
}

fn path_priority(path: &str) -> u8 {
    if path.ends_with(".md") {
        0
    } else if path.ends_with(".html") {
        1
    } else if path.ends_with(".json") {
        2
    } else {
        3
    }
}

fn perform_export_selection(
    codex_home: &Path,
    workspace_root: &Path,
    selection: &ExportSelectionInput,
    app_server_command: &str,
    app_server_args: &[String],
) -> Result<ThreadExportResult> {
    let request = ExportRequest {
        connector: ConnectorKind::Codex,
        source: ExportSource::AppServer,
        selector: ExportSelector::ThreadId(selection.thread_id.clone()),
        format: OutputFormat::Markdown,
        output_target: OutputTarget::WorkspaceConversations {
            workspace_root: workspace_root.to_path_buf(),
        },
        app_server: AppServerLaunchConfig {
            command: app_server_command.to_string(),
            args: app_server_args.to_vec(),
        },
        codex_home: Some(codex_home.to_path_buf()),
        ai_summary: AiSummaryOptions::default(),
    };

    let transcript = connectors::export(&request)?;
    let display_name = transcript
        .thread_display_name()
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| selection.thread_id.clone());
    let exported_at = Utc::now().to_rfc3339();
    let archive_title = transcript.archive_title(&request.output_target);
    let markdown_parts = markdown::render_markdown_parts(
        &transcript,
        &archive_title,
        &exported_at,
        DEFAULT_MAX_LINES_PER_PART,
    );
    let markdown_output = crate::core::archive::write_markdown_parts(
        &transcript,
        &request.output_target,
        &markdown_parts,
    )?;
    let document = html_output::render_html_document(
        &transcript,
        &archive_title,
        &exported_at,
        Some(html_output::WorkspaceHtmlNavigation {
            archive_shell_href: "index.html".to_string(),
            reports_shell_href: "../Search/Reports/index.html".to_string(),
            integration_shell_href: "../Integration/Reports/index.html".to_string(),
        })
        .as_ref(),
    );
    let html_output =
        crate::core::archive::write_html_document(&transcript, &request.output_target, &document)?;
    let mut output_paths = markdown_output.output_paths.clone();
    output_paths.extend(html_output.output_paths.clone());
    let output = ExportOutcome {
        output_paths,
        exported_part_count: markdown_output.exported_part_count + html_output.exported_part_count,
        exported_item_count: markdown_output.exported_item_count,
        exported_turn_count: markdown_output.exported_turn_count,
        completeness: markdown_output.completeness,
    };

    Ok(ThreadExportResult {
        thread_id: selection.thread_id.clone(),
        display_name,
        workspace_label: selection
            .workspace_label
            .clone()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| workspace_label(workspace_root)),
        transcript,
        exported_at,
        output,
        ai_summary: None,
        ai_summary_warning: None,
    })
}

fn run_publish_command(current_exe: &Path, workspace_root: &Path) -> Result<()> {
    let output = Command::new(current_exe)
        .arg("publish")
        .arg("archive-index")
        .arg("--workspace-root")
        .arg(workspace_root)
        .output()
        .with_context(|| {
            format!(
                "failed to launch publish command `{}`",
                current_exe.display()
            )
        })?;
    if !output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!(
            "publish command failed with status {}.\nstdout:\n{}\nstderr:\n{}",
            output.status,
            stdout,
            stderr
        );
    }
    Ok(())
}

fn ensure_integration_reports_shell(workspace_root: &Path) -> Result<()> {
    let html_entries = collect_integration_report_entries(workspace_root)?;
    let generated_at = chrono::Utc::now().to_rfc3339();
    let title = format!(
        "{} integration reports",
        workspace_root
            .file_name()
            .and_then(|name| name.to_str())
            .filter(|value| !value.trim().is_empty())
            .unwrap_or("agent-exporter")
    );
    let index_document =
        render_integration_reports_index_document(&title, &generated_at, &html_entries);
    write_integration_reports_index_document(workspace_root, &index_document)?;

    let json_entries = collect_integration_report_json_documents(workspace_root)?;
    let index_json_document =
        build_integration_reports_index_json_document(&title, &generated_at, &json_entries);
    write_integration_reports_index_json_document(workspace_root, &index_json_document)?;

    Ok(())
}

#[derive(Debug)]
struct ApiError(anyhow::Error);

impl From<anyhow::Error> for ApiError {
    fn from(value: anyhow::Error) -> Self {
        Self(value)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let payload = Json(json!({
            "status": "error",
            "message": self.0.to_string(),
        }));
        (StatusCode::INTERNAL_SERVER_ERROR, payload).into_response()
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::Arc;

    use axum::Json;
    use axum::extract::State;
    use rusqlite::{Connection, params};
    use tempfile::tempdir;

    use crate::connectors::state_index::CodexThreadMetadata;

    use super::{COCKPIT_HTML, COCKPIT_JS, CockpitState, discovery, thread_display_name};

    #[tokio::test]
    async fn discovery_lists_workspace_relevant_threads() {
        let workspace = tempdir().expect("workspace");
        let codex_home = tempdir().expect("codex home");
        std::fs::create_dir_all(codex_home.path().join("sessions")).expect("sessions dir");
        std::fs::write(
            codex_home.path().join("sessions").join("thread-1.jsonl"),
            "{\"type\":\"event_msg\",\"payload\":{\"type\":\"user_message\",\"message\":\"hello cockpit\"}}\n",
        )
        .expect("rollout 1");
        std::fs::write(
            codex_home.path().join("sessions").join("thread-2.jsonl"),
            "{\"type\":\"event_msg\",\"payload\":{\"type\":\"user_message\",\"message\":\"ignore me\"}}\n",
        )
        .expect("rollout 2");
        let state_db = codex_home.path().join("state_5.sqlite");
        let connection = Connection::open(&state_db).expect("sqlite db");
        connection
            .execute_batch(
                "CREATE TABLE threads (
                    id TEXT PRIMARY KEY,
                    rollout_path TEXT,
                    cwd TEXT,
                    model_provider TEXT,
                    created_at INTEGER,
                    updated_at INTEGER,
                    source TEXT,
                    archived INTEGER NOT NULL DEFAULT 0,
                    cli_version TEXT,
                    title TEXT,
                    first_user_message TEXT
                );",
            )
            .expect("schema");
        connection
            .execute(
                "INSERT INTO threads (id, rollout_path, cwd, model_provider, created_at, updated_at, source, archived, cli_version, title, first_user_message)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                params![
                    "thread-1",
                    "sessions/thread-1.jsonl",
                    workspace.path().display().to_string(),
                    "openai",
                    1000_i64,
                    1001_i64,
                    "vscode",
                    0_i64,
                    "0.1.0",
                    "Renamed in UI",
                    "hello cockpit"
                ],
            )
            .expect("insert");
        connection
            .execute(
                "INSERT INTO threads (id, rollout_path, cwd, model_provider, created_at, updated_at, source, archived, cli_version, title, first_user_message)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                params![
                    "thread-2",
                    "sessions/thread-2.jsonl",
                    "/tmp/elsewhere",
                    "openai",
                    999_i64,
                    999_i64,
                    "vscode",
                    0_i64,
                    "0.1.0",
                    "Ignore me",
                    "ignore me"
                ],
            )
            .expect("insert elsewhere");
        connection
            .execute(
                "INSERT INTO threads (id, rollout_path, cwd, model_provider, created_at, updated_at, source, archived, cli_version, title, first_user_message)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                params![
                    "thread-3",
                    "sessions/thread-3.jsonl",
                    "/tmp/elsewhere",
                    "openai",
                    888_i64,
                    999_i64,
                    "exec",
                    0_i64,
                    "0.1.0",
                    "Ignore exec",
                    "ignore exec"
                ],
            )
            .expect("insert exec");
        connection
            .execute(
                "INSERT INTO threads (id, rollout_path, cwd, model_provider, created_at, updated_at, source, archived, cli_version, title, first_user_message)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                params![
                    "thread-4",
                    "sessions/thread-4.jsonl",
                    "/tmp/elsewhere",
                    "openai",
                    777_i64,
                    998_i64,
                    "vscode",
                    1_i64,
                    "0.1.0",
                    "Ignore archived",
                    "ignore archived"
                ],
            )
            .expect("insert archived");

        let state = Arc::new(CockpitState {
            workspace_root: workspace.path().to_path_buf(),
            codex_home: Some(codex_home.path().to_path_buf()),
            jobs: Arc::new(std::sync::Mutex::new(std::collections::BTreeMap::new())),
        });

        let Json(response) = discovery(State(state)).await.expect("discovery response");
        assert_eq!(
            response.discovery_mode,
            "persisted-codex-state-vscode-fallback"
        );
        assert_eq!(response.threads.len(), 2);
        assert_eq!(response.threads[0].thread_id, "thread-1");
        assert_eq!(response.threads[0].display_name, "Renamed in UI");
        assert_eq!(
            response.threads[0].workspace_label,
            workspace
                .path()
                .file_name()
                .and_then(|value| value.to_str())
                .expect("workspace label")
        );
        assert_eq!(
            response.threads[0].workspace_path,
            workspace.path().display().to_string()
        );
        assert_eq!(response.threads[1].thread_id, "thread-2");
    }

    #[test]
    fn cockpit_assets_include_search_and_command_preview_surfaces() {
        assert!(COCKPIT_HTML.contains("thread-search"));
        assert!(COCKPIT_HTML.contains("command-preview"));
        assert!(COCKPIT_JS.contains("renderCommandPreview"));
        assert!(COCKPIT_JS.contains("threadSearchStatusEl"));
        assert!(COCKPIT_JS.contains("groupThreadsByWorkspace"));
        assert!(COCKPIT_HTML.contains("workspace-group"));
    }

    #[test]
    fn thread_display_name_ignores_titles_that_only_repeat_the_first_prompt() {
        let entry = CodexThreadMetadata {
            thread_id: "019dad4b-7cbe-7351-a8c2-931bd140bc90".to_string(),
            rollout_path: PathBuf::from("sessions/thread.jsonl"),
            cwd: Some(PathBuf::from("/tmp/workspace")),
            model_provider: Some("openai".to_string()),
            created_at: Some(1000),
            updated_at: Some(1001),
            title: Some("Repo: /tmp/workspace . Read-only UI/public-surface audit only.".to_string()),
            first_user_message: Some(
                "Repo: /tmp/workspace . Read-only UI/public-surface audit only. Scope files: docs/index.md."
                    .to_string(),
            ),
        };

        assert_eq!(thread_display_name(&entry), "Untitled thread 019dad4b");
    }
}
