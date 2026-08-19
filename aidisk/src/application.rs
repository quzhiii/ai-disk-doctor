use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::Serialize;

use crate::explainability;
pub use crate::explainability::ExplainabilityReport;
use crate::history;
pub use crate::history::ScanSnapshot;
use crate::model_inventory;
pub use crate::model_inventory::ModelInventoryReport;
use crate::policy::{self, Policy};
use crate::rules::{self, Rule};
use crate::rules_repo;
use crate::scanner;
pub use crate::scanner::{ScanProgressEvent, ScanReport};

#[derive(Debug, Clone)]
pub struct ScanRequest {
    pub rules_dir: Option<PathBuf>,
    pub rules_repo: Option<String>,
    pub category: Option<String>,
    pub policy: Option<PathBuf>,
    pub default_rules_dir: PathBuf,
    pub default_policy_path: PathBuf,
    pub reports_dir: Option<PathBuf>,
    pub persist_snapshot: SnapshotPersistence,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SnapshotPersistence {
    Save,
    Skip,
}

#[derive(Debug)]
pub struct ScanResult {
    pub report: ScanReport,
    pub snapshot_path: Option<PathBuf>,
    pub rules_dir: PathBuf,
}

#[derive(Debug)]
pub struct ExplainableScanResult {
    pub scan: ScanResult,
    pub explainability: ExplainabilityReport,
}

struct ScanExecution {
    result: ScanResult,
    rules: Vec<Rule>,
}

#[derive(Debug, Clone)]
pub struct AssetInventoryRequest {
    pub root: Option<PathBuf>,
    pub tool: ApplicationInventoryTool,
    pub max_depth: usize,
    pub stale_after_days: u64,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ApplicationInventoryTool {
    Auto,
    Ollama,
    Huggingface,
    LmStudio,
    Generic,
}

impl From<ApplicationInventoryTool> for model_inventory::InventoryTool {
    fn from(tool: ApplicationInventoryTool) -> Self {
        match tool {
            ApplicationInventoryTool::Auto => Self::Auto,
            ApplicationInventoryTool::Ollama => Self::Ollama,
            ApplicationInventoryTool::Huggingface => Self::Huggingface,
            ApplicationInventoryTool::LmStudio => Self::LmStudio,
            ApplicationInventoryTool::Generic => Self::Generic,
        }
    }
}

#[derive(Debug, Clone)]
pub struct HistoryRequest {
    pub reports_dir: Option<PathBuf>,
}

#[derive(Debug, Serialize)]
pub struct HistoryResult {
    pub reports_dir: PathBuf,
    pub snapshots: Vec<ScanSnapshot>,
    pub latest_snapshot: Option<ScanSnapshot>,
    pub latest_pair: Option<ScanSnapshotPair>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct ScanSnapshotPair {
    pub before: ScanSnapshot,
    pub after: ScanSnapshot,
}

pub fn run_scan(request: ScanRequest) -> Result<ScanResult> {
    run_scan_with_progress(request, |_| {})
}

pub fn run_scan_with_progress<F>(request: ScanRequest, on_progress: F) -> Result<ScanResult>
where
    F: FnMut(ScanProgressEvent<'_>),
{
    Ok(run_scan_execution_with_progress(request, on_progress)?.result)
}

pub fn run_explainable_scan(request: ScanRequest) -> Result<ExplainableScanResult> {
    run_explainable_scan_with_progress(request, |_| {})
}

pub fn run_explainable_scan_with_progress<F>(
    request: ScanRequest,
    on_progress: F,
) -> Result<ExplainableScanResult>
where
    F: FnMut(ScanProgressEvent<'_>),
{
    let execution = run_scan_execution_with_progress(request, on_progress)?;
    let explainability = explainability::build(&execution.result.report, &execution.rules);

    Ok(ExplainableScanResult {
        scan: execution.result,
        explainability,
    })
}

fn run_scan_execution_with_progress<F>(
    request: ScanRequest,
    on_progress: F,
) -> Result<ScanExecution>
where
    F: FnMut(ScanProgressEvent<'_>),
{
    let rules_dir = resolve_rules_dir(
        request.rules_dir,
        request.rules_repo,
        &request.default_rules_dir,
    )?;
    let policy = load_scan_policy(request.policy, &request.default_policy_path)?;
    let rules = load_scan_rules(&rules_dir, request.category.as_deref())?;
    let mut report =
        scanner::scan_with_progress(&rules, policy.planner.max_scan_depth, on_progress)?;
    report.policy = Some(policy.snapshot());

    let snapshot_path = match request.persist_snapshot {
        SnapshotPersistence::Save => {
            let reports_dir = request
                .reports_dir
                .unwrap_or_else(history::default_reports_dir);
            Some(history::save_scan_snapshot(&report, &reports_dir)?)
        }
        SnapshotPersistence::Skip => None,
    };

    Ok(ScanExecution {
        result: ScanResult {
            report,
            snapshot_path,
            rules_dir,
        },
        rules,
    })
}

pub(crate) fn load_scan_rules(rules_dir: &Path, category: Option<&str>) -> Result<Vec<Rule>> {
    let rules = rules::load_rules(rules_dir)?;
    Ok(rules::filter_rules(rules, category))
}

pub(crate) fn load_scan_policy(
    explicit_policy: Option<PathBuf>,
    default_policy_path: &Path,
) -> Result<Policy> {
    if let Some(explicit_policy) = explicit_policy {
        return policy::load_policy(&explicit_policy);
    }

    match policy::load_policy(default_policy_path) {
        Ok(policy) => Ok(policy),
        Err(_) => Ok(default_scan_policy()),
    }
}

pub(crate) fn default_scan_policy() -> Policy {
    Policy {
        sensitive_markers: vec!["token".to_string()],
        planner: policy::PlannerPolicy {
            skip_modified_within_minutes: 30,
            allow_actions: vec![
                "quarantine".to_string(),
                "report-only".to_string(),
                "guide".to_string(),
            ],
            max_scan_depth: 20,
        },
    }
}

pub(crate) fn resolve_rules_dir(
    rules_dir: Option<PathBuf>,
    rules_repo: Option<String>,
    default_rules_dir: &Path,
) -> Result<PathBuf> {
    if let Some(rules_dir) = rules_dir {
        return Ok(rules_dir);
    }

    if let Some(rules_repo) = rules_repo {
        return rules_repo::resolve_rules_repo(
            &rules_repo,
            &rules_repo::default_rules_repo_cache_root(),
        );
    }

    Ok(default_rules_dir.to_path_buf())
}

pub fn inventory_assets(request: AssetInventoryRequest) -> Result<ModelInventoryReport> {
    model_inventory::build_inventory(&model_inventory::InventoryOptions {
        root: request.root,
        tool: request.tool.into(),
        max_depth: request.max_depth,
        stale_after_days: request.stale_after_days,
    })
}

pub fn read_history(request: HistoryRequest) -> Result<HistoryResult> {
    let reports_dir = request
        .reports_dir
        .unwrap_or_else(history::default_reports_dir);
    let snapshots = history::list_scan_snapshots(&reports_dir)?;
    let latest_snapshot = snapshots.last().cloned();
    let latest_pair = match snapshots.as_slice() {
        [.., before, after] => Some(ScanSnapshotPair {
            before: before.clone(),
            after: after.clone(),
        }),
        _ => None,
    };

    Ok(HistoryResult {
        reports_dir,
        snapshots,
        latest_snapshot,
        latest_pair,
    })
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::{
        inventory_assets, read_history, run_explainable_scan, run_scan, ApplicationInventoryTool,
        AssetInventoryRequest, HistoryRequest, ScanRequest, SnapshotPersistence,
    };

    fn write_policy(path: &std::path::Path) {
        fs::write(
            path,
            r#"sensitive_markers:
  - token
planner:
  skip_modified_within_minutes: 30
  allow_actions:
    - quarantine
    - report-only
    - guide
  max_scan_depth: 20
"#,
        )
        .expect("policy should be written");
    }

    fn write_rule(path: &std::path::Path, target: &std::path::Path) {
        write_rule_with_method(path, target, "report-only");
    }

    fn write_rule_with_method(path: &std::path::Path, target: &std::path::Path, method: &str) {
        fs::write(
            path,
            format!(
                r#"id: app-boundary-cache
name: App Boundary Cache
category: test
platform: cross-platform
paths:
  - '{}'
risk: review
cleanup:
  method: {method}
exclusions: []
reason: "fixture"
warnings: []
"#,
                target.display()
            ),
        )
        .expect("rule should be written");
    }

    #[test]
    fn scan_boundary_reuses_rules_policy_scanner_and_can_skip_snapshot_persistence() {
        let temp = tempdir().expect("tempdir should exist");
        let rules_dir = temp.path().join("rules");
        let cache = temp.path().join("cache");
        let policy = temp.path().join("policy.yaml");
        fs::create_dir_all(&rules_dir).expect("rules dir should exist");
        fs::create_dir_all(&cache).expect("cache dir should exist");
        fs::write(cache.join("artifact.bin"), vec![0_u8; 12]).expect("artifact should write");
        write_rule(&rules_dir.join("cache.yaml"), &cache);
        write_policy(&policy);

        let result = run_scan(ScanRequest {
            rules_dir: Some(rules_dir.clone()),
            rules_repo: None,
            category: Some("test".to_string()),
            policy: Some(policy),
            default_rules_dir: temp.path().join("unused-rules"),
            default_policy_path: temp.path().join("unused-policy.yaml"),
            reports_dir: Some(temp.path().join("reports")),
            persist_snapshot: SnapshotPersistence::Skip,
        })
        .expect("scan boundary should run");

        assert_eq!(result.rules_dir, rules_dir);
        assert_eq!(result.report.summary.total_rules, 1);
        assert_eq!(result.report.summary.matched_paths, 1);
        assert_eq!(result.report.summary.report_only_bytes, 12);
        assert!(result.report.policy.is_some());
        assert_eq!(result.snapshot_path, None);
        assert!(
            !temp.path().join("reports").exists(),
            "explicit Skip must not create AI Disk Doctor snapshots"
        );
    }

    #[test]
    fn explainable_scan_boundary_adds_contract_without_changing_scan_result() {
        let temp = tempdir().expect("tempdir should exist");
        let rules_dir = temp.path().join("rules");
        let cache = temp.path().join("cache");
        let policy = temp.path().join("policy.yaml");
        fs::create_dir_all(&rules_dir).expect("rules dir should exist");
        fs::create_dir_all(&cache).expect("cache dir should exist");
        fs::write(cache.join("artifact.bin"), vec![0_u8; 12]).expect("artifact should write");
        write_rule(&rules_dir.join("cache.yaml"), &cache);
        write_policy(&policy);

        let result = run_explainable_scan(ScanRequest {
            rules_dir: Some(rules_dir.clone()),
            rules_repo: None,
            category: Some("test".to_string()),
            policy: Some(policy),
            default_rules_dir: temp.path().join("unused-rules"),
            default_policy_path: temp.path().join("unused-policy.yaml"),
            reports_dir: Some(temp.path().join("reports")),
            persist_snapshot: SnapshotPersistence::Skip,
        })
        .expect("explainable scan boundary should run");

        assert_eq!(result.scan.rules_dir, rules_dir);
        assert_eq!(result.scan.report.summary.observed_bytes, 12);
        assert_eq!(result.explainability.contract, "explainability-v1");
        assert_eq!(result.explainability.storage.observed_bytes, 12);
        assert_eq!(result.explainability.categories.len(), 1);
        assert!(
            !temp.path().join("reports").exists(),
            "snapshot skip must not create reports as a side effect"
        );
    }

    #[test]
    fn scan_boundary_can_persist_snapshot_when_requested() {
        let temp = tempdir().expect("tempdir should exist");
        let rules_dir = temp.path().join("rules");
        let cache = temp.path().join("cache");
        let policy = temp.path().join("policy.yaml");
        let reports_dir = temp.path().join("reports");
        fs::create_dir_all(&rules_dir).expect("rules dir should exist");
        fs::create_dir_all(&cache).expect("cache dir should exist");
        write_rule(&rules_dir.join("cache.yaml"), &cache);
        write_policy(&policy);

        let result = run_scan(ScanRequest {
            rules_dir: Some(rules_dir),
            rules_repo: None,
            category: None,
            policy: Some(policy),
            default_rules_dir: temp.path().join("unused-rules"),
            default_policy_path: temp.path().join("unused-policy.yaml"),
            reports_dir: Some(reports_dir.clone()),
            persist_snapshot: SnapshotPersistence::Save,
        })
        .expect("scan boundary should save snapshot");

        let snapshot_path = result.snapshot_path.expect("snapshot should be returned");
        assert!(snapshot_path.starts_with(&reports_dir));
        assert!(snapshot_path.exists());
    }

    #[test]
    fn scan_boundary_does_not_execute_cleanup_or_restore_actions() {
        let temp = tempdir().expect("tempdir should exist");
        let rules_dir = temp.path().join("rules");
        let cache = temp.path().join("cache");
        let policy = temp.path().join("policy.yaml");
        fs::create_dir_all(&rules_dir).expect("rules dir should exist");
        fs::create_dir_all(&cache).expect("cache dir should exist");
        let source_file = cache.join("artifact.bin");
        fs::write(&source_file, vec![0_u8; 12]).expect("artifact should write");
        write_rule_with_method(&rules_dir.join("cache.yaml"), &cache, "quarantine");
        write_policy(&policy);

        let result = run_scan(ScanRequest {
            rules_dir: Some(rules_dir),
            rules_repo: None,
            category: None,
            policy: Some(policy),
            default_rules_dir: temp.path().join("unused-rules"),
            default_policy_path: temp.path().join("unused-policy.yaml"),
            reports_dir: Some(temp.path().join("reports")),
            persist_snapshot: SnapshotPersistence::Skip,
        })
        .expect("scan boundary should run without mutating source files");

        assert_eq!(result.report.summary.quarantine_bytes, 12);
        assert!(
            source_file.exists(),
            "application scan must not move or delete files"
        );
        assert!(
            cache.exists(),
            "application scan must not execute quarantine"
        );
        assert!(
            !temp.path().join("reports").exists(),
            "snapshot skip must not create reports as a side effect"
        );
    }

    #[test]
    fn asset_inventory_boundary_reuses_existing_model_inventory() {
        let temp = tempdir().expect("tempdir should exist");
        fs::write(temp.path().join("demo.gguf"), vec![0_u8; 7]).expect("model should write");

        let report = inventory_assets(AssetInventoryRequest {
            root: Some(temp.path().to_path_buf()),
            tool: ApplicationInventoryTool::Generic,
            max_depth: 20,
            stale_after_days: 90,
        })
        .expect("inventory boundary should run");

        assert_eq!(report.schema_version, 1);
        assert_eq!(report.summary.total_assets, 1);
        assert_eq!(report.summary.report_only_assets, 1);
        assert!(temp.path().join("demo.gguf").exists());
    }

    #[test]
    fn history_boundary_lists_latest_snapshot_and_pair_without_mutation() {
        let temp = tempdir().expect("tempdir should exist");
        let reports_dir = temp.path().join("reports");
        fs::create_dir_all(&reports_dir).expect("reports dir should exist");
        fs::write(reports_dir.join("scan-20260101-000000-000.json"), "{}").unwrap();
        fs::write(reports_dir.join("ignore.json"), "{}").unwrap();
        fs::write(reports_dir.join("scan-20260103-000000-000.json"), "{}").unwrap();
        fs::write(reports_dir.join("scan-20260102-000000-000.json"), "{}").unwrap();

        let report = read_history(HistoryRequest {
            reports_dir: Some(reports_dir.clone()),
        })
        .expect("history boundary should run");

        assert_eq!(report.reports_dir, reports_dir);
        assert_eq!(report.snapshots.len(), 3);
        assert_eq!(
            report
                .latest_snapshot
                .as_ref()
                .map(|snapshot| snapshot.file_name.as_str()),
            Some("scan-20260103-000000-000.json")
        );
        let pair = report.latest_pair.expect("latest pair should exist");
        assert_eq!(pair.before.file_name, "scan-20260102-000000-000.json");
        assert_eq!(pair.after.file_name, "scan-20260103-000000-000.json");
    }
}
