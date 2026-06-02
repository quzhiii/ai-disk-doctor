mod cleaner;
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
        safe_only: bool,
        #[arg(long)]
        category: Option<String>,
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
            let report = scanner::scan(&rules)?;
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
            let scan_report = scanner::scan(&rules)?;
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
            safe_only,
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
            let scan_report = scanner::scan(&rules)?;
            let plan_report = planner::build_plan(
                &scan_report,
                planner::PlanOptions {
                    safe_only,
                    skip_modified_within_minutes: policy.planner.skip_modified_within_minutes,
                    policy,
                },
            );

            if !dry_run {
                anyhow::bail!("clean currently only supports --dry-run");
            }

            let clean_report = cleaner::build_dry_run(&plan_report);
            println!("{}", reporter::render_clean(&clean_report, effective_format)?);
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
