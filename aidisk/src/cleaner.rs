use std::fs;
use std::io::Write;
use std::path::Path;
use std::time::SystemTime;

use anyhow::Result;
use chrono::{DateTime, Duration, Local, Utc};
use serde::{Deserialize, Serialize};

use crate::planner::{ActionGroup, PlanReport, SkippedItem};

#[derive(Debug, Serialize)]
pub struct CleanReport {
    pub generated_at: DateTime<Local>,
    pub mode: String,
    pub candidate_count: usize,
    pub reclaimable_bytes: u64,
    pub groups: Vec<ActionGroup>,
    pub actions: Vec<CleanAction>,
    pub skipped: Vec<SkippedItem>,
}

#[derive(Debug, Serialize)]
pub struct CleanAction {
    pub path: String,
    pub action: String,
    pub size_bytes: u64,
    pub reason: String,
}

#[derive(Debug, Serialize)]
pub struct QuarantinePlan {
    pub root: String,
    pub skip_modified_within_minutes: u64,
    pub entries: Vec<QuarantineEntry>,
}

#[derive(Debug, Serialize)]
pub struct QuarantineEntry {
    pub source_path: String,
    pub destination_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecutionReport {
    pub generated_at: DateTime<Local>,
    pub mode: String,
    pub root: String,
    pub success_count: usize,
    pub failure_count: usize,
    pub index_path: String,
    pub log_path: String,
    pub results: Vec<ExecutionResult>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub source_path: String,
    pub destination_path: String,
    pub status: String,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct RestoreReport {
    pub generated_at: DateTime<Local>,
    pub mode: String,
    pub index_path: String,
    pub root: String,
    pub entry_count: usize,
    pub success_count: usize,
    pub failure_count: usize,
    pub results: Vec<RestoreResult>,
}

#[derive(Debug, Serialize)]
pub struct RestoreResult {
    pub source_path: String,
    pub destination_path: String,
    pub status: String,
    pub message: String,
}

pub fn build_dry_run(plan: &PlanReport) -> CleanReport {
    let actions = plan
        .candidates
        .iter()
        .map(|candidate| CleanAction {
            path: candidate.path.clone(),
            action: candidate.action.clone(),
            size_bytes: candidate.size_bytes,
            reason: candidate.reason.clone(),
        })
        .collect::<Vec<_>>();

    CleanReport {
        generated_at: Local::now(),
        mode: "dry-run".to_string(),
        candidate_count: plan.candidates.len(),
        reclaimable_bytes: plan.summary.reclaimable_bytes,
        groups: plan.groups.clone(),
        actions,
        skipped: plan.skipped.clone(),
    }
}

pub fn build_quarantine_plan(plan: &PlanReport, root: &str) -> QuarantinePlan {
    let entries = plan
        .candidates
        .iter()
        .filter(|candidate| candidate.action == "quarantine")
        .map(|candidate| QuarantineEntry {
            source_path: candidate.path.clone(),
            destination_path: format!("{}\\{}", root.trim_end_matches(['\\', '/']), sanitize_path(&candidate.path)),
        })
        .collect();

    QuarantinePlan {
        root: root.to_string(),
        skip_modified_within_minutes: plan.skip_modified_within_minutes,
        entries,
    }
}

pub fn execute_quarantine(plan: &QuarantinePlan) -> Result<ExecutionReport> {
    let mut results = Vec::new();
    let mut success_count = 0_usize;
    let mut failure_count = 0_usize;

    for entry in &plan.entries {
        match move_to_quarantine(entry, plan.skip_modified_within_minutes) {
            Ok(message) => {
                success_count += 1;
                results.push(ExecutionResult {
                    source_path: entry.source_path.clone(),
                    destination_path: entry.destination_path.clone(),
                    status: "moved".to_string(),
                    message,
                });
            }
            Err(error) => {
                failure_count += 1;
                let (status, message) = classify_execution_error(&error);
                results.push(ExecutionResult {
                    source_path: entry.source_path.clone(),
                    destination_path: entry.destination_path.clone(),
                    status,
                    message,
                });
            }
        }
    }

    let metadata_dir = Path::new(&plan.root).join(".aidisk");
    fs::create_dir_all(&metadata_dir)?;

    let timestamp = Local::now().format("%Y%m%d-%H%M%S").to_string();
    let index_path = metadata_dir.join(format!("quarantine-index-{timestamp}.json"));
    let log_path = metadata_dir.join(format!("quarantine-log-{timestamp}.log"));

    let report = ExecutionReport {
        generated_at: Local::now(),
        mode: "quarantine".to_string(),
        root: plan.root.clone(),
        success_count,
        failure_count,
        index_path: index_path.display().to_string(),
        log_path: log_path.display().to_string(),
        results,
    };

    write_execution_index(&index_path, &report)?;
    write_execution_log(&log_path, &report)?;

    Ok(report)
}

pub fn restore_from_index(index_path: &Path, dry_run: bool) -> Result<RestoreReport> {
    let content = fs::read_to_string(index_path)?;
    let execution_report: ExecutionReport = serde_json::from_str(&content)?;
    let mut results = Vec::new();
    let mut success_count = 0_usize;
    let mut failure_count = 0_usize;

    for entry in execution_report
        .results
        .iter()
        .filter(|result| result.status == "moved")
    {
        if dry_run {
            results.push(RestoreResult {
                source_path: entry.destination_path.clone(),
                destination_path: entry.source_path.clone(),
                status: "planned".to_string(),
                message: "restore dry-run only".to_string(),
            });
            continue;
        }

        match restore_entry(&entry.destination_path, &entry.source_path) {
            Ok(message) => {
                success_count += 1;
                results.push(RestoreResult {
                    source_path: entry.destination_path.clone(),
                    destination_path: entry.source_path.clone(),
                    status: "restored".to_string(),
                    message,
                });
            }
            Err(error) => {
                failure_count += 1;
                results.push(RestoreResult {
                    source_path: entry.destination_path.clone(),
                    destination_path: entry.source_path.clone(),
                    status: "failed".to_string(),
                    message: error.to_string(),
                });
            }
        }
    }

    Ok(RestoreReport {
        generated_at: Local::now(),
        mode: if dry_run {
            "dry-run".to_string()
        } else {
            "restore".to_string()
        },
        index_path: index_path.display().to_string(),
        root: execution_report.root,
        entry_count: results.len(),
        success_count,
        failure_count,
        results,
    })
}

fn move_to_quarantine(entry: &QuarantineEntry, skip_modified_within_minutes: u64) -> Result<String> {
    let source = Path::new(&entry.source_path);
    if !source.exists() {
        anyhow::bail!("source path does not exist");
    }

    if was_modified_recently(source, skip_modified_within_minutes) {
        anyhow::bail!(
            "source path was recently modified within {} minutes",
            skip_modified_within_minutes
        );
    }

    let destination = Path::new(&entry.destination_path);
    let parent = destination
        .parent()
        .ok_or_else(|| anyhow::anyhow!("destination parent is missing"))?;
    fs::create_dir_all(parent)?;
    fs::rename(source, destination)?;

    Ok("moved to quarantine".to_string())
}

fn restore_entry(source: &str, destination: &str) -> Result<String> {
    let source = Path::new(source);
    if !source.exists() {
        anyhow::bail!("quarantined source path does not exist");
    }

    let destination = Path::new(destination);
    let parent = destination
        .parent()
        .ok_or_else(|| anyhow::anyhow!("restore destination parent is missing"))?;
    fs::create_dir_all(parent)?;
    fs::rename(source, destination)?;

    Ok("restored from quarantine".to_string())
}

fn write_execution_index(path: &Path, report: &ExecutionReport) -> Result<()> {
    let content = serde_json::to_string_pretty(report)?;
    fs::write(path, content)?;
    Ok(())
}

fn write_execution_log(path: &Path, report: &ExecutionReport) -> Result<()> {
    let mut file = fs::File::create(path)?;
    writeln!(file, "Windows AI Space Quarantine Log")?;
    writeln!(file, "Generated At: {}", report.generated_at)?;
    writeln!(file, "Root: {}", report.root)?;
    writeln!(file, "Success Count: {}", report.success_count)?;
    writeln!(file, "Failure Count: {}", report.failure_count)?;
    writeln!(file)?;

    for result in &report.results {
        writeln!(
            file,
            "{} => {} | {} | {}",
            result.source_path, result.destination_path, result.status, result.message
        )?;
    }

    Ok(())
}

fn sanitize_path(path: &str) -> String {
    path.replace(':', "").replace('\\', "__").replace('/', "__")
}

fn classify_execution_error(error: &anyhow::Error) -> (String, String) {
    let message = error.to_string();
    let lowered = message.to_ascii_lowercase();

    if lowered.contains("recently modified") {
        return ("skipped-active".to_string(), message);
    }

    if let Some(io_error) = error.downcast_ref::<std::io::Error>() {
        if io_error.kind() == std::io::ErrorKind::PermissionDenied {
            return (
                "skipped-locked".to_string(),
                "permission denied or path may be locked by another process".to_string(),
            );
        }
    }

    ("failed".to_string(), message)
}

fn was_modified_recently(path: &Path, within_minutes: u64) -> bool {
    if within_minutes == 0 {
        return false;
    }

    let latest = latest_modified_time(path);
    let Some(latest) = latest else {
        return false;
    };

    let modified_at_utc: DateTime<Utc> = latest.into();
    let threshold = Utc::now() - Duration::minutes(within_minutes as i64);
    modified_at_utc >= threshold
}

fn latest_modified_time(path: &Path) -> Option<SystemTime> {
    let metadata = fs::metadata(path).ok()?;
    if metadata.is_file() {
        return metadata.modified().ok();
    }

    let mut latest = metadata.modified().ok();
    for entry in walkdir::WalkDir::new(path).follow_links(false) {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        let metadata = match entry.metadata() {
            Ok(metadata) => metadata,
            Err(_) => continue,
        };
        let modified = match metadata.modified() {
            Ok(modified) => modified,
            Err(_) => continue,
        };

        latest = match latest {
            Some(current) if current >= modified => Some(current),
            _ => Some(modified),
        };
    }

    latest
}

#[cfg(test)]
mod tests {
    use chrono::Local;
    use std::path::Path;
    use std::thread;
    use std::time::Duration as StdDuration;

    use tempfile::tempdir;

    use super::{
        build_dry_run, build_quarantine_plan, execute_quarantine, restore_from_index,
        QuarantineEntry, QuarantinePlan,
    };
    use crate::planner::{ActionGroup, PlanCandidate, PlanReport, PlanSummary, SkippedItem};
    use crate::rules::RiskLevel;

    fn sample_plan() -> PlanReport {
        PlanReport {
            generated_at: Local::now(),
            mode: "dry-run".to_string(),
            safe_only: true,
            skip_modified_within_minutes: 30,
            summary: PlanSummary {
                total_findings: 2,
                eligible_candidates: 1,
                skipped_findings: 1,
                reclaimable_bytes: 100,
                blocked_sensitive_paths: 0,
                skipped_recently_modified: 0,
            },
            groups: vec![ActionGroup {
                action: "quarantine".to_string(),
                candidate_count: 1,
                total_bytes: 100,
            }],
            candidates: vec![PlanCandidate {
                id: "safe-cache".to_string(),
                path: "C:\\temp\\cache".to_string(),
                risk: RiskLevel::Safe,
                size_bytes: 100,
                action: "quarantine".to_string(),
                reason: "safe".to_string(),
            }],
            skipped: vec![SkippedItem {
                id: "skip-me".to_string(),
                path: "C:\\skip".to_string(),
                reason: "path does not exist".to_string(),
            }],
        }
    }

    #[test]
    fn dry_run_inherits_groups_and_skipped() {
        let report = build_dry_run(&sample_plan());
        assert_eq!(report.groups.len(), 1);
        assert_eq!(report.skipped.len(), 1);
        assert_eq!(report.actions.len(), 1);
    }

    #[test]
    fn quarantine_plan_builds_destination_paths() {
        let plan = sample_plan();
        let quarantine = build_quarantine_plan(&plan, "F:\\archives");
        assert_eq!(quarantine.entries.len(), 1);
        assert!(quarantine.entries[0].destination_path.contains("F:\\archives"));
        assert!(quarantine.entries[0].destination_path.contains("C__temp__cache"));
    }

    #[test]
    fn execute_quarantine_moves_source_into_destination() {
        let temp = tempdir().expect("tempdir should exist");
        let source = temp.path().join("cache-dir");
        let destination_root = temp.path().join("archives");
        std::fs::create_dir_all(&source).expect("source dir should be created");
        std::fs::write(source.join("file.txt"), b"demo").expect("source file should be written");

        let plan = QuarantinePlan {
            root: destination_root.display().to_string(),
            skip_modified_within_minutes: 0,
            entries: vec![QuarantineEntry {
                source_path: source.display().to_string(),
                destination_path: destination_root.join("cache-dir").display().to_string(),
            }],
        };

        let report = execute_quarantine(&plan).expect("execution should succeed");
        assert_eq!(report.success_count, 1);
        assert_eq!(report.failure_count, 0);
        assert!(!source.exists());
        assert!(destination_root.join("cache-dir").exists());
        assert!(Path::new(&report.index_path).exists());
        assert!(Path::new(&report.log_path).exists());
    }

    #[test]
    fn execute_quarantine_skips_recently_modified_sources() {
        let temp = tempdir().expect("tempdir should exist");
        let source = temp.path().join("active-dir");
        let destination_root = temp.path().join("archives");
        std::fs::create_dir_all(&source).expect("source dir should be created");
        std::fs::write(source.join("file.txt"), b"demo").expect("source file should be written");

        let plan = QuarantinePlan {
            root: destination_root.display().to_string(),
            skip_modified_within_minutes: 60,
            entries: vec![QuarantineEntry {
                source_path: source.display().to_string(),
                destination_path: destination_root.join("active-dir").display().to_string(),
            }],
        };

        let report = execute_quarantine(&plan).expect("execution should finish with report");
        assert_eq!(report.success_count, 0);
        assert_eq!(report.failure_count, 1);
        assert_eq!(report.results[0].status, "skipped-active");
        assert!(source.exists());
    }

    #[test]
    fn execute_quarantine_allows_older_sources() {
        let temp = tempdir().expect("tempdir should exist");
        let source = temp.path().join("older-dir");
        let destination_root = temp.path().join("archives");
        std::fs::create_dir_all(&source).expect("source dir should be created");
        std::fs::write(source.join("file.txt"), b"demo").expect("source file should be written");
        thread::sleep(StdDuration::from_millis(1100));

        let plan = QuarantinePlan {
            root: destination_root.display().to_string(),
            skip_modified_within_minutes: 0,
            entries: vec![QuarantineEntry {
                source_path: source.display().to_string(),
                destination_path: destination_root.join("older-dir").display().to_string(),
            }],
        };

        let report = execute_quarantine(&plan).expect("execution should succeed");
        assert_eq!(report.success_count, 1);
        assert_eq!(report.results[0].status, "moved");
    }

    #[test]
    fn restore_from_index_supports_dry_run() {
        let temp = tempdir().expect("tempdir should exist");
        let source = temp.path().join("restore-dir");
        let destination_root = temp.path().join("archives");
        std::fs::create_dir_all(&source).expect("source dir should be created");
        std::fs::write(source.join("file.txt"), b"demo").expect("source file should be written");

        let plan = QuarantinePlan {
            root: destination_root.display().to_string(),
            skip_modified_within_minutes: 0,
            entries: vec![QuarantineEntry {
                source_path: source.display().to_string(),
                destination_path: destination_root.join("restore-dir").display().to_string(),
            }],
        };

        let execution = execute_quarantine(&plan).expect("execution should succeed");
        let restore = restore_from_index(Path::new(&execution.index_path), true)
            .expect("restore dry-run should succeed");

        assert_eq!(restore.entry_count, 1);
        assert_eq!(restore.success_count, 0);
        assert_eq!(restore.results[0].status, "planned");
    }
}
