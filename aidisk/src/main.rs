mod cleaner;
mod diff;
mod doctor;
mod history;
mod planner;
mod policy;
mod reporter;
mod rules;
mod rules_repo;
mod scanner;
#[cfg(test)]
mod test_support;

use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use indicatif::{ProgressBar, ProgressStyle};

#[derive(Parser, Debug)]
#[command(name = "aidisk")]
#[command(about = "Windows AI space diagnosis CLI")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    Scan {
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
        #[arg(long)]
        json: bool,
        #[arg(long)]
        markdown: bool,
        #[arg(long)]
        category: Option<String>,
        #[arg(long)]
        rules_dir: Option<PathBuf>,
        #[arg(long)]
        rules_repo: Option<String>,
        #[arg(long)]
        large_files: bool,
        #[arg(
            long,
            default_value = "500MB",
            value_parser = parse_size_arg,
            help = "Minimum size in bytes or B/KB/MB/GB/TB"
        )]
        min_size: u64,
        #[arg(long)]
        root: Option<PathBuf>,
        #[arg(long)]
        policy: Option<PathBuf>,
    },
    Plan {
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
        #[arg(long)]
        json: bool,
        #[arg(long)]
        markdown: bool,
        #[arg(long)]
        safe_only: bool,
        #[arg(long, default_value_t = 30)]
        skip_modified_within_minutes: u64,
        #[arg(long)]
        category: Option<String>,
        #[arg(long)]
        rules_dir: Option<PathBuf>,
        #[arg(long)]
        rules_repo: Option<String>,
        #[arg(long)]
        policy: Option<PathBuf>,
    },
    Clean {
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
        #[arg(long)]
        json: bool,
        #[arg(long)]
        markdown: bool,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        yes: bool,
        #[arg(long)]
        safe_only: bool,
        #[arg(long)]
        category: Option<String>,
        #[arg(long)]
        rules_dir: Option<PathBuf>,
        #[arg(long)]
        rules_repo: Option<String>,
        #[arg(long)]
        policy: Option<PathBuf>,
        #[arg(long)]
        quarantine_root: Option<String>,
    },
    Restore {
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
        #[arg(long)]
        json: bool,
        #[arg(long)]
        markdown: bool,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        yes: bool,
        #[arg(long)]
        index: PathBuf,
    },
    Diff {
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
        #[arg(long)]
        json: bool,
        #[arg(long)]
        markdown: bool,
        #[arg(long)]
        latest: bool,
        #[arg(long)]
        reports_dir: Option<PathBuf>,
        #[arg(long)]
        before: Option<PathBuf>,
        #[arg(long)]
        after: Option<PathBuf>,
    },
    Doctor {
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
        #[arg(long)]
        json: bool,
        #[arg(long)]
        markdown: bool,
        #[arg(long)]
        docker: bool,
        #[arg(long)]
        wsl: bool,
        #[arg(long)]
        ollama: bool,
        #[arg(long)]
        playwright: bool,
        #[arg(long)]
        huggingface: bool,
        #[arg(long)]
        agents: bool,
        #[arg(long)]
        probe_tools: bool,
        #[arg(long)]
        latest: bool,
        #[arg(long)]
        reports_dir: Option<PathBuf>,
        #[arg(long)]
        rules_dir: Option<PathBuf>,
        #[arg(long)]
        rules_repo: Option<String>,
        #[arg(long)]
        policy: Option<PathBuf>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum OutputFormat {
    Text,
    Json,
    Markdown,
}

fn main() -> std::process::ExitCode {
    let raw_args = std::env::args().collect::<Vec<_>>();
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(error) => {
            let context = ErrorContext::from_raw_args(&raw_args);
            return if context.format == OutputFormat::Json {
                emit_clap_error(&context, &error);
                std::process::ExitCode::FAILURE
            } else {
                let _ = error.print();
                std::process::ExitCode::from(error.exit_code() as u8)
            };
        }
    };
    let error_context = ErrorContext::from_command(&cli.command);

    match run(cli) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            emit_error(&error_context, &error);
            std::process::ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Command::Scan {
            format,
            json,
            markdown,
            category,
            rules_dir,
            rules_repo,
            large_files,
            min_size,
            root,
            policy,
        } => {
            let effective_format = effective_format(format, json, markdown);

            if large_files {
                let root = root.unwrap_or_else(large_files_default_root);
                let report = scanner::scan_large_files(&root, min_size)?;
                println!(
                    "{}",
                    reporter::render_large_files(&report, effective_format)?
                );
                return Ok(());
            }

            let rules_dir = resolve_rules_dir(rules_dir, rules_repo)?;
            let policy = load_scan_policy(policy, &default_policy_path())?;
            let rules = rules::load_rules(&rules_dir)?;
            let rules = rules::filter_rules(rules, category.as_deref());
            let mut report = scan_with_optional_progress(
                &rules,
                policy.planner.max_scan_depth,
                effective_format,
            )?;
            report.policy = Some(policy.snapshot());
            history::save_scan_snapshot(&report, &history::default_reports_dir())?;
            println!("{}", reporter::render(&report, effective_format)?);
        }
        Command::Plan {
            format,
            json,
            markdown,
            safe_only,
            skip_modified_within_minutes,
            category,
            rules_dir,
            rules_repo,
            policy,
        } => {
            let effective_format = effective_format(format, json, markdown);

            let rules_dir = resolve_rules_dir(rules_dir, rules_repo)?;
            let policy_path = policy.unwrap_or_else(default_policy_path);
            let policy = policy::load_policy(&policy_path)?;
            let rules = rules::load_rules(&rules_dir)?;
            let rules = rules::filter_rules(rules, category.as_deref());
            let scan_report = scan_with_optional_progress(
                &rules,
                policy.planner.max_scan_depth,
                effective_format,
            )?;
            let plan_report = planner::build_plan(
                &scan_report,
                planner::PlanOptions {
                    safe_only,
                    skip_modified_within_minutes,
                    policy,
                },
            );
            println!("{}", reporter::render_plan(&plan_report, effective_format)?);
        }
        Command::Clean {
            format,
            json,
            markdown,
            dry_run,
            yes,
            safe_only,
            category,
            rules_dir,
            rules_repo,
            policy,
            quarantine_root,
        } => {
            let effective_format = effective_format(format, json, markdown);

            let rules_dir = resolve_rules_dir(rules_dir, rules_repo)?;
            let policy_path = policy.unwrap_or_else(default_policy_path);
            let policy = policy::load_policy(&policy_path)?;
            let rules = rules::load_rules(&rules_dir)?;
            let rules = rules::filter_rules(rules, category.as_deref());
            let scan_report = scan_with_optional_progress(
                &rules,
                policy.planner.max_scan_depth,
                effective_format,
            )?;
            let plan_report = planner::build_plan(
                &scan_report,
                planner::PlanOptions {
                    safe_only,
                    skip_modified_within_minutes: policy.planner.skip_modified_within_minutes,
                    policy,
                },
            );

            if dry_run {
                let clean_report = cleaner::build_dry_run(&plan_report);
                if effective_format == OutputFormat::Json {
                    let quarantine_plan = quarantine_root
                        .as_deref()
                        .map(|root| cleaner::build_quarantine_plan(&plan_report, root));
                    let output = cleaner::CleanDryRunOutput {
                        clean: clean_report,
                        quarantine_plan,
                    };
                    println!(
                        "{}",
                        reporter::render_clean_dry_run_output(&output, effective_format)?
                    );
                } else {
                    println!(
                        "{}",
                        reporter::render_clean(&clean_report, effective_format)?
                    );

                    if let Some(quarantine_root) = quarantine_root {
                        let quarantine_plan =
                            cleaner::build_quarantine_plan(&plan_report, &quarantine_root);
                        println!();
                        println!(
                            "{}",
                            reporter::render_quarantine_plan(&quarantine_plan, effective_format)?
                        );
                    }
                }
            } else {
                if !yes {
                    anyhow::bail!("clean execution requires --yes");
                }

                let quarantine_root = quarantine_root
                    .ok_or_else(|| anyhow::anyhow!("clean execution requires --quarantine-root"))?;
                let quarantine_plan =
                    cleaner::build_quarantine_plan(&plan_report, &quarantine_root);
                let execution_report = cleaner::execute_quarantine(&quarantine_plan)?;
                println!(
                    "{}",
                    reporter::render_execution(&execution_report, effective_format)?
                );
            }
        }
        Command::Restore {
            format,
            json,
            markdown,
            dry_run,
            yes,
            index,
        } => {
            let effective_format = effective_format(format, json, markdown);

            if !dry_run && !yes {
                anyhow::bail!("restore execution requires --yes or use --dry-run");
            }

            let report = cleaner::restore_from_index(&index, dry_run)?;
            println!("{}", reporter::render_restore(&report, effective_format)?);
        }
        Command::Diff {
            format,
            json,
            markdown,
            latest,
            reports_dir,
            before,
            after,
        } => {
            let effective_format = effective_format(format, json, markdown);

            let (before, after) = if latest {
                let reports_dir = reports_dir.unwrap_or_else(history::default_reports_dir);
                history::latest_scan_pair(&reports_dir)?
            } else {
                let before = before.ok_or_else(|| {
                    anyhow::anyhow!("diff requires --before unless --latest is used")
                })?;
                let after = after.ok_or_else(|| {
                    anyhow::anyhow!("diff requires --after unless --latest is used")
                })?;
                (before, after)
            };

            let report = diff::build_diff(&before, &after)?;
            println!("{}", reporter::render_diff(&report, effective_format)?);
        }
        Command::Doctor {
            format,
            json,
            markdown,
            docker,
            wsl,
            ollama,
            playwright,
            huggingface,
            agents,
            probe_tools,
            latest,
            reports_dir,
            rules_dir,
            rules_repo,
            policy,
        } => {
            let effective_format = effective_format(format, json, markdown);

            let rules_dir = resolve_rules_dir(rules_dir, rules_repo)?;
            let policy_path = policy.unwrap_or_else(default_policy_path);
            let loaded_policy = policy::load_policy(&policy_path)?;
            let rules = rules::load_rules(&rules_dir)?;
            let scan_report = scan_with_optional_progress(
                &rules,
                loaded_policy.planner.max_scan_depth,
                effective_format,
            )?;
            let latest_diff = if latest {
                let reports_dir = reports_dir.unwrap_or_else(history::default_reports_dir);
                let (before, after) =
                    history::latest_scan_pair_for_command(&reports_dir, "doctor --latest")?;
                let diff_report = diff::build_diff(&before, &after)?;
                Some(doctor::build_latest_diff_section(&diff_report, 10))
            } else {
                None
            };
            let mut doctor_options = doctor::DoctorOptions {
                docker,
                wsl,
                ollama,
                playwright,
                huggingface,
                agents,
                probe_tools,
            };
            doctor::apply_default_topics_if_none_selected(&mut doctor_options);
            let doctor_report = if let Some(latest_diff) = latest_diff {
                doctor::build_doctor_with_latest_diff(
                    &scan_report,
                    doctor_options,
                    &loaded_policy,
                    Some(latest_diff),
                )
            } else {
                doctor::build_doctor(&scan_report, doctor_options, &loaded_policy)
            };
            println!(
                "{}",
                reporter::render_doctor(&doctor_report, effective_format)?
            );
        }
    }

    Ok(())
}

#[derive(Clone, Copy)]
struct ErrorContext {
    command: &'static str,
    format: OutputFormat,
}

#[derive(serde::Serialize)]
struct JsonErrorEnvelope {
    ok: bool,
    error: JsonErrorBody,
}

#[derive(serde::Serialize)]
struct JsonErrorBody {
    #[serde(rename = "type")]
    error_type: String,
    message: String,
    command: String,
    details: Vec<String>,
}

impl ErrorContext {
    fn from_raw_args(args: &[String]) -> Self {
        let command = args
            .iter()
            .skip(1)
            .find_map(|arg| match arg.as_str() {
                "scan" => Some("scan"),
                "plan" => Some("plan"),
                "clean" => Some("clean"),
                "restore" => Some("restore"),
                "diff" => Some("diff"),
                "doctor" => Some("doctor"),
                _ => None,
            })
            .unwrap_or("aidisk");
        let format = if raw_args_request_json(args) {
            OutputFormat::Json
        } else {
            OutputFormat::Text
        };

        Self { command, format }
    }

    fn from_command(command: &Command) -> Self {
        match command {
            Command::Scan {
                format,
                json,
                markdown,
                ..
            } => Self {
                command: "scan",
                format: effective_format(*format, *json, *markdown),
            },
            Command::Plan {
                format,
                json,
                markdown,
                ..
            } => Self {
                command: "plan",
                format: effective_format(*format, *json, *markdown),
            },
            Command::Clean {
                format,
                json,
                markdown,
                ..
            } => Self {
                command: "clean",
                format: effective_format(*format, *json, *markdown),
            },
            Command::Restore {
                format,
                json,
                markdown,
                ..
            } => Self {
                command: "restore",
                format: effective_format(*format, *json, *markdown),
            },
            Command::Diff {
                format,
                json,
                markdown,
                ..
            } => Self {
                command: "diff",
                format: effective_format(*format, *json, *markdown),
            },
            Command::Doctor {
                format,
                json,
                markdown,
                ..
            } => Self {
                command: "doctor",
                format: effective_format(*format, *json, *markdown),
            },
        }
    }
}

fn raw_args_request_json(args: &[String]) -> bool {
    args.iter().any(|arg| arg == "--json")
        || args
            .windows(2)
            .any(|window| window[0] == "--format" && window[1].eq_ignore_ascii_case("json"))
        || args.iter().any(|arg| {
            arg.strip_prefix("--format=")
                .is_some_and(|value| value.eq_ignore_ascii_case("json"))
        })
}

fn effective_format(format: OutputFormat, json: bool, markdown: bool) -> OutputFormat {
    if json {
        OutputFormat::Json
    } else if markdown {
        OutputFormat::Markdown
    } else {
        format
    }
}

fn parse_size_arg(value: &str) -> std::result::Result<u64, String> {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err("size cannot be empty".to_string());
    }

    let upper = trimmed.to_ascii_uppercase();
    let (number, multiplier) = if let Some(number) = upper.strip_suffix("TB") {
        (number, TB)
    } else if let Some(number) = upper.strip_suffix("GB") {
        (number, GB)
    } else if let Some(number) = upper.strip_suffix("MB") {
        (number, MB)
    } else if let Some(number) = upper.strip_suffix("KB") {
        (number, KB)
    } else if let Some(number) = upper.strip_suffix('B') {
        (number, 1)
    } else if upper.chars().all(|ch| ch.is_ascii_digit()) {
        (upper.as_str(), 1)
    } else {
        return Err(format!(
            "unsupported size suffix in '{value}'; expected bytes or B/KB/MB/GB/TB"
        ));
    };

    let amount = number
        .trim()
        .parse::<u64>()
        .map_err(|error| error.to_string())?;

    amount
        .checked_mul(multiplier)
        .ok_or_else(|| format!("size '{value}' is too large"))
}

fn emit_error(context: &ErrorContext, error: &anyhow::Error) {
    if context.format == OutputFormat::Json {
        let envelope = JsonErrorEnvelope {
            ok: false,
            error: JsonErrorBody {
                error_type: classify_cli_error(error).to_string(),
                message: error.to_string(),
                command: context.command.to_string(),
                details: Vec::new(),
            },
        };
        match serde_json::to_string_pretty(&envelope) {
            Ok(output) => eprintln!("{output}"),
            Err(render_error) => eprintln!(
                "{{\"ok\":false,\"error\":{{\"type\":\"internal\",\"message\":\"failed to render JSON error: {render_error}\",\"command\":\"{}\",\"details\":[]}}}}",
                context.command
            ),
        }
    } else {
        eprintln!("Error: {error:?}");
    }
}

fn emit_clap_error(context: &ErrorContext, error: &clap::Error) {
    let envelope = JsonErrorEnvelope {
        ok: false,
        error: JsonErrorBody {
            error_type: "usage".to_string(),
            message: error.to_string(),
            command: context.command.to_string(),
            details: Vec::new(),
        },
    };
    match serde_json::to_string_pretty(&envelope) {
        Ok(output) => eprintln!("{output}"),
        Err(render_error) => eprintln!(
            "{{\"ok\":false,\"error\":{{\"type\":\"internal\",\"message\":\"failed to render JSON error: {render_error}\",\"command\":\"{}\",\"details\":[]}}}}",
            context.command
        ),
    }
}

fn classify_cli_error(error: &anyhow::Error) -> &'static str {
    let message = error.to_string().to_ascii_lowercase();
    if message.contains("requires --yes")
        || message.contains("requires --before")
        || message.contains("requires --after")
        || message.contains("requires --quarantine-root")
    {
        return "usage";
    }
    if message.contains("failed to read")
        || message.contains("failed to parse")
        || message.contains("requires at least two scan snapshots")
        || message.contains("no such file")
        || message.contains("not found")
        || error.downcast_ref::<std::io::Error>().is_some()
    {
        return "input";
    }
    "execution"
}

fn default_rules_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("rules")
}

fn resolve_rules_dir(rules_dir: Option<PathBuf>, rules_repo: Option<String>) -> Result<PathBuf> {
    if let Some(rules_dir) = rules_dir {
        return Ok(rules_dir);
    }

    if let Some(rules_repo) = rules_repo {
        return rules_repo::resolve_rules_repo(
            &rules_repo,
            &rules_repo::default_rules_repo_cache_root(),
        );
    }

    Ok(default_rules_dir())
}

fn scan_with_optional_progress(
    rules: &[rules::Rule],
    max_scan_depth: usize,
    format: OutputFormat,
) -> Result<scanner::ScanReport> {
    if !progress_enabled(format) {
        return scanner::scan(rules, max_scan_depth);
    }

    let progress = ProgressBar::new(rules.len() as u64);
    let style = ProgressStyle::with_template(
        "{spinner:.green} scanning [{bar:20.cyan/blue}] {pos}/{len} {msg}",
    )
    .unwrap_or_else(|_| ProgressStyle::default_bar())
    .progress_chars("=> ");
    progress.set_style(style);

    let result = scanner::scan_with_progress(rules, max_scan_depth, |event| {
        progress.set_length(event.total as u64);
        progress.set_position(event.current as u64);
        progress.set_message(event.rule_id.to_string());
    });
    progress.finish_and_clear();
    result
}

fn progress_enabled(format: OutputFormat) -> bool {
    progress_enabled_for(
        format,
        std::env::var_os("CI").is_some(),
        console::Term::stderr().is_term(),
    )
}

fn progress_enabled_for(format: OutputFormat, ci_present: bool, stderr_is_term: bool) -> bool {
    format != OutputFormat::Json && !ci_present && stderr_is_term
}

fn default_policy_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("config")
        .join("policy.yaml")
}

fn default_scan_policy() -> policy::Policy {
    policy::Policy {
        sensitive_markers: vec!["token".to_string()],
        planner: policy::PlannerPolicy {
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

fn load_scan_policy(
    explicit_policy: Option<PathBuf>,
    default_policy_path: &std::path::Path,
) -> Result<policy::Policy> {
    if let Some(explicit_policy) = explicit_policy {
        return policy::load_policy(&explicit_policy);
    }

    match policy::load_policy(default_policy_path) {
        Ok(policy) => Ok(policy),
        Err(_) => Ok(default_scan_policy()),
    }
}

fn large_files_default_root() -> PathBuf {
    std::env::var_os("USERPROFILE")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("C:\\"))
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::{load_scan_policy, parse_size_arg, progress_enabled_for, OutputFormat};

    #[test]
    fn progress_enabled_for_only_allows_interactive_non_json_output() {
        assert!(progress_enabled_for(OutputFormat::Text, false, true));
        assert!(progress_enabled_for(OutputFormat::Markdown, false, true));
        assert!(!progress_enabled_for(OutputFormat::Json, false, true));
        assert!(!progress_enabled_for(OutputFormat::Text, true, true));
        assert!(!progress_enabled_for(OutputFormat::Text, false, false));
    }

    #[test]
    fn parse_size_arg_accepts_bytes_and_human_units() {
        assert_eq!(parse_size_arg("500").unwrap(), 500);
        assert_eq!(parse_size_arg("500MB").unwrap(), 524_288_000);
        assert_eq!(parse_size_arg("2 gb").unwrap(), 2_147_483_648);
    }

    #[test]
    fn parse_size_arg_rejects_unknown_suffixes() {
        assert!(parse_size_arg("500XB").is_err());
    }

    #[test]
    fn load_scan_policy_uses_builtin_defaults_when_default_file_missing() {
        let temp = tempdir().expect("tempdir should exist");
        let missing_default = temp.path().join("missing-policy.yaml");

        let policy = load_scan_policy(None, &missing_default).expect("fallback policy should load");

        assert_eq!(policy.planner.max_scan_depth, 20);
        assert!(policy
            .planner
            .allow_actions
            .iter()
            .any(|action| action == "quarantine"));
        assert!(policy.sensitive_markers.iter().any(|marker| marker == "token"));
    }

    #[test]
    fn load_scan_policy_prefers_explicit_override() {
        let temp = tempdir().expect("tempdir should exist");
        let missing_default = temp.path().join("missing-policy.yaml");
        let explicit_policy = temp.path().join("explicit-policy.yaml");
        fs::write(
            &explicit_policy,
            r#"sensitive_markers:
  - explicit
planner:
  skip_modified_within_minutes: 9
  allow_actions:
    - report-only
  max_scan_depth: 4
"#,
        )
        .expect("policy should be written");

        let policy = load_scan_policy(Some(explicit_policy), &missing_default)
            .expect("explicit policy should load");

        assert_eq!(policy.planner.max_scan_depth, 4);
        assert_eq!(policy.planner.skip_modified_within_minutes, 9);
        assert_eq!(policy.sensitive_markers, vec!["explicit".to_string()]);
    }
}
