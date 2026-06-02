use anyhow::Result;

use crate::cleaner::{CleanReport, ExecutionReport, QuarantinePlan, RestoreReport};
use crate::doctor::DoctorReport;
use crate::planner::PlanReport;
use crate::scanner::ScanReport;
use crate::OutputFormat;

pub fn render(report: &ScanReport, format: OutputFormat) -> Result<String> {
    let output = match format {
        OutputFormat::Json => serde_json::to_string_pretty(report)?,
        OutputFormat::Markdown => render_markdown(report),
        OutputFormat::Text => render_text(report),
    };

    Ok(output)
}

pub fn render_plan(report: &PlanReport, format: OutputFormat) -> Result<String> {
    let output = match format {
        OutputFormat::Json => serde_json::to_string_pretty(report)?,
        OutputFormat::Markdown => render_plan_markdown(report),
        OutputFormat::Text => render_plan_text(report),
    };

    Ok(output)
}

pub fn render_clean(report: &CleanReport, format: OutputFormat) -> Result<String> {
    let output = match format {
        OutputFormat::Json => serde_json::to_string_pretty(report)?,
        OutputFormat::Markdown => render_clean_markdown(report),
        OutputFormat::Text => render_clean_text(report),
    };

    Ok(output)
}

pub fn render_quarantine_plan(report: &QuarantinePlan, format: OutputFormat) -> Result<String> {
    let output = match format {
        OutputFormat::Json => serde_json::to_string_pretty(report)?,
        OutputFormat::Markdown => render_quarantine_markdown(report),
        OutputFormat::Text => render_quarantine_text(report),
    };

    Ok(output)
}

pub fn render_execution(report: &ExecutionReport, format: OutputFormat) -> Result<String> {
    let output = match format {
        OutputFormat::Json => serde_json::to_string_pretty(report)?,
        OutputFormat::Markdown => render_execution_markdown(report),
        OutputFormat::Text => render_execution_text(report),
    };

    Ok(output)
}

pub fn render_restore(report: &RestoreReport, format: OutputFormat) -> Result<String> {
    let output = match format {
        OutputFormat::Json => serde_json::to_string_pretty(report)?,
        OutputFormat::Markdown => render_restore_markdown(report),
        OutputFormat::Text => render_restore_text(report),
    };

    Ok(output)
}

pub fn render_doctor(report: &DoctorReport, format: OutputFormat) -> Result<String> {
    let output = match format {
        OutputFormat::Json => serde_json::to_string_pretty(report)?,
        OutputFormat::Markdown => render_doctor_markdown(report),
        OutputFormat::Text => render_doctor_text(report),
    };

    Ok(output)
}

fn render_text(report: &ScanReport) -> String {
    let mut lines = vec![
        "Windows AI Space Report".to_string(),
        format!("Scan Time: {}", report.scan_time),
        format!("Rules: {}", report.summary.total_rules),
        format!("Matched Paths: {}", report.summary.matched_paths),
        format!("Total Size: {}", human_bytes(report.summary.total_size_bytes)),
        format!("Safe Bytes: {}", human_bytes(report.summary.safe_bytes)),
        format!("Review Bytes: {}", human_bytes(report.summary.review_bytes)),
        format!("Dangerous Bytes: {}", human_bytes(report.summary.dangerous_bytes)),
        format!("System Bytes: {}", human_bytes(report.summary.system_bytes)),
        format!(
            "Reclaimable Safe Bytes: {}",
            human_bytes(report.summary.reclaimable_safe_bytes)
        ),
        String::new(),
        "Top Findings:".to_string(),
    ];

    for finding in &report.summary.top_findings {
        lines.push(format!(
            "- [{}] {} | {}",
            risk_label(&finding.risk),
            finding.path,
            human_bytes(finding.size_bytes)
        ));
    }

    lines.push(String::new());
    lines.push("Findings:".to_string());

    if !report.volumes.is_empty() {
        lines.splice(11..11, [String::new(), "Volumes:".to_string()]);
        let insert_at = 13;
        for (index, volume) in report.volumes.iter().enumerate() {
            lines.insert(
                insert_at + index,
                format!(
                    "- {} ({}) | free={} | total={}",
                    display_volume_name(volume),
                    volume.mount_point,
                    human_bytes(volume.available_bytes),
                    human_bytes(volume.total_bytes)
                ),
            );
        }
        lines.push("Volumes:".to_string());
    }

    for finding in &report.findings {
        lines.push(format!(
            "- [{}] {} | exists={} | {} | {}",
            risk_label(&finding.risk),
            finding.path,
            finding.exists,
            human_bytes(finding.size_bytes),
            finding.reason
        ));
    }

    lines.join("\n")
}

fn render_markdown(report: &ScanReport) -> String {
    let mut lines = vec![
        "# Windows AI Space Report".to_string(),
        String::new(),
        format!("- Scan Time: {}", report.scan_time),
        format!("- Rules: {}", report.summary.total_rules),
        format!("- Matched Paths: {}", report.summary.matched_paths),
        format!("- Total Size: {}", human_bytes(report.summary.total_size_bytes)),
        format!("- Safe Bytes: {}", human_bytes(report.summary.safe_bytes)),
        format!("- Review Bytes: {}", human_bytes(report.summary.review_bytes)),
        format!("- Dangerous Bytes: {}", human_bytes(report.summary.dangerous_bytes)),
        format!("- System Bytes: {}", human_bytes(report.summary.system_bytes)),
        format!(
            "- Reclaimable Safe Bytes: {}",
            human_bytes(report.summary.reclaimable_safe_bytes)
        ),
        String::new(),
        "## Top Findings".to_string(),
        String::new(),
        "| Risk | Path | Size |".to_string(),
        "|---|---|---:|".to_string(),
        String::new(),
        "## Findings".to_string(),
        String::new(),
        "| Risk | Path | Exists | Size | Action |".to_string(),
        "|---|---|---:|---:|---|".to_string(),
    ];

    for finding in &report.summary.top_findings {
        lines.insert(
            15,
            format!(
                "| {} | `{}` | {} |",
                risk_label(&finding.risk),
                finding.path,
                human_bytes(finding.size_bytes)
            ),
        );
    }

    if !report.volumes.is_empty() {
        lines.splice(
            12..12,
            [
                "## Volumes".to_string(),
                String::new(),
                "| Name | Mount | Free | Total |".to_string(),
                "|---|---|---:|---:|".to_string(),
            ],
        );

        for volume in &report.volumes {
            lines.insert(
                16,
                format!(
                    "| {} | `{}` | {} | {} |",
                    display_volume_name(volume),
                    volume.mount_point,
                    human_bytes(volume.available_bytes),
                    human_bytes(volume.total_bytes)
                ),
            );
        }
    }

    for finding in &report.findings {
        lines.push(format!(
            "| {} | `{}` | {} | {} | {} |",
            risk_label(&finding.risk),
            finding.path,
            finding.exists,
            human_bytes(finding.size_bytes),
            finding.action
        ));
    }

    lines.join("\n")
}

fn render_plan_text(report: &PlanReport) -> String {
    let mut lines = vec![
        "Windows AI Space Plan".to_string(),
        format!("Generated At: {}", report.generated_at),
        format!("Mode: {}", report.mode),
        format!("Safe Only: {}", report.safe_only),
        format!(
            "Skip Modified Within Minutes: {}",
            report.skip_modified_within_minutes
        ),
        format!("Total Findings: {}", report.summary.total_findings),
        format!("Eligible Candidates: {}", report.summary.eligible_candidates),
        format!("Skipped Findings: {}", report.summary.skipped_findings),
        format!(
            "Blocked Sensitive Paths: {}",
            report.summary.blocked_sensitive_paths
        ),
        format!(
            "Skipped Recently Modified: {}",
            report.summary.skipped_recently_modified
        ),
        format!(
            "Reclaimable Bytes: {}",
            human_bytes(report.summary.reclaimable_bytes)
        ),
        String::new(),
        "Action Groups:".to_string(),
    ];

    for group in &report.groups {
        lines.push(format!(
            "- {} | candidates={} | {}",
            group.action,
            group.candidate_count,
            human_bytes(group.total_bytes)
        ));
    }

    lines.extend([
        String::new(),
        "Candidates:".to_string(),
    ]);

    for candidate in &report.candidates {
        lines.push(format!(
            "- [{}] {} | {} | {}",
            risk_label(&candidate.risk),
            candidate.path,
            human_bytes(candidate.size_bytes),
            candidate.action
        ));
    }

    if !report.skipped.is_empty() {
        lines.push(String::new());
        lines.push("Skipped:".to_string());
        for skipped in &report.skipped {
            lines.push(format!("- {} | {}", skipped.path, skipped.reason));
        }
    }

    lines.join("\n")
}

fn render_plan_markdown(report: &PlanReport) -> String {
    let mut lines = vec![
        "# Windows AI Space Plan".to_string(),
        String::new(),
        format!("- Generated At: {}", report.generated_at),
        format!("- Mode: {}", report.mode),
        format!("- Safe Only: {}", report.safe_only),
        format!(
            "- Skip Modified Within Minutes: {}",
            report.skip_modified_within_minutes
        ),
        format!("- Total Findings: {}", report.summary.total_findings),
        format!("- Eligible Candidates: {}", report.summary.eligible_candidates),
        format!("- Skipped Findings: {}", report.summary.skipped_findings),
        format!(
            "- Blocked Sensitive Paths: {}",
            report.summary.blocked_sensitive_paths
        ),
        format!(
            "- Skipped Recently Modified: {}",
            report.summary.skipped_recently_modified
        ),
        format!(
            "- Reclaimable Bytes: {}",
            human_bytes(report.summary.reclaimable_bytes)
        ),
        String::new(),
        "## Action Groups".to_string(),
        String::new(),
        "| Action | Candidates | Size |".to_string(),
        "|---|---:|---:|".to_string(),
    ];

    for group in &report.groups {
        lines.push(format!(
            "| {} | {} | {} |",
            group.action,
            group.candidate_count,
            human_bytes(group.total_bytes)
        ));
    }

    lines.push(String::new());
    lines.push("## Candidates".to_string());
    lines.push(String::new());
    lines.push("| Risk | Path | Size | Action |".to_string());
    lines.push("|---|---|---:|---|".to_string());

    for candidate in &report.candidates {
        lines.push(format!(
            "| {} | `{}` | {} | {} |",
            risk_label(&candidate.risk),
            candidate.path,
            human_bytes(candidate.size_bytes),
            candidate.action
        ));
    }

    if !report.skipped.is_empty() {
        lines.push(String::new());
        lines.push("## Skipped".to_string());
        lines.push(String::new());
        lines.push("| Path | Reason |".to_string());
        lines.push("|---|---|".to_string());
        for skipped in &report.skipped {
            lines.push(format!("| `{}` | {} |", skipped.path, skipped.reason));
        }
    }

    lines.join("\n")
}

fn render_clean_text(report: &CleanReport) -> String {
    let mut lines = vec![
        "Windows AI Space Clean Preview".to_string(),
        format!("Generated At: {}", report.generated_at),
        format!("Mode: {}", report.mode),
        format!("Candidate Count: {}", report.candidate_count),
        format!("Reclaimable Bytes: {}", human_bytes(report.reclaimable_bytes)),
        String::new(),
        "Action Groups:".to_string(),
    ];

    for group in &report.groups {
        lines.push(format!(
            "- {} | candidates={} | {}",
            group.action,
            group.candidate_count,
            human_bytes(group.total_bytes)
        ));
    }

    lines.extend([
        String::new(),
        "Actions:".to_string(),
    ]);

    for action in &report.actions {
        lines.push(format!(
            "- {} | {} | {}",
            action.path,
            action.action,
            human_bytes(action.size_bytes)
        ));
    }

    if !report.skipped.is_empty() {
        lines.push(String::new());
        lines.push("Skipped:".to_string());
        for skipped in &report.skipped {
            lines.push(format!("- {} | {}", skipped.path, skipped.reason));
        }
    }

    lines.join("\n")
}

fn render_clean_markdown(report: &CleanReport) -> String {
    let mut lines = vec![
        "# Windows AI Space Clean Preview".to_string(),
        String::new(),
        format!("- Generated At: {}", report.generated_at),
        format!("- Mode: {}", report.mode),
        format!("- Candidate Count: {}", report.candidate_count),
        format!("- Reclaimable Bytes: {}", human_bytes(report.reclaimable_bytes)),
        String::new(),
        "## Action Groups".to_string(),
        String::new(),
        "| Action | Candidates | Size |".to_string(),
        "|---|---:|---:|".to_string(),
    ];

    for group in &report.groups {
        lines.push(format!(
            "| {} | {} | {} |",
            group.action,
            group.candidate_count,
            human_bytes(group.total_bytes)
        ));
    }

    lines.push(String::new());
    lines.push("## Actions".to_string());
    lines.push(String::new());
    lines.push("| Path | Action | Size |".to_string());
    lines.push("|---|---|---:|".to_string());

    for action in &report.actions {
        lines.push(format!(
            "| `{}` | {} | {} |",
            action.path,
            action.action,
            human_bytes(action.size_bytes)
        ));
    }

    if !report.skipped.is_empty() {
        lines.push(String::new());
        lines.push("## Skipped".to_string());
        lines.push(String::new());
        lines.push("| Path | Reason |".to_string());
        lines.push("|---|---|".to_string());
        for skipped in &report.skipped {
            lines.push(format!("| `{}` | {} |", skipped.path, skipped.reason));
        }
    }

    lines.join("\n")
}

fn render_quarantine_text(report: &QuarantinePlan) -> String {
    let mut lines = vec![
        "Windows AI Space Quarantine Plan".to_string(),
        format!("Root: {}", report.root),
        String::new(),
        "Entries:".to_string(),
    ];

    for entry in &report.entries {
        lines.push(format!("- {} => {}", entry.source_path, entry.destination_path));
    }

    lines.join("\n")
}

fn render_quarantine_markdown(report: &QuarantinePlan) -> String {
    let mut lines = vec![
        "# Windows AI Space Quarantine Plan".to_string(),
        String::new(),
        format!("- Root: {}", report.root),
        String::new(),
        "## Entries".to_string(),
        String::new(),
        "| Source | Destination |".to_string(),
        "|---|---|".to_string(),
    ];

    for entry in &report.entries {
        lines.push(format!("| `{}` | `{}` |", entry.source_path, entry.destination_path));
    }

    lines.join("\n")
}

fn render_execution_text(report: &ExecutionReport) -> String {
    let mut lines = vec![
        "Windows AI Space Quarantine Result".to_string(),
        format!("Generated At: {}", report.generated_at),
        format!("Mode: {}", report.mode),
        format!("Root: {}", report.root),
        format!("Success Count: {}", report.success_count),
        format!("Failure Count: {}", report.failure_count),
        format!("Index Path: {}", report.index_path),
        format!("Log Path: {}", report.log_path),
        String::new(),
        "Results:".to_string(),
    ];

    for result in &report.results {
        lines.push(format!(
            "- {} => {} | {} | {}",
            result.source_path, result.destination_path, result.status, result.message
        ));
    }

    lines.join("\n")
}

fn render_execution_markdown(report: &ExecutionReport) -> String {
    let mut lines = vec![
        "# Windows AI Space Quarantine Result".to_string(),
        String::new(),
        format!("- Generated At: {}", report.generated_at),
        format!("- Mode: {}", report.mode),
        format!("- Root: {}", report.root),
        format!("- Success Count: {}", report.success_count),
        format!("- Failure Count: {}", report.failure_count),
        format!("- Index Path: `{}`", report.index_path),
        format!("- Log Path: `{}`", report.log_path),
        String::new(),
        "## Results".to_string(),
        String::new(),
        "| Source | Destination | Status | Message |".to_string(),
        "|---|---|---|---|".to_string(),
    ];

    for result in &report.results {
        lines.push(format!(
            "| `{}` | `{}` | {} | {} |",
            result.source_path, result.destination_path, result.status, result.message
        ));
    }

    lines.join("\n")
}

fn render_restore_text(report: &RestoreReport) -> String {
    let mut lines = vec![
        "Windows AI Space Restore Report".to_string(),
        format!("Generated At: {}", report.generated_at),
        format!("Mode: {}", report.mode),
        format!("Index Path: {}", report.index_path),
        format!("Root: {}", report.root),
        format!("Entry Count: {}", report.entry_count),
        format!("Success Count: {}", report.success_count),
        format!("Failure Count: {}", report.failure_count),
        String::new(),
        "Results:".to_string(),
    ];

    for result in &report.results {
        lines.push(format!(
            "- {} => {} | {} | {}",
            result.source_path, result.destination_path, result.status, result.message
        ));
    }

    lines.join("\n")
}

fn render_restore_markdown(report: &RestoreReport) -> String {
    let mut lines = vec![
        "# Windows AI Space Restore Report".to_string(),
        String::new(),
        format!("- Generated At: {}", report.generated_at),
        format!("- Mode: {}", report.mode),
        format!("- Index Path: `{}`", report.index_path),
        format!("- Root: {}", report.root),
        format!("- Entry Count: {}", report.entry_count),
        format!("- Success Count: {}", report.success_count),
        format!("- Failure Count: {}", report.failure_count),
        String::new(),
        "## Results".to_string(),
        String::new(),
        "| Source | Destination | Status | Message |".to_string(),
        "|---|---|---|---|".to_string(),
    ];

    for result in &report.results {
        lines.push(format!(
            "| `{}` | `{}` | {} | {} |",
            result.source_path, result.destination_path, result.status, result.message
        ));
    }

    lines.join("\n")
}

fn render_doctor_text(report: &DoctorReport) -> String {
    let mut lines = vec![
        "Windows AI Space Doctor".to_string(),
        format!("Generated At: {}", report.generated_at),
    ];

    for topic in &report.topics {
        lines.push(String::new());
        lines.push(format!("[{}] {}", topic.name.to_uppercase(), topic.summary));
        for finding in &topic.findings {
            lines.push(format!(
                "- {} | exists={} | {} bytes | {}",
                finding.path, finding.exists, finding.size_bytes, finding.reason
            ));
        }
        for recommendation in &topic.recommendations {
            lines.push(format!("- Recommendation: {}", recommendation));
        }
    }

    lines.join("\n")
}

fn render_doctor_markdown(report: &DoctorReport) -> String {
    let mut lines = vec![
        "# Windows AI Space Doctor".to_string(),
        String::new(),
        format!("- Generated At: {}", report.generated_at),
    ];

    for topic in &report.topics {
        lines.push(String::new());
        lines.push(format!("## {}", topic.name));
        lines.push(String::new());
        lines.push(format!("- Summary: {}", topic.summary));
        lines.push(String::new());
        lines.push("| Path | Exists | Size | Risk | Action |".to_string());
        lines.push("|---|---:|---:|---|---|".to_string());
        for finding in &topic.findings {
            lines.push(format!(
                "| `{}` | {} | {} | {} | {} |",
                finding.path,
                finding.exists,
                finding.size_bytes,
                finding.risk,
                finding.action
            ));
        }
        lines.push(String::new());
        lines.push("Recommendations:".to_string());
        for recommendation in &topic.recommendations {
            lines.push(format!("- {}", recommendation));
        }
    }

    lines.join("\n")
}

fn display_volume_name(volume: &crate::scanner::Volume) -> &str {
    if volume.name.is_empty() {
        volume.mount_point.as_str()
    } else {
        volume.name.as_str()
    }
}

fn risk_label(risk: &crate::rules::RiskLevel) -> &'static str {
    match risk {
        crate::rules::RiskLevel::Safe => "SAFE",
        crate::rules::RiskLevel::Review => "REVIEW",
        crate::rules::RiskLevel::Dangerous => "DANGEROUS",
        crate::rules::RiskLevel::System => "SYSTEM",
    }
}

fn human_bytes(bytes: u64) -> String {
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
