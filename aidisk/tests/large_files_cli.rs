use std::fs;
use std::path::PathBuf;
use std::process::Command;

use tempfile::tempdir;

fn aidisk_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_aidisk"))
}

fn write_scan_rule(path: &std::path::Path, target_root: &std::path::Path) {
    let escaped_path = target_root.display().to_string().replace('\\', "\\\\");
    let content = format!(
        r#"id: test-cache
name: Test cache
category: test
platform: windows
paths:
  - "{escaped_path}"
risk: safe
cleanup:
  method: quarantine
reason: "test cache"
"#
    );

    fs::write(path.join("test-cache.yaml"), content).expect("rule should be written");
}

fn write_policy(path: &std::path::Path, skip_minutes: u64, max_scan_depth: usize) {
    fs::write(
        path,
        format!(
            r#"sensitive_markers:
  - token
planner:
  skip_modified_within_minutes: {skip_minutes}
  allow_actions:
    - quarantine
    - report-only
    - guide
  max_scan_depth: {max_scan_depth}
"#
        ),
    )
    .expect("policy should be written");
}

#[test]
fn scan_large_files_json_outputs_parseable_report() {
    let temp = tempdir().expect("tempdir should exist");
    let root = temp.path();
    fs::create_dir_all(root.join("big")).expect("big dir should exist");
    fs::write(root.join("big").join("large.bin"), vec![0_u8; 600])
        .expect("large file should write");

    let output = Command::new(aidisk_bin())
        .args([
            "scan",
            "--large-files",
            "--min-size",
            "500",
            "--root",
            root.to_str().unwrap(),
            "--json",
        ])
        .output()
        .expect("scan --large-files should run");

    assert!(
        output.status.success(),
        "scan --large-files should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let parsed: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("stdout should be parseable JSON");

    assert_eq!(parsed["scan_root"], root.display().to_string());
    assert_eq!(parsed["min_size_bytes"], 500);
    assert!(
        parsed["entries"].as_array().unwrap().len() >= 1,
        "should find at least the big directory"
    );
    assert!(
        parsed["entries"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["path"].as_str().unwrap().ends_with("big")),
        "should find big directory entry"
    );
    assert!(
        parsed["entries"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["is_directory"].as_bool().unwrap_or(false)),
        "should find at least one directory entry"
    );
}

#[test]
fn scan_large_files_filters_below_min_size() {
    let temp = tempdir().expect("tempdir should exist");
    let root = temp.path();
    fs::write(root.join("small.txt"), vec![0_u8; 10]).expect("small file should write");

    let output = Command::new(aidisk_bin())
        .args([
            "scan",
            "--large-files",
            "--min-size",
            "500",
            "--root",
            root.to_str().unwrap(),
            "--json",
        ])
        .output()
        .expect("scan --large-files should run");

    assert!(output.status.success());
    let parsed: serde_json::Value = serde_json::from_slice(&output.stdout)
        .expect("stdout should be parseable JSON");

    assert!(
        !parsed["entries"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["path"].as_str().unwrap().ends_with("small.txt")),
        "small.txt should not appear below threshold"
    );
}

#[test]
fn scan_large_files_accepts_human_readable_min_size() {
    let temp = tempdir().expect("tempdir should exist");
    let root = temp.path();

    let output = Command::new(aidisk_bin())
        .args([
            "scan",
            "--large-files",
            "--min-size",
            "500MB",
            "--root",
            root.to_str().unwrap(),
            "--json",
        ])
        .output()
        .expect("scan --large-files should run");

    assert!(
        output.status.success(),
        "scan --large-files should accept 500MB: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let parsed: serde_json::Value = serde_json::from_slice(&output.stdout)
        .expect("stdout should be parseable JSON");

    assert_eq!(parsed["min_size_bytes"], 524_288_000_u64);
}

#[test]
fn scan_json_accepts_policy_override_and_reports_snapshot() {
    let temp = tempdir().expect("tempdir should exist");
    let rules_dir = temp.path().join("rules");
    let target_root = temp.path().join("cache-root");
    let policy_path = temp.path().join("scan-policy.yaml");
    fs::create_dir_all(&rules_dir).expect("rules dir should exist");
    fs::create_dir_all(&target_root).expect("target root should exist");
    fs::write(target_root.join("file.bin"), vec![0_u8; 32]).expect("file should exist");
    write_scan_rule(&rules_dir, &target_root);
    write_policy(&policy_path, 12, 3);

    let output = Command::new(aidisk_bin())
        .args([
            "scan",
            "--rules-dir",
            rules_dir.to_str().unwrap(),
            "--policy",
            policy_path.to_str().unwrap(),
            "--json",
        ])
        .output()
        .expect("scan should run");

    assert!(
        output.status.success(),
        "scan should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let parsed: serde_json::Value = serde_json::from_slice(&output.stdout)
        .expect("stdout should be parseable JSON");
    assert_eq!(parsed["policy"]["planner"]["skip_modified_within_minutes"], 12);
    assert_eq!(parsed["policy"]["planner"]["max_scan_depth"], 3);
}
