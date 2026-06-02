use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    pub sensitive_markers: Vec<String>,
    pub planner: PlannerPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannerPolicy {
    pub skip_modified_within_minutes: u64,
    pub allow_actions: Vec<String>,
    #[serde(default = "default_max_scan_depth")]
    pub max_scan_depth: usize,
}

fn default_max_scan_depth() -> usize {
    20
}

pub fn load_policy(path: &Path) -> Result<Policy> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("failed to read policy file {}", path.display()))?;
    let policy = serde_yaml::from_str(&content)
        .with_context(|| format!("failed to parse policy file {}", path.display()))?;
    Ok(policy)
}
