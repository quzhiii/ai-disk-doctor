use chrono::{DateTime, Local};
use serde::Serialize;

use crate::policy::Policy;
use crate::scanner::{Finding, ScanReport};

#[derive(Debug, Serialize)]
pub struct DoctorReport {
    pub generated_at: DateTime<Local>,
    pub policy_summary: String,
    pub topics: Vec<DoctorTopic>,
}

#[derive(Debug, Serialize)]
pub struct DoctorTopic {
    pub name: String,
    pub summary: String,
    pub findings: Vec<DoctorFinding>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct DoctorFinding {
    pub id: String,
    pub path: String,
    pub exists: bool,
    pub size_bytes: u64,
    pub risk: String,
    pub action: String,
    pub reason: String,
}

#[derive(Debug, Clone, Copy)]
pub struct DoctorOptions {
    pub docker: bool,
    pub wsl: bool,
    pub ollama: bool,
    pub playwright: bool,
    pub huggingface: bool,
}

pub fn build_doctor(scan_report: &ScanReport, options: DoctorOptions, policy: &Policy) -> DoctorReport {
    let mut topics = Vec::new();
    let policy_summary = format!(
        "sensitive markers: [{}]; planner actions: [{}]; skip modified within: {}min",
        policy.sensitive_markers.join(", "),
        policy.planner.allow_actions.join(", "),
        policy.planner.skip_modified_within_minutes
    );

    if options.docker {
        topics.push(build_topic(
            "docker",
            scan_report,
            |finding| finding.category == "docker",
            vec![
                "Use `docker system df` to inspect image, cache, and volume usage.".to_string(),
                "Prefer `docker builder prune` and targeted prune commands over filesystem deletion.".to_string(),
                "Treat Docker virtual disk files and volume storage as report-only assets.".to_string(),
            ],
        ));
    }

    if options.wsl {
        topics.push(build_topic(
            "wsl",
            scan_report,
            |finding| finding.category == "wsl",
            vec![
                "Do not delete ext4.vhdx directly; use WSL export, compact, or relocation workflows.".to_string(),
                "Check whether large distros should be exported and re-imported to another drive.".to_string(),
            ],
        ));
    }

    if options.ollama {
        topics.push(build_topic(
            "ollama",
            scan_report,
            |finding| finding.id == "ollama-models",
            vec![
                "Run `ollama list` before cleanup so model names and sizes are explicit.".to_string(),
                "Remove unused models through model-aware commands instead of deleting blob paths directly.".to_string(),
            ],
        ));
    }

    if options.huggingface {
        topics.push(build_topic(
            "huggingface",
            scan_report,
            |finding| finding.id == "huggingface-cache",
            vec![
                "Review which projects share this cache before deleting reusable downloads.".to_string(),
                "Prefer targeted cache cleanup or relocation over wiping the cache root blindly.".to_string(),
            ],
        ));
    }

    if options.playwright {
        topics.push(build_topic(
            "playwright",
            scan_report,
            |finding| finding.id == "playwright-project-browsers" || finding.path.contains("ms-playwright"),
            vec![
                "Check whether browsers are being downloaded per project instead of via a shared cache.".to_string(),
                "If browser downloads are repeated, centralize Playwright browser storage before deleting caches.".to_string(),
            ],
        ));
    }

    DoctorReport {
        generated_at: Local::now(),
        policy_summary,
        topics,
    }
}

fn build_topic<F>(
    name: &str,
    scan_report: &ScanReport,
    predicate: F,
    recommendations: Vec<String>,
) -> DoctorTopic
where
    F: Fn(&Finding) -> bool,
{
    let findings = scan_report
        .findings
        .iter()
        .filter(|finding| predicate(finding))
        .map(|finding| DoctorFinding {
            id: finding.id.clone(),
            path: finding.path.clone(),
            exists: finding.exists,
            size_bytes: finding.size_bytes,
            risk: format!("{:?}", finding.risk).to_ascii_lowercase(),
            action: finding.action.clone(),
            reason: finding.reason.clone(),
        })
        .collect::<Vec<_>>();

    let existing_count = findings.iter().filter(|finding| finding.exists).count();
    let total_bytes = findings
        .iter()
        .filter(|finding| finding.exists)
        .map(|finding| finding.size_bytes)
        .sum::<u64>();

    let summary = if findings.is_empty() {
        format!(
            "No {} rules matched the current scan set. Check whether the related rules are enabled.",
            name
        )
    } else if existing_count == 0 {
        format!(
            "{} {} rules matched, but no existing paths were found on this machine right now.",
            findings.len(), name
        )
    } else {
        format!(
            "{} matching items, {} existing, total observed size {} bytes",
            findings.len(),
            existing_count,
            total_bytes
        )
    };

    DoctorTopic {
        name: name.to_string(),
        summary,
        findings,
        recommendations: enrich_recommendations(name, existing_count, recommendations),
    }
}

fn enrich_recommendations(
    name: &str,
    existing_count: usize,
    mut recommendations: Vec<String>,
) -> Vec<String> {
    if existing_count == 0 {
        recommendations.insert(
            0,
            format!(
                "No active {} paths were detected. If usage is expected, re-run doctor after reproducing the workload.",
                name
            ),
        );
    }

    match name {
        "docker" => {
            recommendations.push(
                "If Docker Desktop is large, compare `aidisk doctor --docker` output with `docker system df` before pruning."
                    .to_string(),
            );
        }
        "wsl" => {
            recommendations.push(
                "If WSL growth is unexpected, inspect distro usage first, then plan export/compact instead of touching files directly."
                    .to_string(),
            );
        }
        "ollama" => {
            recommendations.push(
                "If model caches are intentional, prefer relocating them to a larger drive over repeated deletion."
                    .to_string(),
            );
        }
        "huggingface" => {
            recommendations.push(
                "If Hugging Face artifacts are still needed across projects, relocate the cache to a larger disk instead of repeatedly pruning it."
                    .to_string(),
            );
        }
        "playwright" => {
            recommendations.push(
                "If browser runtimes are expected, track whether `.playwright-browsers` appears in multiple worktrees and consolidate where possible."
                    .to_string(),
            );
        }
        _ => {}
    }

    recommendations
}

#[cfg(test)]
mod tests {
    use chrono::Local;

    use super::{build_doctor, DoctorOptions};
    use crate::policy::{PlannerPolicy, Policy};
    use crate::rules::RiskLevel;
    use crate::scanner::{Finding, ScanReport, Summary, Volume};

    fn empty_scan() -> ScanReport {
        ScanReport {
            scan_time: Local::now(),
            volumes: Vec::<Volume>::new(),
            findings: Vec::<Finding>::new(),
            summary: Summary::default(),
        }
    }

    fn test_policy() -> Policy {
        Policy {
            sensitive_markers: vec!["token".to_string(), ".env".to_string()],
            planner: PlannerPolicy {
                skip_modified_within_minutes: 30,
                allow_actions: vec!["quarantine".to_string(), "report-only".to_string(), "guide".to_string()],
                max_scan_depth: 20,
            },
        }
    }

    #[test]
    fn doctor_explains_empty_topics() {
        let report = build_doctor(
            &empty_scan(),
            DoctorOptions {
                docker: true,
                wsl: false,
                ollama: false,
                playwright: false,
                huggingface: false,
            },
            &test_policy(),
        );

        assert_eq!(report.topics.len(), 1);
        assert!(report.topics[0].summary.contains("No docker rules matched"));
    }

    #[test]
    fn doctor_explains_missing_paths() {
        let report = ScanReport {
            scan_time: Local::now(),
            volumes: Vec::<Volume>::new(),
            findings: vec![Finding {
                id: "wsl-ext4-vhdx".to_string(),
                name: "WSL ext4 virtual disk".to_string(),
                category: "wsl".to_string(),
                path: "C:\\Users\\demo\\AppData\\Local\\Packages\\X\\LocalState\\ext4.vhdx".to_string(),
                exists: false,
                size_bytes: 0,
                risk: RiskLevel::System,
                action: "guide".to_string(),
                reason: "demo".to_string(),
                warnings: Vec::new(),
            }],
            summary: Summary::default(),
        };

        let doctor = build_doctor(
            &report,
            DoctorOptions {
                docker: false,
                wsl: true,
                ollama: false,
                playwright: false,
                huggingface: false,
            },
            &test_policy(),
        );

        assert!(doctor.topics[0].summary.contains("no existing paths were found"));
    }

    #[test]
    fn doctor_can_split_ollama_and_huggingface_topics() {
        let report = ScanReport {
            scan_time: Local::now(),
            volumes: Vec::<Volume>::new(),
            findings: vec![
                Finding {
                    id: "ollama-models".to_string(),
                    name: "Ollama model store".to_string(),
                    category: "models".to_string(),
                    path: "C:\\Users\\demo\\.ollama\\models".to_string(),
                    exists: true,
                    size_bytes: 100,
                    risk: RiskLevel::Review,
                    action: "guide".to_string(),
                    reason: "ollama".to_string(),
                    warnings: Vec::new(),
                },
                Finding {
                    id: "huggingface-cache".to_string(),
                    name: "Hugging Face cache".to_string(),
                    category: "models".to_string(),
                    path: "C:\\Users\\demo\\.cache\\huggingface".to_string(),
                    exists: true,
                    size_bytes: 200,
                    risk: RiskLevel::Review,
                    action: "guide".to_string(),
                    reason: "hf".to_string(),
                    warnings: Vec::new(),
                },
            ],
            summary: Summary::default(),
        };

        let doctor = build_doctor(
            &report,
            DoctorOptions {
                docker: false,
                wsl: false,
                ollama: true,
                playwright: false,
                huggingface: true,
            },
            &test_policy(),
        );

        assert_eq!(doctor.topics.len(), 2);
        assert_eq!(doctor.topics[0].name, "ollama");
        assert_eq!(doctor.topics[1].name, "huggingface");
    }
}
