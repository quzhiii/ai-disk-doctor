use std::fs;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

use anyhow::Result;
use chrono::{DateTime, Local};
use glob::glob;
use serde::Serialize;
use sysinfo::Disks;
use walkdir::WalkDir;

use crate::rules::{expand_windows_path, RiskLevel, Rule};

#[derive(Debug, Serialize)]
pub struct ScanReport {
    pub scan_time: DateTime<Local>,
    pub volumes: Vec<Volume>,
    pub findings: Vec<Finding>,
    pub summary: Summary,
}

#[derive(Debug, Serialize)]
pub struct Volume {
    pub name: String,
    pub mount_point: String,
    pub total_bytes: u64,
    pub available_bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct Finding {
    pub id: String,
    pub name: String,
    pub category: String,
    pub path: String,
    pub exists: bool,
    pub size_bytes: u64,
    pub risk: RiskLevel,
    pub action: String,
    pub reason: String,
    pub warnings: Vec<String>,
}

#[derive(Debug, Default, Serialize)]
pub struct Summary {
    pub total_rules: usize,
    pub matched_paths: usize,
    pub total_size_bytes: u64,
    pub safe_bytes: u64,
    pub review_bytes: u64,
    pub dangerous_bytes: u64,
    pub system_bytes: u64,
    pub top_findings: Vec<TopFinding>,
    pub reclaimable_safe_bytes: u64,
}

#[derive(Debug, Serialize)]
pub struct TopFinding {
    pub id: String,
    pub path: String,
    pub risk: RiskLevel,
    pub size_bytes: u64,
}

pub struct ScanProgressEvent<'a> {
    pub current: usize,
    pub total: usize,
    pub rule_id: &'a str,
}

pub fn scan(rules: &[Rule], max_scan_depth: usize) -> Result<ScanReport> {
    scan_with_progress(rules, max_scan_depth, |_| {})
}

pub fn scan_with_progress<F>(
    rules: &[Rule],
    max_scan_depth: usize,
    mut on_progress: F,
) -> Result<ScanReport>
where
    F: FnMut(ScanProgressEvent<'_>),
{
    let mut findings = Vec::new();
    let volumes = collect_volumes();
    let mut summary = Summary {
        total_rules: rules.len(),
        ..Summary::default()
    };

    for (index, rule) in rules.iter().enumerate() {
        let mut seen_paths = HashSet::new();
        on_progress(ScanProgressEvent {
            current: index + 1,
            total: rules.len(),
            rule_id: &rule.id,
        });

        for raw_path in &rule.paths {
            let Some(expanded_path) = expand_windows_path(raw_path) else {
                continue;
            };
            let matched_paths = resolve_rule_paths(&expanded_path)?;

            if matched_paths.is_empty() {
                if !seen_paths.insert(expanded_path.clone()) {
                    continue;
                }
                findings.push(Finding {
                    id: rule.id.clone(),
                    name: rule.name.clone(),
                    category: rule.category.clone(),
                    path: expanded_path.display().to_string(),
                    exists: false,
                    size_bytes: 0,
                    risk: rule.risk.clone(),
                    action: rule.cleanup.method.clone(),
                    reason: rule.reason.clone(),
                    warnings: rule.warnings.clone(),
                });
                continue;
            }

            for matched_path in matched_paths {
                if !seen_paths.insert(matched_path.clone()) {
                    continue;
                }
                let exists = matched_path.exists();
                let size_bytes = if exists { compute_size(&matched_path, max_scan_depth)? } else { 0 };

                if exists {
                    summary.matched_paths += 1;
                    summary.total_size_bytes = summary.total_size_bytes.saturating_add(size_bytes);
                    match rule.risk {
                        RiskLevel::Safe => {
                            summary.safe_bytes = summary.safe_bytes.saturating_add(size_bytes);
                            summary.reclaimable_safe_bytes = summary.reclaimable_safe_bytes.saturating_add(size_bytes);
                        }
                        RiskLevel::Review => {
                            summary.review_bytes = summary.review_bytes.saturating_add(size_bytes);
                        }
                        RiskLevel::Dangerous => {
                            summary.dangerous_bytes = summary.dangerous_bytes.saturating_add(size_bytes);
                        }
                        RiskLevel::System => {
                            summary.system_bytes = summary.system_bytes.saturating_add(size_bytes);
                        }
                    }
                }

                findings.push(Finding {
                    id: rule.id.clone(),
                    name: rule.name.clone(),
                    category: rule.category.clone(),
                    path: matched_path.display().to_string(),
                    exists,
                    size_bytes,
                    risk: rule.risk.clone(),
                    action: rule.cleanup.method.clone(),
                    reason: rule.reason.clone(),
                    warnings: rule.warnings.clone(),
                });
            }
        }
    }

    findings.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes).then_with(|| a.id.cmp(&b.id)));
    summary.top_findings = findings
        .iter()
        .filter(|finding| finding.exists)
        .take(5)
        .map(|finding| TopFinding {
            id: finding.id.clone(),
            path: finding.path.clone(),
            risk: finding.risk,
            size_bytes: finding.size_bytes,
        })
        .collect();

    Ok(ScanReport {
        scan_time: Local::now(),
        volumes,
        findings,
        summary,
    })
}

fn resolve_rule_paths(path: &Path) -> Result<Vec<PathBuf>> {
    let pattern = path.display().to_string();
    if contains_glob(&pattern) {
        let mut matched = Vec::new();
        for entry in glob(&pattern)? {
            let matched_path = match entry {
                Ok(path) => path,
                Err(_) => continue,
            };
            matched.push(matched_path);
        }

        matched.sort();
        matched.dedup();
        return Ok(matched);
    }

    Ok(vec![path.to_path_buf()])
}

fn contains_glob(path: &str) -> bool {
    path.contains('*') || path.contains('?') || path.contains('[')
}

fn collect_volumes() -> Vec<Volume> {
    Disks::new_with_refreshed_list()
        .list()
        .iter()
        .map(|disk| Volume {
            name: disk.name().to_string_lossy().into_owned(),
            mount_point: disk.mount_point().display().to_string(),
            total_bytes: disk.total_space(),
            available_bytes: disk.available_space(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::{resolve_rule_paths, Summary, TopFinding};
    use crate::rules::RiskLevel;
    use crate::test_support::{env_lock, EnvSnapshot};

    #[test]
    fn resolves_globbed_rule_paths() {
        let temp = tempdir().expect("tempdir should exist");
        let root = temp.path();
        let nested = root.join("demo").join(".playwright-browsers");
        fs::create_dir_all(&nested).expect("nested directory should be created");

        let pattern = root.join("**").join(".playwright-browsers");
        let matched = resolve_rule_paths(&pattern).expect("glob should resolve");

        assert_eq!(matched.len(), 1);
        assert_eq!(matched[0], nested);
    }

    #[test]
    fn summary_defaults_to_empty_top_findings() {
        let summary = Summary::default();
        assert!(summary.top_findings.is_empty());
    }

    #[test]
    fn top_finding_keeps_risk_and_size() {
        let finding = TopFinding {
            id: "demo".to_string(),
            path: "C:\\demo".to_string(),
            risk: RiskLevel::Safe,
            size_bytes: 42,
        };

        assert_eq!(finding.size_bytes, 42);
        assert_eq!(finding.risk, RiskLevel::Safe);
    }

    #[test]
    fn scan_with_progress_reports_rule_steps() {
        let temp = tempdir().expect("tempdir should exist");
        let first = temp.path().join("first-cache");
        let second = temp.path().join("second-cache");
        fs::create_dir_all(&first).expect("first dir should exist");
        fs::create_dir_all(&second).expect("second dir should exist");

        let rules = vec![
            crate::rules::Rule {
                id: "first".to_string(),
                name: "First".to_string(),
                category: "test".to_string(),
                platform: "windows".to_string(),
                paths: vec![first.display().to_string()],
                risk: RiskLevel::Safe,
                cleanup: crate::rules::Cleanup {
                    method: "quarantine".to_string(),
                },
                exclusions: Vec::new(),
                reason: "first".to_string(),
                warnings: Vec::new(),
            },
            crate::rules::Rule {
                id: "second".to_string(),
                name: "Second".to_string(),
                category: "test".to_string(),
                platform: "windows".to_string(),
                paths: vec![second.display().to_string()],
                risk: RiskLevel::Review,
                cleanup: crate::rules::Cleanup {
                    method: "report-only".to_string(),
                },
                exclusions: Vec::new(),
                reason: "second".to_string(),
                warnings: Vec::new(),
            },
        ];
        let mut events = Vec::new();

        let report = super::scan_with_progress(&rules, 20, |event| {
            events.push((event.current, event.total, event.rule_id.to_string()));
        })
        .expect("scan should succeed");

        assert_eq!(report.summary.total_rules, 2);
        assert_eq!(
            events,
            vec![(1, 2, "first".to_string()), (2, 2, "second".to_string())]
        );
    }

    #[test]
    fn scan_large_files_discovers_entries_above_threshold() {
        let temp = tempdir().expect("tempdir should exist");
        let root = temp.path();
        fs::create_dir_all(root.join("big-dir")).expect("big dir should exist");
        fs::write(root.join("small.txt"), vec![0_u8; 100]).expect("small file should write");
        fs::write(root.join("big-dir").join("large.bin"), vec![0_u8; 1000])
            .expect("large file should write");
        fs::write(root.join("big-dir").join("tiny.bin"), vec![0_u8; 10])
            .expect("tiny file should write");

        let report = super::scan_large_files(root, 500).expect("scan should succeed");

        assert_eq!(report.scan_root, root.display().to_string());
        assert_eq!(report.min_size_bytes, 500);
        assert!(
            report.entries.len() >= 1,
            "should find at least one entry above 500 bytes"
        );

        let big_dir = report.entries.iter().find(|e| e.path.ends_with("big-dir"));
        assert!(big_dir.is_some(), "should find big-dir directory");
        assert!(big_dir.unwrap().is_directory);

        let large_file = report
            .entries
            .iter()
            .find(|e| e.path.ends_with("large.bin"));
        assert!(large_file.is_some(), "should find large.bin");
        assert!(!large_file.unwrap().is_directory);

        assert!(
            !report.entries.iter().any(|e| e.path.ends_with("small.txt")),
            "small.txt should not appear below threshold"
        );
        assert!(
            !report.entries.iter().any(|e| e.path.ends_with("tiny.bin")),
            "tiny.bin should not appear below threshold"
        );
    }

    #[test]
    fn scan_skips_unresolved_home_tilde_paths() {
        let _env_lock = env_lock();
        let _env_snapshot = EnvSnapshot::capture(&["HOME"]);
        std::env::remove_var("HOME");
        let rules = vec![crate::rules::Rule {
            id: "unix-home".to_string(),
            name: "Unix home path".to_string(),
            category: "models".to_string(),
            platform: "cross-platform".to_string(),
            paths: vec!["~/.cache/huggingface".to_string()],
            risk: RiskLevel::Review,
            cleanup: crate::rules::Cleanup {
                method: "guide".to_string(),
            },
            exclusions: Vec::new(),
            reason: "test".to_string(),
            warnings: Vec::new(),
        }];

        let report = super::scan(&rules, 20).expect("scan should succeed");

        assert_eq!(report.summary.total_rules, 1);
        assert!(
            report.findings.is_empty(),
            "unresolved ~/ paths should be skipped instead of producing bogus findings"
        );
    }

    #[test]
    fn scan_skips_unresolved_windows_env_paths() {
        let _env_lock = env_lock();
        let _env_snapshot = EnvSnapshot::capture(&["AIDISK_TEST_HOME"]);
        std::env::remove_var("AIDISK_TEST_HOME");
        let rules = vec![crate::rules::Rule {
            id: "windows-env".to_string(),
            name: "Windows env path".to_string(),
            category: "models".to_string(),
            platform: "cross-platform".to_string(),
            paths: vec!["%AIDISK_TEST_HOME%\\cache".to_string()],
            risk: RiskLevel::Review,
            cleanup: crate::rules::Cleanup {
                method: "guide".to_string(),
            },
            exclusions: Vec::new(),
            reason: "test".to_string(),
            warnings: Vec::new(),
        }];

        let report = super::scan(&rules, 20).expect("scan should succeed");

        assert_eq!(report.summary.total_rules, 1);
        assert!(
            report.findings.is_empty(),
            "unresolved %VAR% paths should be skipped instead of producing bogus findings"
        );
    }

    #[test]
    fn scan_deduplicates_equivalent_rule_paths() {
        let _env_lock = env_lock();
        let _env_snapshot = EnvSnapshot::capture(&["HOME", "AIDISK_TEST_HOME"]);
        let temp = tempdir().expect("tempdir should exist");
        let home = temp.path();
        let target = home.join(".cache").join("huggingface");
        fs::create_dir_all(&target).expect("cache dir should exist");
        fs::write(target.join("model.bin"), vec![0_u8; 256]).expect("model should write");

        std::env::set_var("HOME", home);
        std::env::set_var("AIDISK_TEST_HOME", home);
        let rules = vec![crate::rules::Rule {
            id: "huggingface-cache".to_string(),
            name: "Hugging Face cache".to_string(),
            category: "models".to_string(),
            platform: "cross-platform".to_string(),
            paths: vec![
                "%AIDISK_TEST_HOME%\\.cache\\huggingface".to_string(),
                "~/.cache/huggingface".to_string(),
            ],
            risk: RiskLevel::Review,
            cleanup: crate::rules::Cleanup {
                method: "guide".to_string(),
            },
            exclusions: Vec::new(),
            reason: "test".to_string(),
            warnings: Vec::new(),
        }];

        let report = super::scan(&rules, 20).expect("scan should succeed");

        assert_eq!(report.summary.matched_paths, 1);
        assert_eq!(report.findings.len(), 1);
        assert_eq!(report.findings[0].path, target.display().to_string());
        assert_eq!(report.summary.total_size_bytes, report.findings[0].size_bytes);
    }
}

#[derive(Debug, Serialize)]
pub struct LargeFilesReport {
    pub scan_root: String,
    pub min_size: String,
    pub min_size_bytes: u64,
    pub scan_time: DateTime<Local>,
    pub entries: Vec<LargeFileEntry>,
}

#[derive(Debug, Serialize)]
pub struct LargeFileEntry {
    pub path: String,
    pub size_bytes: u64,
    pub is_directory: bool,
}

pub fn scan_large_files(root: &Path, min_size_bytes: u64) -> Result<LargeFilesReport> {
    let mut entries = Vec::new();

    for entry in WalkDir::new(root).follow_links(false).max_depth(20) {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        let metadata = match entry.metadata() {
            Ok(metadata) => metadata,
            Err(_) => continue,
        };

        let size_bytes = if metadata.is_dir() {
            compute_size(entry.path(), 20).unwrap_or(0)
        } else {
            metadata.len()
        };

        if size_bytes >= min_size_bytes {
            entries.push(LargeFileEntry {
                path: entry.path().display().to_string(),
                size_bytes,
                is_directory: metadata.is_dir(),
            });
        }
    }

    entries.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));

    Ok(LargeFilesReport {
        scan_root: root.display().to_string(),
        min_size: format_size(min_size_bytes),
        min_size_bytes,
        scan_time: Local::now(),
        entries,
    })
}

fn compute_size(path: &Path, max_depth: usize) -> Result<u64> {
    let metadata = fs::metadata(path)?;
    if metadata.is_file() {
        return Ok(metadata.len());
    }

    let mut total = 0_u64;
    for entry in WalkDir::new(path).follow_links(false).max_depth(max_depth) {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        let metadata = match entry.metadata() {
            Ok(metadata) => metadata,
            Err(_) => continue,
        };
        if metadata.is_file() {
            total = total.saturating_add(metadata.len());
        }
    }

    Ok(total)
}

fn format_size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];

    let mut value = bytes as f64;
    let mut unit = 0_usize;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }

    if unit == 0 {
        format!("{} {}", bytes, UNITS[unit])
    } else {
        format!("{value:.2} {}", UNITS[unit])
    }
}
