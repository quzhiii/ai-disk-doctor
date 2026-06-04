use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use chrono::Local;

use crate::scanner::ScanReport;

pub fn default_reports_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(".aidisk")
        .join("reports")
}

pub fn save_scan_snapshot(report: &ScanReport, reports_dir: &Path) -> Result<PathBuf> {
    fs::create_dir_all(reports_dir)?;

    let timestamp = Local::now().format("%Y%m%d-%H%M%S-%3f");
    let mut candidate = reports_dir.join(format!("scan-{timestamp}.json"));
    let mut suffix = 1_u32;
    while candidate.exists() {
        candidate = reports_dir.join(format!("scan-{timestamp}-{suffix}.json"));
        suffix += 1;
    }

    let content = serde_json::to_string_pretty(report)?;
    fs::write(&candidate, content)?;
    Ok(candidate)
}

pub fn latest_scan_pair(reports_dir: &Path) -> Result<(PathBuf, PathBuf)> {
    latest_scan_pair_for_command(reports_dir, "diff --latest")
}

pub fn latest_scan_pair_for_command(
    reports_dir: &Path,
    command_name: &str,
) -> Result<(PathBuf, PathBuf)> {
    let mut snapshots = Vec::new();

    if reports_dir.exists() {
        for entry in fs::read_dir(reports_dir)? {
            let entry = entry?;
            let path = entry.path();
            let file_name = path.file_name().and_then(|value| value.to_str()).unwrap_or("");
            if file_name.starts_with("scan-") && file_name.ends_with(".json") {
                snapshots.push(path);
            }
        }
    }

    snapshots.sort_by(|a, b| a.file_name().cmp(&b.file_name()));

    if snapshots.len() < 2 {
        anyhow::bail!(
            "{} requires at least two scan snapshots in {}",
            command_name,
            reports_dir.display()
        );
    }

    let after = snapshots.pop().expect("snapshot count was checked");
    let before = snapshots.pop().expect("snapshot count was checked");
    Ok((before, after))
}

#[cfg(test)]
mod tests {
    use std::fs;

    use chrono::Local;
    use tempfile::tempdir;

    use super::{latest_scan_pair, latest_scan_pair_for_command, save_scan_snapshot};
    use crate::scanner::{ScanReport, Summary};

    fn sample_scan_report() -> ScanReport {
        ScanReport {
            scan_time: Local::now(),
            volumes: Vec::new(),
            findings: Vec::new(),
            summary: Summary::default(),
        }
    }

    #[test]
    fn saves_scan_snapshot_as_json_report() {
        let temp = tempdir().expect("tempdir should exist");
        let report = sample_scan_report();

        let path = save_scan_snapshot(&report, temp.path()).expect("snapshot should save");

        assert!(path
            .file_name()
            .expect("snapshot should have a file name")
            .to_string_lossy()
            .starts_with("scan-"));
        assert_eq!(
            path.extension().and_then(|extension| extension.to_str()),
            Some("json")
        );
        let parsed: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(&path).expect("snapshot should be readable"),
        )
        .expect("snapshot should be json");
        assert!(parsed.get("findings").is_some());
    }

    #[test]
    fn saving_twice_creates_distinct_snapshots() {
        let temp = tempdir().expect("tempdir should exist");
        let report = sample_scan_report();

        let first = save_scan_snapshot(&report, temp.path()).expect("first snapshot should save");
        let second = save_scan_snapshot(&report, temp.path()).expect("second snapshot should save");

        assert_ne!(first, second);
        assert!(first.exists());
        assert!(second.exists());
    }

    #[test]
    fn latest_scan_pair_returns_newest_two_snapshots() {
        let temp = tempdir().expect("tempdir should exist");
        let old = temp.path().join("scan-20260101-000000-000.json");
        let before = temp.path().join("scan-20260102-000000-000.json");
        let after = temp.path().join("scan-20260103-000000-000.json");
        fs::write(&old, "{}").expect("old snapshot should be written");
        fs::write(&before, "{}").expect("before snapshot should be written");
        fs::write(&after, "{}").expect("after snapshot should be written");
        fs::write(temp.path().join("other.json"), "{}").expect("other file should be written");

        let pair = latest_scan_pair(temp.path()).expect("latest pair should exist");

        assert_eq!(pair.0, before);
        assert_eq!(pair.1, after);
    }

    #[test]
    fn latest_scan_pair_requires_two_snapshots() {
        let temp = tempdir().expect("tempdir should exist");
        fs::write(temp.path().join("scan-20260101-000000-000.json"), "{}")
            .expect("snapshot should be written");

        let error = latest_scan_pair(temp.path()).expect_err("two snapshots should be required");

        assert!(error.to_string().contains("at least two"));
    }

    #[test]
    fn latest_scan_pair_for_command_uses_caller_name_in_error() {
        let temp = tempdir().expect("tempdir should exist");

        let error = latest_scan_pair_for_command(temp.path(), "doctor --latest")
            .expect_err("two snapshots should be required");

        assert!(error
            .to_string()
            .contains("doctor --latest requires at least two scan snapshots in"));
    }
}
