use anyhow::Result;

use crate::cleaner::{
    CleanDryRunOutput, CleanReport, ExecutionReport, QuarantinePlan, RestoreReport,
};
use crate::diff::DiffReport;
use crate::doctor::DoctorReport;
use crate::planner::PlanReport;
use crate::policy::PolicySnapshot;
use crate::scanner::LargeFilesReport;
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

pub fn render_clean_dry_run_output(
    report: &CleanDryRunOutput,
    format: OutputFormat,
) -> Result<String> {
    let output = match format {
        OutputFormat::Json => serde_json::to_string_pretty(report)?,
        OutputFormat::Markdown => render_clean_markdown(&report.clean),
        OutputFormat::Text => render_clean_text(&report.clean),
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
    ];
    lines.extend(render_policy_lines(report.policy.as_ref(), false));
    lines.extend(render_scan_limit_lines(report.policy.as_ref(), false));
    lines.extend([
        format!("Rules: {}", report.summary.total_rules),
        format!("Matched Paths: {}", report.summary.matched_paths),
        format!("Partial Findings: {}", report.summary.partial_findings),
        format!(
            "Total Size: {}",
            human_bytes(report.summary.total_size_bytes)
        ),
        format!("Safe Bytes: {}", human_bytes(report.summary.safe_bytes)),
        format!("Review Bytes: {}", human_bytes(report.summary.review_bytes)),
        format!(
            "Dangerous Bytes: {}",
            human_bytes(report.summary.dangerous_bytes)
        ),
        format!("System Bytes: {}", human_bytes(report.summary.system_bytes)),
        format!(
            "Reclaimable Safe Bytes: {}",
            human_bytes(report.summary.reclaimable_safe_bytes)
        ),
    ]);

    lines.push(String::new());
    lines.extend(render_scan_executive_summary_lines(report, false));

    if !report.volumes.is_empty() {
        lines.push(String::new());
        lines.push("Volumes:".to_string());
        for volume in &report.volumes {
            lines.push(format!(
                "- {} ({}) | free={} | total={}",
                display_volume_name(volume),
                volume.mount_point,
                human_bytes(volume.available_bytes),
                human_bytes(volume.total_bytes)
            ));
        }
    }

    lines.push(String::new());
    lines.push("Top Findings:".to_string());
    for finding in &report.summary.top_findings {
        lines.push(format!(
            "- [{}] {} | {}",
            risk_label(&finding.risk),
            finding.path,
            human_bytes_with_partial(finding.size_bytes, finding.partial)
        ));
    }

    lines.push(String::new());
    lines.push("Findings:".to_string());
    for finding in &report.findings {
        lines.push(format!(
            "- [{}] {} | exists={} | {} | {}",
            risk_label(&finding.risk),
            finding.path,
            finding.exists,
            human_bytes_with_partial(finding.size_bytes, finding.partial),
            finding.reason
        ));
        if finding.partial {
            lines.push(format!(
                "  Partial size warning: {}",
                partial_reasons_text(&finding.partial_reasons)
            ));
        }
    }

    lines.join("\n")
}

fn render_markdown(report: &ScanReport) -> String {
    let mut lines = vec![
        "# Windows AI Space Report".to_string(),
        String::new(),
        format!("- Scan Time: {}", report.scan_time),
    ];
    lines.extend(render_policy_lines(report.policy.as_ref(), true));
    lines.extend(render_scan_limit_lines(report.policy.as_ref(), true));
    lines.extend([
        format!("- Rules: {}", report.summary.total_rules),
        format!("- Matched Paths: {}", report.summary.matched_paths),
        format!("- Partial Findings: {}", report.summary.partial_findings),
        format!(
            "- Total Size: {}",
            human_bytes(report.summary.total_size_bytes)
        ),
        format!("- Safe Bytes: {}", human_bytes(report.summary.safe_bytes)),
        format!(
            "- Review Bytes: {}",
            human_bytes(report.summary.review_bytes)
        ),
        format!(
            "- Dangerous Bytes: {}",
            human_bytes(report.summary.dangerous_bytes)
        ),
        format!(
            "- System Bytes: {}",
            human_bytes(report.summary.system_bytes)
        ),
        format!(
            "- Reclaimable Safe Bytes: {}",
            human_bytes(report.summary.reclaimable_safe_bytes)
        ),
    ]);

    lines.push(String::new());
    lines.extend(render_scan_executive_summary_lines(report, true));

    if !report.volumes.is_empty() {
        lines.push(String::new());
        lines.push("## Volumes".to_string());
        lines.push(String::new());
        lines.push("| Name | Mount | Free | Total |".to_string());
        lines.push("|---|---|---:|---:|".to_string());
        for volume in &report.volumes {
            lines.push(format!(
                "| {} | `{}` | {} | {} |",
                display_volume_name(volume),
                volume.mount_point,
                human_bytes(volume.available_bytes),
                human_bytes(volume.total_bytes)
            ));
        }
    }

    lines.push(String::new());
    lines.push("## Top Findings".to_string());
    lines.push(String::new());
    lines.push("| Risk | Path | Size |".to_string());
    lines.push("|---|---|---:|".to_string());
    for finding in &report.summary.top_findings {
        lines.push(format!(
            "| {} | `{}` | {} |",
            risk_label(&finding.risk),
            finding.path,
            human_bytes_with_partial(finding.size_bytes, finding.partial)
        ));
    }

    lines.push(String::new());
    lines.push("## Findings".to_string());
    lines.push(String::new());
    lines.push("| Risk | Path | Exists | Size | Action |".to_string());
    lines.push("|---|---|---:|---:|---|".to_string());
    for finding in &report.findings {
        lines.push(format!(
            "| {} | `{}` | {} | {} | {} |",
            risk_label(&finding.risk),
            finding.path,
            finding.exists,
            human_bytes_with_partial(finding.size_bytes, finding.partial),
            finding.action
        ));
        if finding.partial {
            lines.push(format!(
                "- Partial size warning for `{}`: {}",
                finding.path,
                partial_reasons_text(&finding.partial_reasons)
            ));
        }
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
    ];
    lines.extend(render_policy_lines(report.policy.as_ref(), false));
    lines.extend([
        format!("Total Findings: {}", report.summary.total_findings),
        format!(
            "Eligible Candidates: {}",
            report.summary.eligible_candidates
        ),
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
    ]);

    for group in &report.groups {
        lines.push(format!(
            "- {} | candidates={} | {}",
            group.action,
            group.candidate_count,
            human_bytes(group.total_bytes)
        ));
    }

    lines.extend([String::new(), "Candidates:".to_string()]);

    for candidate in &report.candidates {
        lines.push(format!(
            "- [{}] {} | {} | {}",
            risk_label(&candidate.risk),
            candidate.path,
            human_bytes_with_partial(candidate.size_bytes, candidate.partial),
            candidate.action
        ));
        if candidate.partial {
            lines.push(format!(
                "  Partial size warning: {}",
                partial_reasons_text(&candidate.partial_reasons)
            ));
        }
    }

    if !report.skipped.is_empty() {
        lines.push(String::new());
        lines.push("Skipped:".to_string());
        for skipped in &report.skipped {
            lines.push(format!("- {} | {}", skipped.path, skipped.reason));
            if skipped.partial {
                lines.push(format!(
                    "  Partial size warning: {}",
                    partial_reasons_text(&skipped.partial_reasons)
                ));
            }
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
    ];
    lines.extend(render_policy_lines(report.policy.as_ref(), true));
    lines.extend([
        format!("- Total Findings: {}", report.summary.total_findings),
        format!(
            "- Eligible Candidates: {}",
            report.summary.eligible_candidates
        ),
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
    ]);

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
            human_bytes_with_partial(candidate.size_bytes, candidate.partial),
            candidate.action
        ));
        if candidate.partial {
            lines.push(format!(
                "- Partial size warning for `{}`: {}",
                candidate.path,
                partial_reasons_text(&candidate.partial_reasons)
            ));
        }
    }

    if !report.skipped.is_empty() {
        lines.push(String::new());
        lines.push("## Skipped".to_string());
        lines.push(String::new());
        lines.push("| Path | Reason |".to_string());
        lines.push("|---|---|".to_string());
        for skipped in &report.skipped {
            lines.push(format!("| `{}` | {} |", skipped.path, skipped.reason));
            if skipped.partial {
                lines.push(format!(
                    "- Partial size warning for `{}`: {}",
                    skipped.path,
                    partial_reasons_text(&skipped.partial_reasons)
                ));
            }
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
        format!(
            "Reclaimable Bytes: {}",
            human_bytes(report.reclaimable_bytes)
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

    lines.extend([String::new(), "Actions:".to_string()]);

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
        format!(
            "- Reclaimable Bytes: {}",
            human_bytes(report.reclaimable_bytes)
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
        lines.push(format!(
            "- {} => {}",
            entry.source_path, entry.destination_path
        ));
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
        lines.push(format!(
            "| `{}` | `{}` |",
            entry.source_path, entry.destination_path
        ));
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
    lines.extend(render_policy_lines(report.policy.as_ref(), false));

    lines.push(String::new());
    lines.extend(render_doctor_executive_summary_lines(report, false));

    if let Some(latest_diff) = &report.latest_diff {
        lines.push(String::new());
        lines.push("Latest Diff:".to_string());
        lines.push(format!("- Before: {}", latest_diff.before));
        lines.push(format!("- After: {}", latest_diff.after));
        lines.push(format!(
            "- Total Growth: {}",
            human_bytes_delta(latest_diff.summary.total_growth_bytes)
        ));
        lines.push(format!(
            "- Counts: grew={} shrunk={} appeared={} disappeared={}",
            latest_diff.summary.grew,
            latest_diff.summary.shrunk,
            latest_diff.summary.appeared,
            latest_diff.summary.disappeared
        ));
        for change in &latest_diff.top_changes {
            lines.push(format!(
                "- [{}] {} | before={} after={} delta={}",
                change.change,
                change.path,
                human_bytes(change.before_bytes),
                human_bytes(change.after_bytes),
                human_bytes_delta(change.delta_bytes)
            ));
        }
    }

    for topic in &report.topics {
        lines.push(String::new());
        lines.push(format!("[{}] {}", topic.name.to_uppercase(), topic.summary));
        let missing_count = topic
            .findings
            .iter()
            .filter(|finding| !finding.exists)
            .count();
        if missing_count > 0 {
            lines.push(format!("Not detected: {}", missing_count));
        }

        let existing_findings = topic
            .findings
            .iter()
            .filter(|finding| finding.exists)
            .collect::<Vec<_>>();
        if existing_findings.is_empty() && !topic.findings.is_empty() {
            lines.push("No active paths detected.".to_string());
        }

        for finding in existing_findings {
            lines.push(format!(
                "- {} | exists={} | {} | {}",
                finding.path,
                finding.exists,
                human_bytes_with_partial(finding.size_bytes, finding.partial),
                finding.reason
            ));
            if finding.partial {
                lines.push(format!(
                    "  Partial size warning: {}",
                    partial_reasons_text(&finding.partial_reasons)
                ));
            }
            if !finding.breakdown.is_empty() {
                lines.push("  Breakdown:".to_string());
                for item in &finding.breakdown {
                    lines.push(format!(
                        "  - {} | {}",
                        item.path,
                        human_bytes(item.size_bytes)
                    ));
                }
            }
        }
        for recommendation in &topic.recommendations {
            lines.push(format!("- Recommendation: {}", recommendation));
        }
        if !topic.probes.is_empty() {
            lines.push("Probes:".to_string());
            for probe in &topic.probes {
                lines.push(format!(
                    "- {} | status={} | {}",
                    probe.name, probe.status, probe.command
                ));
                lines.push(format!("  Summary: {}", probe.summary));
                lines.push(format!("  Output: {}", probe.output));
            }
        }
    }

    lines.join("\n")
}

pub fn render_diff(report: &DiffReport, format: OutputFormat) -> Result<String> {
    let output = match format {
        OutputFormat::Json => serde_json::to_string_pretty(report)?,
        OutputFormat::Markdown => render_diff_markdown(report),
        OutputFormat::Text => render_diff_text(report),
    };

    Ok(output)
}

pub fn render_large_files(report: &LargeFilesReport, format: OutputFormat) -> Result<String> {
    let output = match format {
        OutputFormat::Json => serde_json::to_string_pretty(report)?,
        OutputFormat::Markdown => render_large_files_markdown(report),
        OutputFormat::Text => render_large_files_text(report),
    };

    Ok(output)
}

fn render_large_files_text(report: &LargeFilesReport) -> String {
    let mut lines = vec![
        "Large Files Discovery".to_string(),
        format!("Scan Root: {}", report.scan_root),
        format!("Min Size: {}", report.min_size),
        format!("Scan Time: {}", report.scan_time),
        format!("Entries: {}", report.entries.len()),
        String::new(),
    ];

    for entry in &report.entries {
        lines.push(format!(
            "{} {} {}",
            if entry.is_directory {
                "[DIR]"
            } else {
                "[FILE]"
            },
            human_bytes(entry.size_bytes),
            entry.path
        ));
    }

    lines.join("\n")
}

fn render_large_files_markdown(report: &LargeFilesReport) -> String {
    let mut lines = vec![
        "# Large Files Discovery".to_string(),
        String::new(),
        format!("- Scan Root: `{}`", report.scan_root),
        format!("- Min Size: {}", report.min_size),
        format!("- Scan Time: {}", report.scan_time),
        format!("- Entries: {}", report.entries.len()),
        String::new(),
        "| Type | Size | Path |".to_string(),
        "|---|---|---|".to_string(),
    ];

    for entry in &report.entries {
        lines.push(format!(
            "| {} | {} | `{}` |",
            if entry.is_directory { "DIR" } else { "FILE" },
            human_bytes(entry.size_bytes),
            entry.path
        ));
    }

    lines.join("\n")
}

fn render_diff_text(report: &DiffReport) -> String {
    let mut lines = vec![
        "Windows AI Space Diff".to_string(),
        format!("Generated At: {}", report.generated_at),
        format!("Before: {}", report.before),
        format!("After: {}", report.after),
        format!(
            "Total Growth: {}",
            human_bytes_delta(report.summary.total_growth_bytes)
        ),
        format!(
            "Grew: {}, Shrunk: {}, Appeared: {}, Disappeared: {}",
            report.summary.grew,
            report.summary.shrunk,
            report.summary.appeared,
            report.summary.disappeared
        ),
        String::new(),
        "Changes:".to_string(),
    ];

    for change in &report.changes {
        lines.push(format!(
            "- [{}] {} | before={} after={} delta={}",
            change.change,
            change.path,
            human_bytes(change.before_bytes),
            human_bytes(change.after_bytes),
            human_bytes_delta(change.delta_bytes)
        ));
    }

    lines.join("\n")
}

fn render_diff_markdown(report: &DiffReport) -> String {
    let mut lines = vec![
        "# Windows AI Space Diff".to_string(),
        String::new(),
        format!("- Generated At: {}", report.generated_at),
        format!("- Before: `{}`", report.before),
        format!("- After: `{}`", report.after),
        format!(
            "- Total Growth: {}",
            human_bytes_delta(report.summary.total_growth_bytes)
        ),
        format!(
            "- Grew: {}, Shrunk: {}, Appeared: {}, Disappeared: {}",
            report.summary.grew,
            report.summary.shrunk,
            report.summary.appeared,
            report.summary.disappeared
        ),
        String::new(),
        "## Changes".to_string(),
        String::new(),
        "| Change | Path | Before | After | Delta |".to_string(),
        "|---|---|---:|---:|---:|".to_string(),
    ];

    for change in &report.changes {
        lines.push(format!(
            "| {} | `{}` | {} | {} | {} |",
            change.change,
            change.path,
            human_bytes(change.before_bytes),
            human_bytes(change.after_bytes),
            human_bytes_delta(change.delta_bytes)
        ));
    }

    lines.join("\n")
}

fn human_bytes_delta(bytes: i64) -> String {
    if bytes >= 0 {
        format!("+{}", human_bytes(bytes as u64))
    } else {
        format!("-{}", human_bytes(bytes.unsigned_abs()))
    }
}

fn render_doctor_markdown(report: &DoctorReport) -> String {
    let mut lines = vec![
        "# Windows AI Space Doctor".to_string(),
        String::new(),
        format!("- Generated At: {}", report.generated_at),
    ];
    lines.extend(render_policy_lines(report.policy.as_ref(), true));

    lines.push(String::new());
    lines.extend(render_doctor_executive_summary_lines(report, true));

    if let Some(latest_diff) = &report.latest_diff {
        lines.push(String::new());
        lines.push("## Latest Diff".to_string());
        lines.push(String::new());
        lines.push(format!("- Before: `{}`", latest_diff.before));
        lines.push(format!("- After: `{}`", latest_diff.after));
        lines.push(format!(
            "- Total Growth: {}",
            human_bytes_delta(latest_diff.summary.total_growth_bytes)
        ));
        lines.push(format!(
            "- Counts: `grew={}` `shrunk={}` `appeared={}` `disappeared={}`",
            latest_diff.summary.grew,
            latest_diff.summary.shrunk,
            latest_diff.summary.appeared,
            latest_diff.summary.disappeared
        ));
        lines.push(String::new());
        lines.push("| Change | Path | Before | After | Delta |".to_string());
        lines.push("|---|---|---:|---:|---:|".to_string());
        for change in &latest_diff.top_changes {
            lines.push(format!(
                "| {} | `{}` | {} | {} | {} |",
                change.change,
                change.path,
                human_bytes(change.before_bytes),
                human_bytes(change.after_bytes),
                human_bytes_delta(change.delta_bytes)
            ));
        }
    }

    for topic in &report.topics {
        lines.push(String::new());
        lines.push(format!("## {}", topic.name));
        lines.push(String::new());
        lines.push(format!("- Summary: {}", topic.summary));
        let missing_count = topic
            .findings
            .iter()
            .filter(|finding| !finding.exists)
            .count();
        if missing_count > 0 {
            lines.push(format!("- Not detected: {}", missing_count));
        }

        let existing_findings = topic
            .findings
            .iter()
            .filter(|finding| finding.exists)
            .collect::<Vec<_>>();
        if existing_findings.is_empty() && !topic.findings.is_empty() {
            lines.push(String::new());
            lines.push("No active paths detected.".to_string());
        }

        if !existing_findings.is_empty() {
            lines.push(String::new());
            lines.push("| Path | Exists | Size | Risk | Action |".to_string());
            lines.push("|---|---:|---:|---|---|".to_string());
            for finding in &existing_findings {
                lines.push(format!(
                    "| `{}` | {} | {} | {} | {} |",
                    finding.path,
                    finding.exists,
                    human_bytes_with_partial(finding.size_bytes, finding.partial),
                    finding.risk,
                    finding.action
                ));
                if finding.partial {
                    lines.push(format!(
                        "- Partial size warning for `{}`: {}",
                        finding.path,
                        partial_reasons_text(&finding.partial_reasons)
                    ));
                }
            }
        }
        let breakdown_items = existing_findings
            .iter()
            .flat_map(|finding| {
                finding
                    .breakdown
                    .iter()
                    .map(move |item| (finding.path.as_str(), item.path.as_str(), item.size_bytes))
            })
            .collect::<Vec<_>>();
        if !breakdown_items.is_empty() {
            lines.push(String::new());
            lines.push("### Breakdown".to_string());
            lines.push(String::new());
            lines.push("| Parent | Child | Size |".to_string());
            lines.push("|---|---|---:|".to_string());
            for (parent, child, size_bytes) in breakdown_items {
                lines.push(format!(
                    "| `{}` | `{}` | {} |",
                    parent,
                    child,
                    human_bytes(size_bytes)
                ));
            }
        }
        lines.push(String::new());
        lines.push("Recommendations:".to_string());
        for recommendation in &topic.recommendations {
            lines.push(format!("- {}", recommendation));
        }
        if !topic.probes.is_empty() {
            lines.push(String::new());
            lines.push("### Probes".to_string());
            lines.push(String::new());
            lines.push("| Probe | Status | Command | Summary |".to_string());
            lines.push("|---|---|---|---|".to_string());
            for probe in &topic.probes {
                lines.push(format!(
                    "| {} | {} | `{}` | {} |",
                    probe.name, probe.status, probe.command, probe.summary
                ));
                lines.push(String::new());
                lines.push("Output: ".to_string());
                lines.push("```text".to_string());
                lines.push(probe.output.clone());
                lines.push("```".to_string());
            }
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

fn human_bytes_with_partial(bytes: u64, partial: bool) -> String {
    let size = human_bytes(bytes);
    if partial {
        format!("{size} (partial)")
    } else {
        size
    }
}

fn partial_reasons_text(reasons: &[String]) -> String {
    if reasons.is_empty() {
        "best-effort, not exact; reason unavailable".to_string()
    } else {
        format!("best-effort, not exact; {}", reasons.join("; "))
    }
}

fn render_policy_lines(policy: Option<&PolicySnapshot>, markdown: bool) -> Vec<String> {
    let Some(policy) = policy else {
        return Vec::new();
    };

    let prefix = if markdown { "- " } else { "" };
    vec![
        if markdown {
            "## Policy:".to_string()
        } else {
            "Policy:".to_string()
        },
        format!(
            "{prefix}Sensitive Markers: {}",
            policy.sensitive_markers.join(", ")
        ),
        format!(
            "{prefix}Allowed Actions: {}",
            policy.planner.allow_actions.join(", ")
        ),
        format!(
            "{prefix}Skip Modified Within Minutes: {}",
            policy.planner.skip_modified_within_minutes
        ),
        format!("{prefix}Max Scan Depth: {}", policy.planner.max_scan_depth),
    ]
}

fn render_scan_limit_lines(policy: Option<&PolicySnapshot>, markdown: bool) -> Vec<String> {
    let Some(policy) = policy else {
        return Vec::new();
    };

    let prefix = if markdown { "- " } else { "" };
    vec![format!(
        "{prefix}Scan Limits: max depth {}",
        policy.planner.max_scan_depth
    )]
}

fn render_scan_executive_summary_lines(report: &ScanReport, markdown: bool) -> Vec<String> {
    let protected_bytes = report
        .summary
        .dangerous_bytes
        .saturating_add(report.summary.system_bytes);
    let total = report.summary.total_size_bytes;
    let prefix = if markdown { "- " } else { "" };
    let mut lines = vec![
        if markdown {
            "## Executive Summary".to_string()
        } else {
            "Executive Summary:".to_string()
        },
        String::new(),
        format!(
            "{prefix}Reclaimable now: {}",
            human_bytes(report.summary.reclaimable_safe_bytes)
        ),
        format!(
            "{prefix}Needs review: {}",
            human_bytes(report.summary.review_bytes)
        ),
        format!(
            "{prefix}High risk/system protected: {}",
            human_bytes(protected_bytes)
        ),
        String::new(),
        if markdown {
            "### Risk Distribution".to_string()
        } else {
            "Risk Distribution:".to_string()
        },
    ];

    for (label, bytes) in [
        ("SAFE", report.summary.safe_bytes),
        ("REVIEW", report.summary.review_bytes),
        ("DANGEROUS", report.summary.dangerous_bytes),
        ("SYSTEM", report.summary.system_bytes),
    ] {
        lines.push(format!(
            "{label}: {} {}",
            risk_bar(bytes, total, 12),
            human_bytes(bytes)
        ));
    }

    lines
}

fn render_doctor_executive_summary_lines(report: &DoctorReport, markdown: bool) -> Vec<String> {
    let active_topics = report
        .topics
        .iter()
        .filter(|topic| topic.status == "active")
        .count();
    let not_detected_topics = report
        .topics
        .iter()
        .filter(|topic| topic.status == "not-detected")
        .count();
    let no_rules_topics = report
        .topics
        .iter()
        .filter(|topic| topic.status == "no-rules")
        .count();
    let topic_sizes = report
        .topics
        .iter()
        .map(|topic| {
            let observed_bytes = topic
                .findings
                .iter()
                .filter(|finding| finding.exists)
                .map(|finding| finding.size_bytes)
                .sum::<u64>();
            (topic.name.as_str(), observed_bytes)
        })
        .collect::<Vec<_>>();
    let total = topic_sizes
        .iter()
        .map(|(_, observed_bytes)| *observed_bytes)
        .sum::<u64>();
    let prefix = if markdown { "- " } else { "" };
    let mut lines = vec![
        if markdown {
            "## Executive Summary".to_string()
        } else {
            "Executive Summary:".to_string()
        },
        String::new(),
        format!("{prefix}Active topics: {active_topics}"),
        format!("{prefix}Not detected topics: {not_detected_topics}"),
        format!("{prefix}No-rules topics: {no_rules_topics}"),
        String::new(),
        if markdown {
            "### Topic Size Distribution".to_string()
        } else {
            "Topic Size Distribution:".to_string()
        },
    ];

    for (name, observed_bytes) in topic_sizes {
        lines.push(format!(
            "{name}: {} {}",
            risk_bar(observed_bytes, total, 12),
            human_bytes(observed_bytes)
        ));
    }

    lines
}

fn risk_bar(value: u64, total: u64, width: usize) -> String {
    if width == 0 {
        return String::new();
    }
    if total == 0 || value == 0 {
        return "░".repeat(width);
    }

    let filled = (((value as f64 / total as f64) * width as f64).round() as usize).clamp(1, width);
    format!("{}{}", "█".repeat(filled), "░".repeat(width - filled))
}

#[cfg(test)]
mod tests {
    use chrono::Local;

    use super::render_doctor;
    use crate::doctor::{DoctorBreakdownItem, DoctorFinding, DoctorReport, DoctorTopic};
    use crate::planner::{PlanCandidate, PlanReport, PlanSummary, SkippedItem};
    use crate::policy::{PlannerPolicySnapshot, PolicySnapshot};
    use crate::OutputFormat;

    fn sample_policy_snapshot() -> PolicySnapshot {
        PolicySnapshot {
            sensitive_markers: vec!["token".to_string(), ".env".to_string()],
            planner: PlannerPolicySnapshot {
                allow_actions: vec!["quarantine".to_string(), "report-only".to_string()],
                skip_modified_within_minutes: 15,
                max_scan_depth: 7,
            },
        }
    }

    fn assert_policy_text(output: &str) {
        assert!(output.contains("Policy:"));
        assert!(output.contains("Sensitive Markers: token, .env"));
        assert!(output.contains("Allowed Actions: quarantine, report-only"));
        assert!(output.contains("Skip Modified Within Minutes: 15"));
        assert!(output.contains("Max Scan Depth: 7"));
    }

    fn assert_policy_json(output: &str) {
        let value: serde_json::Value = serde_json::from_str(output).expect("json should parse");
        assert_eq!(value["policy"]["sensitive_markers"][0], "token");
        assert_eq!(value["policy"]["sensitive_markers"][1], ".env");
        assert_eq!(value["policy"]["planner"]["allow_actions"][0], "quarantine");
        assert_eq!(
            value["policy"]["planner"]["allow_actions"][1],
            "report-only"
        );
        assert_eq!(
            value["policy"]["planner"]["skip_modified_within_minutes"],
            15
        );
        assert_eq!(value["policy"]["planner"]["max_scan_depth"], 7);
    }

    #[test]
    fn scan_report_renders_structured_policy_snapshot() {
        let report = crate::scanner::ScanReport {
            scan_time: Local::now(),
            policy: Some(sample_policy_snapshot()),
            volumes: Vec::new(),
            findings: Vec::new(),
            summary: crate::scanner::Summary::default(),
        };

        let text = super::render(&report, OutputFormat::Text).expect("scan text should render");
        let markdown =
            super::render(&report, OutputFormat::Markdown).expect("scan markdown should render");
        let json = super::render(&report, OutputFormat::Json).expect("scan json should render");

        assert_policy_text(&text);
        assert_policy_text(&markdown);
        assert_policy_json(&json);
    }

    #[test]
    fn plan_report_renders_structured_policy_snapshot() {
        let report = PlanReport {
            generated_at: Local::now(),
            mode: "dry-run".to_string(),
            safe_only: true,
            skip_modified_within_minutes: 15,
            policy: Some(sample_policy_snapshot()),
            summary: PlanSummary::default(),
            groups: Vec::new(),
            candidates: Vec::new(),
            skipped: Vec::new(),
        };

        let text =
            super::render_plan(&report, OutputFormat::Text).expect("plan text should render");
        let markdown = super::render_plan(&report, OutputFormat::Markdown)
            .expect("plan markdown should render");
        let json =
            super::render_plan(&report, OutputFormat::Json).expect("plan json should render");

        assert_policy_text(&text);
        assert_policy_text(&markdown);
        assert_policy_json(&json);
    }

    #[test]
    fn doctor_report_renders_structured_policy_snapshot() {
        let report = DoctorReport {
            generated_at: Local::now(),
            policy_summary: "sensitive markers: [token, .env]; planner actions: [quarantine, report-only]; skip modified within: 15min; max scan depth: 7".to_string(),
            policy: Some(sample_policy_snapshot()),
            latest_diff: None,
            topics: Vec::new(),
        };

        let text = render_doctor(&report, OutputFormat::Text).expect("doctor text should render");
        let markdown =
            render_doctor(&report, OutputFormat::Markdown).expect("doctor markdown should render");
        let json = render_doctor(&report, OutputFormat::Json).expect("doctor json should render");

        assert_policy_text(&text);
        assert_policy_text(&markdown);
        assert_policy_json(&json);
    }

    #[test]
    fn doctor_json_keeps_legacy_policy_summary_for_compatibility() {
        let report = DoctorReport {
            generated_at: Local::now(),
            policy_summary: "sensitive markers: [token, .env]; planner actions: [quarantine, report-only]; skip modified within: 15min; max scan depth: 7".to_string(),
            policy: Some(sample_policy_snapshot()),
            latest_diff: None,
            topics: Vec::new(),
        };

        let json = render_doctor(&report, OutputFormat::Json).expect("doctor json should render");
        let value: serde_json::Value = serde_json::from_str(&json).expect("json should parse");

        assert!(value.get("policy").is_some(), "new structured policy field should exist");
        assert_eq!(
            value["policy_summary"],
            "sensitive markers: [token, .env]; planner actions: [quarantine, report-only]; skip modified within: 15min; max scan depth: 7"
        );
    }

    #[test]
    fn scan_text_and_markdown_label_partial_sizes() {
        let report = crate::scanner::ScanReport {
            scan_time: Local::now(),
            policy: None,
            volumes: Vec::new(),
            findings: vec![crate::scanner::Finding {
                id: "partial-cache".to_string(),
                name: "Partial Cache".to_string(),
                category: "test".to_string(),
                path: "C:\\cache".to_string(),
                exists: true,
                size_bytes: 10,
                partial: true,
                partial_reasons: vec!["max scan depth 1 reached".to_string()],
                risk: crate::rules::RiskLevel::Safe,
                action: "quarantine".to_string(),
                reason: "cache".to_string(),
                warnings: Vec::new(),
            }],
            summary: crate::scanner::Summary {
                total_rules: 1,
                matched_paths: 1,
                total_size_bytes: 10,
                safe_bytes: 10,
                review_bytes: 0,
                dangerous_bytes: 0,
                system_bytes: 0,
                top_findings: vec![crate::scanner::TopFinding {
                    id: "partial-cache".to_string(),
                    path: "C:\\cache".to_string(),
                    risk: crate::rules::RiskLevel::Safe,
                    size_bytes: 10,
                    partial: true,
                }],
                reclaimable_safe_bytes: 10,
                partial_findings: 1,
            },
        };

        let text = super::render(&report, OutputFormat::Text).expect("scan text should render");
        let markdown =
            super::render(&report, OutputFormat::Markdown).expect("scan markdown should render");

        assert!(text.contains("Partial Findings: 1"));
        assert!(text.contains("10 B (partial)"));
        assert!(text.contains("max scan depth 1 reached"));
        assert!(markdown.contains("Partial Findings: 1"));
        assert!(markdown.contains("10 B (partial)"));
        assert!(markdown.contains("max scan depth 1 reached"));
    }

    #[test]
    fn scan_text_and_markdown_explain_active_scan_limits() {
        let report = crate::scanner::ScanReport {
            scan_time: Local::now(),
            policy: Some(sample_policy_snapshot()),
            volumes: Vec::new(),
            findings: Vec::new(),
            summary: crate::scanner::Summary {
                total_rules: 1,
                matched_paths: 1,
                total_size_bytes: 10,
                safe_bytes: 10,
                review_bytes: 0,
                dangerous_bytes: 0,
                system_bytes: 0,
                top_findings: Vec::new(),
                reclaimable_safe_bytes: 10,
                partial_findings: 2,
            },
        };

        let text = super::render(&report, OutputFormat::Text).expect("scan text should render");
        let markdown =
            super::render(&report, OutputFormat::Markdown).expect("scan markdown should render");

        assert!(text.contains("Scan Limits: max depth 7"));
        assert!(text.contains("Partial Findings: 2"));
        assert!(markdown.contains("Scan Limits: max depth 7"));
        assert!(markdown.contains("Partial Findings: 2"));
    }

    #[test]
    fn plan_text_and_markdown_preserve_partial_candidates_and_skipped_items() {
        let report = PlanReport {
            generated_at: Local::now(),
            mode: "dry-run".to_string(),
            safe_only: false,
            skip_modified_within_minutes: 0,
            policy: None,
            summary: PlanSummary {
                total_findings: 2,
                eligible_candidates: 1,
                skipped_findings: 1,
                reclaimable_bytes: 10,
                blocked_sensitive_paths: 0,
                skipped_recently_modified: 0,
            },
            groups: Vec::new(),
            candidates: vec![PlanCandidate {
                id: "partial-candidate".to_string(),
                path: "C:\\candidate".to_string(),
                risk: crate::rules::RiskLevel::Safe,
                size_bytes: 10,
                partial: true,
                partial_reasons: vec!["traversal error: denied".to_string()],
                action: "quarantine".to_string(),
                reason: "candidate".to_string(),
            }],
            skipped: vec![SkippedItem {
                id: "partial-skipped".to_string(),
                path: "C:\\skipped".to_string(),
                reason: "filtered".to_string(),
                partial: true,
                partial_reasons: vec!["metadata unavailable: denied".to_string()],
            }],
        };

        let text =
            super::render_plan(&report, OutputFormat::Text).expect("plan text should render");
        let markdown = super::render_plan(&report, OutputFormat::Markdown)
            .expect("plan markdown should render");

        assert!(text.contains("10 B (partial)"));
        assert!(text.contains("best-effort, not exact"));
        assert!(text.contains("traversal error: denied"));
        assert!(text.contains("metadata unavailable: denied"));
        assert!(markdown.contains("10 B (partial)"));
        assert!(markdown.contains("best-effort, not exact"));
        assert!(markdown.contains("traversal error: denied"));
        assert!(markdown.contains("metadata unavailable: denied"));
    }

    #[test]
    fn doctor_text_and_markdown_surface_partial_findings_as_warnings() {
        let report = DoctorReport {
            generated_at: Local::now(),
            policy_summary: String::new(),
            policy: None,
            latest_diff: None,
            topics: vec![DoctorTopic {
                name: "agents".to_string(),
                status: "active".to_string(),
                summary: "1 matching item".to_string(),
                findings: vec![DoctorFinding {
                    id: "partial-agent".to_string(),
                    path: "C:\\agent".to_string(),
                    exists: true,
                    size_bytes: 10,
                    partial: true,
                    partial_reasons: vec!["max scan depth 1 reached".to_string()],
                    risk: "review".to_string(),
                    action: "report-only".to_string(),
                    reason: "agent".to_string(),
                    breakdown: Vec::new(),
                }],
                recommendations: Vec::new(),
                probes: Vec::new(),
            }],
        };

        let text = render_doctor(&report, OutputFormat::Text).expect("doctor text should render");
        let markdown =
            render_doctor(&report, OutputFormat::Markdown).expect("doctor markdown should render");

        assert!(text.contains("Partial size warning"));
        assert!(text.contains("10 B (partial)"));
        assert!(text.contains("best-effort, not exact"));
        assert!(text.contains("max scan depth 1 reached"));
        assert!(markdown.contains("Partial size warning"));
        assert!(markdown.contains("10 B (partial)"));
        assert!(markdown.contains("best-effort, not exact"));
        assert!(markdown.contains("max scan depth 1 reached"));
    }

    #[test]
    fn doctor_markdown_renders_breakdown_items() {
        let report = DoctorReport {
            generated_at: Local::now(),
            policy_summary: String::new(),
            policy: None,
            latest_diff: None,
            topics: vec![DoctorTopic {
                name: "agents".to_string(),
                status: "active".to_string(),
                summary: "1 matching item".to_string(),
                findings: vec![DoctorFinding {
                    id: "claude-home".to_string(),
                    path: "C:\\Users\\demo\\.claude".to_string(),
                    exists: true,
                    size_bytes: 1024,
                    partial: false,
                    partial_reasons: Vec::new(),
                    risk: "review".to_string(),
                    action: "report-only".to_string(),
                    reason: "agent state".to_string(),
                    breakdown: vec![DoctorBreakdownItem {
                        path: "C:\\Users\\demo\\.claude\\cache".to_string(),
                        size_bytes: 512,
                    }],
                }],
                recommendations: vec!["Review cache-like child first.".to_string()],
                probes: Vec::new(),
            }],
        };

        let output = render_doctor(&report, OutputFormat::Markdown).expect("doctor should render");

        assert!(output.contains("Breakdown"));
        assert!(output.contains(".claude\\cache"));
        assert!(output.contains("512 B"));
    }

    #[test]
    fn doctor_markdown_summarizes_missing_findings() {
        let report = DoctorReport {
            generated_at: Local::now(),
            policy_summary: String::new(),
            policy: None,
            latest_diff: None,
            topics: vec![DoctorTopic {
                name: "agents".to_string(),
                status: "active".to_string(),
                summary: "3 matching items".to_string(),
                findings: vec![
                    DoctorFinding {
                        id: "existing".to_string(),
                        path: "C:\\Users\\demo\\.claude".to_string(),
                        exists: true,
                        size_bytes: 1024,
                        partial: false,
                        partial_reasons: Vec::new(),
                        risk: "review".to_string(),
                        action: "report-only".to_string(),
                        reason: "agent state".to_string(),
                        breakdown: Vec::new(),
                    },
                    DoctorFinding {
                        id: "missing-one".to_string(),
                        path: "C:\\Users\\demo\\missing-one".to_string(),
                        exists: false,
                        size_bytes: 0,
                        partial: false,
                        partial_reasons: Vec::new(),
                        risk: "review".to_string(),
                        action: "report-only".to_string(),
                        reason: "not installed".to_string(),
                        breakdown: Vec::new(),
                    },
                    DoctorFinding {
                        id: "missing-two".to_string(),
                        path: "C:\\Users\\demo\\missing-two".to_string(),
                        exists: false,
                        size_bytes: 0,
                        partial: false,
                        partial_reasons: Vec::new(),
                        risk: "review".to_string(),
                        action: "report-only".to_string(),
                        reason: "not installed".to_string(),
                        breakdown: Vec::new(),
                    },
                ],
                recommendations: Vec::new(),
                probes: Vec::new(),
            }],
        };

        let output = render_doctor(&report, OutputFormat::Markdown).expect("doctor should render");

        assert!(output.contains("Not detected: 2"));
        assert!(output.contains(".claude"));
        assert!(!output.contains("missing-one"));
        assert!(!output.contains("missing-two"));
    }

    #[test]
    fn doctor_text_summarizes_missing_findings() {
        let report = DoctorReport {
            generated_at: Local::now(),
            policy_summary: String::new(),
            policy: None,
            latest_diff: None,
            topics: vec![DoctorTopic {
                name: "agents".to_string(),
                status: "active".to_string(),
                summary: "2 matching items".to_string(),
                findings: vec![
                    DoctorFinding {
                        id: "existing".to_string(),
                        path: "C:\\Users\\demo\\.codex".to_string(),
                        exists: true,
                        size_bytes: 512,
                        partial: false,
                        partial_reasons: Vec::new(),
                        risk: "review".to_string(),
                        action: "report-only".to_string(),
                        reason: "agent state".to_string(),
                        breakdown: Vec::new(),
                    },
                    DoctorFinding {
                        id: "missing-cli".to_string(),
                        path: "C:\\Users\\demo\\missing-cli".to_string(),
                        exists: false,
                        size_bytes: 0,
                        partial: false,
                        partial_reasons: Vec::new(),
                        risk: "review".to_string(),
                        action: "report-only".to_string(),
                        reason: "not installed".to_string(),
                        breakdown: Vec::new(),
                    },
                ],
                recommendations: Vec::new(),
                probes: Vec::new(),
            }],
        };

        let output = render_doctor(&report, OutputFormat::Text).expect("doctor should render");

        assert!(output.contains("Not detected: 1"));
        assert!(output.contains(".codex"));
        assert!(!output.contains("missing-cli"));
    }

    #[test]
    fn doctor_json_preserves_missing_findings() {
        let report = DoctorReport {
            generated_at: Local::now(),
            policy_summary: String::new(),
            policy: None,
            latest_diff: None,
            topics: vec![DoctorTopic {
                name: "agents".to_string(),
                status: "not-detected".to_string(),
                summary: "1 matching item".to_string(),
                findings: vec![DoctorFinding {
                    id: "missing-json".to_string(),
                    path: "C:\\Users\\demo\\missing-json".to_string(),
                    exists: false,
                    size_bytes: 0,
                    partial: false,
                    partial_reasons: Vec::new(),
                    risk: "review".to_string(),
                    action: "report-only".to_string(),
                    reason: "not installed".to_string(),
                    breakdown: Vec::new(),
                }],
                recommendations: Vec::new(),
                probes: Vec::new(),
            }],
        };

        let output = render_doctor(&report, OutputFormat::Json).expect("doctor should render");

        assert!(output.contains("missing-json"));
        assert!(output.contains("\"exists\": false"));
    }

    #[test]
    fn doctor_markdown_renders_probe_section() {
        let report = DoctorReport {
            generated_at: Local::now(),
            policy_summary: String::new(),
            policy: None,
            latest_diff: None,
            topics: vec![DoctorTopic {
                name: "docker".to_string(),
                status: "active".to_string(),
                summary: "1 matching item".to_string(),
                findings: vec![DoctorFinding {
                    id: "docker-root".to_string(),
                    path: "C:\\Users\\demo\\AppData\\Local\\Docker".to_string(),
                    exists: true,
                    size_bytes: 2048,
                    partial: false,
                    partial_reasons: Vec::new(),
                    risk: "review".to_string(),
                    action: "report-only".to_string(),
                    reason: "docker state".to_string(),
                    breakdown: Vec::new(),
                }],
                recommendations: vec!["Review docker usage first.".to_string()],
                probes: vec![crate::doctor::DoctorProbe {
                    name: "docker-system-df".to_string(),
                    status: "ok".to_string(),
                    command: "docker system df".to_string(),
                    summary: "docker-system-df probe status: ok".to_string(),
                    output: "TYPE TOTAL ACTIVE SIZE RECLAIMABLE".to_string(),
                }],
            }],
        };

        let output = render_doctor(&report, OutputFormat::Markdown).expect("doctor should render");

        assert!(output.contains("### Probes"));
        assert!(output.contains("docker-system-df"));
        assert!(output.contains("docker system df"));
        assert!(output.contains("RECLAIMABLE"));
        assert!(output.contains("```text"));
        assert!(!output.contains("| output |  |  |"));
    }

    #[test]
    fn doctor_markdown_renders_latest_diff_section() {
        let report = DoctorReport {
            generated_at: Local::now(),
            policy_summary: String::new(),
            policy: None,
            latest_diff: Some(crate::doctor::DoctorLatestDiff {
                before: "before.json".to_string(),
                after: "after.json".to_string(),
                summary: crate::doctor::DoctorLatestDiffSummary {
                    total_growth_bytes: 120,
                    grew: 1,
                    shrunk: 0,
                    appeared: 0,
                    disappeared: 0,
                },
                top_changes: vec![crate::doctor::DoctorLatestDiffEntry {
                    path: "C:\\demo\\.claude".to_string(),
                    change: "grew".to_string(),
                    before_bytes: 100,
                    after_bytes: 220,
                    delta_bytes: 120,
                }],
            }),
            topics: Vec::new(),
        };

        let output = render_doctor(&report, OutputFormat::Markdown).expect("doctor should render");

        assert!(output.contains("## Latest Diff"));
        assert!(output.contains("before.json"));
        assert!(output.contains("after.json"));
        assert!(output.contains("+120 B"));
        assert!(output.contains("grew=1"));
        assert!(output.contains("shrunk=0"));
        assert!(output.contains("appeared=0"));
        assert!(output.contains("disappeared=0"));
        assert!(output.contains("C:\\demo\\.claude"));
    }

    #[test]
    fn doctor_text_renders_latest_diff_summary_counters() {
        let report = DoctorReport {
            generated_at: Local::now(),
            policy_summary: String::new(),
            policy: None,
            latest_diff: Some(crate::doctor::DoctorLatestDiff {
                before: "before.json".to_string(),
                after: "after.json".to_string(),
                summary: crate::doctor::DoctorLatestDiffSummary {
                    total_growth_bytes: 120,
                    grew: 2,
                    shrunk: 1,
                    appeared: 3,
                    disappeared: 4,
                },
                top_changes: vec![crate::doctor::DoctorLatestDiffEntry {
                    path: "C:\\demo\\.claude".to_string(),
                    change: "grew".to_string(),
                    before_bytes: 100,
                    after_bytes: 220,
                    delta_bytes: 120,
                }],
            }),
            topics: Vec::new(),
        };

        let output = render_doctor(&report, OutputFormat::Text).expect("doctor should render");

        assert!(output.contains("grew=2"));
        assert!(output.contains("shrunk=1"));
        assert!(output.contains("appeared=3"));
        assert!(output.contains("disappeared=4"));
    }

    #[test]
    fn scan_markdown_renders_executive_summary_and_unicode_risk_bars() {
        let report = crate::scanner::ScanReport {
            scan_time: Local::now(),
            policy: None,
            volumes: Vec::new(),
            findings: Vec::new(),
            summary: crate::scanner::Summary {
                total_rules: 3,
                matched_paths: 2,
                total_size_bytes: 10,
                safe_bytes: 6,
                review_bytes: 3,
                dangerous_bytes: 1,
                system_bytes: 0,
                top_findings: Vec::new(),
                reclaimable_safe_bytes: 6,
                partial_findings: 0,
            },
        };

        let output =
            super::render(&report, OutputFormat::Markdown).expect("scan markdown should render");

        assert!(output.contains("## Executive Summary"));
        assert!(output.contains("Reclaimable now: 6 B"));
        assert!(output.contains("Risk Distribution"));
        assert!(output.contains("SAFE"));
        assert!(output.contains("REVIEW"));
        assert!(output.contains("DANGEROUS"));
        assert!(output.contains('█'));
    }

    #[test]
    fn scan_json_does_not_gain_executive_summary_text() {
        let report = crate::scanner::ScanReport {
            scan_time: Local::now(),
            policy: None,
            volumes: Vec::new(),
            findings: Vec::new(),
            summary: crate::scanner::Summary::default(),
        };

        let output = super::render(&report, OutputFormat::Json).expect("scan json should render");

        assert!(!output.contains("Executive Summary"));
        assert!(!output.contains('█'));
    }

    #[test]
    fn doctor_markdown_renders_executive_summary_and_topic_bars() {
        let report = DoctorReport {
            generated_at: Local::now(),
            policy_summary: String::new(),
            policy: None,
            latest_diff: None,
            topics: vec![
                DoctorTopic {
                    name: "agents".to_string(),
                    status: "active".to_string(),
                    summary: "2 matching items".to_string(),
                    findings: vec![DoctorFinding {
                        id: "agent".to_string(),
                        path: "C:\\Users\\demo\\.claude".to_string(),
                        exists: true,
                        size_bytes: 6,
                        partial: false,
                        partial_reasons: Vec::new(),
                        risk: "review".to_string(),
                        action: "report-only".to_string(),
                        reason: "agent state".to_string(),
                        breakdown: Vec::new(),
                    }],
                    recommendations: Vec::new(),
                    probes: Vec::new(),
                },
                DoctorTopic {
                    name: "docker".to_string(),
                    status: "not-detected".to_string(),
                    summary: "not found".to_string(),
                    findings: Vec::new(),
                    recommendations: Vec::new(),
                    probes: Vec::new(),
                },
            ],
        };

        let output =
            render_doctor(&report, OutputFormat::Markdown).expect("doctor markdown should render");

        assert!(output.contains("## Executive Summary"));
        assert!(output.contains("Active topics: 1"));
        assert!(output.contains("Not detected topics: 1"));
        assert!(output.contains("Topic Size Distribution"));
        assert!(output.contains("agents"));
        assert!(output.contains('█'));
    }

    #[test]
    fn doctor_json_does_not_gain_executive_summary_text() {
        let report = DoctorReport {
            generated_at: Local::now(),
            policy_summary: String::new(),
            policy: None,
            latest_diff: None,
            topics: Vec::new(),
        };

        let output = render_doctor(&report, OutputFormat::Json).expect("doctor json should render");

        assert!(!output.contains("Executive Summary"));
        assert!(!output.contains('█'));
    }
}
