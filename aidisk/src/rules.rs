use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct Rule {
    pub id: String,
    pub name: String,
    pub category: String,
    pub platform: String,
    #[serde(deserialize_with = "deserialize_paths")]
    pub paths: Vec<String>,
    pub risk: RiskLevel,
    pub cleanup: Cleanup,
    #[serde(default)]
    pub exclusions: Vec<String>,
    pub reason: String,
    #[serde(default)]
    pub warnings: Vec<String>,
}

impl<'de> Deserialize<'de> for Rule {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawRule {
            id: String,
            name: String,
            category: String,
            #[serde(default)]
            platform: Option<String>,
            #[serde(default)]
            platforms: Option<Vec<String>>,
            #[serde(deserialize_with = "deserialize_paths")]
            paths: Vec<String>,
            risk: RiskLevel,
            cleanup: Cleanup,
            #[serde(default)]
            exclusions: Vec<String>,
            reason: String,
            #[serde(default)]
            warnings: Vec<String>,
        }

        let raw = RawRule::deserialize(deserializer)?;

        let platform = raw.platform.unwrap_or_else(|| {
            raw.platforms
                .map(|p| p.join(", "))
                .unwrap_or_else(|| "cross-platform".to_string())
        });

        Ok(Rule {
            id: raw.id,
            name: raw.name,
            category: raw.category,
            platform,
            paths: raw.paths,
            risk: raw.risk,
            cleanup: raw.cleanup,
            exclusions: raw.exclusions,
            reason: raw.reason,
            warnings: raw.warnings,
        })
    }
}

fn deserialize_paths<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de;

    #[derive(Deserialize)]
    #[serde(untagged)]
    enum PathsValue {
        Flat(Vec<String>),
        PlatformMap(std::collections::HashMap<String, Vec<String>>),
    }

    match PathsValue::deserialize(deserializer)? {
        PathsValue::Flat(paths) => Ok(paths),
        PathsValue::PlatformMap(map) => {
            #[cfg(target_os = "windows")]
            let key = "windows";
            #[cfg(target_os = "macos")]
            let key = "macos";
            #[cfg(target_os = "linux")]
            let key = "linux";

            Ok(map.get(key).cloned().unwrap_or_default())
        }
    }
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

pub fn expand_windows_path(pattern: &str) -> Option<PathBuf> {
    expand_path(pattern)
}

pub fn expand_path(pattern: &str) -> Option<PathBuf> {
    let mut expanded = pattern.to_owned();

    if expanded.starts_with("~/") {
        let home = env::var_os("HOME")?;
        let suffix = expanded.trim_start_matches("~/");
        expanded = PathBuf::from(home).join(suffix).to_string_lossy().into_owned();
    }

    expanded = expand_windows_env_tokens(&expanded)?;

    Some(PathBuf::from(expanded))
}

fn expand_windows_env_tokens(pattern: &str) -> Option<String> {
    let mut expanded = String::with_capacity(pattern.len());
    let mut rest = pattern;

    while let Some(start) = rest.find('%') {
        expanded.push_str(&rest[..start]);

        let token_start = &rest[start + 1..];
        let Some(end) = token_start.find('%') else {
            expanded.push('%');
            expanded.push_str(token_start);
            return Some(expanded);
        };

        let token = &token_start[..end];
        if token.is_empty() || !token.chars().all(|ch| ch.is_ascii_alphanumeric() || ch == '_') {
            expanded.push('%');
            rest = token_start;
            continue;
        }

        let value = env::var_os(token)?;
        expanded.push_str(&PathBuf::from(value).to_string_lossy());
        rest = &token_start[end + 1..];
    }

    expanded.push_str(rest);
    Some(expanded)
}

#[cfg(test)]
mod tests {
    use crate::test_support::{env_lock, EnvSnapshot};

    use super::*;

    #[test]
    fn expand_path_supports_home_tilde_and_windows_vars() {
        let _env_lock = env_lock();
        let _env_snapshot = EnvSnapshot::capture(&["HOME", "USERPROFILE", "AIDISK_TEST_HOME"]);

        // unix ~ expansion
        std::env::set_var("HOME", "/home/demo");
        assert_eq!(
            expand_path("~/.cache/huggingface"),
            Some(PathBuf::from("/home/demo/.cache/huggingface"))
        );

        // unresolved unix home paths are skipped when HOME is unavailable
        std::env::remove_var("HOME");
        assert_eq!(expand_path("~/unknown"), None);

        // windows %VAR% expansion
        std::env::set_var("USERPROFILE", "C:\\Users\\demo");
        assert_eq!(
            expand_path("%USERPROFILE%\\.cache\\huggingface"),
            Some(PathBuf::from("C:\\Users\\demo\\.cache\\huggingface"))
        );

        std::env::remove_var("AIDISK_TEST_HOME");
        assert_eq!(expand_path("%AIDISK_TEST_HOME%\\cache"), None);

        // unchanged path
        assert_eq!(
            expand_path("/usr/local/bin/tool"),
            Some(PathBuf::from("/usr/local/bin/tool"))
        );
    }
}
