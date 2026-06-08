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
pub struct PolicySnapshot {
    pub sensitive_markers: Vec<String>,
    pub planner: PlannerPolicySnapshot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannerPolicySnapshot {
    pub allow_actions: Vec<String>,
    pub skip_modified_within_minutes: u64,
    pub max_scan_depth: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannerPolicy {
    pub skip_modified_within_minutes: u64,
    pub allow_actions: Vec<String>,
    #[serde(default = "default_max_scan_depth")]
    pub max_scan_depth: usize,
}

impl Policy {
    pub fn snapshot(&self) -> PolicySnapshot {
        PolicySnapshot {
            sensitive_markers: self.sensitive_markers.clone(),
            planner: PlannerPolicySnapshot {
                allow_actions: self.planner.allow_actions.clone(),
                skip_modified_within_minutes: self.planner.skip_modified_within_minutes,
                max_scan_depth: self.planner.max_scan_depth,
            },
        }
    }

    pub fn policy_summary(&self) -> String {
        format!(
            "sensitive markers: [{}]; planner actions: [{}]; skip modified within: {}min; max scan depth: {}",
            self.sensitive_markers.join(", "),
            self.planner.allow_actions.join(", "),
            self.planner.skip_modified_within_minutes,
            self.planner.max_scan_depth
        )
    }
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
