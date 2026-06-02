mod cleaner;
mod diff;
mod doctor;
mod planner;
mod policy;
mod reporter;
mod rules;
mod scanner;

use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};

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
        before: PathBuf,
        #[arg(long)]
        after: PathBuf,
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
        rules_dir: Option<PathBuf>,
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

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Scan {
            format,
            json,
            markdown,
            category,
            rules_dir,
        } => {
            let effective_format = if json {
                OutputFormat::Json
            } else if markdown {
                OutputFormat::Markdown
            } else {
                format
            };

            let rules_dir = rules_dir.unwrap_or_else(default_rules_dir);
            let rules = rules::load_rules(&rules_dir)?;
            let rules = rules::filter_rules(rules, category.as_deref());
            let report = scanner::scan(&rules, 20)?;
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
            policy,
        } => {
            let effective_format = if json {
                OutputFormat::Json
            } else if markdown {
                OutputFormat::Markdown
            } else {
                format
            };

            let rules_dir = rules_dir.unwrap_or_else(default_rules_dir);
            let policy_path = policy.unwrap_or_else(default_policy_path);
            let policy = policy::load_policy(&policy_path)?;
            let rules = rules::load_rules(&rules_dir)?;
            let rules = rules::filter_rules(rules, category.as_deref());
            let scan_report = scanner::scan(&rules, policy.planner.max_scan_depth)?;
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
            policy,
            quarantine_root,
        } => {
            let effective_format = if json {
                OutputFormat::Json
            } else if markdown {
                OutputFormat::Markdown
            } else {
                format
            };

            let rules_dir = rules_dir.unwrap_or_else(default_rules_dir);
            let policy_path = policy.unwrap_or_else(default_policy_path);
            let policy = policy::load_policy(&policy_path)?;
            let rules = rules::load_rules(&rules_dir)?;
            let rules = rules::filter_rules(rules, category.as_deref());
            let scan_report = scanner::scan(&rules, policy.planner.max_scan_depth)?;
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
                println!("{}", reporter::render_clean(&clean_report, effective_format)?);

                if let Some(quarantine_root) = quarantine_root {
                    let quarantine_plan = cleaner::build_quarantine_plan(&plan_report, &quarantine_root);
                    println!();
                    println!("{}", reporter::render_quarantine_plan(&quarantine_plan, effective_format)?);
                }
            } else {
                if !yes {
                    anyhow::bail!("clean execution requires --yes");
                }

                let quarantine_root = quarantine_root
                    .ok_or_else(|| anyhow::anyhow!("clean execution requires --quarantine-root"))?;
                let quarantine_plan = cleaner::build_quarantine_plan(&plan_report, &quarantine_root);
                let execution_report = cleaner::execute_quarantine(&quarantine_plan)?;
                println!("{}", reporter::render_execution(&execution_report, effective_format)?);
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
            let effective_format = if json {
                OutputFormat::Json
            } else if markdown {
                OutputFormat::Markdown
            } else {
                format
            };

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
            before,
            after,
        } => {
            let effective_format = if json {
                OutputFormat::Json
            } else if markdown {
                OutputFormat::Markdown
            } else {
                format
            };

            let report = diff::build_diff(&before, &after)?;
            println!("{}", reporter::render_diff(&report, effective_format)?);
        }
        Command::Doctor {
            format,
            json,
            markdown,
            mut docker,
            mut wsl,
            mut ollama,
            mut playwright,
            mut huggingface,
            rules_dir,
            policy,
        } => {
            let effective_format = if json {
                OutputFormat::Json
            } else if markdown {
                OutputFormat::Markdown
            } else {
                format
            };

            if !(docker || wsl || ollama || playwright || huggingface) {
                docker = true;
                wsl = true;
                ollama = true;
                playwright = true;
                huggingface = true;
            }

            let rules_dir = rules_dir.unwrap_or_else(default_rules_dir);
            let policy_path = policy.unwrap_or_else(default_policy_path);
            let loaded_policy = policy::load_policy(&policy_path)?;
            let rules = rules::load_rules(&rules_dir)?;
            let scan_report = scanner::scan(&rules, loaded_policy.planner.max_scan_depth)?;
            let doctor_report = doctor::build_doctor(
                &scan_report,
                doctor::DoctorOptions {
                    docker,
                    wsl,
                    ollama,
                    playwright,
                    huggingface,
                },
                &loaded_policy,
            );
            println!("{}", reporter::render_doctor(&doctor_report, effective_format)?);
        }
    }

    Ok(())
}

fn default_rules_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("rules")
}

fn default_policy_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("config")
        .join("policy.yaml")
}
