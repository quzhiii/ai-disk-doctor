mod planner;
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
        #[arg(long)]
        category: Option<String>,
        #[arg(long)]
        rules_dir: Option<PathBuf>,
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
            let scan_report = scanner::scan(&rules)?;
            let plan_report = planner::build_plan(&scan_report, safe_only);
            println!("{}", reporter::render_plan(&plan_report, effective_format)?);
        }
    }

    Ok(())
}

fn default_rules_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("rules")
}
