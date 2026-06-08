use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use chrono::{DateTime, Local};
use glob::glob;
use serde::Serialize;
use sysinfo::Disks;
use walkdir::WalkDir;

use crate::policy::PolicySnapshot;
use crate::rules::{expand_windows_path, RiskLevel, Rule};

#[derive(Debug, Serialize)]
pub struct ScanReport {
    pub scan_time: DateTime<Local>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy: Option<PolicySnapshot>,
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
    pub partial: bool,
    pub partial_reasons: Vec<String>,
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
    pub partial_findings: usize,
}

#[derive(Debug, Serialize)]
pub struct TopFinding {
    pub id: String,
    pub path: String,
    pub risk: RiskLevel,
    pub size_bytes: u64,
    pub partial: bool,
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
                    partial: false,
                    partial_reasons: Vec::new(),
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
                let computed_size = if exists {
                    compute_size(&matched_path, max_scan_depth)?
                } else {
                    ComputedSize::exact(0)
                };
                let size_bytes = computed_size.size_bytes;

                if exists {
                    summary.matched_paths += 1;
                    summary.total_size_bytes = summary.total_size_bytes.saturating_add(size_bytes);
                    if computed_size.partial {
                        summary.partial_findings += 1;
                    }
                    match rule.risk {
                        RiskLevel::Safe => {
                            summary.safe_bytes = summary.safe_bytes.saturating_add(size_bytes);
                            summary.reclaimable_safe_bytes =
                                summary.reclaimable_safe_bytes.saturating_add(size_bytes);
                        }
                        RiskLevel::Review => {
                            summary.review_bytes = summary.review_bytes.saturating_add(size_bytes);
                        }
                        RiskLevel::Dangerous => {
                            summary.dangerous_bytes =
                                summary.dangerous_bytes.saturating_add(size_bytes);
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
                    partial: computed_size.partial,
                    partial_reasons: computed_size.partial_reasons,
                    risk: rule.risk.clone(),
                    action: rule.cleanup.method.clone(),
                    reason: rule.reason.clone(),
                    warnings: rule.warnings.clone(),
                });
            }
        }
    }

    findings.sort_by(|a, b| {
        b.size_bytes
            .cmp(&a.size_bytes)
            .then_with(|| a.id.cmp(&b.id))
    });
    summary.top_findings = findings
        .iter()
        .filter(|finding| finding.exists)
        .take(5)
        .map(|finding| TopFinding {
            id: finding.id.clone(),
            path: finding.path.clone(),
            risk: finding.risk,
            size_bytes: finding.size_bytes,
            partial: finding.partial,
        })
        .collect();

    Ok(ScanReport {
        scan_time: Local::now(),
        policy: None,
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

#[derive(Debug)]
struct ComputedSize {
    size_bytes: u64,
    partial: bool,
    partial_reasons: Vec<String>,
}

impl ComputedSize {
    fn exact(size_bytes: u64) -> Self {
        Self {
            size_bytes,
            partial: false,
            partial_reasons: Vec::new(),
        }
    }

    fn mark_partial(&mut self, reason: String) {
        self.partial = true;
        if !self.partial_reasons.contains(&reason) {
            self.partial_reasons.push(reason);
        }
    }
}

#[derive(Debug)]
struct WalkSizeEntry {
    depth: usize,
    metadata: std::result::Result<WalkSizeMetadata, String>,
}

#[derive(Debug)]
struct WalkSizeMetadata {
    is_file: bool,
    is_dir: bool,
    has_unscanned_children: bool,
    len: u64,
}

fn compute_size_from_walk_entries<I>(entries: I, max_depth: usize) -> ComputedSize
where
    I: IntoIterator<Item = WalkSizeEntry>,
{
    let mut computed = ComputedSize::exact(0);
    for entry in entries {
        let metadata = match entry.metadata {
            Ok(metadata) => metadata,
            Err(error) => {
                computed.mark_partial(format!("metadata unavailable: {error}"));
                continue;
            }
        };

        if metadata.is_file {
            computed.size_bytes = computed.size_bytes.saturating_add(metadata.len);
        } else if metadata.is_dir && entry.depth == max_depth && metadata.has_unscanned_children {
            computed.mark_partial(format!(
                "max scan depth {max_depth} reached; deeper children were not scanned"
            ));
        }
    }

    computed
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
            partial: false,
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
        assert_eq!(
            report.summary.total_size_bytes,
            report.findings[0].size_bytes
        );
    }

    #[test]
    fn scan_marks_finding_partial_when_max_depth_truncates_directory_size() {
        let temp = tempdir().expect("tempdir should exist");
        let root = temp.path().join("cache-root");
        let nested = root.join("nested");
        fs::create_dir_all(&nested).expect("nested dir should exist");
        fs::write(root.join("visible.bin"), vec![0_u8; 10]).expect("visible file should write");
        fs::write(nested.join("hidden.bin"), vec![0_u8; 99]).expect("hidden file should write");

        let report = super::scan(&[test_rule_for_path(&root)], 1).expect("scan should succeed");

        assert_eq!(report.findings.len(), 1);
        assert_eq!(report.findings[0].size_bytes, 10);
        assert!(report.findings[0].partial);
        assert!(report.findings[0]
            .partial_reasons
            .iter()
            .any(|reason| reason.contains("max scan depth")));
        assert_eq!(report.summary.partial_findings, 1);
    }

    #[test]
    fn compute_size_marks_partial_when_descendant_metadata_is_unreadable() {
        let computed = super::compute_size_from_walk_entries(
            vec![
                super::WalkSizeEntry {
                    depth: 1,
                    metadata: Ok(super::WalkSizeMetadata {
                        is_file: true,
                        is_dir: false,
                        has_unscanned_children: false,
                        len: 5,
                    }),
                },
                super::WalkSizeEntry {
                    depth: 2,
                    metadata: Err("access denied".to_string()),
                },
            ],
            20,
        );

        assert_eq!(computed.size_bytes, 5);
        assert!(computed.partial);
        assert!(computed
            .partial_reasons
            .iter()
            .any(|reason| reason.contains("metadata")));
    }

    #[test]
    fn scan_does_not_mark_empty_root_at_depth_boundary_as_partial() {
        let temp = tempdir().expect("tempdir should exist");
        let root = temp.path().join("empty-root");
        fs::create_dir_all(&root).expect("empty root should exist");

        let report = super::scan(&[test_rule_for_path(&root)], 0).expect("scan should succeed");

        assert_eq!(report.findings.len(), 1);
        assert_eq!(report.findings[0].size_bytes, 0);
        assert!(!report.findings[0].partial, "empty boundary directory should not be partial");
        assert_eq!(report.summary.partial_findings, 0);
    }

    fn test_rule_for_path(path: &std::path::Path) -> crate::rules::Rule {
        crate::rules::Rule {
            id: "cache-root".to_string(),
            name: "Cache Root".to_string(),
            category: "test".to_string(),
            platform: "windows".to_string(),
            paths: vec![path.display().to_string()],
            risk: RiskLevel::Safe,
            cleanup: crate::rules::Cleanup {
                method: "quarantine".to_string(),
            },
            exclusions: Vec::new(),
            reason: "test cache".to_string(),
            warnings: Vec::new(),
        }
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
            compute_size(entry.path(), 20)
                .map(|computed| computed.size_bytes)
                .unwrap_or(0)
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

fn compute_size(path: &Path, max_depth: usize) -> Result<ComputedSize> {
    let metadata = fs::metadata(path)?;
    if metadata.is_file() {
        return Ok(ComputedSize::exact(metadata.len()));
    }

    let mut entries = Vec::new();
    for entry in WalkDir::new(path).follow_links(false).max_depth(max_depth) {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                entries.push(WalkSizeEntry {
                    depth: max_depth,
                    metadata: Err(format!("traversal error: {error}")),
                });
                continue;
            }
        };
        let depth = entry.depth();
        let metadata = entry
            .metadata()
            .map(|metadata| WalkSizeMetadata {
                is_file: metadata.is_file(),
                is_dir: metadata.is_dir(),
                has_unscanned_children: metadata.is_dir() && entry.depth() == max_depth && fs::read_dir(entry.path()).map(|mut children| children.next().is_some()).unwrap_or(true),
                len: metadata.len(),
            })
            .map_err(|error| error.to_string());
        entries.push(WalkSizeEntry { depth, metadata });
    }

    Ok(compute_size_from_walk_entries(entries, max_depth))
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
