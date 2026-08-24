use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use indicatif::{ProgressBar, ProgressStyle};
use serde::Serialize;

use crate::application::ApplicationInventoryTool;
use crate::{
    action_proposal, anomaly, application, cleaner, diff, doctor, explainability, history,
    model_inventory, planner, reporter, rules, rules_repo, scanner, visualize,
};

const AGENT_DIAGNOSTIC_CLI_CONTRACT: &str = "agent-diagnostic-cli-v1";
const AGENT_CAPABILITIES_CONTRACT: &str = "agent-capabilities-v1";

#[derive(Parser, Debug)]
#[command(name = "aidisk")]
#[command(about = "Windows AI space diagnosis CLI")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    Explain {
        #[arg(long)]
        json: bool,
        #[arg(long)]
        category: Option<String>,
        #[arg(long, value_enum, default_value_t = SnapshotMode::Save)]
        snapshot: SnapshotMode,
    },
    Capabilities {
        #[arg(long)]
        json: bool,
    },
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
    Anomaly {
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
        #[arg(long, default_value = "1GB", value_parser = parse_size_arg)]
        min_growth: u64,
        #[arg(long, default_value_t = 30.0)]
        min_growth_percent: f64,
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
        ai_footprint: bool,
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
    Rules {
        #[command(subcommand)]
        command: RulesCommand,
    },
    Models {
        #[command(subcommand)]
        command: ModelsCommand,
    },
    Visualize {
        #[arg(long, default_value = "true")]
        html: bool,

        #[arg(long, default_value = ".aidisk/reports")]
        reports_dir: PathBuf,

        #[arg(long, default_value = "aidisk-footprint.html")]
        output: PathBuf,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
enum SnapshotMode {
    Save,
    Skip,
}

#[derive(Debug, Serialize)]
struct ExplainCliOutput {
    ok: bool,
    command: &'static str,
    contract: &'static str,
    schema_version: u16,
    core_version: &'static str,
    snapshot: SnapshotOutput,
    explainability: application::ExplainabilityReport,
}

#[derive(Debug, Serialize)]
struct SnapshotOutput {
    requested: &'static str,
    persisted: bool,
    path: Option<PathBuf>,
}

#[derive(Debug, Serialize)]
struct CapabilitiesOutput {
    ok: bool,
    command: &'static str,
    contract: &'static str,
    schema_version: u16,
    core_version: &'static str,
    capabilities: AgentCapabilities,
}

#[derive(Debug, Serialize)]
struct AgentCapabilities {
    explainability: ExplainabilityCapabilities,
    action_proposals: ActionProposalCapabilities,
}

#[derive(Debug, Serialize)]
struct ExplainabilityCapabilities {
    contract: &'static str,
    schema_versions: Vec<u16>,
    cli_available: bool,
    snapshot_modes: Vec<&'static str>,
    bounded_path_groups: bool,
}

#[derive(Debug, Serialize)]
struct ActionProposalCapabilities {
    contract: &'static str,
    schema_versions: Vec<u16>,
    application_api_available: bool,
    read_only: bool,
    human_preview_required: bool,
    mutation_authorized: bool,
}

#[derive(Subcommand, Debug)]
enum RulesCommand {
    Lint {
        #[arg(long)]
        json: bool,
        #[arg(long)]
        rules_dir: Option<PathBuf>,
        #[arg(long)]
        rules_repo: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
enum ModelsCommand {
    Inventory {
        #[arg(long, value_enum, default_value_t = model_inventory::InventoryTool::Auto)]
        tool: model_inventory::InventoryTool,
        #[arg(long)]
        root: Option<PathBuf>,
        #[arg(long, default_value_t = 20)]
        max_depth: usize,
        #[arg(long, default_value_t = 90)]
        stale_after_days: u64,
        #[arg(long)]
        json: bool,
        #[arg(long)]
        markdown: bool,
    },
    Adapters {
        #[arg(long, value_enum, default_value_t = model_inventory::AdapterTool::Auto)]
        tool: model_inventory::AdapterTool,
        #[arg(long)]
        root: Option<PathBuf>,
        #[arg(long, default_value_t = 20)]
        max_depth: usize,
        #[arg(
            long,
            help = "Explicitly probe only official CLI version/help commands"
        )]
        probe_official_cli: bool,
        #[arg(
            long,
            help = "Explicitly run only allowlisted official dry-run or read-only list commands"
        )]
        run_official_dry_run: bool,
        #[arg(
            long,
            default_value_t = model_inventory::DEFAULT_OFFICIAL_CLI_PROBE_TIMEOUT_MS
        )]
        probe_timeout_ms: u64,
        #[arg(
            long,
            default_value_t = model_inventory::DEFAULT_OFFICIAL_CLI_OUTPUT_CHARS
        )]
        probe_output_chars: usize,
        #[arg(long)]
        json: bool,
        #[arg(long)]
        markdown: bool,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum OutputFormat {
    Text,
    Json,
    Markdown,
}

pub fn run_from_env() -> std::process::ExitCode {
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
        Command::Explain {
            json,
            category,
            snapshot,
        } => {
            if !json {
                anyhow::bail!("explain requires --json");
            }

            let persist_snapshot = match snapshot {
                SnapshotMode::Save => application::SnapshotPersistence::Save,
                SnapshotMode::Skip => application::SnapshotPersistence::Skip,
            };
            let result = application::run_explainable_scan(application::ScanRequest {
                rules_dir: None,
                rules_repo: None,
                category,
                policy: None,
                default_rules_dir: default_rules_dir(),
                default_policy_path: default_policy_path(),
                reports_dir: None,
                persist_snapshot,
            })?;
            let persisted = result.scan.snapshot_path.is_some();
            let requested = match snapshot {
                SnapshotMode::Save => "save",
                SnapshotMode::Skip => "skip",
            };
            let output = ExplainCliOutput {
                ok: true,
                command: "explain",
                contract: AGENT_DIAGNOSTIC_CLI_CONTRACT,
                schema_version: 1,
                core_version: env!("CARGO_PKG_VERSION"),
                snapshot: SnapshotOutput {
                    requested,
                    persisted,
                    path: result.scan.snapshot_path,
                },
                explainability: result.explainability,
            };
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        Command::Capabilities { json } => {
            if !json {
                anyhow::bail!("capabilities requires --json");
            }

            let output = CapabilitiesOutput {
                ok: true,
                command: "capabilities",
                contract: AGENT_CAPABILITIES_CONTRACT,
                schema_version: 1,
                core_version: env!("CARGO_PKG_VERSION"),
                capabilities: AgentCapabilities {
                    explainability: ExplainabilityCapabilities {
                        contract: explainability::EXPLAINABILITY_CONTRACT,
                        schema_versions: vec![explainability::EXPLAINABILITY_SCHEMA_VERSION],
                        cli_available: true,
                        snapshot_modes: vec!["save", "skip"],
                        bounded_path_groups: true,
                    },
                    action_proposals: ActionProposalCapabilities {
                        contract: action_proposal::ACTION_PROPOSAL_CONTRACT,
                        schema_versions: vec![action_proposal::ACTION_PROPOSAL_SCHEMA_VERSION],
                        application_api_available: true,
                        read_only: true,
                        human_preview_required: true,
                        mutation_authorized: false,
                    },
                },
            };
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
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

            let result = run_application_scan_with_optional_progress(
                application::ScanRequest {
                    rules_dir,
                    rules_repo,
                    category,
                    policy,
                    default_rules_dir: default_rules_dir(),
                    default_policy_path: default_policy_path(),
                    reports_dir: None,
                    persist_snapshot: application::SnapshotPersistence::Save,
                },
                effective_format,
            )?;
            println!("{}", reporter::render(&result.report, effective_format)?);
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
            let policy = application::load_scan_policy(Some(policy_path), &default_policy_path())?;
            let rules = application::load_scan_rules(&rules_dir, category.as_deref())?;
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
            let policy = application::load_scan_policy(Some(policy_path), &default_policy_path())?;
            let rules = application::load_scan_rules(&rules_dir, category.as_deref())?;
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
                latest_scan_pair_from_application(reports_dir, "diff --latest")?
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
        Command::Anomaly {
            format,
            json,
            markdown,
            latest,
            reports_dir,
            before,
            after,
            min_growth,
            min_growth_percent,
        } => {
            let effective_format = effective_format(format, json, markdown);

            let (before, after) = if latest {
                let reports_dir = reports_dir.unwrap_or_else(history::default_reports_dir);
                latest_scan_pair_from_application(reports_dir, "anomaly --latest")?
            } else {
                let before = before.ok_or_else(|| {
                    anyhow::anyhow!("anomaly requires --before unless --latest is used")
                })?;
                let after = after.ok_or_else(|| {
                    anyhow::anyhow!("anomaly requires --after unless --latest is used")
                })?;
                (before, after)
            };

            let diff_report = diff::build_diff(&before, &after)?;
            let report = anomaly::build_anomaly_report(
                &diff_report,
                anomaly::AnomalyThresholds {
                    min_growth_bytes: min_growth,
                    min_growth_percent,
                },
            );
            println!("{}", reporter::render_anomaly(&report, effective_format)?);
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
            ai_footprint,
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
            let loaded_policy =
                application::load_scan_policy(Some(policy_path), &default_policy_path())?;
            let rules = application::load_scan_rules(&rules_dir, None)?;
            let scan_report = scan_with_optional_progress(
                &rules,
                loaded_policy.planner.max_scan_depth,
                effective_format,
            )?;
            let latest_diff = if latest {
                let reports_dir = reports_dir.unwrap_or_else(history::default_reports_dir);
                let (before, after) =
                    latest_scan_pair_from_application(reports_dir, "doctor --latest")?;
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
                ai_footprint,
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
        Command::Rules { command } => match command {
            RulesCommand::Lint {
                json,
                rules_dir,
                rules_repo,
            } => {
                let rules_dir = resolve_rules_dir(rules_dir, rules_repo)?;
                let report = rules::lint_rules(&rules_dir)?;
                if json {
                    println!("{}", serde_json::to_string_pretty(&report)?);
                } else {
                    println!("AI Disk Rule Lint");
                    println!("Rules Directory: {}", rules_dir.display());
                    println!("Total Rules: {}", report.total_rules);
                    for (version, count) in report.schema_versions {
                        println!("Schema v{version}: {count}");
                    }
                    println!("Sources:");
                    for source in report.rules {
                        println!(
                            "- schema=v{} | {} | {}",
                            source.schema_version, source.digest, source.path
                        );
                    }
                }
            }
        },
        Command::Models { command } => match command {
            ModelsCommand::Inventory {
                tool,
                root,
                max_depth,
                stale_after_days,
                json,
                markdown,
            } => {
                let report = application::inventory_assets(application::AssetInventoryRequest {
                    root,
                    tool: match tool {
                        model_inventory::InventoryTool::Auto => ApplicationInventoryTool::Auto,
                        model_inventory::InventoryTool::Ollama => ApplicationInventoryTool::Ollama,
                        model_inventory::InventoryTool::Huggingface => {
                            ApplicationInventoryTool::Huggingface
                        }
                        model_inventory::InventoryTool::LmStudio => {
                            ApplicationInventoryTool::LmStudio
                        }
                        model_inventory::InventoryTool::Generic => {
                            ApplicationInventoryTool::Generic
                        }
                    },
                    max_depth,
                    stale_after_days,
                })?;
                if json {
                    println!("{}", serde_json::to_string_pretty(&report)?);
                } else if markdown {
                    println!("# Model Asset Inventory");
                    println!();
                    println!("- Schema Version: {}", report.schema_version);
                    println!("- Assets: {}", report.summary.total_assets);
                    println!(
                        "- Logical Bytes: {}",
                        format_inventory_bytes(report.summary.logical_bytes)
                    );
                    println!(
                        "- Exclusive Physical Bytes: {}",
                        format_inventory_bytes(report.summary.exclusive_physical_bytes)
                    );
                    println!(
                        "- Shared Physical Bytes: {}",
                        format_inventory_bytes(report.summary.shared_physical_bytes)
                    );
                    println!("- Referenced Assets: {}", report.summary.referenced_assets);
                    println!(
                        "- Detached Revision Assets: {}",
                        report.summary.detached_revision_assets
                    );
                    println!(
                        "- Orphan Blob Assets: {}",
                        report.summary.orphan_blob_assets
                    );
                    println!(
                        "- Incomplete Download Assets: {}",
                        report.summary.incomplete_download_assets
                    );
                    println!("- Stale Assets: {}", report.summary.stale_assets);
                    println!(
                        "- Duplicate Logical Model Assets: {}",
                        report.summary.duplicate_logical_model_assets
                    );
                    println!(
                        "- External Drive Candidate Assets: {}",
                        report.summary.external_drive_candidate_assets
                    );
                    println!(
                        "- Expected Reclaim Bytes: {}",
                        format_inventory_bytes(report.summary.expected_reclaim_bytes)
                    );
                    println!(
                        "- Recovery Size: {}",
                        format_inventory_bytes(report.summary.recovery_size_bytes)
                    );
                    println!(
                        "- High Utility Eviction Assets: {}",
                        report.summary.high_utility_eviction_assets
                    );
                    println!(
                        "- Blocked Eviction Assets: {}",
                        report.summary.blocked_eviction_assets
                    );
                    println!();
                    println!("| Model | Format | Manager | State | Size | Action |");
                    println!("|---|---|---|---|---:|---|");
                    for asset in report.assets {
                        println!(
                            "| `{}` | `{}` | `{}` | `{}` | {} | `{}` |",
                            asset.logical_name,
                            asset.format,
                            asset.manager,
                            asset.state,
                            format_inventory_bytes(asset.logical_size_bytes),
                            asset.action
                        );
                    }
                } else {
                    println!("Model Asset Inventory");
                    println!("Schema Version: {}", report.schema_version);
                    println!("Assets: {}", report.summary.total_assets);
                    println!(
                        "Logical Bytes: {}",
                        format_inventory_bytes(report.summary.logical_bytes)
                    );
                    println!(
                        "Exclusive Physical Bytes: {}",
                        format_inventory_bytes(report.summary.exclusive_physical_bytes)
                    );
                    println!(
                        "Shared Physical Bytes: {}",
                        format_inventory_bytes(report.summary.shared_physical_bytes)
                    );
                    println!("Referenced Assets: {}", report.summary.referenced_assets);
                    println!(
                        "Detached Revision Assets: {}",
                        report.summary.detached_revision_assets
                    );
                    println!("Orphan Blob Assets: {}", report.summary.orphan_blob_assets);
                    println!(
                        "Incomplete Download Assets: {}",
                        report.summary.incomplete_download_assets
                    );
                    println!("Stale Assets: {}", report.summary.stale_assets);
                    println!(
                        "Duplicate Logical Model Assets: {}",
                        report.summary.duplicate_logical_model_assets
                    );
                    println!(
                        "External Drive Candidate Assets: {}",
                        report.summary.external_drive_candidate_assets
                    );
                    println!(
                        "Expected Reclaim Bytes: {}",
                        format_inventory_bytes(report.summary.expected_reclaim_bytes)
                    );
                    println!(
                        "Recovery Size: {}",
                        format_inventory_bytes(report.summary.recovery_size_bytes)
                    );
                    println!(
                        "High Utility Eviction Assets: {}",
                        report.summary.high_utility_eviction_assets
                    );
                    println!(
                        "Blocked Eviction Assets: {}",
                        report.summary.blocked_eviction_assets
                    );
                    for asset in report.assets {
                        println!(
                            "- [{}] {} | manager={} | format={} | state={} | size={} | action={} | confidence={}/100",
                            asset.recoverability,
                            asset.logical_name,
                            asset.manager,
                            asset.format,
                            asset.state,
                            format_inventory_bytes(asset.logical_size_bytes),
                            asset.action,
                            asset.reclaim_confidence
                        );
                    }
                }
            }
            ModelsCommand::Adapters {
                tool,
                root,
                max_depth,
                probe_official_cli,
                run_official_dry_run,
                probe_timeout_ms,
                probe_output_chars,
                json,
                markdown,
            } => {
                let report =
                    model_inventory::build_adapter_report(&model_inventory::AdapterOptions {
                        root,
                        tool,
                        max_depth,
                        probe_official_cli,
                        run_official_dry_run,
                        probe_timeout_ms,
                        probe_output_chars,
                    })?;
                if json {
                    println!("{}", serde_json::to_string_pretty(&report)?);
                } else if markdown {
                    println!("# Model Adapter Capability Report");
                    println!();
                    println!("- Schema Version: {}", report.schema_version);
                    println!("- Adapters: {}", report.summary.total_adapters);
                    println!(
                        "- Parseable Indexes: {}",
                        report.summary.index_parseable_adapters
                    );
                    println!(
                        "- Official CLI Probed: {}",
                        report.summary.official_cli_probed_adapters
                    );
                    println!(
                        "- Official Dry-Run Capable: {}",
                        report.summary.official_dry_run_capable_adapters
                    );
                    println!(
                        "- Official Dry-Run Invoked: {}",
                        report.summary.official_dry_run_invoked_adapters
                    );
                    println!(
                        "- Official Read-Only Lists: {}",
                        report.summary.official_read_only_list_invoked_adapters
                    );
                    println!(
                        "- Official Cleanup Plan Items: {}",
                        report.summary.official_cleanup_plan_items
                    );
                    println!(
                        "- Official Rollback Capable Items: {}",
                        report.summary.official_cleanup_rollback_capable_items
                    );
                    println!();
                    println!("| Tool | Root | Index | Official CLI | Dry-Run | Official Run | Plan Mode | Action |");
                    println!("|---|---|---|---|---|---|---|---|");
                    for adapter in report.adapters {
                        let official_cli = adapter.official_cli.as_ref();
                        let official_dry_run = adapter.official_dry_run.as_ref();
                        println!(
                            "| `{}` | `{}` | `{}` | `{}` | `{}` | `{}` | `{}` | `{}` |",
                            adapter.tool,
                            adapter.root.unwrap_or_else(|| "not detected".to_string()),
                            if adapter.index_parseable {
                                "parseable"
                            } else if adapter.index_present {
                                "present/unresolved"
                            } else {
                                "missing"
                            },
                            official_cli
                                .map(|cli| cli.available.map_or("not-probed", |available| {
                                    if available {
                                        "available"
                                    } else {
                                        "not-available"
                                    }
                                }))
                                .unwrap_or("unknown"),
                            official_cli.and_then(|cli| cli.supports_dry_run).map_or(
                                "unknown",
                                |supported| if supported { "yes" } else { "no" }
                            ),
                            official_dry_run
                                .map(|dry_run| dry_run.status.as_str())
                                .unwrap_or("unknown"),
                            adapter.plan_mode,
                            adapter.action
                        );
                    }
                } else {
                    println!("Model Adapter Capability Report");
                    println!("Schema Version: {}", report.schema_version);
                    println!("Adapters: {}", report.summary.total_adapters);
                    println!(
                        "Parseable Indexes: {}",
                        report.summary.index_parseable_adapters
                    );
                    println!(
                        "Official CLI Probed: {}",
                        report.summary.official_cli_probed_adapters
                    );
                    println!(
                        "Official Dry-Run Invoked: {}",
                        report.summary.official_dry_run_invoked_adapters
                    );
                    println!(
                        "Official Read-Only Lists: {}",
                        report.summary.official_read_only_list_invoked_adapters
                    );
                    println!(
                        "Official Cleanup Plan Items: {}",
                        report.summary.official_cleanup_plan_items
                    );
                    println!(
                        "Official Rollback Capable Items: {}",
                        report.summary.official_cleanup_rollback_capable_items
                    );
                    for adapter in report.adapters {
                        let official_cli = adapter.official_cli.as_ref();
                        let official_dry_run = adapter.official_dry_run.as_ref();
                        println!(
                            "- {} | root={} | index_present={} | index_parseable={} | cli={} | dry_run={} | official_run={} | plan={} | action={}",
                            adapter.tool,
                            adapter.root.unwrap_or_else(|| "not detected".to_string()),
                            adapter.index_present,
                            adapter.index_parseable,
                            official_cli
                                .map(|cli| cli.available.map_or("not-probed", |available| if available { "available" } else { "not-available" }))
                                .unwrap_or("unknown"),
                            official_cli
                                .and_then(|cli| cli.supports_dry_run)
                                .map_or("unknown", |supported| if supported { "yes" } else { "no" }),
                            official_dry_run
                                .map(|dry_run| dry_run.status.as_str())
                                .unwrap_or("unknown"),
                            adapter.plan_mode,
                            adapter.action
                        );
                    }
                }
            }
        },
        Command::Visualize {
            html,
            reports_dir,
            output,
        } => {
            let _ = html;
            visualize::generate_dashboard(&reports_dir, &output)?;
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
                "explain" => Some("explain"),
                "capabilities" => Some("capabilities"),
                "scan" => Some("scan"),
                "plan" => Some("plan"),
                "clean" => Some("clean"),
                "restore" => Some("restore"),
                "diff" => Some("diff"),
                "anomaly" => Some("anomaly"),
                "doctor" => Some("doctor"),
                "rules" => Some("rules"),
                "models" => Some("models"),
                "visualize" => Some("visualize"),
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
            Command::Explain { json, .. } => Self {
                command: "explain",
                format: if *json {
                    OutputFormat::Json
                } else {
                    OutputFormat::Text
                },
            },
            Command::Capabilities { json } => Self {
                command: "capabilities",
                format: if *json {
                    OutputFormat::Json
                } else {
                    OutputFormat::Text
                },
            },
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
            Command::Anomaly {
                format,
                json,
                markdown,
                ..
            } => Self {
                command: "anomaly",
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
            Command::Rules { command } => Self {
                command: "rules",
                format: match command {
                    RulesCommand::Lint { json, .. } if *json => OutputFormat::Json,
                    _ => OutputFormat::Text,
                },
            },
            Command::Models { command } => Self {
                command: "models",
                format: match command {
                    ModelsCommand::Inventory { json, .. } if *json => OutputFormat::Json,
                    ModelsCommand::Inventory { markdown, .. } if *markdown => {
                        OutputFormat::Markdown
                    }
                    ModelsCommand::Adapters { json, .. } if *json => OutputFormat::Json,
                    ModelsCommand::Adapters { markdown, .. } if *markdown => OutputFormat::Markdown,
                    _ => OutputFormat::Text,
                },
            },
            Command::Visualize { .. } => Self {
                command: "visualize",
                format: OutputFormat::Text,
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

fn format_inventory_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut value = bytes as f64;
    let mut unit = 0_usize;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} B")
    } else {
        format!("{value:.2} {}", UNITS[unit])
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
        || message.contains("duplicate rule id")
        || message.contains("unsupported rule schema")
        || message.contains("schema v2")
        || error.downcast_ref::<std::io::Error>().is_some()
    {
        return "input";
    }
    "execution"
}

fn default_rules_dir() -> PathBuf {
    portable_resource_path("rules")
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("rules"))
}

fn portable_resource_path(relative: &str) -> Option<PathBuf> {
    let mut candidates = Vec::new();

    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            candidates.push(exe_dir.join(relative));
        }
    }

    if let Ok(current_dir) = std::env::current_dir() {
        candidates.push(current_dir.join(relative));
        candidates.push(current_dir.join("aidisk").join(relative));
    }

    candidates.into_iter().find(|candidate| candidate.exists())
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

fn run_application_scan_with_optional_progress(
    request: application::ScanRequest,
    format: OutputFormat,
) -> Result<application::ScanResult> {
    if !progress_enabled(format) {
        return application::run_scan(request);
    }

    let progress = ProgressBar::new(0);
    let style = ProgressStyle::with_template(
        "{spinner:.green} scanning [{bar:20.cyan/blue}] {pos}/{len} {msg}",
    )
    .unwrap_or_else(|_| ProgressStyle::default_bar())
    .progress_chars("=> ");
    progress.set_style(style);

    let result = application::run_scan_with_progress(request, |event| {
        progress.set_length(event.total as u64);
        progress.set_position(event.current as u64);
        progress.set_message(event.rule_id.to_string());
    });
    progress.finish_and_clear();
    result
}

fn latest_scan_pair_from_application(
    reports_dir: PathBuf,
    command_name: &str,
) -> Result<(PathBuf, PathBuf)> {
    let history = application::read_history(application::HistoryRequest {
        reports_dir: Some(reports_dir.clone()),
    })?;
    let Some(pair) = history.latest_pair else {
        anyhow::bail!(
            "{} requires at least two scan snapshots in {}",
            command_name,
            reports_dir.display()
        );
    };

    Ok((pair.before.path, pair.after.path))
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
    portable_resource_path("config")
        .map(|config_dir| config_dir.join("policy.yaml"))
        .filter(|policy_path| policy_path.exists())
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("config")
                .join("policy.yaml")
        })
}

fn large_files_default_root() -> PathBuf {
    std::env::var_os("USERPROFILE")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("C:\\"))
}

#[cfg(test)]
mod tests {
    use super::{parse_size_arg, progress_enabled_for, OutputFormat};

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
}
