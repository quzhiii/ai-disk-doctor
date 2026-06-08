use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::json;
use tempfile::tempdir;

fn aidisk_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_aidisk"))
}

fn write_snapshot(path: &Path, findings: serde_json::Value) {
    let content = serde_json::to_string_pretty(&json!({ "findings": findings }))
        .expect("snapshot json should serialize");
    fs::write(path, content).expect("snapshot should be written");
}

#[test]
fn anomaly_json_reports_growth_above_dual_thresholds() {
    let temp = tempdir().expect("tempdir should exist");
    let before = temp.path().join("before.json");
    let after = temp.path().join("after.json");

    write_snapshot(
        &before,
        json!([
            { "path": "C:\\demo\\cache", "exists": true, "size_bytes": 1_000 },
            { "path": "C:\\demo\\noise", "exists": true, "size_bytes": 10_000 }
        ]),
    );
    write_snapshot(
        &after,
        json!([
            { "path": "C:\\demo\\cache", "exists": true, "size_bytes": 2_100 },
            { "path": "C:\\demo\\noise", "exists": true, "size_bytes": 10_500 }
        ]),
    );

    let output = Command::new(aidisk_bin())
        .args([
            "anomaly",
            "--before",
            before.to_str().unwrap(),
            "--after",
            after.to_str().unwrap(),
            "--min-growth",
            "1KB",
            "--min-growth-percent",
            "30",
            "--json",
        ])
        .output()
        .expect("anomaly command should run");

    assert!(
        output.status.success(),
        "anomaly should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let parsed: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("stdout should be parseable JSON");
    assert_eq!(parsed["summary"]["anomalies"], 1);
    assert_eq!(parsed["thresholds"]["min_growth_bytes"], 1_024);
    assert_eq!(parsed["anomalies"][0]["path"], "C:\\demo\\cache");
    assert_eq!(parsed["anomalies"][0]["growth_percent"], 110.0);
}

#[test]
fn anomaly_latest_uses_newest_two_snapshots_from_reports_dir() {
    let temp = tempdir().expect("tempdir should exist");
    let reports_dir = temp.path().join("reports");
    fs::create_dir_all(&reports_dir).expect("reports dir should exist");

    write_snapshot(
        &reports_dir.join("scan-20260101-000000-000.json"),
        json!([{ "path": "C:\\demo\\cache", "exists": true, "size_bytes": 1_000 }]),
    );
    write_snapshot(
        &reports_dir.join("scan-20260102-000000-000.json"),
        json!([{ "path": "C:\\demo\\cache", "exists": true, "size_bytes": 4_000 }]),
    );

    let output = Command::new(aidisk_bin())
        .args([
            "anomaly",
            "--latest",
            "--reports-dir",
            reports_dir.to_str().unwrap(),
            "--min-growth",
            "1KB",
            "--min-growth-percent",
            "30",
            "--json",
        ])
        .output()
        .expect("anomaly command should run");

    assert!(
        output.status.success(),
        "anomaly should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let parsed: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("stdout should be parseable JSON");
    assert_eq!(parsed["summary"]["anomalies"], 1);
    assert_eq!(parsed["anomalies"][0]["delta_bytes"], 3_000);
}
