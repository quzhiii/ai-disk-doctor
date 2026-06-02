use anyhow::Result;

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
        format!("Total Findings: {}", report.summary.total_findings),
        format!("Eligible Candidates: {}", report.summary.eligible_candidates),
        format!("Skipped Findings: {}", report.summary.skipped_findings),
        format!(
            "Reclaimable Bytes: {}",
            human_bytes(report.summary.reclaimable_bytes)
        ),
        String::new(),
        "Candidates:".to_string(),
    ];

    for candidate in &report.candidates {
        lines.push(format!(
            "- [{}] {} | {} | {}",
            risk_label(&candidate.risk),
            candidate.path,
            human_bytes(candidate.size_bytes),
            candidate.action
        ));
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
        format!("- Total Findings: {}", report.summary.total_findings),
        format!("- Eligible Candidates: {}", report.summary.eligible_candidates),
        format!("- Skipped Findings: {}", report.summary.skipped_findings),
        format!(
            "- Reclaimable Bytes: {}",
            human_bytes(report.summary.reclaimable_bytes)
        ),
        String::new(),
        "## Candidates".to_string(),
        String::new(),
        "| Risk | Path | Size | Action |".to_string(),
        "|---|---|---:|---|".to_string(),
    ];

    for candidate in &report.candidates {
        lines.push(format!(
            "| {} | `{}` | {} | {} |",
            risk_label(&candidate.risk),
            candidate.path,
            human_bytes(candidate.size_bytes),
            candidate.action
        ));
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
