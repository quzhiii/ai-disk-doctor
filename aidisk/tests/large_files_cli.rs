use std::fs;
use std::path::PathBuf;
use std::process::Command;

use tempfile::tempdir;

fn aidisk_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_aidisk"))
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
