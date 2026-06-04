use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::json;
use tempfile::tempdir;

fn aidisk_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_aidisk"))
}

fn write_policy(path: &Path) {
    fs::write(
        path,
        r#"sensitive_markers:
  - token
  - credential
  - secret
  - .env
  - cookies
  - login data
  - auth.json

planner:
  skip_modified_within_minutes: 30
  allow_actions:
    - quarantine
    - report-only
    - guide
  max_scan_depth: 20
"#,
    )
    .expect("policy file should be written");
}

fn write_agent_rule(path: &Path, agent_root: &Path) {
    let escaped_path = agent_root
        .display()
        .to_string()
        .replace('\\', "\\\\");
    let content = format!(
        r#"id: test-agent-root
name: Test agent root
category: ai-agent
platform: windows
paths:
  - "{escaped_path}"
risk: review
cleanup:
  method: report-only
reason: "test agent state"
"#
    );

    fs::write(path.join("test-agent.yaml"), content).expect("rule file should be written");
}

fn write_snapshot(path: &Path, finding_path: &str, exists: bool, size_bytes: u64) {
    let content = serde_json::to_string_pretty(&json!({
        "findings": [
            {
                "path": finding_path,
                "exists": exists,
                "size_bytes": size_bytes
            }
        ]
    }))
    .expect("snapshot json should serialize");

    fs::write(path, content).expect("snapshot should be written");
}

#[test]
fn doctor_latest_reports_dir_emits_latest_diff_json() {
    let temp = tempdir().expect("tempdir should exist");
    let rules_dir = temp.path().join("rules");
    let reports_dir = temp.path().join("reports");
    let policy_path = temp.path().join("policy.yaml");
    let agent_root = temp.path().join("agent-root");
    let before = reports_dir.join("scan-20260101-000000-000.json");
    let after = reports_dir.join("scan-20260102-000000-000.json");

    fs::create_dir_all(&rules_dir).expect("rules dir should exist");
    fs::create_dir_all(&reports_dir).expect("reports dir should exist");
    fs::create_dir_all(&agent_root).expect("agent root should exist");
    fs::write(agent_root.join("session.log"), vec![1_u8; 220]).expect("agent file should exist");

    write_policy(&policy_path);
    write_agent_rule(&rules_dir, &agent_root);

    let agent_path = agent_root.display().to_string();
    write_snapshot(&before, &agent_path, true, 100);
    write_snapshot(&after, &agent_path, true, 220);

    let output = Command::new(aidisk_bin())
        .args([
            "doctor",
            "--agents",
            "--latest",
            "--reports-dir",
            reports_dir.to_str().expect("reports dir should be utf-8"),
            "--rules-dir",
            rules_dir.to_str().expect("rules dir should be utf-8"),
            "--policy",
            policy_path.to_str().expect("policy path should be utf-8"),
            "--json",
        ])
        .output()
        .expect("doctor command should run");

    assert!(
        output.status.success(),
        "doctor should succeed, stderr was: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let parsed: serde_json::Value = serde_json::from_slice(&output.stdout)
        .expect("doctor json output should parse");

    assert_eq!(parsed["latest_diff"]["summary"]["grew"], 1);
    assert_eq!(parsed["latest_diff"]["summary"]["appeared"], 0);
    assert_eq!(parsed["topics"][0]["name"], "agents");
    assert_eq!(parsed["topics"][0]["status"], "active");
}

#[test]
fn doctor_latest_requires_two_snapshots_with_doctor_specific_message() {
    let temp = tempdir().expect("tempdir should exist");
    let rules_dir = temp.path().join("rules");
    let reports_dir = temp.path().join("reports");
    let policy_path = temp.path().join("policy.yaml");
    let agent_root = temp.path().join("agent-root");
    let only_snapshot = reports_dir.join("scan-20260101-000000-000.json");

    fs::create_dir_all(&rules_dir).expect("rules dir should exist");
    fs::create_dir_all(&reports_dir).expect("reports dir should exist");
    fs::create_dir_all(&agent_root).expect("agent root should exist");

    write_policy(&policy_path);
    write_agent_rule(&rules_dir, &agent_root);
    write_snapshot(&only_snapshot, &agent_root.display().to_string(), true, 100);

    let output = Command::new(aidisk_bin())
        .args([
            "doctor",
            "--agents",
            "--latest",
            "--reports-dir",
            reports_dir.to_str().expect("reports dir should be utf-8"),
            "--rules-dir",
            rules_dir.to_str().expect("rules dir should be utf-8"),
            "--policy",
            policy_path.to_str().expect("policy path should be utf-8"),
            "--json",
        ])
        .output()
        .expect("doctor command should run");

    assert!(!output.status.success(), "doctor should fail with one snapshot");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("doctor --latest requires at least two scan snapshots in"));
    assert!(!stderr.contains("diff --latest requires"));
}
