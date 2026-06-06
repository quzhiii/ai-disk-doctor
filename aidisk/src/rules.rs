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
    expand_path(pattern)
}

pub fn expand_path(pattern: &str) -> PathBuf {
    let mut expanded = pattern.to_owned();

    if expanded.starts_with("~/") {
        if let Ok(home) = env::var("HOME") {
            expanded = expanded.replacen("~", &home, 1);
        } else {
            expanded = expanded.replacen("~", "/tmp", 1);
        }
    }

    for (key, value) in env::vars() {
        let token = format!("%{key}%");
        if expanded.contains(&token) {
            expanded = expanded.replace(&token, &value);
        }
    }

    PathBuf::from(expanded)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expand_path_supports_home_tilde_and_windows_vars() {
        // unix ~ expansion
        std::env::set_var("HOME", "/home/demo");
        assert_eq!(
            expand_path("~/.cache/huggingface"),
            PathBuf::from("/home/demo/.cache/huggingface")
        );

        // fallback without HOME
        std::env::remove_var("HOME");
        assert_eq!(expand_path("~/unknown"), PathBuf::from("/tmp/unknown"));

        // windows %VAR% expansion
        std::env::set_var("USERPROFILE", "C:\\Users\\demo");
        assert_eq!(
            expand_path("%USERPROFILE%\\.cache\\huggingface"),
            PathBuf::from("C:\\Users\\demo\\.cache\\huggingface")
        );

        // unchanged path
        assert_eq!(
            expand_path("/usr/local/bin/tool"),
            PathBuf::from("/usr/local/bin/tool")
        );
    }
}
