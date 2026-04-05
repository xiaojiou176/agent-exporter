use std::fs;
use std::path::{Path, PathBuf};

use assert_cmd::Command;
use predicates::str::contains;

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("mock_codex_app_server.py")
}

fn banned_runtime_patterns() -> &'static [&'static str] {
    &[
        "killall",
        "pkill",
        "kill -9",
        "process.kill(",
        "os.kill(",
        "killpg(",
        "osascript",
        "system events",
        "appleevent",
        "loginwindow",
        "showforcequitpanel",
        "detached: true",
        ".unref()",
    ]
}

fn strip_host_safety_rule_tables(content: &str) -> String {
    let mut cleaned = String::new();
    let mut skipping = false;

    for line in content.lines() {
        if line.contains("HOST_SAFETY_RULES_BEGIN") {
            skipping = true;
            continue;
        }
        if line.contains("HOST_SAFETY_RULES_END") {
            skipping = false;
            continue;
        }
        if !skipping {
            cleaned.push_str(line);
            cleaned.push('\n');
        }
    }

    cleaned
}

fn collect_rust_files(root: &Path, files: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(root).expect("source directory should exist");
    for entry in entries {
        let entry = entry.expect("directory entry");
        let path = entry.path();
        if path.is_dir() {
            collect_rust_files(&path, files);
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            files.push(path);
        }
    }
}

#[test]
fn runtime_code_stays_free_of_banned_host_control_primitives() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src_root = repo_root.join("src");
    let mut rust_files = Vec::new();
    collect_rust_files(&src_root, &mut rust_files);
    rust_files.sort();

    let mut violations = Vec::new();
    for file in rust_files {
        let original = fs::read_to_string(&file).expect("source file should be readable");
        let sanitized = strip_host_safety_rule_tables(&original).to_ascii_lowercase();
        for pattern in banned_runtime_patterns() {
            if sanitized.contains(pattern) {
                let relative = file
                    .strip_prefix(&repo_root)
                    .expect("file should stay inside repo")
                    .display()
                    .to_string();
                violations.push(format!("{relative}: {pattern}"));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "runtime code reintroduced banned host-control primitives:\n{}",
        violations.join("\n")
    );
}

#[test]
fn codex_export_rejects_host_control_override_commands() {
    let mut command = Command::cargo_bin("agent-exporter").expect("binary should build");
    command
        .arg("export")
        .arg("codex")
        .arg("--thread-id")
        .arg("complete-thread")
        .arg("--app-server-command")
        .arg("osascript")
        .arg("--app-server-arg=-e")
        .arg("--app-server-arg")
        .arg("tell application \"System Events\" to key code 36");

    command
        .assert()
        .failure()
        .stderr(contains("refuses to launch host-control utility"));
}

#[test]
fn codex_export_rejects_inline_eval_launchers() {
    let mut command = Command::cargo_bin("agent-exporter").expect("binary should build");
    command
        .arg("export")
        .arg("codex")
        .arg("--thread-id")
        .arg("complete-thread")
        .arg("--app-server-command")
        .arg("python3")
        .arg("--app-server-arg=-c")
        .arg("--app-server-arg")
        .arg("print('not an app server')");

    command
        .assert()
        .failure()
        .stderr(contains("refuses inline-eval launcher"));
}

#[test]
fn codex_export_allows_repo_owned_fixture_server() {
    let mut command = Command::cargo_bin("agent-exporter").expect("binary should build");
    command
        .arg("export")
        .arg("codex")
        .arg("--thread-id")
        .arg("complete-thread")
        .arg("--app-server-command")
        .arg(std::env::var("PYTHON").unwrap_or_else(|_| "python3".to_string()))
        .arg("--app-server-arg")
        .arg(fixture_path());

    command
        .assert()
        .success()
        .stdout(contains("Export completed"));
}
