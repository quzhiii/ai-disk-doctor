use std::fs;
use std::path::PathBuf;
use std::process::Command;

use tempfile::tempdir;

fn aidisk_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_aidisk"))
}

#[test]
fn visualize_html_generates_valid_output() {
    let temp = tempdir().expect("tempdir should exist");
    let reports_dir = temp.path().join("reports");
    fs::create_dir_all(&reports_dir).expect("reports dir should exist");

    let scan_json = r#"{
  "scan_time": "2026-06-11T10:30:00+08:00",
  "volumes": [],
  "findings": [
    {"id": "cursor-cache", "name": "Cursor Cache", "category": "ai-ide", "path": "C:\\Users\\test\\AppData\\Roaming\\Cursor\\Cache", "exists": true, "size_bytes": 524288000, "partial": false, "partial_reasons": [], "risk": "safe", "action": "quarantine", "reason": "IDE cache", "warnings": []},
    {"id": "not-installed", "name": "Not Installed Tool", "category": "ai-cli", "path": "C:\\Users\\test\\.nonexistent", "exists": false, "size_bytes": 0, "partial": false, "partial_reasons": [], "risk": "safe", "action": "quarantine", "reason": "Not installed", "warnings": []}
  ],
  "summary": {"total_rules": 5, "matched_paths": 1, "total_size_bytes": 524288000, "safe_bytes": 524288000, "review_bytes": 0, "dangerous_bytes": 0, "system_bytes": 0}
}"#;
    fs::write(reports_dir.join("scan-20260611-103000-000.json"), scan_json)
        .expect("scan json should be written");

    let output = temp.path().join("dashboard.html");

    let result = Command::new(aidisk_bin())
        .arg("visualize")
        .arg("--html")
        .arg("--reports-dir")
        .arg(reports_dir)
        .arg("--output")
        .arg(&output)
        .output()
        .expect("command should run");

    assert!(result.status.success(), "stderr: {}", String::from_utf8_lossy(&result.stderr));

    let html = fs::read_to_string(&output).expect("output should be readable");
    assert!(html.contains("<!DOCTYPE html>"));
    assert!(html.contains("AI Disk Footprint"));
    assert!(html.contains("Total AI Footprint"));
    assert!(html.contains("Safe to Reclaim"));
    assert!(html.contains("Tools Detected"));
    assert!(!html.contains("cdn"));
    assert!(!html.contains("http://"));
    assert!(!html.contains("https://"));
    assert!(!html.contains("border-radius"));
    assert!(!html.contains("box-shadow"));
}

#[test]
fn visualize_html_handles_empty_reports_dir() {
    let temp = tempdir().expect("tempdir should exist");
    let reports_dir = temp.path().join("empty-reports");
    fs::create_dir_all(&reports_dir).expect("reports dir should exist");

    let output = temp.path().join("empty-dashboard.html");

    let result = Command::new(aidisk_bin())
        .arg("visualize")
        .arg("--html")
        .arg("--reports-dir")
        .arg(reports_dir)
        .arg("--output")
        .arg(&output)
        .output()
        .expect("command should run");

    assert!(!result.status.success());
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(stderr.contains("no scan snapshots"));
}
