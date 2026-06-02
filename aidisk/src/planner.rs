use std::fs;
use std::path::Path;
use std::time::SystemTime;

use chrono::{DateTime, Duration, Local, Utc};
use serde::Serialize;
use walkdir::WalkDir;

use crate::policy::Policy;
use crate::rules::RiskLevel;
use crate::scanner::{Finding, ScanReport};

#[derive(Debug, Serialize)]
pub struct PlanReport {
    pub generated_at: DateTime<Local>,
    pub mode: String,
    pub safe_only: bool,
    pub skip_modified_within_minutes: u64,
    pub summary: PlanSummary,
    pub groups: Vec<ActionGroup>,
    pub candidates: Vec<PlanCandidate>,
    pub skipped: Vec<SkippedItem>,
}

#[derive(Debug, Default, Serialize)]
pub struct PlanSummary {
    pub total_findings: usize,
    pub eligible_candidates: usize,
    pub skipped_findings: usize,
    pub reclaimable_bytes: u64,
    pub blocked_sensitive_paths: usize,
    pub skipped_recently_modified: usize,
}

#[derive(Debug, Serialize)]
pub struct PlanCandidate {
    pub id: String,
    pub path: String,
    pub risk: RiskLevel,
    pub size_bytes: u64,
    pub action: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ActionGroup {
    pub action: String,
    pub candidate_count: usize,
    pub total_bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SkippedItem {
    pub id: String,
    pub path: String,
    pub reason: String,
}

#[derive(Debug, Clone)]
pub struct PlanOptions {
    pub safe_only: bool,
    pub skip_modified_within_minutes: u64,
    pub policy: Policy,
}

pub fn build_plan(scan_report: &ScanReport, options: PlanOptions) -> PlanReport {
    let mut candidates = Vec::new();
    let mut skipped = Vec::new();
    let mut summary = PlanSummary {
        total_findings: scan_report.findings.len(),
        ..PlanSummary::default()
    };

    for finding in &scan_report.findings {
        let skip_reason = skip_reason(finding, &options);
        if let Some(reason) = skip_reason {
            summary.skipped_findings += 1;
            if reason.contains("sensitive") {
                summary.blocked_sensitive_paths += 1;
            }
            if reason.contains("recently modified") {
                summary.skipped_recently_modified += 1;
            }
            skipped.push(SkippedItem {
                id: finding.id.clone(),
                path: finding.path.clone(),
                reason,
            });
            continue;
        }

        summary.eligible_candidates += 1;
        summary.reclaimable_bytes = summary.reclaimable_bytes.saturating_add(finding.size_bytes);
        candidates.push(PlanCandidate {
            id: finding.id.clone(),
            path: finding.path.clone(),
            risk: finding.risk,
            size_bytes: finding.size_bytes,
            action: finding.action.clone(),
            reason: finding.reason.clone(),
        });
    }

    let groups = build_action_groups(&candidates);

    PlanReport {
        generated_at: Local::now(),
        mode: "dry-run".to_string(),
        safe_only: options.safe_only,
        skip_modified_within_minutes: options.skip_modified_within_minutes,
        summary,
        groups,
        candidates,
        skipped,
    }
}

fn skip_reason(finding: &Finding, options: &PlanOptions) -> Option<String> {
    if !finding.exists {
        return Some("path does not exist".to_string());
    }
    if finding.size_bytes == 0 {
        return Some("path has no reclaimable size".to_string());
    }
    if !options
        .policy
        .planner
        .allow_actions
        .iter()
        .any(|action| action == &finding.action)
    {
        return Some("action is not supported by planner".to_string());
    }
    if options.safe_only && finding.risk != RiskLevel::Safe {
        return Some("filtered out by safe-only mode".to_string());
    }
    if is_sensitive_path(&finding.path, &options.policy.sensitive_markers) {
        return Some("blocked because path looks sensitive".to_string());
    }
    if was_modified_recently(&finding.path, options.skip_modified_within_minutes) {
        return Some(format!(
            "skipped because path was recently modified within {} minutes",
            options.skip_modified_within_minutes
        ));
    }

    None
}

fn build_action_groups(candidates: &[PlanCandidate]) -> Vec<ActionGroup> {
    let mut groups: Vec<ActionGroup> = Vec::new();

    for candidate in candidates {
        if let Some(group) = groups.iter_mut().find(|group| group.action == candidate.action) {
            group.candidate_count += 1;
            group.total_bytes = group.total_bytes.saturating_add(candidate.size_bytes);
            continue;
        }

        groups.push(ActionGroup {
            action: candidate.action.clone(),
            candidate_count: 1,
            total_bytes: candidate.size_bytes,
        });
    }

    groups.sort_by(|a, b| b.total_bytes.cmp(&a.total_bytes).then_with(|| a.action.cmp(&b.action)));
    groups
}

fn is_sensitive_path(path: &str, sensitive_markers: &[String]) -> bool {
    let normalized = path.to_ascii_lowercase();
    sensitive_markers
        .iter()
        .any(|marker| normalized.contains(&marker.to_ascii_lowercase()))
}

fn was_modified_recently(path: &str, within_minutes: u64) -> bool {
    if within_minutes == 0 {
        return false;
    }

    let latest = latest_modified_time(Path::new(path));
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
    for entry in WalkDir::new(path).follow_links(false) {
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

    use super::{build_plan, is_sensitive_path, PlanOptions};
    use crate::policy::{PlannerPolicy, Policy};
    use crate::rules::RiskLevel;
    use crate::scanner::{Finding, ScanReport, Summary, Volume};

    #[test]
    fn safe_only_plan_keeps_only_safe_findings() {
        let report = ScanReport {
            scan_time: Local::now(),
            volumes: Vec::<Volume>::new(),
            findings: vec![
                Finding {
                    id: "safe-cache".to_string(),
                    name: "Safe cache".to_string(),
                    category: "browser-cache".to_string(),
                    path: "C:\\safe".to_string(),
                    exists: true,
                    size_bytes: 100,
                    risk: RiskLevel::Safe,
                    action: "quarantine".to_string(),
                    reason: "safe".to_string(),
                    warnings: Vec::new(),
                },
                Finding {
                    id: "review-cache".to_string(),
                    name: "Review cache".to_string(),
                    category: "ai-agent".to_string(),
                    path: "C:\\review".to_string(),
                    exists: true,
                    size_bytes: 200,
                    risk: RiskLevel::Review,
                    action: "report-only".to_string(),
                    reason: "review".to_string(),
                    warnings: Vec::new(),
                },
            ],
            summary: Summary::default(),
        };

        let plan = build_plan(
            &report,
            PlanOptions {
                safe_only: true,
                skip_modified_within_minutes: 0,
                policy: test_policy(),
            },
        );

        assert_eq!(plan.summary.eligible_candidates, 1);
        assert_eq!(plan.summary.skipped_findings, 1);
        assert_eq!(plan.summary.reclaimable_bytes, 100);
        assert_eq!(plan.candidates.len(), 1);
        assert_eq!(plan.candidates[0].id, "safe-cache");
    }

    #[test]
    fn non_safe_only_plan_keeps_existing_positive_size_candidates() {
        let report = ScanReport {
            scan_time: Local::now(),
            volumes: Vec::<Volume>::new(),
            findings: vec![
                Finding {
                    id: "system-guide".to_string(),
                    name: "System guide".to_string(),
                    category: "wsl".to_string(),
                    path: "C:\\system".to_string(),
                    exists: true,
                    size_bytes: 300,
                    risk: RiskLevel::System,
                    action: "guide".to_string(),
                    reason: "system".to_string(),
                    warnings: Vec::new(),
                },
                Finding {
                    id: "missing-safe".to_string(),
                    name: "Missing safe".to_string(),
                    category: "dev-cache".to_string(),
                    path: "C:\\missing".to_string(),
                    exists: false,
                    size_bytes: 0,
                    risk: RiskLevel::Safe,
                    action: "quarantine".to_string(),
                    reason: "missing".to_string(),
                    warnings: Vec::new(),
                },
            ],
            summary: Summary::default(),
        };

        let plan = build_plan(
            &report,
            PlanOptions {
                safe_only: false,
                skip_modified_within_minutes: 0,
                policy: test_policy(),
            },
        );

        assert_eq!(plan.summary.eligible_candidates, 1);
        assert_eq!(plan.summary.skipped_findings, 1);
        assert_eq!(plan.summary.reclaimable_bytes, 300);
        assert_eq!(plan.candidates[0].id, "system-guide");
    }

    #[test]
    fn blocks_sensitive_paths() {
        let markers = vec!["cookies".to_string(), "login data".to_string(), "auth.json".to_string()];
        assert!(is_sensitive_path("C:\\Users\\demo\\cookies", &markers));
        assert!(is_sensitive_path("C:\\Users\\demo\\Login Data", &markers));
        assert!(is_sensitive_path("C:\\Users\\demo\\auth.json", &markers));
        assert!(!is_sensitive_path("C:\\Users\\demo\\Cache", &markers));
    }

    fn test_policy() -> Policy {
        Policy {
            sensitive_markers: vec![
                "token".to_string(),
                "credential".to_string(),
                "secret".to_string(),
                ".env".to_string(),
                "cookies".to_string(),
                "login data".to_string(),
                "auth.json".to_string(),
            ],
            planner: PlannerPolicy {
                skip_modified_within_minutes: 30,
                allow_actions: vec![
                    "quarantine".to_string(),
                    "report-only".to_string(),
                    "guide".to_string(),
                ],
            },
        }
    }
}
