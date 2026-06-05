use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use tempfile::tempdir;

fn aidisk_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_aidisk"))
}

fn parse_json(bytes: &[u8]) -> serde_json::Value {
    serde_json::from_slice(bytes).expect("stderr should be a single JSON document")
}

fn assert_json_error(output: &std::process::Output, command: &str) -> serde_json::Value {
    assert!(!output.status.success(), "command should fail");
    assert!(
        output.stdout.is_empty(),
        "stdout must stay empty on JSON errors: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    let parsed = parse_json(&output.stderr);
    assert_eq!(parsed["ok"], false);
    assert_eq!(parsed["error"]["command"], command);
    assert!(parsed["error"]["message"]
        .as_str()
        .expect("message should be string")
        .len()
        > 0);
    assert!(parsed["error"]["type"]
        .as_str()
        .expect("type should be string")
        .len()
        > 0);
    assert!(
        parsed["error"]["details"].as_array().is_some(),
        "details should be an array"
    );
    parsed
}

fn write_policy(path: &Path) {
    fs::write(
        path,
        r#"sensitive_markers:
  - token
planner:
  skip_modified_within_minutes: 30
  allow_actions:
    - quarantine
    - report-only
    - guide
  max_scan_depth: 20
"#,
    )
    .expect("policy should be written");
}

#[test]
fn scan_json_error_is_parseable_and_keeps_stdout_empty() {
    let temp = tempdir().expect("tempdir should exist");
    let missing_rules = temp.path().join("missing-rules");

    let output = Command::new(aidisk_bin())
        .args(["scan", "--rules-dir", missing_rules.to_str().unwrap(), "--json"])
        .output()
        .expect("scan should run");

    let parsed = assert_json_error(&output, "scan");
    assert_eq!(parsed["error"]["type"], "input");
}

#[test]
fn plan_json_error_uses_same_contract() {
    let temp = tempdir().expect("tempdir should exist");
    let rules_dir = temp.path().join("rules");
    let missing_policy = temp.path().join("missing-policy.yaml");
    fs::create_dir_all(&rules_dir).expect("rules dir should exist");

    let output = Command::new(aidisk_bin())
        .args([
            "plan",
            "--rules-dir",
            rules_dir.to_str().unwrap(),
            "--policy",
            missing_policy.to_str().unwrap(),
            "--json",
        ])
        .output()
        .expect("plan should run");

    let parsed = assert_json_error(&output, "plan");
    assert_eq!(parsed["error"]["type"], "input");
}

#[test]
fn clean_json_error_uses_same_contract_for_usage_errors() {
    let temp = tempdir().expect("tempdir should exist");
    let rules_dir = temp.path().join("rules");
    let policy = temp.path().join("policy.yaml");
    fs::create_dir_all(&rules_dir).expect("rules dir should exist");
    write_policy(&policy);

    let output = Command::new(aidisk_bin())
        .args([
            "clean",
            "--rules-dir",
            rules_dir.to_str().unwrap(),
            "--policy",
            policy.to_str().unwrap(),
            "--json",
        ])
        .output()
        .expect("clean should run");

    let parsed = assert_json_error(&output, "clean");
    assert_eq!(parsed["error"]["type"], "usage");
    assert!(parsed["error"]["message"]
        .as_str()
        .unwrap()
        .contains("--yes"));
}

#[test]
fn restore_json_error_uses_same_contract_for_usage_errors() {
    let temp = tempdir().expect("tempdir should exist");
    let index = temp.path().join("index.json");

    let output = Command::new(aidisk_bin())
        .args(["restore", "--index", index.to_str().unwrap(), "--json"])
        .output()
        .expect("restore should run");

    let parsed = assert_json_error(&output, "restore");
    assert_eq!(parsed["error"]["type"], "usage");
}

#[test]
fn clap_parse_json_error_uses_same_contract_for_missing_required_args() {
    let output = Command::new(aidisk_bin())
        .args(["restore", "--json"])
        .output()
        .expect("restore should run");

    let parsed = assert_json_error(&output, "restore");
    assert_eq!(parsed["error"]["type"], "usage");
    assert!(parsed["error"]["message"]
        .as_str()
        .unwrap()
        .contains("--index"));
}

#[test]
fn diff_json_error_uses_same_contract() {
    let output = Command::new(aidisk_bin())
        .args(["diff", "--json"])
        .output()
        .expect("diff should run");

    let parsed = assert_json_error(&output, "diff");
    assert_eq!(parsed["error"]["type"], "usage");
}

#[test]
fn doctor_json_error_uses_same_contract() {
    let temp = tempdir().expect("tempdir should exist");
    let rules_dir = temp.path().join("rules");
    let reports_dir = temp.path().join("reports");
    let policy = temp.path().join("policy.yaml");
    fs::create_dir_all(&rules_dir).expect("rules dir should exist");
    fs::create_dir_all(&reports_dir).expect("reports dir should exist");
    write_policy(&policy);

    let output = Command::new(aidisk_bin())
        .args([
            "doctor",
            "--latest",
            "--reports-dir",
            reports_dir.to_str().unwrap(),
            "--rules-dir",
            rules_dir.to_str().unwrap(),
            "--policy",
            policy.to_str().unwrap(),
            "--json",
        ])
        .output()
        .expect("doctor should run");

    let parsed = assert_json_error(&output, "doctor");
    assert_eq!(parsed["error"]["type"], "input");
}

#[test]
fn text_error_remains_non_json() {
    let output = Command::new(aidisk_bin())
        .args(["diff"])
        .output()
        .expect("diff should run");

    assert!(
        !output.status.success(),
        "diff should fail without --before/--after"
    );
    assert!(
        output.stdout.is_empty(),
        "stdout should be empty on text errors"
    );
    assert!(
        serde_json::from_slice::<serde_json::Value>(&output.stderr).is_err(),
        "text-mode stderr should not become JSON"
    );
    assert!(String::from_utf8_lossy(&output.stderr).contains("diff requires --before"));
}
