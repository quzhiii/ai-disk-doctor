use std::fs;
use std::path::Path;
use std::process::Command;

use serde_json::Value;
use tempfile::tempdir;

fn binary() -> &'static str {
    env!("CARGO_BIN_EXE_aidisk")
}

#[test]
fn rules_lint_reports_v1_v2_counts_and_digests() {
    let output = Command::new(binary())
        .args(["rules", "lint", "--json"])
        .output()
        .expect("rules lint should run");

    assert!(output.status.success());
    let report: Value = serde_json::from_slice(&output.stdout).expect("lint output should be json");
    assert_eq!(report["total_rules"], 26);
    assert_eq!(report["schema_versions"]["1"], 23);
    assert_eq!(report["schema_versions"]["2"], 3);
    assert!(report["rules"]
        .as_array()
        .expect("rules should be an array")
        .iter()
        .all(|source| source["digest"]
            .as_str()
            .is_some_and(|digest| digest.starts_with("sha256:"))));
}

#[test]
fn rules_lint_rejects_duplicate_ids() {
    let temp = tempdir().expect("tempdir should exist");
    let first = r#"
id: duplicate
name: First
category: test
platform: windows
paths: [C:\\first]
risk: safe
cleanup:
  method: quarantine
reason: first
"#;
    let second = first.replace("First", "Second");
    fs::write(temp.path().join("first.yaml"), first).expect("first rule should write");
    fs::write(temp.path().join("second.yaml"), second).expect("second rule should write");

    let output = Command::new(binary())
        .args([
            "rules",
            "lint",
            "--json",
            "--rules-dir",
            temp.path().to_str().expect("temp path should be utf8"),
        ])
        .output()
        .expect("rules lint should run");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("duplicate rule id duplicate"));
}

#[test]
fn scan_json_exposes_rule_sources_and_v2_model_rule_action() {
    let root = Path::new("tests/fixtures/windows-user");
    let output = Command::new(binary())
        .args([
            "scan",
            "--json",
            "--rules-dir",
            "rules",
            "--policy",
            "config/policy.yaml",
        ])
        .env("USERPROFILE", root)
        .env("HOME", root)
        .output()
        .expect("scan should run");

    assert!(output.status.success());
    let report: Value = serde_json::from_slice(&output.stdout).expect("scan output should be json");
    let sources = report["summary"]["rule_sources"]
        .as_array()
        .expect("rule sources should be present");
    assert_eq!(sources.len(), 26);
    assert!(sources.iter().any(|source| source["schema_version"] == 2));
    let model_finding = report["findings"]
        .as_array()
        .expect("findings should be present")
        .iter()
        .find(|finding| finding["id"] == "model-files");
    if let Some(finding) = model_finding {
        assert_eq!(finding["action"], "report-only");
        assert_eq!(finding["risk"], "review");
    }
}
