use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read_repo_file(relative_path: &str) -> String {
    fs::read_to_string(repo_root().join(relative_path))
        .unwrap_or_else(|error| panic!("failed to read {relative_path}: {error}"))
}

fn assert_contains_all(content: &str, expected: &[&str], path: &str) {
    for needle in expected {
        assert!(
            content.contains(needle),
            "expected `{needle}` in {path}, but it was missing"
        );
    }
}

fn contains_openai_style_secret(content: &str) -> bool {
    let bytes = content.as_bytes();
    let mut index = 0;

    while index + 3 <= bytes.len() {
        if &bytes[index..index + 3] != b"sk-" {
            index += 1;
            continue;
        }

        let mut tail_len = 0;
        let mut cursor = index + 3;
        while cursor < bytes.len() {
            let byte = bytes[cursor];
            if byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-') {
                tail_len += 1;
                cursor += 1;
                continue;
            }
            break;
        }

        if tail_len >= 20 {
            return true;
        }

        index = cursor.max(index + 1);
    }

    false
}

#[test]
fn gitignore_covers_sensitive_local_tooling_paths() {
    let content = read_repo_file(".gitignore");
    assert_contains_all(
        &content,
        &[
            ".agents/",
            ".agent/",
            ".codex/",
            ".claude/",
            ".cursor/",
            ".venv/",
            ".runtime-cache/",
            ".env",
        ],
        ".gitignore",
    );
}

#[test]
fn ci_keeps_clippy_and_security_contract_on_the_required_path() {
    let content = read_repo_file(".github/workflows/ci.yml");
    assert_contains_all(
        &content,
        &[
            "cargo fmt --check",
            "cargo clippy --all-targets --all-features -- -D warnings",
            "cargo test",
            "cargo test --test security_contract",
            "cargo test --test public_surface_contract",
        ],
        ".github/workflows/ci.yml",
    );
}

#[test]
fn tracked_text_files_do_not_contain_live_secret_material() {
    let output = Command::new("git")
        .args(["ls-files"])
        .current_dir(repo_root())
        .output()
        .expect("git ls-files should succeed");
    assert!(
        output.status.success(),
        "git ls-files failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let mut violations = Vec::new();

    for line in String::from_utf8_lossy(&output.stdout).lines() {
        let relative = line.trim();
        if relative.is_empty() {
            continue;
        }

        let path_buf = PathBuf::from(relative);
        let extension = path_buf
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or_default();
        let likely_text = matches!(
            extension,
            "" | "md"
                | "txt"
                | "json"
                | "jsonl"
                | "toml"
                | "yaml"
                | "yml"
                | "rs"
                | "py"
                | "sh"
                | "html"
                | "css"
                | "js"
                | "ts"
                | "svg"
        );
        if !likely_text {
            continue;
        }

        let path = repo_root().join(relative);
        let Ok(content) = fs::read_to_string(&path) else {
            continue;
        };
        let lowercase = content.to_ascii_lowercase();
        if lowercase.contains("ghp_") {
            violations.push(format!("{relative}: ghp_"));
        }
        if contains_openai_style_secret(&content) {
            violations.push(format!("{relative}: sk-<long-secret>"));
        }
        for marker in [
            "xoxp-",
            "xoxb-",
            "-----begin private key-----",
            "-----begin openssh private key-----",
            "aws_secret_access_key",
            "authorization: bearer ",
            "\"api_key\":",
        ] {
            if lowercase.contains(marker) {
                violations.push(format!("{relative}: {marker}"));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "tracked text files appear to contain live secret material:\n{}",
        violations.join("\n")
    );
}
