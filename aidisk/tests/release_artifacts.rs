use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("aidisk crate should be inside repository root")
        .to_path_buf()
}

fn read_repo_file(path: &str) -> String {
    fs::read_to_string(repo_root().join(path))
        .unwrap_or_else(|error| panic!("failed to read repository file {path}: {error}"))
}

#[test]
fn changelog_and_release_notes_cover_v1_2_scope() {
    let changelog = read_repo_file("CHANGELOG.md");
    let release_notes = read_repo_file("docs/release-notes/v1.2.0.md");
    let required_terms = [
        "scan --large-files",
        "500MB",
        "node_modules",
        "__pycache__",
        "structured JSON",
        "stderr",
        "~/",
        "%VAR%",
        "policy snapshot",
        "best-effort, not exact",
        "scan --policy",
    ];

    assert!(changelog.contains("## 1.2.0"));
    assert!(release_notes.contains("# Windows AI Space Manager v1.2.0"));
    assert!(release_notes.contains("## Test Plan"));
    assert!(release_notes.contains("## Safety Boundaries"));
    assert!(release_notes.contains("## Known Limits"));

    for term in required_terms {
        assert!(changelog.contains(term), "CHANGELOG.md should mention {term}");
        assert!(
            release_notes.contains(term),
            "release notes should mention {term}"
        );
    }

    assert!(
        changelog.contains("rule-driven `scan`, `plan`, and `doctor`"),
        "CHANGELOG.md should scope policy metadata to rule-driven reports"
    );
    assert!(
        release_notes.contains("rule-driven `scan`, `plan`, and `doctor`"),
        "release notes should scope policy metadata to rule-driven reports"
    );
    assert!(
        changelog.contains("rule-driven read-only scans"),
        "CHANGELOG.md should scope scan --policy to rule-driven read-only scans"
    );
    assert!(
        release_notes.contains("rule-driven read-only scans"),
        "release notes should scope scan --policy to rule-driven read-only scans"
    );
    assert!(
        changelog.contains("mark sizes as `(partial)`")
            && changelog.contains("best-effort, not exact"),
        "CHANGELOG.md should describe partial marker plus warning wording"
    );
    assert!(
        release_notes.contains("mark sizes as `(partial)`")
            && release_notes.contains("best-effort, not exact"),
        "release notes should describe partial marker plus warning wording"
    );
}

#[test]
fn changelog_and_release_notes_cover_v1_scope() {
    let changelog = read_repo_file("CHANGELOG.md");
    let release_notes = read_repo_file("docs/release-notes/v1.0.0.md");
    let required_terms = [
        "scan",
        "plan",
        "clean",
        "restore",
        "doctor",
        "diff --latest",
        "--rules-repo",
        "quarantine",
    ];

    assert!(changelog.contains("## 1.0.0"));
    assert!(release_notes.contains("# Windows AI Space Manager v1.0.0"));
    assert!(release_notes.contains("## Test Plan"));
    assert!(release_notes.contains("## Safety Boundaries"));
    assert!(release_notes.contains("## Known Limits"));

    for term in required_terms {
        assert!(changelog.contains(term), "CHANGELOG.md should mention {term}");
        assert!(
            release_notes.contains(term),
            "release notes should mention {term}"
        );
    }
}

#[test]
fn changelog_and_release_notes_cover_v1_1_scope() {
    let changelog = read_repo_file("CHANGELOG.md");
    let release_notes = read_repo_file("docs/release-notes/v1.1.0.md");
    let required_terms = [
        "doctor --agents",
        "--probe-tools",
        "doctor --latest",
        "DoctorTopicSpec",
        "topic registry",
    ];

    assert!(changelog.contains("## 1.1.0"));
    assert!(release_notes.contains("# Windows AI Space Manager v1.1.0"));
    assert!(release_notes.contains("## Test Plan"));
    assert!(release_notes.contains("## Safety Boundaries"));
    assert!(release_notes.contains("## Known Limits"));

    for term in required_terms {
        assert!(changelog.contains(term), "CHANGELOG.md should mention {term}");
        assert!(
            release_notes.contains(term),
            "release notes should mention {term}"
        );
    }
}

#[test]
fn smoke_script_is_non_destructive_and_covers_core_commands() {
    let script = read_repo_file("scripts/release-smoke.ps1");
    let required_commands = [
        "cargo test",
        "scan --rules-repo",
        "scan --large-files --min-size 500MB",
        "plan --safe-only",
        "clean --dry-run",
        "doctor --markdown",
        "diff --before",
    ];

    for command in required_commands {
        assert!(
            script.contains(command),
            "release-smoke.ps1 should include {command}"
        );
    }

    assert!(!script.contains("--yes"));
    assert!(!script.contains("clean --yes"));
}

#[test]
fn governance_script_is_non_destructive_and_covers_scan_anomaly_workflow() {
    let script = read_repo_file("scripts/governance/run-governance.ps1");

    assert!(script.contains("cargo run -- scan --json"));
    assert!(script.contains("cargo run -- anomaly --latest"));
    assert!(script.contains("-NotifierAdapter"));
    assert!(script.contains("-WebhookUrl"));
    assert!(script.contains("Copy-Item"));
    assert!(script.contains("requires at least two scan snapshots"));
    assert!(script.contains("Invoke-RestMethod"));
    assert!(script.contains("ContentType \"application/json\""));
    assert!(script.contains("governance-event.json"));
    assert!(script.contains("anomaly_found"));
    assert!(script.contains("pending_history"));
    assert!(script.contains("no_anomaly"));
    assert!(!script.contains("clean --yes"));
    assert!(!script.contains("Remove-Item"));
}

#[test]
fn crate_version_and_readme_reference_release_artifacts() {
    let cargo_toml = read_repo_file("aidisk/Cargo.toml");
    let readme = read_repo_file("README.md");
    let readme_zh = read_repo_file("README.zh-CN.md");

    assert!(cargo_toml.contains("version = \"1.2.0\""));
    assert!(readme.contains("CHANGELOG.md"));
    assert!(readme.contains("docs/release-notes/v1.2.0.md"));
    assert!(readme_zh.contains("docs/release-notes/v1.2.0.md"));
    assert!(readme.contains("scripts/release-smoke.ps1"));
}

#[test]
fn cargo_toml_defines_release_profile_for_distributable_binary() {
    let cargo_toml = read_repo_file("aidisk/Cargo.toml");

    assert!(cargo_toml.contains("[profile.release]"));
    assert!(cargo_toml.contains("lto = \"thin\""));
    assert!(cargo_toml.contains("strip = \"symbols\""));
    assert!(cargo_toml.contains("codegen-units = 1"));
    assert!(cargo_toml.contains("opt-level = \"z\""));
}

#[test]
fn github_actions_run_tests_and_build_windows_release_artifact() {
    let ci = read_repo_file(".github/workflows/ci.yml");
    let release = read_repo_file(".github/workflows/release-artifacts.yml");

    assert!(ci.contains("cargo test"));
    assert!(ci.contains("working-directory: aidisk"));
    assert!(ci.contains("runs-on: windows-2025"));
    assert!(ci.contains("actions/checkout@v5"));
    assert!(release.contains("cargo build --release"));
    assert!(release.contains("runs-on: windows-2025"));
    assert!(release.contains("actions/checkout@v5"));
    assert!(release.contains("aidisk.exe"));
    assert!(release.contains("actions/upload-artifact@v7"));
}

#[test]
fn cargo_toml_includes_progress_terminal_dependencies() {
    let cargo_toml = read_repo_file("aidisk/Cargo.toml");

    assert!(cargo_toml.contains("indicatif"));
    assert!(cargo_toml.contains("console"));
}

#[test]
fn repository_uses_dual_license_files_without_duplicate_root_license() {
    let root = repo_root();

    assert!(root.join("LICENSE-MIT").is_file());
    assert!(root.join("LICENSE-APACHE").is_file());
    assert!(
        !root.join("LICENSE").exists(),
        "root LICENSE duplicates the dual-license tabs on GitHub"
    );
}
