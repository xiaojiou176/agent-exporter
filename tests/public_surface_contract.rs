use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read_repo_file(relative_path: &str) -> String {
    fs::read_to_string(repo_root().join(relative_path))
        .unwrap_or_else(|error| panic!("failed to read {relative_path}: {error}"))
}

fn contains_han_character(content: &str) -> bool {
    content.chars().any(|ch| {
        matches!(
            ch,
            '\u{3400}'..='\u{4dbf}' | '\u{4e00}'..='\u{9fff}' | '\u{f900}'..='\u{faff}'
        )
    })
}

fn assert_no_han_characters(path: &str) {
    let content = read_repo_file(path);
    assert!(
        !contains_han_character(&content),
        "public-facing file still contains unplanned Han characters: {path}"
    );
}

fn assert_contains_all(content: &str, expected: &[&str], path: &str) {
    for needle in expected {
        assert!(
            content.contains(needle),
            "expected `{needle}` in {path}, but it was missing"
        );
    }
}

fn public_front_door_files() -> &'static [&'static str] {
    &[
        "README.md",
        "docs/README.md",
        "docs/index.md",
        "docs/archive-shell-proof.md",
        "docs/repo-map.md",
        "docs/_layouts/default.html",
        "public-skills/README.md",
        "public-skills/agent-exporter-archive-governance-workbench/README.md",
        "public-skills/agent-exporter-archive-governance-workbench/SKILL.md",
        "public-skills/agent-exporter-archive-governance-workbench/manifest.yaml",
    ]
}

#[test]
fn public_front_door_files_stay_language_normalized() {
    for path in public_front_door_files() {
        assert!(
            repo_root().join(path).exists(),
            "expected tracked public-facing file to exist: {path}"
        );
        assert_no_han_characters(path);
    }
}

#[test]
fn public_skill_packet_truth_matches_live_and_pending_lanes() {
    let manifest_path = "public-skills/agent-exporter-archive-governance-workbench/manifest.yaml";
    let manifest = read_repo_file(manifest_path);

    assert_contains_all(
        &manifest,
        &[
            "clawhub:",
            "status: listed-live",
            "read_back: clawhub inspect agent-exporter-archive-governance-workbench --no-input",
            "openhands-extensions:",
            "status: review-pending",
            "submission_ref: https://github.com/OpenHands/extensions/pull/162",
            "listing_state_summary: ClawHub listed-live; OpenHands/extensions review-pending.",
            "No listed-live OpenHands/extensions entry exists yet",
        ],
        manifest_path,
    );

    for stale in [
        "ready-but-not-listed",
        "not-yet-listed",
        "No live ClawHub listing exists yet",
        "No live OpenHands/extensions listing exists yet",
    ] {
        assert!(
            !manifest.contains(stale),
            "stale registry truth `{stale}` reappeared in {manifest_path}"
        );
    }
}

#[test]
fn public_skill_packet_prose_keeps_live_lane_truth_in_sync() {
    let expected = [
        "`ClawHub`: `listed-live`",
        "`OpenHands/extensions`: `review-pending`",
    ];

    for path in [
        "public-skills/README.md",
        "public-skills/agent-exporter-archive-governance-workbench/README.md",
        "public-skills/agent-exporter-archive-governance-workbench/SKILL.md",
    ] {
        let content = read_repo_file(path);
        assert_contains_all(&content, &expected, path);

        for stale in [
            "ready-but-not-listed",
            "not-yet-listed",
            "No live ClawHub listing exists yet",
            "No live OpenHands/extensions listing exists yet",
        ] {
            assert!(
                !content.contains(stale),
                "stale packet truth `{stale}` reappeared in {path}"
            );
        }
    }
}

#[test]
fn front_door_docs_keep_cli_first_and_local_first_truth() {
    for path in [
        "README.md",
        "docs/README.md",
        "docs/index.md",
        "public-skills/README.md",
        "public-skills/agent-exporter-archive-governance-workbench/README.md",
        "public-skills/agent-exporter-archive-governance-workbench/SKILL.md",
    ] {
        let content = read_repo_file(path);
        assert!(
            content.contains("CLI-first"),
            "CLI-first primary truth disappeared from {path}"
        );
        assert!(
            content.contains("local-first"),
            "local-first product identity disappeared from {path}"
        );
    }
}

#[test]
fn front_door_docs_keep_release_shelf_truth_explicit() {
    for path in ["README.md", "docs/README.md", "docs/index.md"] {
        let content = read_repo_file(path);
        assert!(
            content.contains("Release Shelf Truth"),
            "release shelf truth section disappeared from {path}"
        );
        assert!(
            content.contains("published"),
            "published release wording disappeared from {path}"
        );
        assert!(
            content.contains("repository-side truth"),
            "current main vs release shelf distinction disappeared from {path}"
        );
    }
}

#[test]
fn pages_index_keeps_main_landmark_and_visibility_styles() {
    let content = read_repo_file("docs/index.md");
    assert!(
        content.contains("<main id=\"main-content\" role=\"main\" markdown=\"1\">"),
        "docs/index.md lost the explicit main landmark wrapper"
    );
    assert!(
        content.contains("text-decoration: underline;"),
        "docs/index.md lost the link visibility style"
    );
    assert!(
        content.contains(".markdown-body .highlight .nb"),
        "docs/index.md lost the code contrast style override"
    );
    let layout = read_repo_file("docs/_layouts/default.html");
    assert!(
        layout.contains("role=\"contentinfo\""),
        "docs/_layouts/default.html lost the footer contentinfo landmark"
    );
}
