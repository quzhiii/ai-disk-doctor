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
    size_bytes: u64,
}

pub fn build_diff(before_path: &Path, after_path: &Path) -> Result<DiffReport> {
    let before = load_scan(before_path)?;
    let after = load_scan(after_path)?;

    let mut changes = Vec::new();
    let mut summary = DiffSummary::default();

    for finding in &after {
        summary.total_paths += 1;
        let before_bytes = before
            .iter()
            .find(|f| f.path == finding.path)
            .map(|f| f.size_bytes)
            .unwrap_or(0);

        if before_bytes == 0 {
            summary.appeared += 1;
            summary.total_growth_bytes += finding.size_bytes as i64;
            changes.push(DiffEntry {
                path: finding.path.clone(),
                before_bytes: 0,
                after_bytes: finding.size_bytes,
                delta_bytes: finding.size_bytes as i64,
                change: "appeared".to_string(),
            });
        } else if finding.size_bytes > before_bytes {
            summary.grew += 1;
            let delta = finding.size_bytes as i64 - before_bytes as i64;
            summary.total_growth_bytes += delta;
            changes.push(DiffEntry {
                path: finding.path.clone(),
                before_bytes,
                after_bytes: finding.size_bytes,
                delta_bytes: delta,
                change: "grew".to_string(),
            });
        } else if finding.size_bytes < before_bytes {
            summary.shrunk += 1;
            let delta = finding.size_bytes as i64 - before_bytes as i64;
            summary.total_growth_bytes += delta;
            changes.push(DiffEntry {
                path: finding.path.clone(),
                before_bytes,
                after_bytes: finding.size_bytes,
                delta_bytes: delta,
                change: "shrunk".to_string(),
            });
        }
    }

    for finding in &before {
        if !after.iter().any(|f| f.path == finding.path) {
            summary.disappeared += 1;
            changes.push(DiffEntry {
                path: finding.path.clone(),
                before_bytes: finding.size_bytes,
                after_bytes: 0,
                delta_bytes: -(finding.size_bytes as i64),
                change: "disappeared".to_string(),
            });
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
