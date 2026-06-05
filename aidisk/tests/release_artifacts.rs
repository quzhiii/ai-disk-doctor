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
fn crate_version_and_readme_reference_release_artifacts() {
    let cargo_toml = read_repo_file("aidisk/Cargo.toml");
    let readme = read_repo_file("README.md");
    let readme_zh = read_repo_file("README.zh-CN.md");

    assert!(cargo_toml.contains("version = \"1.1.0\""));
    assert!(readme.contains("CHANGELOG.md"));
    assert!(readme.contains("docs/release-notes/v1.1.0.md"));
    assert!(readme_zh.contains("docs/release-notes/v1.1.0.md"));
    assert!(readme.contains("scripts/release-smoke.ps1"));
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
