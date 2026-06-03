use std::collections::BTreeSet;
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
fn skill_lists_all_existing_scripts_and_listed_scripts_exist() {
    let skill = read_repo_file("skills/windows-ai-space-manager/SKILL.md");
    let scripts_dir = repo_root().join("skills/windows-ai-space-manager/scripts");
    let actual_scripts = fs::read_dir(&scripts_dir)
        .expect("scripts directory should exist")
        .map(|entry| {
            entry
                .expect("script directory entry should be readable")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .filter(|file_name| file_name.ends_with(".ps1"))
        .collect::<BTreeSet<_>>();

    for script in &actual_scripts {
        assert!(
            skill.contains(&format!("scripts/{script}")),
            "SKILL.md should list scripts/{script}"
        );
    }

    for line in skill.lines().filter(|line| line.contains("`scripts/")) {
        let Some(script) = line
            .split("`scripts/")
            .nth(1)
            .and_then(|value| value.split('`').next())
        else {
            continue;
        };
        assert!(
            scripts_dir.join(script).exists(),
            "listed script scripts/{script} should exist"
        );
    }
}

#[test]
fn skill_response_style_includes_all_next_step_commands() {
    let skill = read_repo_file("skills/windows-ai-space-manager/SKILL.md");

    assert!(
        skill.contains("scan / plan / clean / restore / doctor / diff"),
        "response style should include diff as a next-step command"
    );
}

#[test]
fn workflow_references_diff_wrapper() {
    let workflow = read_repo_file("skills/windows-ai-space-manager/references/workflow.md");

    assert!(
        workflow.contains("run-diff.ps1"),
        "workflow reference should tell agents which wrapper runs diff"
    );
}

#[test]
fn skill_documents_diff_latest_workflow() {
    let skill = read_repo_file("skills/windows-ai-space-manager/SKILL.md");
    let run_diff = read_repo_file("skills/windows-ai-space-manager/scripts/run-diff.ps1");

    assert!(
        skill.contains("run-diff.ps1 -Latest"),
        "SKILL.md should document the diff latest wrapper workflow"
    );
    assert!(
        run_diff.contains("[switch]$Latest"),
        "run-diff.ps1 should expose -Latest"
    );
    assert!(
        run_diff.contains("[string]$ReportsDir"),
        "run-diff.ps1 should expose -ReportsDir"
    );
}

#[test]
fn risk_cheatsheet_covers_execution_and_restore_statuses() {
    let risk = read_repo_file("skills/windows-ai-space-manager/references/risk-cheatsheet.md");
    let expected_statuses = [
        "moved",
        "planned",
        "restored",
        "skipped-active",
        "skipped-conflict",
        "skipped-locked",
        "failed",
    ];

    for status in expected_statuses {
        assert!(
            risk.contains(status),
            "risk cheatsheet should explain status {status}"
        );
    }
}

#[test]
fn category_map_covers_rule_categories() {
    let category_map = read_repo_file("skills/windows-ai-space-manager/references/category-map.md");
    let rules_dir = repo_root().join("aidisk/rules");
    let categories = fs::read_dir(&rules_dir)
        .expect("rules directory should exist")
        .filter_map(|entry| {
            let path = entry.expect("rule entry should be readable").path();
            let is_yaml = matches!(
                path.extension().and_then(|extension| extension.to_str()),
                Some("yaml" | "yml")
            );
            is_yaml.then_some(path)
        })
        .flat_map(|path| {
            fs::read_to_string(&path)
                .unwrap_or_else(|error| {
                    panic!("failed to read rule file {}: {error}", path.display())
                })
                .lines()
                .filter_map(|line| {
                    line.strip_prefix("category: ")
                        .map(str::trim)
                        .map(str::to_string)
                })
                .collect::<Vec<_>>()
        })
        .collect::<BTreeSet<_>>();

    for category in categories {
        assert!(
            category_map.contains(&format!("`{category}`")),
            "category map should cover `{category}`"
        );
    }
}
