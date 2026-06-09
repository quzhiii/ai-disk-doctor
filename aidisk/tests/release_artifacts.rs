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
fn changelog_and_release_notes_cover_v1_3_scope() {
    let changelog = read_repo_file("CHANGELOG.md");
    let release_notes = read_repo_file("docs/release-notes/v1.3.0.md");
    let required_terms = [
        "Local Scheduled Governance",
        "anomaly",
        "absolute + relative",
        "run-governance.ps1",
        "governance-event.json",
        "anomaly_found",
        "pending_history",
        "no_anomaly",
        "generic webhook",
        "webhook-failure.json",
        "Windows Task Scheduler",
        "test-run-governance-task.ps1",
        "Start-ScheduledTask",
    ];

    assert!(changelog.contains("## 1.3.0"));
    assert!(release_notes.contains("# Windows AI Space Manager v1.3.0"));
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
        changelog.contains("does not perform cleanup")
            && release_notes.contains("does not perform cleanup"),
        "v1.3.0 release artifacts should preserve the no-cleanup governance boundary"
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
        "cargo build",
        "target\\debug\\aidisk.exe",
        "scan --rules-repo",
        "scan --large-files --min-size 500MB",
        "--root",
        "plan --safe-only",
        "clean --dry-run",
        "doctor --markdown",
        "diff --before",
        "anomaly --before",
    ];

    for command in required_commands {
        assert!(
            script.contains(command),
            "release-smoke.ps1 should include {command}"
        );
    }

    assert!(script.contains("tests\\fixtures\\windows-user"));
    assert!(script.contains("$env:USERPROFILE"));
    assert!(script.contains("$env:LOCALAPPDATA"));
    assert!(script.contains("$env:APPDATA"));
    assert!(script.contains("$env:HOME"));
    assert!(!script.contains("cargo run"));
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
    assert!(script.contains("-WebhookTimeoutSeconds"));
    assert!(script.contains("Copy-Item"));
    assert!(script.contains("requires at least two scan snapshots"));
    assert!(script.contains("Invoke-RestMethod"));
    assert!(script.contains("ContentType \"application/json\""));
    assert!(script.contains("governance-event.json"));
    assert!(script.contains("anomaly_found"));
    assert!(script.contains("pending_history"));
    assert!(script.contains("no_anomaly"));
    assert!(script.contains("headline"));
    assert!(script.contains("summary_markdown"));
    assert!(script.contains("top_anomaly_path"));
    assert!(script.contains("top_anomaly_growth_bytes"));
    assert!(script.contains("webhook-failure.json"));
    assert!(script.contains("delivery_status"));
    assert!(script.contains("function New-GovernanceEvent"));
    assert!(script.contains("function Send-NotifierEvent"));
    assert!(!script.contains("clean --yes"));
    assert!(!script.contains("Remove-Item"));
}

#[test]
fn scheduler_setup_script_registers_windows_task_for_governance() {
    let script = read_repo_file("scripts/governance/register-governance-task.ps1");

    assert!(script.contains("Register-ScheduledTask"));
    assert!(script.contains("New-ScheduledTaskAction"));
    assert!(script.contains("New-ScheduledTaskTrigger"));
    assert!(script.contains("run-governance.ps1"));
    assert!(script.contains("-TaskName"));
    assert!(script.contains("-DailyAt"));
    assert!(!script.contains("clean --yes"));
    assert!(!script.contains("Remove-Item"));
}

#[test]
fn scheduler_management_scripts_can_show_and_unregister_task() {
    let show_script = read_repo_file("scripts/governance/show-governance-task.ps1");
    let unregister_script = read_repo_file("scripts/governance/unregister-governance-task.ps1");

    assert!(show_script.contains("Get-ScheduledTask"));
    assert!(show_script.contains("Get-ScheduledTaskInfo"));
    assert!(show_script.contains("-TaskName"));

    assert!(unregister_script.contains("Unregister-ScheduledTask"));
    assert!(unregister_script.contains("-TaskName"));
    assert!(unregister_script.contains("-Confirm:$false"));
    assert!(!unregister_script.contains("Remove-Item"));
    assert!(!unregister_script.contains("clean --yes"));
}

#[test]
fn scheduler_test_run_script_starts_existing_governance_task() {
    let script = read_repo_file("scripts/governance/test-run-governance-task.ps1");

    assert!(script.contains("Start-ScheduledTask"));
    assert!(script.contains("Get-ScheduledTask"));
    assert!(script.contains("-TaskName"));
    assert!(script.contains("AI Disk Doctor Governance"));
    assert!(script.contains("Write-Host"));
    assert!(script.contains("show-governance-task.ps1"));
    assert!(!script.contains("Register-ScheduledTask"));
    assert!(!script.contains("Unregister-ScheduledTask"));
    assert!(!script.contains("Remove-Item"));
    assert!(!script.contains("clean --yes"));
}

#[test]
fn phase_9_roadmap_marks_local_governance_complete() {
    let roadmap = read_repo_file("docs/execution-plan.md");
    let phase_plan = read_repo_file("docs/plans/2026-06-08-phase-9-local-scheduled-governance.md");

    let required_terms = [
        "Phase 9 status: Completed",
        "`aidisk anomaly --latest`",
        "`governance-event.json`",
        "generic webhook",
        "Windows Task Scheduler",
        "register-governance-task.ps1",
        "show-governance-task.ps1",
        "unregister-governance-task.ps1",
        "test-run-governance-task.ps1",
        "不做后台常驻、不自动清理、不绑定单一 IM 服务",
        "Phase 9 Immediate Next Steps",
        "v1.3.0 release readiness",
    ];

    for term in required_terms {
        assert!(roadmap.contains(term), "Phase 9 roadmap should mention {term}");
    }

    assert!(
        phase_plan.contains("Extended completion notes")
            && phase_plan.contains("test-run-governance-task.ps1")
            && phase_plan.contains("webhook-failure.json")
            && phase_plan.contains("Phase 9 status: Completed"),
        "Phase 9 implementation plan should summarize the completed extended scope"
    );
}

#[test]
fn roadmap_and_reference_docs_reflect_post_phase_9_state() {
    let roadmap = read_repo_file("docs/execution-plan.md");
    let phase_8_plan = read_repo_file("docs/plans/2026-06-07-phase-8-hardening-operability.md");
    let phase_10_plan = read_repo_file("docs/plans/2026-06-09-phase-10-cross-platform-scheduler-adapters.md");
    let rules_spec = read_repo_file("docs/rules-spec.md");
    let storage_map = read_repo_file("docs/windows-ai-storage-map.md");

    let roadmap_terms = [
        "Doctor V2 status: Completed",
        "Phase 8 status: Completed",
        "v1.3.0 release readiness is complete",
        "Phase 10",
        "cross-platform scheduler adapters",
        "scheduler-first",
        "notifier adapter expansion",
    ];

    for term in roadmap_terms {
        assert!(roadmap.contains(term), "execution plan should mention {term}");
    }

    assert!(
        phase_8_plan.contains("Phase 8 Hardening And Operability Implementation Plan"),
        "Phase 8 implementation plan should be tracked as a repository artifact"
    );

    assert!(
        phase_10_plan.contains("Phase 10 Cross-Platform Scheduler Adapters Implementation Plan")
            && phase_10_plan.contains("cron")
            && phase_10_plan.contains("launchd")
            && phase_10_plan.contains("systemd timer")
            && phase_10_plan.contains("scheduler-first")
            && phase_10_plan.contains("notifier adapter")
            && phase_10_plan.contains("run-governance.ps1"),
        "Phase 10 implementation plan should define scheduler-first roadmap and boundaries"
    );

    assert!(
        rules_spec.contains("glob 递归匹配")
            && rules_spec.contains("环境变量占位")
            && rules_spec.contains("~/"),
        "rules spec should describe current glob and path expansion behavior"
    );

    assert!(
        storage_map.contains("已完成覆盖")
            && storage_map.contains("Docker build cache / volumes 更精细解释与原生命令联动")
            && storage_map.contains("同步盘高频项目识别")
            && storage_map.contains("npm / uv / pip / cargo 开发缓存"),
        "storage map should separate completed coverage from remaining roadmap items"
    );
}

#[test]
fn crate_version_and_readme_reference_release_artifacts() {
    let cargo_toml = read_repo_file("aidisk/Cargo.toml");
    let cargo_lock = read_repo_file("aidisk/Cargo.lock");
    let readme = read_repo_file("README.md");
    let readme_zh = read_repo_file("README.zh-CN.md");
    let roadmap = read_repo_file("docs/execution-plan.md");

    assert!(cargo_toml.contains("version = \"1.3.0\""));
    let normalized_cargo_lock = cargo_lock.replace("\r\n", "\n");
    assert!(normalized_cargo_lock.contains("name = \"aidisk\"\nversion = \"1.3.0\""));
    assert!(readme.contains("CHANGELOG.md"));
    assert!(readme.contains("docs/release-notes/v1.3.0.md"));
    assert!(readme_zh.contains("docs/release-notes/v1.3.0.md"));
    assert!(roadmap.contains("docs/release-notes/v1.3.0.md"));
    assert!(roadmap.contains("`aidisk` crate version `1.3.0`"));
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
