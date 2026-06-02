use chrono::{DateTime, Local};
use serde::Serialize;

use crate::rules::RiskLevel;
use crate::scanner::{Finding, ScanReport};

#[derive(Debug, Serialize)]
pub struct PlanReport {
    pub generated_at: DateTime<Local>,
    pub mode: String,
    pub safe_only: bool,
    pub summary: PlanSummary,
    pub candidates: Vec<PlanCandidate>,
}

#[derive(Debug, Default, Serialize)]
pub struct PlanSummary {
    pub total_findings: usize,
    pub eligible_candidates: usize,
    pub skipped_findings: usize,
    pub reclaimable_bytes: u64,
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

pub fn build_plan(scan_report: &ScanReport, safe_only: bool) -> PlanReport {
    let mut candidates = Vec::new();
    let mut summary = PlanSummary {
        total_findings: scan_report.findings.len(),
        ..PlanSummary::default()
    };

    for finding in &scan_report.findings {
        if !is_eligible(finding, safe_only) {
            summary.skipped_findings += 1;
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

    PlanReport {
        generated_at: Local::now(),
        mode: "dry-run".to_string(),
        safe_only,
        summary,
        candidates,
    }
}

fn is_eligible(finding: &Finding, safe_only: bool) -> bool {
    finding.exists
        && finding.size_bytes > 0
        && matches!(finding.action.as_str(), "quarantine" | "report-only" | "guide")
        && (!safe_only || finding.risk == RiskLevel::Safe)
}

#[cfg(test)]
mod tests {
    use chrono::Local;

    use super::build_plan;
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

        let plan = build_plan(&report, true);

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

        let plan = build_plan(&report, false);

        assert_eq!(plan.summary.eligible_candidates, 1);
        assert_eq!(plan.summary.skipped_findings, 1);
        assert_eq!(plan.summary.reclaimable_bytes, 300);
        assert_eq!(plan.candidates[0].id, "system-guide");
    }
}
