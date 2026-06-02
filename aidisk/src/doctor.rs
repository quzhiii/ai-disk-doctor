use chrono::{DateTime, Local};
use serde::Serialize;

use crate::scanner::{Finding, ScanReport};

#[derive(Debug, Serialize)]
pub struct DoctorReport {
    pub generated_at: DateTime<Local>,
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
}

pub fn build_doctor(scan_report: &ScanReport, options: DoctorOptions) -> DoctorReport {
    let mut topics = Vec::new();

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
            |finding| finding.id == "ollama-models" || finding.id == "huggingface-cache",
            vec![
                "Review model inventory before deletion; prefer model-aware commands over manual removal.".to_string(),
                "Move large model stores to a larger drive if model usage is frequent.".to_string(),
            ],
        ));
    }

    DoctorReport {
        generated_at: Local::now(),
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

    DoctorTopic {
        name: name.to_string(),
        summary: format!(
            "{} matching items, {} existing, total observed size {} bytes",
            findings.len(),
            existing_count,
            total_bytes
        ),
        findings,
        recommendations,
    }
}
