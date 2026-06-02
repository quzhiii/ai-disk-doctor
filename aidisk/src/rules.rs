use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub id: String,
    pub name: String,
    pub category: String,
    pub platform: String,
    pub paths: Vec<String>,
    pub risk: RiskLevel,
    pub cleanup: Cleanup,
    #[serde(default)]
    pub exclusions: Vec<String>,
    pub reason: String,
    #[serde(default)]
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RiskLevel {
    Safe,
    Review,
    Dangerous,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cleanup {
    pub method: String,
}

pub fn load_rules(rules_dir: &Path) -> Result<Vec<Rule>> {
    let mut rules = Vec::new();

    for entry in fs::read_dir(rules_dir)
        .with_context(|| format!("failed to read rules directory {}", rules_dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();

        if !matches!(path.extension().and_then(|value| value.to_str()), Some("yaml" | "yml")) {
            continue;
        }

        let content = fs::read_to_string(&path)
            .with_context(|| format!("failed to read rule file {}", path.display()))?;
        let rule: Rule = serde_yaml::from_str(&content)
            .with_context(|| format!("failed to parse rule file {}", path.display()))?;
        rules.push(rule);
    }

    rules.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(rules)
}

pub fn filter_rules(rules: Vec<Rule>, category: Option<&str>) -> Vec<Rule> {
    match category {
        Some(category) => rules
            .into_iter()
            .filter(|rule| rule.category.eq_ignore_ascii_case(category))
            .collect(),
        None => rules,
    }
}

pub fn expand_windows_path(pattern: &str) -> PathBuf {
    let mut expanded = pattern.to_owned();

    for (key, value) in env::vars() {
        let token = format!("%{key}%");
        if expanded.contains(&token) {
            expanded = expanded.replace(&token, &value);
        }
    }

    PathBuf::from(expanded)
}
