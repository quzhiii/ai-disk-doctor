use chrono::{DateTime, Local};
use serde::Serialize;

use crate::diff::DiffReport;

#[derive(Clone, Copy, Debug, Serialize)]
pub struct AnomalyThresholds {
    pub min_growth_bytes: u64,
    pub min_growth_percent: f64,
}

#[derive(Debug, Serialize)]
pub struct AnomalyReport {
    pub generated_at: DateTime<Local>,
    pub before: String,
    pub after: String,
    pub thresholds: AnomalyThresholds,
    pub summary: AnomalySummary,
    pub anomalies: Vec<GrowthAnomaly>,
}

#[derive(Debug, Default, Serialize)]
pub struct AnomalySummary {
    pub anomalies: usize,
    pub total_growth_bytes: u64,
}

#[derive(Debug, Serialize)]
pub struct GrowthAnomaly {
    pub path: String,
    pub before_bytes: u64,
    pub after_bytes: u64,
    pub delta_bytes: u64,
    pub growth_percent: Option<f64>,
}

pub fn build_anomaly_report(
    diff_report: &DiffReport,
    thresholds: AnomalyThresholds,
) -> AnomalyReport {
    let mut anomalies = diff_report
        .changes
        .iter()
        .filter_map(|change| {
            let delta_bytes = u64::try_from(change.delta_bytes).ok()?;
            if delta_bytes < thresholds.min_growth_bytes {
                return None;
            }

            let growth_percent = if change.before_bytes == 0 {
                None
            } else {
                Some((delta_bytes as f64 / change.before_bytes as f64) * 100.0)
            };

            if growth_percent
                .map(|percent| percent < thresholds.min_growth_percent)
                .unwrap_or(false)
            {
                return None;
            }

            Some(GrowthAnomaly {
                path: change.path.clone(),
                before_bytes: change.before_bytes,
                after_bytes: change.after_bytes,
                delta_bytes,
                growth_percent,
            })
        })
        .collect::<Vec<_>>();

    anomalies.sort_by(|a, b| b.delta_bytes.cmp(&a.delta_bytes));
    let total_growth_bytes = anomalies.iter().map(|anomaly| anomaly.delta_bytes).sum();

    AnomalyReport {
        generated_at: Local::now(),
        before: diff_report.before.clone(),
        after: diff_report.after.clone(),
        thresholds,
        summary: AnomalySummary {
            anomalies: anomalies.len(),
            total_growth_bytes,
        },
        anomalies,
    }
}

#[cfg(test)]
mod tests {
    use crate::diff::{DiffEntry, DiffReport, DiffSummary};

    use super::{build_anomaly_report, AnomalyThresholds};

    fn diff_report(changes: Vec<DiffEntry>) -> DiffReport {
        DiffReport {
            generated_at: chrono::Local::now(),
            before: "before.json".to_string(),
            after: "after.json".to_string(),
            summary: DiffSummary::default(),
            changes,
        }
    }

    fn grew(path: &str, before_bytes: u64, after_bytes: u64) -> DiffEntry {
        DiffEntry {
            path: path.to_string(),
            before_bytes,
            after_bytes,
            delta_bytes: after_bytes as i64 - before_bytes as i64,
            change: "grew".to_string(),
        }
    }

    #[test]
    fn requires_absolute_and_relative_growth_for_existing_paths() {
        let report = diff_report(vec![
            grew("C:\\demo\\small-relative", 10_000, 11_500),
            grew("C:\\demo\\small-absolute", 1_000, 1_500),
            grew("C:\\demo\\alert", 10_000, 15_000),
        ]);

        let anomaly = build_anomaly_report(
            &report,
            AnomalyThresholds {
                min_growth_bytes: 1_000,
                min_growth_percent: 30.0,
            },
        );

        assert_eq!(anomaly.summary.anomalies, 1);
        assert_eq!(anomaly.anomalies[0].path, "C:\\demo\\alert");
        assert_eq!(anomaly.anomalies[0].growth_percent, Some(50.0));
    }

    #[test]
    fn alerts_on_large_new_paths_without_percent_baseline() {
        let report = diff_report(vec![grew("C:\\demo\\appeared", 0, 2_000)]);

        let anomaly = build_anomaly_report(
            &report,
            AnomalyThresholds {
                min_growth_bytes: 1_000,
                min_growth_percent: 30.0,
            },
        );

        assert_eq!(anomaly.summary.anomalies, 1);
        assert_eq!(anomaly.anomalies[0].path, "C:\\demo\\appeared");
        assert_eq!(anomaly.anomalies[0].growth_percent, None);
    }

    #[test]
    fn ignores_shrinking_and_unchanged_paths() {
        let report = diff_report(vec![
            DiffEntry {
                path: "C:\\demo\\shrunk".to_string(),
                before_bytes: 2_000,
                after_bytes: 1_000,
                delta_bytes: -1_000,
                change: "shrunk".to_string(),
            },
            grew("C:\\demo\\unchanged", 2_000, 2_000),
        ]);

        let anomaly = build_anomaly_report(
            &report,
            AnomalyThresholds {
                min_growth_bytes: 1_000,
                min_growth_percent: 30.0,
            },
        );

        assert!(anomaly.anomalies.is_empty());
    }
}
