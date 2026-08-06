use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
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
pub struct CleanDryRunOutput {
    #[serde(flatten)]
    pub clean: CleanReport,
    pub quarantine_plan: Option<QuarantinePlan>,
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
    #[serde(default = "default_execution_schema_version")]
    pub schema_version: u16,
    pub generated_at: DateTime<Local>,
    pub mode: String,
    pub root: String,
    pub success_count: usize,
    pub failure_count: usize,
    pub index_path: String,
    pub log_path: String,
    #[serde(default)]
    pub journal_path: String,
    pub results: Vec<ExecutionResult>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub source_path: String,
    pub destination_path: String,
    pub status: String,
    #[serde(default = "default_execution_stage")]
    pub stage: String,
    #[serde(default)]
    pub recovery: String,
    pub message: String,
}

fn default_execution_schema_version() -> u16 {
    1
}

fn default_execution_stage() -> String {
    "unknown".to_string()
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
    pub journal_path: String,
    pub results: Vec<RestoreResult>,
}

#[derive(Debug, Serialize)]
pub struct RestoreResult {
    pub source_path: String,
    pub destination_path: String,
    pub status: String,
    pub stage: String,
    pub recovery: String,
    pub message: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FailureInjection {
    None,
    PartialCopy,
    Verification,
    SourceRemove,
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
            destination_path: Path::new(root)
                .join(sanitize_path(&candidate.path))
                .display()
                .to_string(),
        })
        .collect();

    QuarantinePlan {
        root: root.to_string(),
        skip_modified_within_minutes: plan.skip_modified_within_minutes,
        entries,
    }
}

pub fn execute_quarantine(plan: &QuarantinePlan) -> Result<ExecutionReport> {
    execute_quarantine_with_injection(plan, FailureInjection::None)
}

fn execute_quarantine_with_injection(
    plan: &QuarantinePlan,
    injection: FailureInjection,
) -> Result<ExecutionReport> {
    for entry in &plan.entries {
        if Path::new(&entry.source_path).exists() {
            validate_quarantine_paths(
                Path::new(&entry.source_path),
                Path::new(&entry.destination_path),
                Path::new(&plan.root),
            )?;
        }
    }

    let metadata_dir = Path::new(&plan.root).join(".aidisk");
    fs::create_dir_all(&metadata_dir)?;

    let timestamp = Local::now().format("%Y%m%d-%H%M%S").to_string();
    let index_path = metadata_dir.join(format!("quarantine-index-{timestamp}.json"));
    let log_path = metadata_dir.join(format!("quarantine-log-{timestamp}.log"));
    let journal_path = metadata_dir.join(format!("quarantine-journal-{timestamp}.log"));
    let mut journal = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&journal_path)?;
    writeln!(journal, "schema_version=2")?;
    writeln!(journal, "generated_at={}", Local::now())?;
    writeln!(journal, "root={}", plan.root)?;

    let mut results = Vec::new();
    let mut success_count = 0_usize;
    let mut failure_count = 0_usize;

    for entry in &plan.entries {
        write_journal_stage(&mut journal, entry, "planned", "action planned")?;
        match move_to_quarantine(
            entry,
            &plan.root,
            plan.skip_modified_within_minutes,
            &mut journal,
            injection,
        ) {
            Ok(message) => {
                success_count += 1;
                write_journal_stage(&mut journal, entry, "quarantined", &message)?;
                results.push(ExecutionResult {
                    source_path: entry.source_path.clone(),
                    destination_path: entry.destination_path.clone(),
                    status: "quarantined".to_string(),
                    stage: "quarantined".to_string(),
                    recovery: "source removed; destination is the quarantine copy".to_string(),
                    message,
                });
            }
            Err(error) => {
                failure_count += 1;
                let (status, message) = classify_execution_error(&error);
                let stage = execution_stage(&status, &message);
                let recovery = execution_recovery(&status);
                write_journal_stage(&mut journal, entry, &stage, &message)?;
                results.push(ExecutionResult {
                    source_path: entry.source_path.clone(),
                    destination_path: entry.destination_path.clone(),
                    status,
                    stage,
                    recovery,
                    message,
                });
            }
        }
    }

    let report = ExecutionReport {
        schema_version: 2,
        generated_at: Local::now(),
        mode: "quarantine".to_string(),
        root: plan.root.clone(),
        success_count,
        failure_count,
        index_path: index_path.display().to_string(),
        log_path: log_path.display().to_string(),
        journal_path: journal_path.display().to_string(),
        results,
    };

    write_execution_index(&index_path, &report)?;
    write_execution_log(&log_path, &report)?;

    Ok(report)
}

fn write_journal_stage(
    journal: &mut fs::File,
    entry: &QuarantineEntry,
    stage: &str,
    message: &str,
) -> Result<()> {
    writeln!(
        journal,
        "{} | {} => {} | {} | {}",
        Local::now(),
        entry.source_path,
        entry.destination_path,
        stage,
        message.replace('\n', " ")
    )?;
    journal.flush()?;
    Ok(())
}

pub fn restore_from_index(index_path: &Path, dry_run: bool) -> Result<RestoreReport> {
    restore_from_index_with_injection(index_path, dry_run, FailureInjection::None)
}

fn restore_from_index_with_injection(
    index_path: &Path,
    dry_run: bool,
    injection: FailureInjection,
) -> Result<RestoreReport> {
    let content = fs::read_to_string(index_path).map_err(|e| {
        anyhow::anyhow!("failed to read index file {}: {}", index_path.display(), e)
    })?;
    let execution_report: ExecutionReport = serde_json::from_str(&content)
        .map_err(|e| anyhow::anyhow!("failed to parse index file: {}", e))?;

    validate_index(&execution_report)?;
    let mut journal = if execution_report.journal_path.is_empty() {
        None
    } else {
        Some(
            fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&execution_report.journal_path)?,
        )
    };
    let mut results = Vec::new();
    let mut success_count = 0_usize;
    let mut failure_count = 0_usize;

    for entry in execution_report
        .results
        .iter()
        .filter(|result| result.status == "quarantined" || result.status == "moved")
    {
        let journal_entry = QuarantineEntry {
            source_path: entry.destination_path.clone(),
            destination_path: entry.source_path.clone(),
        };
        if dry_run {
            if let Some(journal) = journal.as_mut() {
                write_journal_stage(
                    journal,
                    &journal_entry,
                    "restore-planned",
                    "restore dry-run",
                )?;
            }
            results.push(RestoreResult {
                source_path: entry.destination_path.clone(),
                destination_path: entry.source_path.clone(),
                status: "planned".to_string(),
                stage: "restore-planned".to_string(),
                recovery: "quarantine copy remains available for a later restore".to_string(),
                message: "restore dry-run only".to_string(),
            });
            continue;
        }

        if let Some(journal) = journal.as_mut() {
            write_journal_stage(
                journal,
                &journal_entry,
                "restore-planned",
                "restore action planned",
            )?;
        }
        match restore_entry(&journal_entry, journal.as_mut(), injection) {
            Ok(message) => {
                success_count += 1;
                if let Some(journal) = journal.as_mut() {
                    write_journal_stage(journal, &journal_entry, "restored", &message)?;
                }
                results.push(RestoreResult {
                    source_path: entry.destination_path.clone(),
                    destination_path: entry.source_path.clone(),
                    status: "restored".to_string(),
                    stage: "restored".to_string(),
                    recovery: "quarantine copy removed; original path restored".to_string(),
                    message,
                });
            }
            Err(error) => {
                failure_count += 1;
                let (status, message) = classify_restore_error(&error);
                let stage = restore_stage(&status, &message);
                let recovery = restore_recovery(&status);
                if let Some(journal) = journal.as_mut() {
                    write_journal_stage(journal, &journal_entry, &stage, &message)?;
                }
                results.push(RestoreResult {
                    source_path: entry.destination_path.clone(),
                    destination_path: entry.source_path.clone(),
                    status,
                    stage,
                    recovery,
                    message,
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
        journal_path: execution_report.journal_path,
        results,
    })
}

fn validate_index(report: &ExecutionReport) -> Result<()> {
    if report.root.is_empty() {
        anyhow::bail!("index is invalid: root path is empty");
    }
    if report.results.is_empty() {
        anyhow::bail!("index is invalid: no results found");
    }

    let allowed_statuses = [
        "quarantined",
        "moved",
        "failed",
        "partial-copy",
        "verification-failed",
        "source-remove-failed",
        "skipped-active",
        "skipped-locked",
    ];
    for result in &report.results {
        if !allowed_statuses.contains(&result.status.as_str()) {
            anyhow::bail!(
                "index is invalid: unknown status '{}' for path '{}'",
                result.status,
                result.source_path
            );
        }
        if result.source_path.is_empty() || result.destination_path.is_empty() {
            anyhow::bail!("index is invalid: empty source or destination path");
        }
    }

    Ok(())
}

fn move_to_quarantine(
    entry: &QuarantineEntry,
    quarantine_root: &str,
    skip_modified_within_minutes: u64,
    journal: &mut fs::File,
    injection: FailureInjection,
) -> Result<String> {
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
    if destination.exists() {
        anyhow::bail!("destination already exists");
    }
    let parent = destination
        .parent()
        .ok_or_else(|| anyhow::anyhow!("destination parent is missing"))?;
    validate_quarantine_paths(source, destination, Path::new(quarantine_root))?;
    fs::create_dir_all(parent)?;

    if injection != FailureInjection::None {
        write_journal_stage(
            journal,
            entry,
            "copying",
            "failure injection forced copy fallback",
        )?;
        let mut optional_journal = Some(journal);
        copy_verify_remove(source, destination, &mut optional_journal, entry, injection)?;
        return Ok("moved to quarantine with copy-verify-remove".to_string());
    }

    write_journal_stage(journal, entry, "renaming", "trying atomic rename")?;
    match fs::rename(source, destination) {
        Ok(()) => Ok("moved to quarantine with rename".to_string()),
        Err(rename_error) => {
            write_journal_stage(
                journal,
                entry,
                "copying",
                &format!("rename failed ({rename_error}); using copy fallback"),
            )?;
            let mut optional_journal = Some(journal);
            copy_verify_remove(
                source,
                destination,
                &mut optional_journal,
                entry,
                FailureInjection::None,
            )
            .map_err(|error| {
                anyhow::anyhow!(
                    "rename failed ({rename_error}); copy-verify-remove failed ({error})"
                )
            })?;
            Ok("moved to quarantine with copy-verify-remove".to_string())
        }
    }
}

fn validate_quarantine_paths(source: &Path, destination: &Path, root: &Path) -> Result<()> {
    let source_canonical = source.canonicalize()?;
    let root_canonical = canonicalize_with_missing(root)?;
    let destination_parent = canonicalize_with_missing(
        destination
            .parent()
            .ok_or_else(|| anyhow::anyhow!("destination parent is missing"))?,
    )?;

    if !destination_parent.starts_with(&root_canonical) {
        anyhow::bail!("destination is outside quarantine root");
    }
    if source_canonical.starts_with(&root_canonical) {
        anyhow::bail!("source path is already inside quarantine root");
    }
    if root_canonical.starts_with(&source_canonical)
        || destination_parent.starts_with(&source_canonical)
    {
        anyhow::bail!("quarantine destination cannot be nested inside source path");
    }

    Ok(())
}

fn canonicalize_with_missing(path: &Path) -> Result<PathBuf> {
    let mut missing = Vec::new();
    let mut current = path;
    while !current.exists() {
        let name = current
            .file_name()
            .ok_or_else(|| anyhow::anyhow!("path has no existing ancestor: {}", path.display()))?;
        missing.push(name.to_os_string());
        current = current
            .parent()
            .ok_or_else(|| anyhow::anyhow!("path has no existing ancestor: {}", path.display()))?;
    }

    let mut canonical = current.canonicalize()?;
    for component in missing.iter().rev() {
        canonical.push(component);
    }
    Ok(canonical)
}

fn copy_recursive(src: &Path, dst: &Path) -> Result<()> {
    if src.is_dir() {
        fs::create_dir_all(dst)?;
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let src_child = entry.path();
            let dst_child = dst.join(entry.file_name());
            copy_recursive(&src_child, &dst_child)?;
        }
    } else {
        fs::copy(src, dst)?;
    }
    Ok(())
}

#[derive(Debug, Default, PartialEq, Eq)]
struct PathStats {
    files: u64,
    directories: u64,
    bytes: u64,
}

fn collect_path_stats(path: &Path) -> Result<PathStats> {
    let metadata = fs::metadata(path)?;
    if metadata.is_file() {
        return Ok(PathStats {
            files: 1,
            directories: 0,
            bytes: metadata.len(),
        });
    }

    let mut stats = PathStats::default();
    for entry in walkdir::WalkDir::new(path).follow_links(false) {
        let entry = entry?;
        let metadata = entry.metadata()?;
        if metadata.is_dir() {
            stats.directories = stats.directories.saturating_add(1);
        } else if metadata.is_file() {
            stats.files = stats.files.saturating_add(1);
            stats.bytes = stats.bytes.saturating_add(metadata.len());
        }
    }
    Ok(stats)
}

fn copy_verify_remove(
    source: &Path,
    destination: &Path,
    journal: &mut Option<&mut fs::File>,
    entry: &QuarantineEntry,
    injection: FailureInjection,
) -> Result<()> {
    let source_stats = collect_path_stats(source)?;
    if injection == FailureInjection::PartialCopy {
        write_optional_journal_stage(journal, entry, "copying", "failure injected before copy")?;
        anyhow::bail!("partial copy failed: injected copy interruption");
    }
    if let Err(error) = copy_recursive(source, destination) {
        let _ = remove_recursive(destination);
        anyhow::bail!("partial copy failed: {error}");
    }
    write_optional_journal_stage(journal, entry, "copied", "copy completed")?;
    if injection == FailureInjection::Verification {
        let _ = remove_recursive(destination);
        anyhow::bail!("verification failed: injected verification failure");
    }
    let destination_stats = collect_path_stats(destination)?;
    if source_stats != destination_stats {
        let _ = remove_recursive(destination);
        anyhow::bail!(
            "verification failed: source stats {:?} != destination stats {:?}",
            source_stats,
            destination_stats
        );
    }
    write_optional_journal_stage(
        journal,
        entry,
        "verified",
        "file, directory, and byte counts match",
    )?;
    if injection == FailureInjection::SourceRemove {
        anyhow::bail!("source remove failed after verified copy: injected source removal failure");
    }
    write_optional_journal_stage(
        journal,
        entry,
        "source-removing",
        "removing original source",
    )?;
    remove_recursive(source)
        .map_err(|error| anyhow::anyhow!("source remove failed after verified copy: {error}"))?;
    Ok(())
}

fn write_optional_journal_stage(
    journal: &mut Option<&mut fs::File>,
    entry: &QuarantineEntry,
    stage: &str,
    message: &str,
) -> Result<()> {
    if let Some(journal) = journal.as_deref_mut() {
        write_journal_stage(journal, entry, stage, message)?;
    }
    Ok(())
}

fn remove_recursive(path: &Path) -> Result<()> {
    if path.is_dir() {
        fs::remove_dir_all(path)?;
    } else {
        fs::remove_file(path)?;
    }
    Ok(())
}

fn restore_entry(
    entry: &QuarantineEntry,
    mut journal: Option<&mut fs::File>,
    injection: FailureInjection,
) -> Result<String> {
    let source = Path::new(&entry.source_path);
    if !source.exists() {
        anyhow::bail!("quarantined source path does not exist");
    }

    let destination = Path::new(&entry.destination_path);
    if destination.exists() {
        anyhow::bail!("restore destination already exists");
    }

    let parent = destination
        .parent()
        .ok_or_else(|| anyhow::anyhow!("restore destination parent is missing"))?;
    fs::create_dir_all(parent)?;
    write_optional_journal_stage(
        &mut journal,
        entry,
        "restoring",
        "trying atomic restore rename",
    )?;

    if injection != FailureInjection::None {
        copy_verify_remove(source, destination, &mut journal, entry, injection)?;
        return Ok("restored from quarantine with copy-verify-remove".to_string());
    }

    match fs::rename(source, destination) {
        Ok(()) => Ok("restored from quarantine with rename".to_string()),
        Err(rename_error) => {
            write_optional_journal_stage(
                &mut journal,
                entry,
                "copying",
                &format!("restore rename failed ({rename_error}); using copy-back fallback"),
            )?;
            copy_verify_remove(
                source,
                destination,
                &mut journal,
                entry,
                FailureInjection::None,
            )
            .map_err(|error| {
                anyhow::anyhow!(
                    "restore rename failed ({rename_error}); copy-back failed ({error})"
                )
            })?;
            Ok("restored from quarantine with copy-verify-remove".to_string())
        }
    }
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
            "{} => {} | {} | {} | {} | {}",
            result.source_path,
            result.destination_path,
            result.status,
            result.stage,
            result.recovery,
            result.message
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

    if lowered.contains("partial copy failed") {
        return ("partial-copy".to_string(), message);
    }

    if lowered.contains("verification failed") {
        return ("verification-failed".to_string(), message);
    }

    if lowered.contains("source remove failed") {
        return ("source-remove-failed".to_string(), message);
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

fn execution_stage(status: &str, message: &str) -> String {
    match status {
        "skipped-active" => "preflight".to_string(),
        "skipped-locked" => "preflight".to_string(),
        "partial-copy" => "copying".to_string(),
        "verification-failed" => "verifying".to_string(),
        "source-remove-failed" => "source-removing".to_string(),
        _ if message.contains("destination already exists") => "preflight".to_string(),
        _ if message.contains("source path does not exist") => "preflight".to_string(),
        _ => "failed".to_string(),
    }
}

fn execution_recovery(status: &str) -> String {
    match status {
        "skipped-active" => "source was left untouched; rerun after the active window expires".to_string(),
        "skipped-locked" => "source was left untouched; close locking processes and retry".to_string(),
        "partial-copy" => "source should remain in place; incomplete destination copy was removed when possible".to_string(),
        "verification-failed" => "source should remain in place; failed destination copy was removed when possible".to_string(),
        "source-remove-failed" => "verified destination copy exists; manually review both source and quarantine copy before retrying".to_string(),
        _ => "review source and destination paths before retrying".to_string(),
    }
}

fn classify_restore_error(error: &anyhow::Error) -> (String, String) {
    let message = error.to_string();
    let lowered = message.to_ascii_lowercase();

    if lowered.contains("already exists") {
        return (
            "skipped-conflict".to_string(),
            "restore destination already exists; existing path was left untouched".to_string(),
        );
    }

    if let Some(io_error) = error.downcast_ref::<std::io::Error>() {
        if io_error.kind() == std::io::ErrorKind::PermissionDenied {
            return (
                "skipped-locked".to_string(),
                "permission denied or restore destination may be locked by another process"
                    .to_string(),
            );
        }
    }

    ("failed".to_string(), message)
}

fn restore_stage(status: &str, message: &str) -> String {
    match status {
        "skipped-conflict" => "restore-preflight".to_string(),
        "skipped-locked" => "restore-preflight".to_string(),
        _ if message.contains("copy-back failed") => "restore-copying".to_string(),
        _ => "restore-failed".to_string(),
    }
}

fn restore_recovery(status: &str) -> String {
    match status {
        "skipped-conflict" => {
            "quarantine copy was left untouched; move or remove destination before retrying"
                .to_string()
        }
        "skipped-locked" => {
            "quarantine copy was left untouched; close locking processes and retry".to_string()
        }
        _ => "review quarantine copy and destination path before retrying restore".to_string(),
    }
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
        build_dry_run, build_quarantine_plan, classify_execution_error, classify_restore_error,
        execute_quarantine, execute_quarantine_with_injection, restore_from_index,
        FailureInjection, QuarantineEntry, QuarantinePlan,
    };
    use crate::planner::{ActionGroup, PlanCandidate, PlanReport, PlanSummary, SkippedItem};
    use crate::rules::RiskLevel;

    fn sample_plan() -> PlanReport {
        PlanReport {
            generated_at: Local::now(),
            mode: "dry-run".to_string(),
            safe_only: true,
            skip_modified_within_minutes: 30,
            policy: None,
            summary: PlanSummary {
                total_findings: 2,
                eligible_candidates: 1,
                skipped_findings: 1,
                reclaimable_bytes: 100,
                actionable_bytes: 100,
                quarantine_bytes: 100,
                blocked_sensitive_paths: 0,
                skipped_recently_modified: 0,
                ..Default::default()
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
                partial: false,
                partial_reasons: Vec::new(),
                action: "quarantine".to_string(),
                reason: "safe".to_string(),
            }],
            skipped: vec![SkippedItem {
                id: "skip-me".to_string(),
                path: "C:\\skip".to_string(),
                reason: "path does not exist".to_string(),
                partial: false,
                partial_reasons: Vec::new(),
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
        assert!(quarantine.entries[0]
            .destination_path
            .contains("F:\\archives"));
        assert!(quarantine.entries[0]
            .destination_path
            .contains("C__temp__cache"));
    }

    #[test]
    fn quarantine_plan_uses_platform_join_for_destination_paths() {
        let plan = sample_plan();
        let temp = tempdir().expect("tempdir should exist");
        let quarantine = build_quarantine_plan(&plan, &temp.path().display().to_string());

        let expected = temp.path().join("C__temp__cache").display().to_string();
        assert_eq!(quarantine.entries[0].destination_path, expected);
    }

    #[test]
    fn execute_quarantine_blocks_sources_inside_quarantine_root() {
        let temp = tempdir().expect("tempdir should exist");
        let destination_root = temp.path().join("archives");
        let source = destination_root.join("already-quarantined");
        std::fs::create_dir_all(&source).expect("source dir should be created");
        std::fs::write(source.join("file.txt"), b"demo").expect("source file should be written");

        let plan = QuarantinePlan {
            root: destination_root.display().to_string(),
            skip_modified_within_minutes: 0,
            entries: vec![QuarantineEntry {
                source_path: source.display().to_string(),
                destination_path: destination_root.join("dest").display().to_string(),
            }],
        };

        let error = execute_quarantine(&plan).expect_err("preflight should reject nested source");

        assert!(error
            .to_string()
            .contains("source path is already inside quarantine root"));
        assert!(source.exists());
        assert!(!destination_root.join(".aidisk").exists());
    }

    #[test]
    fn execute_quarantine_blocks_destination_nested_inside_source() {
        let temp = tempdir().expect("tempdir should exist");
        let source = temp.path().join("source-root");
        let destination_root = source.join("archives");
        std::fs::create_dir_all(&source).expect("source dir should be created");
        std::fs::write(source.join("file.txt"), b"demo").expect("source file should be written");

        let plan = QuarantinePlan {
            root: destination_root.display().to_string(),
            skip_modified_within_minutes: 0,
            entries: vec![QuarantineEntry {
                source_path: source.display().to_string(),
                destination_path: destination_root.join("dest").display().to_string(),
            }],
        };

        let error =
            execute_quarantine(&plan).expect_err("preflight should reject destination loop");

        assert!(error
            .to_string()
            .contains("quarantine destination cannot be nested inside source path"));
        assert!(source.exists());
        assert!(!destination_root.exists());
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
        assert_eq!(report.schema_version, 2);
        assert!(!source.exists());
        assert!(destination_root.join("cache-dir").exists());
        assert_eq!(
            report.results[0].recovery,
            "source removed; destination is the quarantine copy"
        );
        assert!(Path::new(&report.index_path).exists());
        assert!(Path::new(&report.log_path).exists());
        assert!(Path::new(&report.journal_path).exists());
        let journal = std::fs::read_to_string(&report.journal_path).expect("journal should read");
        assert!(journal.contains("planned"));
        assert!(journal.contains("renaming"));
        assert!(journal.contains("quarantined"));
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
        assert_eq!(report.results[0].status, "quarantined");
        assert_eq!(report.results[0].stage, "quarantined");
    }

    fn injected_plan(
        kind: FailureInjection,
    ) -> (
        tempfile::TempDir,
        QuarantinePlan,
        std::path::PathBuf,
        std::path::PathBuf,
    ) {
        let temp = tempdir().expect("tempdir should exist");
        let source = temp.path().join("copy-source");
        let destination_root = temp.path().join("archives");
        std::fs::create_dir_all(&source).expect("source dir should be created");
        std::fs::write(source.join("file.txt"), b"demo").expect("source file should be written");
        let destination = destination_root.join(format!("copy-source-{kind:?}"));
        let plan = QuarantinePlan {
            root: destination_root.display().to_string(),
            skip_modified_within_minutes: 0,
            entries: vec![QuarantineEntry {
                source_path: source.display().to_string(),
                destination_path: destination.display().to_string(),
            }],
        };
        (temp, plan, source, destination)
    }

    #[test]
    fn execute_quarantine_records_copy_verify_remove_stages() {
        let (_temp, plan, source, destination) = injected_plan(FailureInjection::PartialCopy);

        let report = execute_quarantine_with_injection(&plan, FailureInjection::SourceRemove)
            .expect("execution should finish with source remove failure report");

        assert_eq!(report.success_count, 0);
        assert_eq!(report.failure_count, 1);
        assert_eq!(report.results[0].status, "source-remove-failed");
        assert_eq!(report.results[0].stage, "source-removing");
        assert!(report.results[0]
            .recovery
            .contains("verified destination copy exists"));
        assert!(source.exists());
        assert!(destination.exists());

        let journal = std::fs::read_to_string(&report.journal_path).expect("journal should read");
        assert!(journal.contains("copying"));
        assert!(journal.contains("copied"));
        assert!(journal.contains("verified"));
        assert!(journal.contains("source-removing"));
    }

    #[test]
    fn execute_quarantine_recovers_from_partial_copy_failure() {
        let (_temp, plan, source, destination) = injected_plan(FailureInjection::PartialCopy);

        let report = execute_quarantine_with_injection(&plan, FailureInjection::PartialCopy)
            .expect("execution should finish with partial copy report");

        assert_eq!(report.results[0].status, "partial-copy");
        assert_eq!(report.results[0].stage, "copying");
        assert!(report.results[0]
            .recovery
            .contains("source should remain in place"));
        assert!(source.exists());
        assert!(!destination.exists());
    }

    #[test]
    fn execute_quarantine_recovers_from_verification_failure() {
        let (_temp, plan, source, destination) = injected_plan(FailureInjection::Verification);

        let report = execute_quarantine_with_injection(&plan, FailureInjection::Verification)
            .expect("execution should finish with verification failure report");

        assert_eq!(report.results[0].status, "verification-failed");
        assert_eq!(report.results[0].stage, "verifying");
        assert!(source.exists());
        assert!(!destination.exists());
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

    #[test]
    fn restore_from_index_executes_restore() {
        let temp = tempdir().expect("tempdir should exist");
        let source = temp.path().join("restore-live-dir");
        let destination_root = temp.path().join("archives");
        std::fs::create_dir_all(&source).expect("source dir should be created");
        std::fs::write(source.join("file.txt"), b"demo").expect("source file should be written");

        let plan = QuarantinePlan {
            root: destination_root.display().to_string(),
            skip_modified_within_minutes: 0,
            entries: vec![QuarantineEntry {
                source_path: source.display().to_string(),
                destination_path: destination_root
                    .join("restore-live-dir")
                    .display()
                    .to_string(),
            }],
        };

        let execution = execute_quarantine(&plan).expect("execution should succeed");
        let quarantined_path = destination_root.join("restore-live-dir");
        assert!(quarantined_path.exists());

        let restore = restore_from_index(Path::new(&execution.index_path), false)
            .expect("restore should succeed");

        assert_eq!(restore.entry_count, 1);
        assert_eq!(restore.success_count, 1);
        assert_eq!(restore.results[0].status, "restored");
        assert!(source.exists());
        assert!(!quarantined_path.exists());
    }

    #[test]
    fn permission_denied_is_classified_as_skipped_locked() {
        let error = anyhow::Error::new(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "access denied",
        ));

        let (status, message) = classify_execution_error(&error);

        assert_eq!(status, "skipped-locked");
        assert!(message.contains("locked") || message.contains("permission denied"));
    }

    #[test]
    fn restore_conflict_is_classified_as_skipped_conflict() {
        let error = anyhow::anyhow!("restore destination already exists");

        let (status, message) = classify_restore_error(&error);

        assert_eq!(status, "skipped-conflict");
        assert!(message.contains("left untouched"));
    }

    #[test]
    fn copy_failure_statuses_are_classified_separately() {
        let partial = anyhow::anyhow!("partial copy failed: disk full");
        let verification = anyhow::anyhow!("verification failed: stats mismatch");
        let remove = anyhow::anyhow!("source remove failed after verified copy: access denied");

        assert_eq!(classify_execution_error(&partial).0, "partial-copy");
        assert_eq!(
            classify_execution_error(&verification).0,
            "verification-failed"
        );
        assert_eq!(classify_execution_error(&remove).0, "source-remove-failed");
    }

    #[test]
    fn legacy_execution_index_defaults_schema_and_stage() {
        let json = r#"{
  "generated_at": "2026-06-01T00:00:00+00:00",
  "mode": "quarantine",
  "root": "C:\\archives",
  "success_count": 1,
  "failure_count": 0,
  "index_path": "C:\\archives\\.aidisk\\quarantine-index.json",
  "log_path": "C:\\archives\\.aidisk\\quarantine-log.log",
  "results": [
    {
      "source_path": "C:\\cache",
      "destination_path": "C:\\archives\\cache",
      "status": "moved",
      "message": "legacy moved"
    }
  ]
}"#;

        let report: super::ExecutionReport =
            serde_json::from_str(json).expect("legacy index should deserialize");

        assert_eq!(report.schema_version, 1);
        assert_eq!(report.journal_path, "");
        assert_eq!(report.results[0].stage, "unknown");
        super::validate_index(&report).expect("legacy moved status should remain valid");
    }

    #[test]
    fn restore_from_index_skips_existing_destination() {
        let temp = tempdir().expect("tempdir should exist");
        let source = temp.path().join("restore-conflict-dir");
        let destination_root = temp.path().join("archives");
        std::fs::create_dir_all(&source).expect("source dir should be created");
        std::fs::write(source.join("file.txt"), b"demo").expect("source file should be written");

        let plan = QuarantinePlan {
            root: destination_root.display().to_string(),
            skip_modified_within_minutes: 0,
            entries: vec![QuarantineEntry {
                source_path: source.display().to_string(),
                destination_path: destination_root
                    .join("restore-conflict-dir")
                    .display()
                    .to_string(),
            }],
        };

        let execution = execute_quarantine(&plan).expect("execution should succeed");
        std::fs::create_dir_all(&source).expect("conflicting destination should be recreated");

        let restore = restore_from_index(Path::new(&execution.index_path), false)
            .expect("restore should complete with conflict report");

        assert_eq!(restore.failure_count, 1);
        assert_eq!(restore.results[0].status, "skipped-conflict");
        assert!(source.exists());
        assert!(destination_root.join("restore-conflict-dir").exists());
    }
}
