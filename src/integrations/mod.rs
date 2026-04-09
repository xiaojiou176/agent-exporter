use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use serde_json::Value as JsonValue;
use toml::Value as TomlValue;

const MCP_SCRIPT_PLACEHOLDER: &str =
    "/absolute/path/to/agent-exporter/scripts/agent_exporter_mcp.py";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IntegrationPlatform {
    Codex,
    ClaudeCode,
    OpenClaw,
}

impl IntegrationPlatform {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Codex => "codex",
            Self::ClaudeCode => "claude-code",
            Self::OpenClaw => "openclaw",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IntegrationReadiness {
    Ready,
    Partial,
    Missing,
}

impl IntegrationReadiness {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Partial => "partial",
            Self::Missing => "missing",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntegrationMaterializeRequest {
    pub platform: IntegrationPlatform,
    pub target_root: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntegrationMaterializeOutcome {
    pub platform: IntegrationPlatform,
    pub target_root: PathBuf,
    pub launcher: LauncherSpec,
    pub written_files: Vec<PathBuf>,
    pub unchanged_files: Vec<PathBuf>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntegrationDoctorRequest {
    pub platform: IntegrationPlatform,
    pub target_root: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntegrationDoctorCheck {
    pub label: &'static str,
    pub readiness: IntegrationReadiness,
    pub detail: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntegrationDoctorOutcome {
    pub platform: IntegrationPlatform,
    pub target_root: PathBuf,
    pub overall_readiness: IntegrationReadiness,
    pub launcher: LauncherSpec,
    pub checks: Vec<IntegrationDoctorCheck>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntegrationOnboardRequest {
    pub platform: IntegrationPlatform,
    pub target_root: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntegrationOnboardOutcome {
    pub platform: IntegrationPlatform,
    pub target_root: PathBuf,
    pub materialized: IntegrationMaterializeOutcome,
    pub doctor: IntegrationDoctorOutcome,
    pub next_steps: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LauncherSpec {
    pub kind: &'static str,
    pub command: String,
    pub args: Vec<String>,
}

impl LauncherSpec {
    pub fn shell_command(&self) -> String {
        let mut rendered = vec![quote_shell_arg_always(&self.command)];
        rendered.extend(self.args.iter().map(|arg| quote_shell_arg(arg)));
        rendered.join(" ")
    }
}

#[derive(Clone, Copy)]
enum RenderMode {
    Plain,
    LauncherCommands,
    McpConfig,
}

#[derive(Clone, Copy)]
struct TemplateAsset {
    source_rel: &'static str,
    destination_rel: &'static str,
    mode: RenderMode,
}

pub fn materialize_integration(
    request: &IntegrationMaterializeRequest,
) -> Result<IntegrationMaterializeOutcome> {
    let repo_root = repo_root();
    let launcher = resolve_launcher(&repo_root)?;
    validate_materialize_target(request.platform, &request.target_root)?;
    prepare_target_dir(&request.target_root)?;

    let mut written_files = Vec::new();
    let mut unchanged_files = Vec::new();

    for asset in assets_for(request.platform) {
        let source_path = repo_root.join(asset.source_rel);
        let raw = fs::read_to_string(&source_path).with_context(|| {
            format!(
                "failed to read integration template `{}`",
                source_path.display()
            )
        })?;
        let rendered = render_template(&raw, asset.mode, &repo_root, &launcher);
        let destination_path = request.target_root.join(asset.destination_rel);
        match write_materialized_file(&destination_path, &rendered)? {
            WriteDisposition::Written => written_files.push(destination_path),
            WriteDisposition::Unchanged => unchanged_files.push(destination_path),
        }
    }

    Ok(IntegrationMaterializeOutcome {
        platform: request.platform,
        target_root: request.target_root.clone(),
        launcher,
        written_files,
        unchanged_files,
    })
}

pub fn doctor_integration(request: &IntegrationDoctorRequest) -> Result<IntegrationDoctorOutcome> {
    let repo_root = repo_root();
    let launcher = resolve_launcher(&repo_root)?;
    let bridge_script = bridge_script_path(&repo_root);
    let assets = assets_for(request.platform);
    let mut checks = Vec::new();

    checks.push(IntegrationDoctorCheck {
        label: "bridge_script",
        readiness: if bridge_script.is_file() {
            IntegrationReadiness::Ready
        } else {
            IntegrationReadiness::Missing
        },
        detail: bridge_script.display().to_string(),
    });

    checks.push(IntegrationDoctorCheck {
        label: "python3",
        readiness: if python3_available() {
            IntegrationReadiness::Ready
        } else {
            IntegrationReadiness::Partial
        },
        detail: "python3".to_string(),
    });

    let missing_assets = assets
        .iter()
        .filter(|asset| !repo_root.join(asset.source_rel).is_file())
        .count();
    checks.push(IntegrationDoctorCheck {
        label: "repo_templates",
        readiness: if missing_assets == 0 {
            IntegrationReadiness::Ready
        } else {
            IntegrationReadiness::Missing
        },
        detail: format!(
            "{}/{} template assets available",
            assets.len() - missing_assets,
            assets.len()
        ),
    });

    if !request.target_root.exists() {
        checks.push(IntegrationDoctorCheck {
            label: "target_root",
            readiness: IntegrationReadiness::Missing,
            detail: request.target_root.display().to_string(),
        });
        return Ok(IntegrationDoctorOutcome {
            platform: request.platform,
            target_root: request.target_root.clone(),
            overall_readiness: IntegrationReadiness::Missing,
            launcher,
            checks,
        });
    }

    if !request.target_root.is_dir() {
        bail!(
            "doctor target must be a directory: {}",
            request.target_root.display()
        );
    }

    checks.push(IntegrationDoctorCheck {
        label: "target_root",
        readiness: IntegrationReadiness::Ready,
        detail: request.target_root.display().to_string(),
    });

    let expected_destinations = assets
        .iter()
        .map(|asset| request.target_root.join(asset.destination_rel))
        .collect::<Vec<_>>();
    let existing_files = expected_destinations
        .iter()
        .filter(|path| path.is_file())
        .count();

    checks.push(IntegrationDoctorCheck {
        label: "target_files",
        readiness: if existing_files == 0 {
            IntegrationReadiness::Missing
        } else if existing_files == expected_destinations.len() {
            IntegrationReadiness::Ready
        } else {
            IntegrationReadiness::Partial
        },
        detail: format!(
            "{existing_files}/{} expected files present",
            expected_destinations.len()
        ),
    });

    let mismatched_files = assets
        .iter()
        .filter_map(|asset| {
            let destination = request.target_root.join(asset.destination_rel);
            let existing = fs::read_to_string(&destination).ok()?;
            let source_path = repo_root.join(asset.source_rel);
            let raw = fs::read_to_string(&source_path).ok()?;
            let expected = render_template(&raw, asset.mode, &repo_root, &launcher);
            (existing != expected).then_some(destination)
        })
        .collect::<Vec<_>>();

    checks.push(IntegrationDoctorCheck {
        label: "target_content_sync",
        readiness: if existing_files == 0 {
            IntegrationReadiness::Missing
        } else if mismatched_files.is_empty() {
            IntegrationReadiness::Ready
        } else {
            IntegrationReadiness::Partial
        },
        detail: if mismatched_files.is_empty() {
            "all materialized files match the current repo-generated content".to_string()
        } else {
            format!(
                "{} files differ from the current repo-generated content",
                mismatched_files.len()
            )
        },
    });

    let unresolved_paths = expected_destinations
        .iter()
        .filter_map(|path| fs::read_to_string(path).ok())
        .filter(|content| {
            content.contains(MCP_SCRIPT_PLACEHOLDER)
                || contains_generic_launcher_reference(
                    content,
                    "agent-exporter publish archive-index",
                )
                || contains_generic_launcher_reference(content, "agent-exporter search semantic")
                || contains_generic_launcher_reference(content, "agent-exporter search hybrid")
        })
        .count();

    checks.push(IntegrationDoctorCheck {
        label: "materialized_paths",
        readiness: if existing_files == 0 {
            IntegrationReadiness::Missing
        } else if unresolved_paths == 0 {
            IntegrationReadiness::Ready
        } else {
            IntegrationReadiness::Partial
        },
        detail: if unresolved_paths == 0 {
            "all detected templates use repo-local launcher/script paths".to_string()
        } else {
            format!(
                "{unresolved_paths} files still contain placeholder or generic PATH launcher references"
            )
        },
    });

    let probe = probe_launcher(&launcher);
    checks.push(IntegrationDoctorCheck {
        label: "launcher_probe",
        readiness: if probe.success {
            IntegrationReadiness::Ready
        } else {
            IntegrationReadiness::Partial
        },
        detail: probe.detail,
    });

    checks.extend(platform_specific_checks(
        request.platform,
        &request.target_root,
    ));

    let overall = collapse_readiness(&checks);
    Ok(IntegrationDoctorOutcome {
        platform: request.platform,
        target_root: request.target_root.clone(),
        overall_readiness: overall,
        launcher,
        checks,
    })
}

pub fn onboard_integration(
    request: &IntegrationOnboardRequest,
) -> Result<IntegrationOnboardOutcome> {
    let materialized = materialize_integration(&IntegrationMaterializeRequest {
        platform: request.platform,
        target_root: request.target_root.clone(),
    })?;
    let doctor = doctor_integration(&IntegrationDoctorRequest {
        platform: request.platform,
        target_root: request.target_root.clone(),
    })?;
    let next_steps = doctor_next_steps(&doctor);

    Ok(IntegrationOnboardOutcome {
        platform: request.platform,
        target_root: request.target_root.clone(),
        materialized,
        doctor,
        next_steps,
    })
}

pub fn doctor_summary(outcome: &IntegrationDoctorOutcome) -> String {
    match outcome.overall_readiness {
        IntegrationReadiness::Ready => format!(
            "{} pack looks ready inside `{}`.",
            outcome.platform.as_str(),
            outcome.target_root.display()
        ),
        IntegrationReadiness::Partial => format!(
            "{} pack is partially ready; at least one check still needs attention in `{}`.",
            outcome.platform.as_str(),
            outcome.target_root.display()
        ),
        IntegrationReadiness::Missing => format!(
            "{} pack is missing required integration material in `{}`.",
            outcome.platform.as_str(),
            outcome.target_root.display()
        ),
    }
}

pub fn doctor_next_steps(outcome: &IntegrationDoctorOutcome) -> Vec<String> {
    let mut next_steps = Vec::new();

    if has_check(outcome, "target_root", IntegrationReadiness::Missing)
        || has_check(outcome, "target_files", IntegrationReadiness::Missing)
    {
        next_steps.push(format!(
            "Run `agent-exporter integrate {} --target {}` first.",
            outcome.platform.as_str(),
            outcome.target_root.display()
        ));
    }

    if has_check(
        outcome,
        "target_content_sync",
        IntegrationReadiness::Partial,
    ) {
        next_steps.push(
            "Re-run `integrate` into an empty target or manually refresh stale materialized files."
                .to_string(),
        );
    }

    if has_check(outcome, "materialized_paths", IntegrationReadiness::Partial) {
        next_steps.push(
            "Replace placeholder or generic launcher strings by re-running `integrate` from the current repo."
                .to_string(),
        );
    }

    if has_check(outcome, "launcher_probe", IntegrationReadiness::Partial) {
        next_steps.push(
            "Give doctor a concrete repo-local binary (`target/debug` or `target/release`) if you want launcher probe to reach `ready`."
                .to_string(),
        );
    }

    if has_check(outcome, "codex_config_shape", IntegrationReadiness::Partial) {
        next_steps.push(
            "Ensure `.codex/config.toml` contains `mcp_servers.agent_exporter.command` and a non-empty `args` array."
                .to_string(),
        );
    }

    if has_check(
        outcome,
        "claude_project_shape",
        IntegrationReadiness::Partial,
    ) {
        next_steps.push(
            "Ensure `.mcp.json` parses and contains `mcpServers.agent-exporter.command`."
                .to_string(),
        );
    }

    if has_check(outcome, "claude_pack_shape", IntegrationReadiness::Partial) {
        next_steps.push(
            "Ensure `CLAUDE.md` keeps a heading and `.claude/commands/*.md` keep a description line plus a bash code block."
                .to_string(),
        );
    }

    if has_check(
        outcome,
        "openclaw_bundle_shape",
        IntegrationReadiness::Partial,
    ) || has_check(
        outcome,
        "openclaw_bundle_shape",
        IntegrationReadiness::Missing,
    ) {
        next_steps.push(
            "Ensure both OpenClaw bundle manifests and `.mcp.json` files are present and parseable before calling the pack ready."
                .to_string(),
        );
    }

    match outcome.platform {
        IntegrationPlatform::Codex => next_steps.push(
            "If this target is your final project root, review `AGENTS.md`, `.agents/skills/`, and `.codex/config.toml` before trusting the project in Codex."
                .to_string(),
        ),
        IntegrationPlatform::ClaudeCode => next_steps.push(
            "If this target is your final project root, keep `CLAUDE.md`, `.claude/commands/*`, and `.mcp.json` together as one project pack."
                .to_string(),
        ),
        IntegrationPlatform::OpenClaw => next_steps.push(
            "Pick the bundle variant you want to use and copy it into your OpenClaw bundle/plugin root; this remains bundle-content readiness, not runtime installation."
                .to_string(),
        ),
    }

    next_steps
}

fn collapse_readiness(checks: &[IntegrationDoctorCheck]) -> IntegrationReadiness {
    let mut has_partial = false;
    for check in checks {
        match check.readiness {
            IntegrationReadiness::Missing => return IntegrationReadiness::Missing,
            IntegrationReadiness::Partial => has_partial = true,
            IntegrationReadiness::Ready => {}
        }
    }
    if has_partial {
        IntegrationReadiness::Partial
    } else {
        IntegrationReadiness::Ready
    }
}

fn has_check(
    outcome: &IntegrationDoctorOutcome,
    label: &'static str,
    readiness: IntegrationReadiness,
) -> bool {
    outcome
        .checks
        .iter()
        .any(|check| check.label == label && check.readiness == readiness)
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn bridge_script_path(repo_root: &Path) -> PathBuf {
    repo_root.join("scripts").join("agent_exporter_mcp.py")
}

fn resolve_launcher(repo_root: &Path) -> Result<LauncherSpec> {
    if let Ok(current_bin) = std::env::var("CARGO_BIN_EXE_agent-exporter") {
        let current_bin_path = PathBuf::from(&current_bin);
        if current_bin_path.is_file() {
            return Ok(LauncherSpec {
                kind: "repo-local-cargo-bin-exe",
                command: current_bin,
                args: Vec::new(),
            });
        }
    }

    let release_bin = repo_root
        .join("target")
        .join("release")
        .join("agent-exporter");
    if release_bin.is_file() {
        return Ok(LauncherSpec {
            kind: "repo-local-release",
            command: release_bin.display().to_string(),
            args: Vec::new(),
        });
    }

    let debug_bin = repo_root
        .join("target")
        .join("debug")
        .join("agent-exporter");
    if debug_bin.is_file() {
        return Ok(LauncherSpec {
            kind: "repo-local-debug",
            command: debug_bin.display().to_string(),
            args: Vec::new(),
        });
    }

    if cargo_available() {
        return Ok(LauncherSpec {
            kind: "cargo-run",
            command: "cargo".to_string(),
            args: vec![
                "run".to_string(),
                "--quiet".to_string(),
                "--manifest-path".to_string(),
                repo_root.join("Cargo.toml").display().to_string(),
                "--bin".to_string(),
                "agent-exporter".to_string(),
                "--".to_string(),
            ],
        });
    }

    bail!(
        "failed to resolve a repo-owned launcher: no target/release, no target/debug, and `cargo` is unavailable"
    )
}

fn cargo_available() -> bool {
    Command::new("cargo")
        .arg("--version")
        .output()
        .is_ok_and(|output| output.status.success())
}

fn python3_available() -> bool {
    Command::new("python3")
        .arg("--version")
        .output()
        .is_ok_and(|output| output.status.success())
}

fn prepare_target_dir(target_root: &Path) -> Result<()> {
    if target_root.exists() && !target_root.is_dir() {
        bail!(
            "integration target must be a directory: {}",
            target_root.display()
        );
    }
    fs::create_dir_all(target_root).with_context(|| {
        format!(
            "failed to prepare integration target `{}`",
            target_root.display()
        )
    })
}

fn validate_materialize_target(platform: IntegrationPlatform, target_root: &Path) -> Result<()> {
    let normalized = normalize_target_root(target_root)?;

    if let Some(home_root) = home_root() {
        let codex_root = home_root.join(".codex");
        if path_is_or_inside(&normalized, &codex_root) {
            bail!(
                "integration target `{}` is forbidden: choose a staging pack directory instead of the live Codex home root `~/.codex`",
                target_root.display()
            );
        }

        if let Some(first_after_home) = first_component_after(&normalized, &home_root) {
            if first_after_home.starts_with(".claude") {
                bail!(
                    "integration target `{}` is forbidden: choose a staging pack directory instead of a live Claude home root such as `~/.claude*`",
                    target_root.display()
                );
            }
        }
    }

    if platform == IntegrationPlatform::OpenClaw && is_openclaw_host_like_root(&normalized) {
        bail!(
            "integration target `{}` is forbidden for OpenClaw: point `--target` at a neutral staging directory above the bundle/plugin roots, not a direct OpenClaw bundle or plugin root",
            target_root.display()
        );
    }

    Ok(())
}

fn normalize_target_root(target_root: &Path) -> Result<PathBuf> {
    let absolute = if target_root.is_absolute() {
        target_root.to_path_buf()
    } else {
        std::env::current_dir()
            .context("failed to resolve current working directory for integration target")?
            .join(target_root)
    };

    if absolute.exists() {
        return fs::canonicalize(&absolute).with_context(|| {
            format!(
                "failed to canonicalize integration target `{}`",
                target_root.display()
            )
        });
    }

    let mut existing_prefix = absolute.clone();
    let mut pending = Vec::new();
    while !existing_prefix.exists() {
        let Some(name) = existing_prefix.file_name() else {
            break;
        };
        pending.push(name.to_os_string());
        if !existing_prefix.pop() {
            break;
        }
    }

    let mut normalized = fs::canonicalize(&existing_prefix).with_context(|| {
        format!(
            "failed to canonicalize integration target parent `{}`",
            existing_prefix.display()
        )
    })?;
    for name in pending.iter().rev() {
        normalized.push(name);
    }
    Ok(normalized)
}

fn home_root() -> Option<PathBuf> {
    let home = std::env::var_os("HOME").map(PathBuf::from)?;
    fs::canonicalize(&home).ok().or(Some(home))
}

fn path_is_or_inside(path: &Path, root: &Path) -> bool {
    path == root || path.starts_with(root)
}

fn first_component_after(path: &Path, root: &Path) -> Option<String> {
    let relative = path.strip_prefix(root).ok()?;
    relative
        .components()
        .next()
        .map(|component| component.as_os_str().to_string_lossy().into_owned())
}

fn is_openclaw_host_like_root(path: &Path) -> bool {
    let Some(name) = path.file_name().map(|value| value.to_string_lossy()) else {
        return false;
    };
    if matches!(
        name.as_ref(),
        "openclaw-codex-bundle"
            | "openclaw-claude-bundle"
            | ".codex-plugin"
            | ".claude-plugin"
            | "plugins"
            | "bundles"
    ) {
        return true;
    }

    path.parent()
        .and_then(|parent| parent.file_name())
        .map(|value| value.to_string_lossy())
        .is_some_and(|parent_name| matches!(parent_name.as_ref(), "plugins" | "bundles"))
}

fn render_template(
    raw: &str,
    mode: RenderMode,
    repo_root: &Path,
    launcher: &LauncherSpec,
) -> String {
    let with_bridge = raw.replace(
        MCP_SCRIPT_PLACEHOLDER,
        &bridge_script_path(repo_root).display().to_string(),
    );
    match mode {
        RenderMode::Plain | RenderMode::McpConfig => with_bridge,
        RenderMode::LauncherCommands => rewrite_launcher_commands(&with_bridge, launcher),
    }
}

fn rewrite_launcher_commands(raw: &str, launcher: &LauncherSpec) -> String {
    let launcher_shell = launcher.shell_command();
    raw.replace(
        "agent-exporter publish archive-index",
        &format!("{launcher_shell} publish archive-index"),
    )
    .replace(
        "agent-exporter search semantic",
        &format!("{launcher_shell} search semantic"),
    )
    .replace(
        "agent-exporter search hybrid",
        &format!("{launcher_shell} search hybrid"),
    )
}

fn quote_shell_arg(value: &str) -> String {
    let safe = value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '/' | '_' | '-' | '.' | ':' | '='));
    if safe {
        value.to_string()
    } else {
        format!("'{}'", value.replace('\'', "'\"'\"'"))
    }
}

fn quote_shell_arg_always(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

fn contains_generic_launcher_reference(content: &str, pattern: &str) -> bool {
    content.match_indices(pattern).any(|(index, _)| {
        let Some(previous) = content[..index].chars().next_back() else {
            return true;
        };

        !matches!(previous, '/' | '\\' | '\'' | '"' | '_' | '-' | '.')
            && !previous.is_ascii_alphanumeric()
    })
}

enum WriteDisposition {
    Written,
    Unchanged,
}

struct LauncherProbeOutcome {
    success: bool,
    detail: String,
}

fn write_materialized_file(path: &Path, content: &str) -> Result<WriteDisposition> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "failed to prepare integration parent directory `{}`",
                parent.display()
            )
        })?;
    }

    if path.exists() {
        let existing = fs::read_to_string(path)
            .with_context(|| format!("failed to read existing target `{}`", path.display()))?;
        if existing == content {
            return Ok(WriteDisposition::Unchanged);
        }
        bail!(
            "integration materializer refuses to overwrite existing file `{}`; choose an empty target or remove the file first",
            path.display()
        );
    }

    fs::write(path, content)
        .with_context(|| format!("failed to write integration file `{}`", path.display()))?;
    Ok(WriteDisposition::Written)
}

fn probe_launcher(launcher: &LauncherSpec) -> LauncherProbeOutcome {
    if launcher.kind == "cargo-run" {
        return LauncherProbeOutcome {
            success: false,
            detail: format!(
                "`{}` is intentionally not executed in read-only doctor mode because `cargo run` may trigger a build",
                launcher.shell_command()
            ),
        };
    }

    match Command::new(&launcher.command)
        .args(&launcher.args)
        .arg("connectors")
        .output()
    {
        Ok(output) if output.status.success() => LauncherProbeOutcome {
            success: true,
            detail: format!("`{}` can execute `connectors`", launcher.shell_command()),
        },
        Ok(output) => LauncherProbeOutcome {
            success: false,
            detail: format!(
                "`{}` failed probe with status {}",
                launcher.shell_command(),
                output.status
            ),
        },
        Err(error) => LauncherProbeOutcome {
            success: false,
            detail: format!("`{}` failed probe: {error}", launcher.shell_command()),
        },
    }
}

fn assets_for(platform: IntegrationPlatform) -> &'static [TemplateAsset] {
    match platform {
        IntegrationPlatform::Codex => &[
            TemplateAsset {
                source_rel: "docs/integrations/templates/codex/AGENTS.md",
                destination_rel: "AGENTS.md",
                mode: RenderMode::LauncherCommands,
            },
            TemplateAsset {
                source_rel: "docs/integrations/templates/codex/.agents/skills/export-archive/SKILL.md",
                destination_rel: ".agents/skills/export-archive/SKILL.md",
                mode: RenderMode::LauncherCommands,
            },
            TemplateAsset {
                source_rel: "docs/integrations/templates/codex/config.toml",
                destination_rel: ".codex/config.toml",
                mode: RenderMode::McpConfig,
            },
        ],
        IntegrationPlatform::ClaudeCode => &[
            TemplateAsset {
                source_rel: "docs/integrations/templates/claude-code/CLAUDE.md",
                destination_rel: "CLAUDE.md",
                mode: RenderMode::LauncherCommands,
            },
            TemplateAsset {
                source_rel: "docs/integrations/templates/claude-code/.claude/commands/publish-archive.md",
                destination_rel: ".claude/commands/publish-archive.md",
                mode: RenderMode::LauncherCommands,
            },
            TemplateAsset {
                source_rel: "docs/integrations/templates/claude-code/.claude/commands/search-semantic-report.md",
                destination_rel: ".claude/commands/search-semantic-report.md",
                mode: RenderMode::LauncherCommands,
            },
            TemplateAsset {
                source_rel: "docs/integrations/templates/claude-code/.claude/commands/search-hybrid-report.md",
                destination_rel: ".claude/commands/search-hybrid-report.md",
                mode: RenderMode::LauncherCommands,
            },
            TemplateAsset {
                source_rel: "docs/integrations/templates/claude-code/.mcp.json",
                destination_rel: ".mcp.json",
                mode: RenderMode::McpConfig,
            },
        ],
        IntegrationPlatform::OpenClaw => &[
            TemplateAsset {
                source_rel: "docs/integrations/templates/openclaw-codex-bundle/.codex-plugin/plugin.json",
                destination_rel: "openclaw-codex-bundle/.codex-plugin/plugin.json",
                mode: RenderMode::Plain,
            },
            TemplateAsset {
                source_rel: "docs/integrations/templates/openclaw-codex-bundle/skills/export-archive/SKILL.md",
                destination_rel: "openclaw-codex-bundle/skills/export-archive/SKILL.md",
                mode: RenderMode::LauncherCommands,
            },
            TemplateAsset {
                source_rel: "docs/integrations/templates/openclaw-codex-bundle/.mcp.json",
                destination_rel: "openclaw-codex-bundle/.mcp.json",
                mode: RenderMode::McpConfig,
            },
            TemplateAsset {
                source_rel: "docs/integrations/templates/openclaw-claude-bundle/.claude-plugin/plugin.json",
                destination_rel: "openclaw-claude-bundle/.claude-plugin/plugin.json",
                mode: RenderMode::Plain,
            },
            TemplateAsset {
                source_rel: "docs/integrations/templates/openclaw-claude-bundle/commands/search-semantic-report.md",
                destination_rel: "openclaw-claude-bundle/commands/search-semantic-report.md",
                mode: RenderMode::LauncherCommands,
            },
            TemplateAsset {
                source_rel: "docs/integrations/templates/openclaw-claude-bundle/commands/search-hybrid-report.md",
                destination_rel: "openclaw-claude-bundle/commands/search-hybrid-report.md",
                mode: RenderMode::LauncherCommands,
            },
            TemplateAsset {
                source_rel: "docs/integrations/templates/openclaw-claude-bundle/.mcp.json",
                destination_rel: "openclaw-claude-bundle/.mcp.json",
                mode: RenderMode::McpConfig,
            },
        ],
    }
}

fn platform_specific_checks(
    platform: IntegrationPlatform,
    target_root: &Path,
) -> Vec<IntegrationDoctorCheck> {
    match platform {
        IntegrationPlatform::Codex => vec![check_codex_config(target_root)],
        IntegrationPlatform::ClaudeCode => {
            vec![
                check_claude_mcp(target_root),
                check_claude_pack(target_root),
            ]
        }
        IntegrationPlatform::OpenClaw => vec![check_openclaw_bundle(target_root)],
    }
}

fn check_codex_config(target_root: &Path) -> IntegrationDoctorCheck {
    let path = target_root.join(".codex").join("config.toml");
    let Some(content) = fs::read_to_string(&path).ok() else {
        return IntegrationDoctorCheck {
            label: "codex_config_shape",
            readiness: IntegrationReadiness::Missing,
            detail: path.display().to_string(),
        };
    };

    let parsed = content.parse::<TomlValue>();
    match parsed {
        Ok(value) => match value
            .get("mcp_servers")
            .and_then(|servers| servers.get("agent_exporter"))
        {
            Some(server)
                if server
                    .get("command")
                    .and_then(TomlValue::as_str)
                    .is_some()
                    && server
                        .get("args")
                        .and_then(TomlValue::as_array)
                        .is_some_and(|args| !args.is_empty()) =>
            {
                IntegrationDoctorCheck {
                    label: "codex_config_shape",
                    readiness: IntegrationReadiness::Ready,
                    detail: "`.codex/config.toml` contains `mcp_servers.agent_exporter.command` and a non-empty `args` array".to_string(),
                }
            }
            Some(_) => IntegrationDoctorCheck {
                label: "codex_config_shape",
                readiness: IntegrationReadiness::Partial,
                detail: "`.codex/config.toml` parsed, but `mcp_servers.agent_exporter` is missing `command` or a non-empty `args` array".to_string(),
            },
            None => IntegrationDoctorCheck {
                label: "codex_config_shape",
                readiness: IntegrationReadiness::Partial,
                detail: "`.codex/config.toml` parsed, but `mcp_servers.agent_exporter` is missing".to_string(),
            },
        },
        Err(error) => IntegrationDoctorCheck {
            label: "codex_config_shape",
            readiness: IntegrationReadiness::Partial,
            detail: format!("`.codex/config.toml` failed to parse: {error}"),
        },
    }
}

fn check_claude_mcp(target_root: &Path) -> IntegrationDoctorCheck {
    let path = target_root.join(".mcp.json");
    let Some(content) = fs::read_to_string(&path).ok() else {
        return IntegrationDoctorCheck {
            label: "claude_project_shape",
            readiness: IntegrationReadiness::Missing,
            detail: path.display().to_string(),
        };
    };

    let parsed = serde_json::from_str::<JsonValue>(&content);
    match parsed {
        Ok(value)
            if value
                .get("mcpServers")
                .and_then(|servers| servers.get("agent-exporter"))
                .and_then(|entry| entry.get("command"))
                .and_then(|value| value.as_str())
                .is_some() =>
        {
            IntegrationDoctorCheck {
                label: "claude_project_shape",
                readiness: IntegrationReadiness::Ready,
                detail: "`.mcp.json` contains `mcpServers.agent-exporter.command`".to_string(),
            }
        }
        Ok(_) => IntegrationDoctorCheck {
            label: "claude_project_shape",
            readiness: IntegrationReadiness::Partial,
            detail: "`.mcp.json` parsed, but `mcpServers.agent-exporter.command` is missing"
                .to_string(),
        },
        Err(error) => IntegrationDoctorCheck {
            label: "claude_project_shape",
            readiness: IntegrationReadiness::Partial,
            detail: format!("`.mcp.json` failed to parse: {error}"),
        },
    }
}

fn check_claude_pack(target_root: &Path) -> IntegrationDoctorCheck {
    let claude_md = target_root.join("CLAUDE.md");
    let publish_command = target_root
        .join(".claude")
        .join("commands")
        .join("publish-archive.md");

    let Some(claude_md_content) = fs::read_to_string(&claude_md).ok() else {
        return IntegrationDoctorCheck {
            label: "claude_pack_shape",
            readiness: IntegrationReadiness::Missing,
            detail: claude_md.display().to_string(),
        };
    };
    let Some(command_content) = fs::read_to_string(&publish_command).ok() else {
        return IntegrationDoctorCheck {
            label: "claude_pack_shape",
            readiness: IntegrationReadiness::Missing,
            detail: publish_command.display().to_string(),
        };
    };

    let has_heading = claude_md_content.lines().any(|line| line.starts_with('#'));
    let has_description = command_content.contains("description:");
    let has_bash_block = command_content.contains("```bash");

    if has_heading && has_description && has_bash_block {
        IntegrationDoctorCheck {
            label: "claude_pack_shape",
            readiness: IntegrationReadiness::Ready,
            detail: "`CLAUDE.md` and `.claude/commands/*.md` look like a valid project pack"
                .to_string(),
        }
    } else {
        IntegrationDoctorCheck {
            label: "claude_pack_shape",
            readiness: IntegrationReadiness::Partial,
            detail: "Claude pack is missing a heading, command description, or bash command block"
                .to_string(),
        }
    }
}

fn check_openclaw_bundle(target_root: &Path) -> IntegrationDoctorCheck {
    let codex_plugin = target_root
        .join("openclaw-codex-bundle")
        .join(".codex-plugin")
        .join("plugin.json");
    let claude_plugin = target_root
        .join("openclaw-claude-bundle")
        .join(".claude-plugin")
        .join("plugin.json");
    let codex_mcp = target_root.join("openclaw-codex-bundle").join(".mcp.json");
    let claude_mcp = target_root.join("openclaw-claude-bundle").join(".mcp.json");

    let checks = [
        parse_manifest(&codex_plugin),
        parse_manifest(&claude_plugin),
        parse_mcp_config(&codex_mcp),
        parse_mcp_config(&claude_mcp),
    ];

    if checks.iter().all(|result| result.is_ok()) {
        IntegrationDoctorCheck {
            label: "openclaw_bundle_shape",
            readiness: IntegrationReadiness::Ready,
            detail: "bundle manifests and `.mcp.json` files parsed successfully".to_string(),
        }
    } else if checks.iter().any(|result| {
        result
            .as_ref()
            .err()
            .is_some_and(|detail| detail.contains("missing"))
    }) {
        IntegrationDoctorCheck {
            label: "openclaw_bundle_shape",
            readiness: IntegrationReadiness::Missing,
            detail: checks
                .into_iter()
                .filter_map(Result::err)
                .next()
                .unwrap_or_else(|| "bundle files are missing".to_string()),
        }
    } else {
        IntegrationDoctorCheck {
            label: "openclaw_bundle_shape",
            readiness: IntegrationReadiness::Partial,
            detail: checks
                .into_iter()
                .filter_map(Result::err)
                .next()
                .unwrap_or_else(|| "bundle files are malformed".to_string()),
        }
    }
}

fn parse_manifest(path: &Path) -> Result<(), String> {
    let content = fs::read_to_string(path).map_err(|_| format!("missing `{}`", path.display()))?;
    let value = serde_json::from_str::<JsonValue>(&content)
        .map_err(|error| format!("`{}` failed to parse: {error}", path.display()))?;
    if value.get("name").and_then(|value| value.as_str()).is_none()
        || value
            .get("version")
            .and_then(|value| value.as_str())
            .is_none()
    {
        return Err(format!(
            "`{}` parsed, but required `name`/`version` keys are missing",
            path.display()
        ));
    }
    Ok(())
}

fn parse_mcp_config(path: &Path) -> Result<(), String> {
    let content = fs::read_to_string(path).map_err(|_| format!("missing `{}`", path.display()))?;
    let value = serde_json::from_str::<JsonValue>(&content)
        .map_err(|error| format!("`{}` failed to parse: {error}", path.display()))?;
    if value
        .get("mcpServers")
        .and_then(|servers| servers.get("agent-exporter"))
        .and_then(|entry| entry.get("command"))
        .and_then(|value| value.as_str())
        .is_none()
    {
        return Err(format!(
            "`{}` parsed, but `mcpServers.agent-exporter.command` is missing",
            path.display()
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{LauncherSpec, probe_launcher};

    #[test]
    fn cargo_run_launcher_is_not_executed_in_doctor_mode() {
        let launcher = LauncherSpec {
            kind: "cargo-run",
            command: "cargo".to_string(),
            args: vec![
                "run".to_string(),
                "--quiet".to_string(),
                "--manifest-path".to_string(),
                "/tmp/demo/Cargo.toml".to_string(),
                "--bin".to_string(),
                "agent-exporter".to_string(),
                "--".to_string(),
            ],
        };

        let outcome = probe_launcher(&launcher);
        assert!(!outcome.success);
        assert!(
            outcome
                .detail
                .contains("not executed in read-only doctor mode")
        );
    }

    #[test]
    fn repo_local_launcher_probe_runs_for_binary_launchers() {
        let launcher = LauncherSpec {
            kind: "repo-local-debug",
            command: "true".to_string(),
            args: Vec::new(),
        };

        let outcome = probe_launcher(&launcher);
        assert!(outcome.success);
        assert!(outcome.detail.contains("can execute `connectors`"));
    }
}
