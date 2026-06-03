use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct DiffReport {
    pub generated_at: DateTime<Local>,
    pub before: String,
    pub after: String,
    pub summary: DiffSummary,
    pub changes: Vec<DiffEntry>,
}

#[derive(Debug, Default, Serialize)]
pub struct DiffSummary {
    pub total_paths: usize,
    pub grew: usize,
    pub shrunk: usize,
    pub appeared: usize,
    pub disappeared: usize,
    pub total_growth_bytes: i64,
}

#[derive(Debug, Serialize)]
pub struct DiffEntry {
    pub path: String,
    pub before_bytes: u64,
    pub after_bytes: u64,
    pub delta_bytes: i64,
    pub change: String,
}

#[derive(Debug, Deserialize)]
struct ScanJson {
    findings: Vec<ScanFinding>,
}

#[derive(Debug, Deserialize)]
struct ScanFinding {
    path: String,
    exists: bool,
    size_bytes: u64,
}

pub fn build_diff(before_path: &Path, after_path: &Path) -> Result<DiffReport> {
    let before = index_findings(load_scan(before_path)?);
    let after = index_findings(load_scan(after_path)?);

    let mut changes = Vec::new();
    let mut summary = DiffSummary::default();
    let paths = before
        .keys()
        .chain(after.keys())
        .cloned()
        .collect::<BTreeSet<_>>();

    for path in paths {
        summary.total_paths += 1;
        let before_finding = before.get(&path);
        let after_finding = after.get(&path);
        let before_exists = before_finding
            .map(|finding| finding.exists)
            .unwrap_or(false);
        let after_exists = after_finding.map(|finding| finding.exists).unwrap_or(false);
        let before_bytes = before_finding
            .map(|finding| finding.size_bytes)
            .unwrap_or(0);
        let after_bytes = after_finding.map(|finding| finding.size_bytes).unwrap_or(0);

        match (before_exists, after_exists) {
            (false, false) => {}
            (false, true) => {
                summary.appeared += 1;
                summary.total_growth_bytes += after_bytes as i64;
                changes.push(DiffEntry {
                    path,
                    before_bytes,
                    after_bytes,
                    delta_bytes: after_bytes as i64,
                    change: "appeared".to_string(),
                });
            }
            (true, false) => {
                summary.disappeared += 1;
                summary.total_growth_bytes -= before_bytes as i64;
                changes.push(DiffEntry {
                    path,
                    before_bytes,
                    after_bytes,
                    delta_bytes: -(before_bytes as i64),
                    change: "disappeared".to_string(),
                });
            }
            (true, true) if after_bytes > before_bytes => {
                summary.grew += 1;
                let delta = after_bytes as i64 - before_bytes as i64;
                summary.total_growth_bytes += delta;
                changes.push(DiffEntry {
                    path,
                    before_bytes,
                    after_bytes,
                    delta_bytes: delta,
                    change: "grew".to_string(),
                });
            }
            (true, true) if after_bytes < before_bytes => {
                summary.shrunk += 1;
                let delta = after_bytes as i64 - before_bytes as i64;
                summary.total_growth_bytes += delta;
                changes.push(DiffEntry {
                    path,
                    before_bytes,
                    after_bytes,
                    delta_bytes: delta,
                    change: "shrunk".to_string(),
                });
            }
            (true, true) => {}
        }
    }

    changes.sort_by_key(|c| -c.delta_bytes.abs());

    Ok(DiffReport {
        generated_at: Local::now(),
        before: before_path.display().to_string(),
        after: after_path.display().to_string(),
        summary,
        changes,
    })
}

fn load_scan(path: &Path) -> Result<Vec<ScanFinding>> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("failed to read scan file {}", path.display()))?;
    let stripped = content.trim_start_matches('\u{feff}');
    let scan: ScanJson = serde_json::from_str(stripped)
        .with_context(|| format!("failed to parse scan file {}", path.display()))?;
    Ok(scan.findings)
}

fn index_findings(findings: Vec<ScanFinding>) -> BTreeMap<String, ScanFinding> {
    findings
        .into_iter()
        .map(|finding| (finding.path.clone(), finding))
        .collect()
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::build_diff;

    #[test]
    fn ignores_paths_missing_in_both_snapshots() {
        let temp = tempdir().expect("tempdir should exist");
        let before = temp.path().join("before.json");
        let after = temp.path().join("after.json");

        fs::write(
            &before,
            r#"{
  "findings": [
    { "path": "C:\\demo\\missing", "exists": false, "size_bytes": 0 }
  ]
}"#,
        )
        .expect("before snapshot should be written");
        fs::write(
            &after,
            r#"{
  "findings": [
    { "path": "C:\\demo\\missing", "exists": false, "size_bytes": 0 }
  ]
}"#,
        )
        .expect("after snapshot should be written");

        let report = build_diff(&before, &after).expect("diff should be built");

        assert_eq!(report.summary.appeared, 0);
        assert_eq!(report.summary.disappeared, 0);
        assert_eq!(report.summary.grew, 0);
        assert_eq!(report.summary.shrunk, 0);
        assert!(report.changes.is_empty());
    }

    #[test]
    fn preserves_zero_byte_existing_paths() {
        let temp = tempdir().expect("tempdir should exist");
        let before = temp.path().join("before.json");
        let after = temp.path().join("after.json");

        fs::write(&before, r#"{ "findings": [] }"#).expect("before snapshot should be written");
        fs::write(
            &after,
            r#"{
  "findings": [
    { "path": "C:\\demo\\empty.txt", "exists": true, "size_bytes": 0 }
  ]
}"#,
        )
        .expect("after snapshot should be written");

        let report = build_diff(&before, &after).expect("diff should be built");

        assert_eq!(report.summary.appeared, 1);
        assert_eq!(report.changes.len(), 1);
        assert_eq!(report.changes[0].change, "appeared");
        assert_eq!(report.changes[0].path, "C:\\demo\\empty.txt");
    }
}
