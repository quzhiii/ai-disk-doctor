use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

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
    #[serde(skip)]
    pub metadata: RuleMetadata,
}

#[derive(Debug, Clone, Serialize)]
pub struct RuleMetadata {
    pub schema_version: u16,
    pub detector: Detector,
    pub data_kind: String,
    pub recoverability: String,
    pub sensitivity: String,
    pub default_liveness: String,
    pub decision: Decision,
    pub action: RuleAction,
    pub content_access: String,
    pub source: RuleSource,
}

impl Default for RuleMetadata {
    fn default() -> Self {
        Self::legacy()
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RuleSource {
    pub path: String,
    pub schema_version: u16,
    pub digest: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Detector {
    pub paths: Vec<String>,
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Decision {
    pub risk: RiskLevel,
    pub confidence: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RuleAction {
    #[serde(rename = "type")]
    pub action_type: String,
    pub adapter: String,
    pub supports_dry_run: bool,
    pub supports_rollback: bool,
}

impl RuleMetadata {
    pub fn legacy() -> Self {
        Self {
            schema_version: 1,
            detector: Detector {
                paths: Vec::new(),
                evidence: vec!["legacy-rule-migration".to_string()],
            },
            data_kind: "unknown".to_string(),
            recoverability: "unknown".to_string(),
            sensitivity: "unknown".to_string(),
            default_liveness: "unknown".to_string(),
            decision: Decision {
                risk: RiskLevel::Review,
                confidence: "low".to_string(),
            },
            action: RuleAction {
                action_type: "report-only".to_string(),
                adapter: "legacy".to_string(),
                supports_dry_run: true,
                supports_rollback: false,
            },
            content_access: "metadata-only".to_string(),
            source: RuleSource {
                path: "inline".to_string(),
                schema_version: 1,
                digest: String::new(),
            },
        }
    }
}

impl<'de> Deserialize<'de> for Rule {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawRule {
            id: String,
            #[serde(default)]
            name: Option<String>,
            category: String,
            #[serde(default)]
            platform: Option<String>,
            #[serde(default)]
            platforms: Option<Vec<String>>,
            #[serde(default, deserialize_with = "deserialize_optional_paths")]
            paths: Option<Vec<String>>,
            #[serde(default)]
            risk: Option<RiskLevel>,
            #[serde(default)]
            cleanup: Option<Cleanup>,
            #[serde(default)]
            exclusions: Vec<String>,
            #[serde(default)]
            reason: Option<String>,
            #[serde(default)]
            warnings: Vec<String>,
            #[serde(default)]
            schema_version: Option<u16>,
            #[serde(default)]
            detector: Option<RawDetector>,
            #[serde(default)]
            data_kind: Option<String>,
            #[serde(default)]
            recoverability: Option<String>,
            #[serde(default)]
            sensitivity: Option<String>,
            #[serde(default)]
            default_liveness: Option<String>,
            #[serde(default)]
            decision: Option<RawDecision>,
            #[serde(default)]
            action: Option<RawAction>,
            #[serde(default)]
            content_access: Option<String>,
        }

        let raw = RawRule::deserialize(deserializer)?;

        let schema_version = raw.schema_version.unwrap_or(1);
        if !matches!(schema_version, 1 | 2) {
            return Err(serde::de::Error::custom(format!(
                "unsupported rule schema_version {schema_version}"
            )));
        }

        let platform = raw.platform.unwrap_or_else(|| {
            raw.platforms
                .map(|p| p.join(", "))
                .unwrap_or_else(|| "cross-platform".to_string())
        });

        if schema_version == 2 {
            let detector = raw
                .detector
                .ok_or_else(|| serde::de::Error::custom("schema v2 requires detector"))?;
            let decision = raw
                .decision
                .ok_or_else(|| serde::de::Error::custom("schema v2 requires decision"))?;
            let action = raw
                .action
                .ok_or_else(|| serde::de::Error::custom("schema v2 requires action"))?;
            let name = raw
                .name
                .ok_or_else(|| serde::de::Error::custom("schema v2 requires name"))?;
            let reason = raw
                .reason
                .ok_or_else(|| serde::de::Error::custom("schema v2 requires reason"))?;
            validate_v2_dimensions(
                &detector,
                raw.data_kind.as_deref(),
                raw.recoverability.as_deref(),
                raw.sensitivity.as_deref(),
                raw.default_liveness.as_deref(),
                &decision,
                &action,
                raw.content_access.as_deref(),
            )
            .map_err(serde::de::Error::custom)?;

            let execution_method = action.execution_method();
            return Ok(Rule {
                id: raw.id,
                name,
                category: raw.category,
                platform,
                paths: detector.paths.clone(),
                risk: decision.risk,
                cleanup: Cleanup {
                    method: execution_method.to_string(),
                },
                exclusions: raw.exclusions,
                reason,
                warnings: raw.warnings,
                metadata: RuleMetadata {
                    schema_version,
                    detector: Detector {
                        paths: detector.paths,
                        evidence: detector.evidence,
                    },
                    data_kind: raw.data_kind.unwrap_or_default(),
                    recoverability: raw.recoverability.unwrap_or_default(),
                    sensitivity: raw.sensitivity.unwrap_or_default(),
                    default_liveness: raw.default_liveness.unwrap_or_default(),
                    decision: Decision {
                        risk: decision.risk,
                        confidence: decision.confidence,
                    },
                    action: RuleAction {
                        action_type: action.action_type,
                        adapter: action.adapter,
                        supports_dry_run: action.supports_dry_run,
                        supports_rollback: action.supports_rollback,
                    },
                    content_access: raw.content_access.unwrap_or_default(),
                    source: RuleSource {
                        path: "inline".to_string(),
                        schema_version,
                        digest: String::new(),
                    },
                },
            });
        }

        let paths = raw
            .paths
            .ok_or_else(|| serde::de::Error::custom("v1 rule requires paths"))?;
        let risk = raw
            .risk
            .ok_or_else(|| serde::de::Error::custom("v1 rule requires risk"))?;
        let cleanup = raw
            .cleanup
            .ok_or_else(|| serde::de::Error::custom("v1 rule requires cleanup"))?;
        let name = raw
            .name
            .ok_or_else(|| serde::de::Error::custom("v1 rule requires name"))?;
        let reason = raw
            .reason
            .ok_or_else(|| serde::de::Error::custom("v1 rule requires reason"))?;

        Ok(Rule {
            id: raw.id,
            name,
            category: raw.category,
            platform,
            paths: paths.clone(),
            risk,
            cleanup: cleanup.clone(),
            exclusions: raw.exclusions,
            reason,
            warnings: raw.warnings,
            metadata: RuleMetadata {
                detector: Detector {
                    paths,
                    evidence: vec!["legacy-path-pattern".to_string()],
                },
                decision: Decision {
                    risk,
                    confidence: "low".to_string(),
                },
                action: RuleAction {
                    action_type: legacy_action_type(&cleanup.method).to_string(),
                    adapter: "legacy".to_string(),
                    supports_dry_run: true,
                    supports_rollback: cleanup.method == "quarantine",
                },
                ..RuleMetadata::legacy()
            },
        })
    }
}

#[derive(Debug, Deserialize)]
struct RawDetector {
    #[serde(deserialize_with = "deserialize_paths")]
    paths: Vec<String>,
    #[serde(default)]
    evidence: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct RawDecision {
    risk: RiskLevel,
    confidence: String,
}

#[derive(Debug, Deserialize)]
struct RawAction {
    #[serde(rename = "type")]
    action_type: String,
    adapter: String,
    supports_dry_run: bool,
    supports_rollback: bool,
}

impl RawAction {
    fn execution_method(&self) -> &'static str {
        match self.action_type.as_str() {
            "quarantine" => "quarantine",
            "official-command" => "guide",
            "report-only" => "report-only",
            _ => "report-only",
        }
    }
}

fn validate_v2_dimensions(
    detector: &RawDetector,
    data_kind: Option<&str>,
    recoverability: Option<&str>,
    sensitivity: Option<&str>,
    liveness: Option<&str>,
    decision: &RawDecision,
    action: &RawAction,
    content_access: Option<&str>,
) -> Result<()> {
    if detector.paths.is_empty() {
        anyhow::bail!("schema v2 detector.paths must not be empty");
    }
    if detector.evidence.is_empty() {
        anyhow::bail!("schema v2 detector.evidence must not be empty");
    }
    for (name, value) in [
        ("data_kind", data_kind),
        ("recoverability", recoverability),
        ("sensitivity", sensitivity),
        ("default_liveness", liveness),
        ("content_access", content_access),
    ] {
        if value.is_none_or(str::is_empty) {
            anyhow::bail!("schema v2 requires non-empty {name}");
        }
    }
    if !matches!(decision.confidence.as_str(), "low" | "medium" | "high") {
        anyhow::bail!("schema v2 decision.confidence must be low, medium, or high");
    }
    if !matches!(
        action.action_type.as_str(),
        "quarantine" | "official-command" | "report-only"
    ) {
        anyhow::bail!("schema v2 action.type must be quarantine, official-command, or report-only");
    }
    if action.adapter.trim().is_empty() {
        anyhow::bail!("schema v2 action.adapter must not be empty");
    }
    Ok(())
}

fn legacy_action_type(method: &str) -> &'static str {
    match method {
        "quarantine" => "quarantine",
        "guide" => "official-command",
        _ => "report-only",
    }
}

fn deserialize_paths<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
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

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum PathsValue {
    Flat(Vec<String>),
    PlatformMap(std::collections::HashMap<String, Vec<String>>),
}

fn deserialize_optional_paths<'de, D>(deserializer: D) -> Result<Option<Vec<String>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Option::<PathsValue>::deserialize(deserializer)?.map_or(Ok(None), |paths| match paths {
        PathsValue::Flat(paths) => Ok(Some(paths)),
        PathsValue::PlatformMap(map) => {
            #[cfg(target_os = "windows")]
            let key = "windows";
            #[cfg(target_os = "macos")]
            let key = "macos";
            #[cfg(target_os = "linux")]
            let key = "linux";
            Ok(Some(map.get(key).cloned().unwrap_or_default()))
        }
    })
}

#[derive(Debug, Serialize)]
pub struct RuleLintReport {
    pub schema_version: u16,
    pub total_rules: usize,
    pub schema_versions: std::collections::BTreeMap<u16, usize>,
    pub rules: Vec<RuleSource>,
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

        if !matches!(
            path.extension().and_then(|value| value.to_str()),
            Some("yaml" | "yml")
        ) {
            continue;
        }

        let content = fs::read_to_string(&path)
            .with_context(|| format!("failed to read rule file {}", path.display()))?;
        let mut rule: Rule = serde_yaml::from_str(&content)
            .with_context(|| format!("failed to parse rule file {}", path.display()))?;
        rule.metadata.source = RuleSource {
            path: path.display().to_string(),
            schema_version: rule.metadata.schema_version,
            digest: digest_content(&content),
        };
        rules.push(rule);
    }

    rules.sort_by(|a, b| a.id.cmp(&b.id));
    for pair in rules.windows(2) {
        if pair[0].id == pair[1].id {
            anyhow::bail!("duplicate rule id {}", pair[0].id);
        }
    }
    Ok(rules)
}

pub fn lint_rules(rules_dir: &Path) -> Result<RuleLintReport> {
    let rules = load_rules(rules_dir)?;
    let mut schema_versions = std::collections::BTreeMap::new();
    for rule in &rules {
        *schema_versions
            .entry(rule.metadata.schema_version)
            .or_insert(0) += 1;
    }
    Ok(RuleLintReport {
        schema_version: 1,
        total_rules: rules.len(),
        schema_versions,
        rules: rules.into_iter().map(|rule| rule.metadata.source).collect(),
    })
}

fn digest_content(content: &str) -> String {
    let digest = Sha256::digest(content.as_bytes());
    format!("sha256:{digest:x}")
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
        expanded = PathBuf::from(home)
            .join(suffix)
            .to_string_lossy()
            .into_owned();
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
        if token.is_empty()
            || !token
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
        {
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
    fn loads_v1_rule_with_compatibility_metadata() {
        let rule: Rule = serde_yaml::from_str(
            r#"
id: legacy-cache
name: Legacy cache
category: cache
platform: windows
paths:
  - C:\\cache
risk: safe
cleanup:
  method: quarantine
reason: Legacy rule
"#,
        )
        .expect("v1 rule should load");

        assert_eq!(rule.metadata.schema_version, 1);
        assert_eq!(rule.metadata.action.action_type, "quarantine");
        assert_eq!(rule.metadata.source.digest, "");
    }

    #[test]
    fn loads_v2_rule_with_separated_decision_and_action() {
        let rule: Rule = serde_yaml::from_str(
            r#"
schema_version: 2
id: model-cache
name: Model cache
category: models
platform: cross-platform
detector:
  paths:
    - ~/.cache/models
  evidence:
    - path-pattern
data_kind: model-cache
recoverability: redownload
sensitivity: none
default_liveness: unknown
decision:
  risk: review
  confidence: medium
action:
  type: official-command
  adapter: huggingface
  supports_dry_run: true
  supports_rollback: false
content_access: metadata-only
reason: Review model cache
"#,
        )
        .expect("v2 rule should load");

        assert_eq!(rule.metadata.schema_version, 2);
        assert_eq!(rule.paths, vec!["~/.cache/models"]);
        assert_eq!(rule.risk, RiskLevel::Review);
        assert_eq!(rule.cleanup.method, "guide");
        assert_eq!(rule.metadata.action.adapter, "huggingface");
    }

    #[test]
    fn rejects_v2_rule_without_required_dimensions() {
        let error = serde_yaml::from_str::<Rule>(
            "schema_version: 2\nid: incomplete\nname: Incomplete\ncategory: models\n",
        )
        .expect_err("incomplete v2 rule should fail");

        assert!(error.to_string().contains("schema v2 requires detector"));
    }

    #[test]
    fn rejects_unknown_rule_schema_version() {
        for version in ["0", "3"] {
            let error = serde_yaml::from_str::<Rule>(&format!(
                "schema_version: {version}\nid: future\nname: Future\ncategory: models\n"
            ))
            .expect_err("unsupported schema should fail");

            assert!(error
                .to_string()
                .contains(&format!("unsupported rule schema_version {version}")));
        }
    }

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
