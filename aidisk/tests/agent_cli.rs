use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use tempfile::tempdir;

fn aidisk_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_aidisk"))
}

fn parse_stdout(output: &std::process::Output) -> serde_json::Value {
    assert!(
        output.status.success(),
        "command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.stderr.is_empty(),
        "success should not write stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("stdout should be one JSON document")
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

fn write_rule(path: &Path, target: &Path, category: &str, method: &str) {
    fs::write(
        path,
        format!(
            r#"id: agent-cli-cache
name: Agent CLI Cache
category: {category}
platform: cross-platform
paths:
  - '{}'
risk: review
cleanup:
  method: {method}
exclusions: []
reason: "agent cli fixture"
"#,
            target.display()
        ),
    )
    .expect("rule should be written");
}

#[test]
fn capabilities_is_structured_and_does_not_trigger_a_scan() {
    let temp = tempdir().expect("tempdir should exist");
    let output = Command::new(aidisk_bin())
        .args(["capabilities", "--json"])
        .current_dir(temp.path())
        .output()
        .expect("capabilities should run");
    let json = parse_stdout(&output);

    assert_eq!(json["contract"], "agent-capabilities-v1");
    assert_eq!(json["schema_version"], 1);
    assert_eq!(
        json["capabilities"]["explainability"]["contract"],
        "explainability-v1"
    );
    assert_eq!(
        json["capabilities"]["explainability"]["schema_versions"][0],
        1
    );
    assert_eq!(
        json["capabilities"]["explainability"]["cli_available"],
        true
    );
    assert_eq!(
        json["capabilities"]["explainability"]["snapshot_modes"][1],
        "skip"
    );
    assert_eq!(
        json["capabilities"]["action_proposals"]["contract"],
        "action-proposal-v1"
    );
    assert_eq!(
        json["capabilities"]["action_proposals"]["schema_versions"][0],
        1
    );
    assert_eq!(
        json["capabilities"]["action_proposals"]["read_only"],
        serde_json::Value::Bool(true)
    );
    assert_eq!(
        json["capabilities"]["action_proposals"]["mutation_authorized"],
        serde_json::Value::Bool(false)
    );
    assert!(!temp.path().join(".aidisk").exists());
}

#[test]
fn explain_skip_returns_bounded_explainability_without_snapshot() {
    let temp = tempdir().expect("tempdir should exist");
    let rules_dir = temp.path().join("rules");
    let config_dir = temp.path().join("config");
    let policy = config_dir.join("policy.yaml");
    let cache = temp.path().join("cache");
    fs::create_dir_all(&rules_dir).expect("rules dir should exist");
    fs::create_dir_all(&config_dir).expect("config dir should exist");
    fs::create_dir_all(&cache).expect("cache should exist");
    fs::write(cache.join("artifact.bin"), vec![0_u8; 17]).expect("artifact should write");
    write_policy(&policy);
    write_rule(
        &rules_dir.join("cache.yaml"),
        &cache,
        "agent",
        "report-only",
    );

    let output = Command::new(aidisk_bin())
        .args([
            "explain",
            "--json",
            "--snapshot",
            "skip",
            "--category",
            "agent",
        ])
        .current_dir(temp.path())
        .output()
        .expect("explain should run");
    let json = parse_stdout(&output);

    assert_eq!(json["contract"], "agent-diagnostic-cli-v1");
    assert_eq!(json["schema_version"], 1);
    assert_eq!(json["snapshot"]["requested"], "skip");
    assert_eq!(json["snapshot"]["persisted"], false);
    assert!(json["snapshot"]["path"].is_null());
    assert_eq!(json["explainability"]["contract"], "explainability-v1");
    assert_eq!(json["explainability"]["schema_version"], 1);
    assert_eq!(
        json["explainability"]["categories"][0]["category_id"],
        "agent"
    );
    assert_eq!(
        json["explainability"]["categories"][0]["rules"][0]["provenance"]["source"]
            ["schema_version"],
        1
    );
    assert_eq!(
        json["explainability"]["categories"][0]["rules"][0]["risk"],
        "review"
    );
    assert_eq!(
        json["explainability"]["categories"][0]["rules"][0]["handling_mode"],
        "report-only"
    );
    assert!(!temp.path().join(".aidisk").exists());
}

#[test]
fn explain_save_preserves_existing_snapshot_behavior() {
    let temp = tempdir().expect("tempdir should exist");
    let rules_dir = temp.path().join("rules");
    let config_dir = temp.path().join("config");
    let policy = config_dir.join("policy.yaml");
    let cache = temp.path().join("cache");
    fs::create_dir_all(&rules_dir).expect("rules dir should exist");
    fs::create_dir_all(&config_dir).expect("config dir should exist");
    fs::create_dir_all(&cache).expect("cache should exist");
    write_policy(&policy);
    write_rule(
        &rules_dir.join("cache.yaml"),
        &cache,
        "agent",
        "report-only",
    );

    let output = Command::new(aidisk_bin())
        .args([
            "explain",
            "--json",
            "--snapshot",
            "save",
            "--category",
            "agent",
        ])
        .current_dir(temp.path())
        .output()
        .expect("explain should run");
    let json = parse_stdout(&output);

    assert_eq!(json["snapshot"]["requested"], "save");
    assert_eq!(json["snapshot"]["persisted"], true);
    let snapshot = json["snapshot"]["path"]
        .as_str()
        .expect("snapshot path should be returned");
    assert!(Path::new(snapshot).exists());
    assert!(snapshot.ends_with(".json"));
}

#[test]
fn explain_json_errors_are_structured() {
    let temp = tempdir().expect("tempdir should exist");
    let output = Command::new(aidisk_bin())
        .args(["explain", "--json", "--snapshot", "unsupported"])
        .current_dir(temp.path())
        .output()
        .expect("explain should run");

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    let json: serde_json::Value =
        serde_json::from_slice(&output.stderr).expect("stderr should be JSON");
    assert_eq!(json["ok"], false);
    assert_eq!(json["error"]["command"], "explain");
    assert_eq!(json["error"]["type"], "usage");
}
