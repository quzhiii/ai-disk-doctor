use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use tempfile::tempdir;

fn aidisk_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_aidisk"))
}

fn write_policy(path: &Path) {
    fs::write(
        path,
        r#"sensitive_markers:
  - token
planner:
  skip_modified_within_minutes: 0
  allow_actions:
    - quarantine
    - report-only
    - guide
  max_scan_depth: 20
"#,
    )
    .expect("policy should be written");
}

fn write_rule(path: &Path, target: &Path) {
    let escaped = target.display().to_string().replace('\\', "\\\\");
    fs::write(
        path.join("cache.yaml"),
        format!(
            r#"id: clean-json-cache
name: Clean JSON cache
category: dev-cache
platform: windows
paths:
  - "{escaped}"
risk: safe
cleanup:
  method: quarantine
reason: test cache
"#
        ),
    )
    .expect("rule should be written");
}

#[test]
fn clean_dry_run_json_with_quarantine_root_emits_single_parseable_document() {
    let temp = tempdir().expect("tempdir should exist");
    let rules_dir = temp.path().join("rules");
    let policy = temp.path().join("policy.yaml");
    let target = temp.path().join("cache");
    let quarantine_root = temp.path().join("archive");
    fs::create_dir_all(&rules_dir).expect("rules dir should exist");
    fs::create_dir_all(&target).expect("target should exist");
    fs::write(target.join("data.bin"), vec![1_u8; 16]).expect("data should exist");
    write_policy(&policy);
    write_rule(&rules_dir, &target);

    let output = Command::new(aidisk_bin())
        .args([
            "clean",
            "--dry-run",
            "--rules-dir",
            rules_dir.to_str().unwrap(),
            "--policy",
            policy.to_str().unwrap(),
            "--quarantine-root",
            quarantine_root.to_str().unwrap(),
            "--json",
        ])
        .output()
        .expect("clean should run");

    assert!(
        output.status.success(),
        "clean should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty(), "stderr should be empty on success");
    let parsed: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("stdout should be one JSON document");
    assert_eq!(parsed["mode"], "dry-run");
    assert_eq!(
        parsed["quarantine_plan"]["root"],
        quarantine_root.display().to_string()
    );
    assert_eq!(
        parsed["quarantine_plan"]["entries"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
}
