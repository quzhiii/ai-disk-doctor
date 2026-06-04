use std::fs;
use std::io::ErrorKind;
use std::path::Path;
use std::process::Command;

use chrono::{DateTime, Local};
use serde::Serialize;
use walkdir::WalkDir;

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
    pub status: String,
    pub summary: String,
    pub findings: Vec<DoctorFinding>,
    pub recommendations: Vec<String>,
    pub probes: Vec<DoctorProbe>,
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
    pub breakdown: Vec<DoctorBreakdownItem>,
}

#[derive(Debug, Serialize)]
pub struct DoctorBreakdownItem {
    pub path: String,
    pub size_bytes: u64,
}

#[derive(Debug, Serialize)]
pub struct DoctorProbe {
    pub name: String,
    pub status: String,
    pub command: String,
    pub summary: String,
    pub output: String,
}

#[derive(Debug, Clone)]
pub struct ProbeCommandResult {
    pub status: String,
    pub output: String,
}

#[derive(Debug, Clone, Copy)]
pub struct DoctorOptions {
    pub docker: bool,
    pub wsl: bool,
    pub ollama: bool,
    pub playwright: bool,
    pub huggingface: bool,
    pub agents: bool,
    pub probe_tools: bool,
}

pub fn build_doctor(
    scan_report: &ScanReport,
    options: DoctorOptions,
    policy: &Policy,
) -> DoctorReport {
    build_doctor_with_probe_runner(scan_report, options, policy, &default_probe_runner)
}

fn build_doctor_with_probe_runner<F>(
    scan_report: &ScanReport,
    options: DoctorOptions,
    policy: &Policy,
    probe_runner: &F,
) -> DoctorReport
where
    F: Fn(&str, &[&str]) -> ProbeCommandResult,
{
    let mut topics = Vec::new();
    let policy_summary = format!(
        "sensitive markers: [{}]; planner actions: [{}]; skip modified within: {}min",
        policy.sensitive_markers.join(", "),
        policy.planner.allow_actions.join(", "),
        policy.planner.skip_modified_within_minutes
    );

    if options.docker {
        topics.push(with_topic_probes(
            build_topic(
            "docker",
            scan_report,
            |finding| finding.category == "docker",
            vec![
                "Use `docker system df` to inspect image, cache, and volume usage.".to_string(),
                "Prefer `docker builder prune` and targeted prune commands over filesystem deletion.".to_string(),
                "Treat Docker virtual disk files and volume storage as report-only assets.".to_string(),
            ],
            ),
            options.probe_tools,
            probe_runner,
        ));
    }

    if options.wsl {
        topics.push(with_topic_probes(
            build_topic(
            "wsl",
            scan_report,
            |finding| finding.category == "wsl",
            vec![
                "Do not delete ext4.vhdx directly; use WSL export, compact, or relocation workflows.".to_string(),
                "Check whether large distros should be exported and re-imported to another drive.".to_string(),
            ],
            ),
            options.probe_tools,
            probe_runner,
        ));
    }

    if options.ollama {
        topics.push(with_topic_probes(
            build_topic(
            "ollama",
            scan_report,
            |finding| finding.id == "ollama-models",
            vec![
                "Run `ollama list` before cleanup so model names and sizes are explicit.".to_string(),
                "Remove unused models through model-aware commands instead of deleting blob paths directly.".to_string(),
            ],
            ),
            options.probe_tools,
            probe_runner,
        ));
    }

    if options.huggingface {
        topics.push(with_topic_probes(
            build_topic(
            "huggingface",
            scan_report,
            |finding| finding.id == "huggingface-cache",
            vec![
                "Review which projects share this cache before deleting reusable downloads."
                    .to_string(),
                "Prefer targeted cache cleanup or relocation over wiping the cache root blindly."
                    .to_string(),
            ],
            ),
            options.probe_tools,
            probe_runner,
        ));
    }

    if options.playwright {
        topics.push(with_topic_probes(
            build_topic(
            "playwright",
            scan_report,
            |finding| finding.id == "playwright-project-browsers" || finding.path.contains("ms-playwright"),
            vec![
                "Check whether browsers are being downloaded per project instead of via a shared cache.".to_string(),
                "If browser downloads are repeated, centralize Playwright browser storage before deleting caches.".to_string(),
            ],
            ),
            options.probe_tools,
            probe_runner,
        ));
    }

    if options.agents {
        topics.push(with_topic_probes(
            build_topic(
            "agents",
            scan_report,
            is_ai_tooling_finding,
            vec![
                "Treat AI agent roots as review-only because they may contain chat history, sessions, settings, and cached tool state.".to_string(),
                "Use the breakdown to identify cache-like children before planning cleanup; do not delete agent roots blindly.".to_string(),
            ],
            ),
            options.probe_tools,
            probe_runner,
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
            breakdown: if finding.exists {
                build_breakdown(Path::new(&finding.path), 5, 20).unwrap_or_default()
            } else {
                Vec::new()
            },
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
            findings.len(),
            name
        )
    } else {
        format!(
            "{} matching items, {} existing, total observed size {} bytes",
            findings.len(),
            existing_count,
            total_bytes
        )
    };

    let status = if findings.is_empty() {
        "no-rules"
    } else if existing_count == 0 {
        "not-detected"
    } else {
        "active"
    };

    let recommendations = enrich_recommendations(name, &findings, recommendations);

    DoctorTopic {
        name: name.to_string(),
        status: status.to_string(),
        summary,
        findings,
        recommendations,
        probes: Vec::new(),
    }
}

fn with_topic_probes<F>(
    mut topic: DoctorTopic,
    probe_tools: bool,
    probe_runner: &F,
) -> DoctorTopic
where
    F: Fn(&str, &[&str]) -> ProbeCommandResult,
{
    if probe_tools {
        topic.probes = build_topic_probes(&topic, probe_runner);
    }

    topic
}

fn build_topic_probes<F>(topic: &DoctorTopic, probe_runner: &F) -> Vec<DoctorProbe>
where
    F: Fn(&str, &[&str]) -> ProbeCommandResult,
{
    if topic.status == "no-rules" {
        return Vec::new();
    }

    let Some((name, program, args)) = topic_probe_command(&topic.name) else {
        return Vec::new();
    };

    let result = probe_runner(program, args);
    vec![DoctorProbe {
        name: name.to_string(),
        status: result.status.clone(),
        command: format!("{} {}", program, args.join(" ")),
        summary: format!("{} probe status: {}", name, result.status),
        output: result.output,
    }]
}

fn topic_probe_command(name: &str) -> Option<(&'static str, &'static str, &'static [&'static str])> {
    match name {
        "docker" => Some(("docker-system-df", "docker", &["system", "df"])),
        "wsl" => Some(("wsl-list-verbose", "wsl", &["--list", "--verbose"])),
        "ollama" => Some(("ollama-list", "ollama", &["list"])),
        _ => None,
    }
}

fn default_probe_runner(program: &str, args: &[&str]) -> ProbeCommandResult {
    match Command::new(program).args(args).output() {
        Ok(output) => {
            let mut combined = decode_probe_output_bytes(&output.stdout);
            let stderr = decode_probe_output_bytes(&output.stderr).trim().to_string();
            if combined.trim().is_empty() && !stderr.is_empty() {
                combined = stderr;
            }

            ProbeCommandResult {
                status: if output.status.success() {
                    "ok".to_string()
                } else {
                    "failed".to_string()
                },
                output: truncate_probe_output(&combined),
            }
        }
        Err(error) if error.kind() == ErrorKind::NotFound => ProbeCommandResult {
            status: "not-available".to_string(),
            output: format!("{} command not found", program),
        },
        Err(error) => ProbeCommandResult {
            status: "error".to_string(),
            output: truncate_probe_output(&error.to_string()),
        },
    }
}

fn truncate_probe_output(output: &str) -> String {
    let normalized = output.replace("\r\n", "\n").trim().to_string();
    if normalized.is_empty() {
        return "(no output)".to_string();
    }

    let max_chars = 2000;
    if normalized.chars().count() <= max_chars {
        return normalized;
    }

    let truncated = normalized.chars().take(max_chars).collect::<String>();
    format!("{}\n...(truncated)", truncated)
}

fn decode_probe_output_bytes(bytes: &[u8]) -> String {
    if bytes.len() >= 2 && bytes.len() % 2 == 0 {
        let utf16 = bytes
            .chunks_exact(2)
            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
            .collect::<Vec<_>>();
        if utf16.first() == Some(&0xfeff)
            || utf16.iter().skip(1).any(|unit| *unit == 0)
            || utf16.iter().all(|unit| *unit <= 0x007f)
        {
            if let Ok(decoded) = String::from_utf16(&utf16) {
                return decoded;
            }
        }
    }

    String::from_utf8_lossy(bytes).into_owned()
}

fn enrich_recommendations(
    name: &str,
    findings: &[DoctorFinding],
    mut recommendations: Vec<String>,
) -> Vec<String> {
    let existing_count = findings.iter().filter(|finding| finding.exists).count();
    let total_bytes = findings
        .iter()
        .filter(|finding| finding.exists)
        .map(|finding| finding.size_bytes)
        .sum::<u64>();

    if existing_count == 0 {
        return vec![format!(
            "No active {} paths were detected. If usage is expected, re-run doctor after reproducing the workload.",
            name
        )];
    } else if total_bytes <= 1024 * 1024 {
        recommendations.insert(
            0,
            format!(
                "Detected {} active {} bytes in {}. This is tiny; no action needed unless it grows later.",
                existing_count, total_bytes, name
            ),
        );
    } else if total_bytes >= 1024 * 1024 * 1024 {
        recommendations.insert(
            0,
            format!(
                "{} is large at {} bytes. Compare recent snapshots with `aidisk diff --latest` before cleanup decisions.",
                name, total_bytes
            ),
        );
    }

    if findings.iter().any(has_cache_like_child) {
        recommendations.insert(
            0,
            "A cache-like child appears in the breakdown. Review that child first instead of deleting the whole root."
                .to_string(),
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

fn is_ai_tooling_finding(finding: &Finding) -> bool {
    matches!(
        finding.category.as_str(),
        "ai-agent"
            | "ai-ide"
            | "ai-cli"
            | "ai-cache"
            | "ai-installer"
            | "ai-installed-app"
            | "ai-test-artifact"
    ) || finding.id.contains("agent")
}

fn has_cache_like_child(finding: &DoctorFinding) -> bool {
    finding.breakdown.iter().any(|item| {
        let path = item.path.to_ascii_lowercase();
        path.contains("cache")
            || path.contains("tmp")
            || path.contains("temp")
            || path.contains("blob")
            || path.contains("artifact")
    })
}

fn build_breakdown(
    path: &Path,
    max_items: usize,
    max_depth: usize,
) -> std::io::Result<Vec<DoctorBreakdownItem>> {
    if !path.is_dir() || max_items == 0 {
        return Ok(Vec::new());
    }

    let mut items = Vec::new();
    for entry in fs::read_dir(path)? {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        let child_path = entry.path();
        let size_bytes = compute_breakdown_size(&child_path, max_depth);
        items.push(DoctorBreakdownItem {
            path: child_path.display().to_string(),
            size_bytes,
        });
    }

    items.sort_by(|a, b| {
        b.size_bytes
            .cmp(&a.size_bytes)
            .then_with(|| a.path.cmp(&b.path))
    });
    items.truncate(max_items);
    Ok(items)
}

fn compute_breakdown_size(path: &Path, max_depth: usize) -> u64 {
    let metadata = match fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(_) => return 0,
    };

    if metadata.is_file() {
        return metadata.len();
    }

    let mut total = 0_u64;
    for entry in WalkDir::new(path).follow_links(false).max_depth(max_depth) {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        let metadata = match entry.metadata() {
            Ok(metadata) => metadata,
            Err(_) => continue,
        };
        if metadata.is_file() {
            total = total.saturating_add(metadata.len());
        }
    }

    total
}

#[cfg(test)]
mod tests {
    use std::fs;

    use chrono::Local;
    use tempfile::tempdir;

    use super::{
        build_breakdown, build_doctor, build_doctor_with_probe_runner,
        decode_probe_output_bytes, DoctorOptions, ProbeCommandResult,
    };
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
                allow_actions: vec![
                    "quarantine".to_string(),
                    "report-only".to_string(),
                    "guide".to_string(),
                ],
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
                agents: false,
                probe_tools: false,
            },
            &test_policy(),
        );

        assert_eq!(report.topics.len(), 1);
        assert_eq!(report.topics[0].status, "no-rules");
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
                path: "C:\\Users\\demo\\AppData\\Local\\Packages\\X\\LocalState\\ext4.vhdx"
                    .to_string(),
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
                agents: false,
                probe_tools: false,
            },
            &test_policy(),
        );

        assert_eq!(doctor.topics[0].status, "not-detected");
        assert!(doctor.topics[0]
            .summary
            .contains("no existing paths were found"));
    }

    #[test]
    fn doctor_not_detected_topic_does_not_emit_generic_tool_advice() {
        let report = ScanReport {
            scan_time: Local::now(),
            volumes: Vec::<Volume>::new(),
            findings: vec![Finding {
                id: "docker-root".to_string(),
                name: "Docker root".to_string(),
                category: "docker".to_string(),
                path: "C:\\Users\\demo\\AppData\\Local\\Docker".to_string(),
                exists: false,
                size_bytes: 0,
                risk: RiskLevel::Review,
                action: "report-only".to_string(),
                reason: "docker".to_string(),
                warnings: Vec::new(),
            }],
            summary: Summary::default(),
        };

        let doctor = build_doctor(
            &report,
            DoctorOptions {
                docker: true,
                wsl: false,
                ollama: false,
                playwright: false,
                huggingface: false,
                agents: false,
                probe_tools: false,
            },
            &test_policy(),
        );

        assert_eq!(doctor.topics[0].status, "not-detected");
        assert!(doctor.topics[0]
            .recommendations
            .iter()
            .any(|recommendation| recommendation.contains("No active docker paths")));
        assert!(!doctor.topics[0]
            .recommendations
            .iter()
            .any(|recommendation| recommendation.contains("docker system df")));
    }

    #[test]
    fn doctor_does_not_probe_tools_by_default() {
        let report = ScanReport {
            scan_time: Local::now(),
            volumes: Vec::<Volume>::new(),
            findings: vec![Finding {
                id: "docker-root".to_string(),
                name: "Docker root".to_string(),
                category: "docker".to_string(),
                path: "C:\\Users\\demo\\AppData\\Local\\Docker".to_string(),
                exists: true,
                size_bytes: 100,
                risk: RiskLevel::Review,
                action: "report-only".to_string(),
                reason: "docker".to_string(),
                warnings: Vec::new(),
            }],
            summary: Summary::default(),
        };

        let doctor = build_doctor_with_probe_runner(
            &report,
            DoctorOptions {
                docker: true,
                wsl: false,
                ollama: false,
                playwright: false,
                huggingface: false,
                agents: false,
                probe_tools: false,
            },
            &test_policy(),
            &|_, _| panic!("probe runner should not be called unless probe_tools is enabled"),
        );

        assert!(doctor.topics[0].probes.is_empty());
    }

    #[test]
    fn doctor_runs_docker_probe_when_enabled() {
        let report = ScanReport {
            scan_time: Local::now(),
            volumes: Vec::<Volume>::new(),
            findings: vec![Finding {
                id: "docker-root".to_string(),
                name: "Docker root".to_string(),
                category: "docker".to_string(),
                path: "C:\\Users\\demo\\AppData\\Local\\Docker".to_string(),
                exists: true,
                size_bytes: 100,
                risk: RiskLevel::Review,
                action: "report-only".to_string(),
                reason: "docker".to_string(),
                warnings: Vec::new(),
            }],
            summary: Summary::default(),
        };

        let doctor = build_doctor_with_probe_runner(
            &report,
            DoctorOptions {
                docker: true,
                wsl: false,
                ollama: false,
                playwright: false,
                huggingface: false,
                agents: false,
                probe_tools: true,
            },
            &test_policy(),
            &|program, args| {
                assert_eq!(program, "docker");
                assert_eq!(args, &["system", "df"]);
                ProbeCommandResult {
                    status: "ok".to_string(),
                    output: "TYPE TOTAL ACTIVE SIZE RECLAIMABLE".to_string(),
                }
            },
        );

        assert_eq!(doctor.topics[0].probes.len(), 1);
        assert_eq!(doctor.topics[0].probes[0].name, "docker-system-df");
        assert_eq!(doctor.topics[0].probes[0].status, "ok");
        assert!(doctor.topics[0].probes[0].command.contains("docker system df"));
        assert!(doctor.topics[0].probes[0].output.contains("RECLAIMABLE"));
    }

    #[test]
    fn doctor_records_probe_command_unavailable_without_failing() {
        let report = ScanReport {
            scan_time: Local::now(),
            volumes: Vec::<Volume>::new(),
            findings: vec![Finding {
                id: "ollama-models".to_string(),
                name: "Ollama model store".to_string(),
                category: "models".to_string(),
                path: "C:\\Users\\demo\\.ollama\\models".to_string(),
                exists: false,
                size_bytes: 0,
                risk: RiskLevel::Review,
                action: "guide".to_string(),
                reason: "ollama".to_string(),
                warnings: Vec::new(),
            }],
            summary: Summary::default(),
        };

        let doctor = build_doctor_with_probe_runner(
            &report,
            DoctorOptions {
                docker: false,
                wsl: false,
                ollama: true,
                playwright: false,
                huggingface: false,
                agents: false,
                probe_tools: true,
            },
            &test_policy(),
            &|program, args| {
                assert_eq!(program, "ollama");
                assert_eq!(args, &["list"]);
                ProbeCommandResult {
                    status: "not-available".to_string(),
                    output: "ollama command not found".to_string(),
                }
            },
        );

        assert_eq!(doctor.topics[0].probes.len(), 1);
        assert_eq!(doctor.topics[0].probes[0].status, "not-available");
        assert!(doctor.topics[0].probes[0]
            .summary
            .contains("not-available"));
    }

    #[test]
    fn decode_probe_output_bytes_handles_utf16le_console_output() {
        let bytes = "NAME\r\nUbuntu".encode_utf16().flat_map(|unit| unit.to_le_bytes()).collect::<Vec<_>>();

        let decoded = decode_probe_output_bytes(&bytes);

        assert!(decoded.contains("NAME"));
        assert!(decoded.contains("Ubuntu"));
        assert!(!decoded.contains('\0'));
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
                agents: false,
                probe_tools: false,
            },
            &test_policy(),
        );

        assert_eq!(doctor.topics.len(), 2);
        assert_eq!(doctor.topics[0].name, "ollama");
        assert_eq!(doctor.topics[1].name, "huggingface");
    }

    #[test]
    fn doctor_can_report_ai_agent_topic() {
        let agent_findings = [
            ("C:\\Users\\demo\\.gemini", 8_000),
            ("C:\\Users\\demo\\.claude", 2_000),
            ("C:\\Users\\demo\\.codex", 1_000),
            ("C:\\Users\\demo\\AppData\\Local\\opencode", 500),
        ];
        let report = ScanReport {
            scan_time: Local::now(),
            volumes: Vec::<Volume>::new(),
            findings: agent_findings
                .into_iter()
                .map(|(path, size_bytes)| Finding {
                    id: "claude-home".to_string(),
                    name: "AI agent state".to_string(),
                    category: "ai-agent".to_string(),
                    path: path.to_string(),
                    exists: true,
                    size_bytes,
                    risk: RiskLevel::Review,
                    action: "report-only".to_string(),
                    reason: "agent state".to_string(),
                    warnings: Vec::new(),
                })
                .collect(),
            summary: Summary::default(),
        };

        let doctor = build_doctor(
            &report,
            DoctorOptions {
                docker: false,
                wsl: false,
                ollama: false,
                playwright: false,
                huggingface: false,
                agents: true,
                probe_tools: false,
            },
            &test_policy(),
        );

        assert_eq!(doctor.topics.len(), 1);
        assert_eq!(doctor.topics[0].name, "agents");
        assert_eq!(doctor.topics[0].status, "active");
        assert!(doctor.topics[0].summary.contains("4 matching items"));
        assert!(doctor.topics[0].summary.contains("11500 bytes"));
    }

    #[test]
    fn doctor_agents_topic_includes_ai_tooling_categories() {
        let categories = [
            "ai-agent",
            "ai-ide",
            "ai-cli",
            "ai-cache",
            "ai-installer",
            "ai-installed-app",
            "ai-test-artifact",
        ];
        let report = ScanReport {
            scan_time: Local::now(),
            volumes: Vec::<Volume>::new(),
            findings: categories
                .into_iter()
                .enumerate()
                .map(|(index, category)| Finding {
                    id: format!("{category}-demo"),
                    name: "AI tooling demo".to_string(),
                    category: category.to_string(),
                    path: format!("C:\\Users\\demo\\{category}"),
                    exists: true,
                    size_bytes: (index as u64) + 1,
                    risk: RiskLevel::Review,
                    action: "report-only".to_string(),
                    reason: "ai tooling state".to_string(),
                    warnings: Vec::new(),
                })
                .collect(),
            summary: Summary::default(),
        };

        let doctor = build_doctor(
            &report,
            DoctorOptions {
                docker: false,
                wsl: false,
                ollama: false,
                playwright: false,
                huggingface: false,
                agents: true,
                probe_tools: false,
            },
            &test_policy(),
        );

        assert_eq!(doctor.topics[0].findings.len(), categories.len());
        assert!(doctor.topics[0].summary.contains("7 matching items"));
    }

    #[test]
    fn breakdown_returns_top_direct_children_by_size() {
        let temp = tempdir().expect("tempdir should exist");
        let root = temp.path();
        fs::create_dir_all(root.join("large")).expect("large dir should exist");
        fs::create_dir_all(root.join("small")).expect("small dir should exist");
        fs::write(root.join("large").join("data.bin"), vec![0_u8; 12])
            .expect("large file should write");
        fs::write(root.join("small").join("data.bin"), vec![0_u8; 3])
            .expect("small file should write");
        fs::write(root.join("middle.log"), vec![0_u8; 7]).expect("middle file should write");

        let breakdown = build_breakdown(root, 2, 20).expect("breakdown should build");

        assert_eq!(breakdown.len(), 2);
        assert!(breakdown[0].path.ends_with("large"));
        assert_eq!(breakdown[0].size_bytes, 12);
        assert!(breakdown[1].path.ends_with("middle.log"));
        assert_eq!(breakdown[1].size_bytes, 7);
    }

    #[test]
    fn doctor_includes_breakdown_for_existing_agent_roots() {
        let temp = tempdir().expect("tempdir should exist");
        let root = temp.path().join(".gemini");
        fs::create_dir_all(root.join("sessions")).expect("sessions dir should exist");
        fs::create_dir_all(root.join("cache")).expect("cache dir should exist");
        fs::write(root.join("sessions").join("history.jsonl"), vec![0_u8; 15])
            .expect("session file should write");
        fs::write(root.join("cache").join("artifact.bin"), vec![0_u8; 6])
            .expect("cache file should write");

        let report = ScanReport {
            scan_time: Local::now(),
            volumes: Vec::<Volume>::new(),
            findings: vec![Finding {
                id: "claude-home".to_string(),
                name: "AI agent state".to_string(),
                category: "ai-agent".to_string(),
                path: root.display().to_string(),
                exists: true,
                size_bytes: 21,
                risk: RiskLevel::Review,
                action: "report-only".to_string(),
                reason: "agent state".to_string(),
                warnings: Vec::new(),
            }],
            summary: Summary::default(),
        };

        let doctor = build_doctor(
            &report,
            DoctorOptions {
                docker: false,
                wsl: false,
                ollama: false,
                playwright: false,
                huggingface: false,
                agents: true,
                probe_tools: false,
            },
            &test_policy(),
        );

        let breakdown = &doctor.topics[0].findings[0].breakdown;
        assert_eq!(breakdown.len(), 2);
        assert!(breakdown[0].path.ends_with("sessions"));
        assert_eq!(breakdown[0].size_bytes, 15);
    }

    #[test]
    fn doctor_recommends_no_action_for_tiny_existing_cache() {
        let temp = tempdir().expect("tempdir should exist");
        let cache = temp.path().join("huggingface");
        fs::write(&cache, [0_u8]).expect("tiny cache placeholder should write");
        let report = ScanReport {
            scan_time: Local::now(),
            volumes: Vec::<Volume>::new(),
            findings: vec![Finding {
                id: "huggingface-cache".to_string(),
                name: "Hugging Face cache".to_string(),
                category: "models".to_string(),
                path: cache.display().to_string(),
                exists: true,
                size_bytes: 1,
                risk: RiskLevel::Review,
                action: "guide".to_string(),
                reason: "hf".to_string(),
                warnings: Vec::new(),
            }],
            summary: Summary::default(),
        };

        let doctor = build_doctor(
            &report,
            DoctorOptions {
                docker: false,
                wsl: false,
                ollama: false,
                playwright: false,
                huggingface: true,
                agents: false,
                probe_tools: false,
            },
            &test_policy(),
        );

        assert!(doctor.topics[0]
            .recommendations
            .iter()
            .any(|recommendation| recommendation.contains("no action needed")));
    }

    #[test]
    fn doctor_recommends_review_for_large_cache_like_agent_child() {
        let temp = tempdir().expect("tempdir should exist");
        let root = temp.path().join(".claude");
        fs::create_dir_all(root.join("cache")).expect("cache dir should exist");
        fs::write(root.join("cache").join("artifact.bin"), vec![0_u8; 12])
            .expect("cache artifact should write");
        let report = ScanReport {
            scan_time: Local::now(),
            volumes: Vec::<Volume>::new(),
            findings: vec![Finding {
                id: "claude-home".to_string(),
                name: "AI agent state".to_string(),
                category: "ai-agent".to_string(),
                path: root.display().to_string(),
                exists: true,
                size_bytes: 2 * 1024 * 1024 * 1024,
                risk: RiskLevel::Review,
                action: "report-only".to_string(),
                reason: "agent state".to_string(),
                warnings: Vec::new(),
            }],
            summary: Summary::default(),
        };

        let doctor = build_doctor(
            &report,
            DoctorOptions {
                docker: false,
                wsl: false,
                ollama: false,
                playwright: false,
                huggingface: false,
                agents: true,
                probe_tools: false,
            },
            &test_policy(),
        );

        assert!(doctor.topics[0]
            .recommendations
            .iter()
            .any(|recommendation| recommendation.contains("cache-like child")));
        assert!(doctor.topics[0]
            .recommendations
            .iter()
            .any(|recommendation| recommendation.contains("diff --latest")));
    }
}
