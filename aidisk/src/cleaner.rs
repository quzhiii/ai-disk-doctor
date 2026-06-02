use chrono::{DateTime, Local};
use serde::Serialize;

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
    pub entries: Vec<QuarantineEntry>,
}

#[derive(Debug, Serialize)]
pub struct QuarantineEntry {
    pub source_path: String,
    pub destination_path: String,
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
        entries,
    }
}

fn sanitize_path(path: &str) -> String {
    path.replace(':', "").replace('\\', "__").replace('/', "__")
}

#[cfg(test)]
mod tests {
    use chrono::Local;

    use super::{build_dry_run, build_quarantine_plan};
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
}
