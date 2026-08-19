use std::collections::BTreeMap;

use serde::Serialize;

use crate::rules::{RiskLevel, Rule};
use crate::scanner::{Finding, ScanReport, Volume};

pub const EXPLAINABILITY_CONTRACT: &str = "explainability-v1";
pub const EXPLAINABILITY_SCHEMA_VERSION: u16 = 1;
const MAX_PATH_GROUPS_PER_RULE: usize = 50;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ExplainabilityReport {
    pub contract: String,
    pub schema_version: u16,
    pub accounting: AccountingSemantics,
    pub storage: StorageExplanation,
    pub evidence: EvidenceSummary,
    pub volumes: Vec<VolumeExplanation>,
    pub categories: Vec<CategoryExplanation>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct AccountingSemantics {
    pub byte_basis: ByteBasis,
    pub deduplication: DeduplicationSemantics,
    pub category_sum_matches_storage: bool,
    pub rule_sum_matches_category: bool,
    pub path_group_sum_matches_rule: bool,
    pub partial_bytes_in_total_size: bool,
    pub partial_bytes_excluded_from_handling_totals: bool,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ByteBasis {
    LogicalRuleMatchLowerBound,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum DeduplicationSemantics {
    PerRulePathOnly,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct StorageExplanation {
    pub observed_bytes: u64,
    pub total_size_bytes: u64,
    pub potential_bytes: u64,
    pub actionable_bytes: u64,
    pub quarantine_bytes: u64,
    pub official_cleanup_bytes: u64,
    pub report_only_bytes: u64,
    pub partial_bytes: u64,
    pub reclaimable_safe_bytes: u64,
    pub safe_bytes: u64,
    pub review_bytes: u64,
    pub dangerous_bytes: u64,
    pub system_bytes: u64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct EvidenceSummary {
    pub status: EvidenceStatus,
    pub partial_findings: usize,
    pub warnings: Vec<EvidenceWarning>,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum EvidenceStatus {
    Complete,
    Partial,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct EvidenceWarning {
    pub code: EvidenceWarningCode,
    pub message: String,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum EvidenceWarningCode {
    PartialLowerBound,
    RuleWarning,
    PartialReason,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct VolumeExplanation {
    pub name: String,
    pub mount_point: String,
    pub total_bytes: u64,
    pub available_bytes: u64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CategoryExplanation {
    pub category_id: String,
    pub category_name: String,
    pub observed_bytes: u64,
    pub partial_bytes: u64,
    pub total_size_bytes: u64,
    pub handling: HandlingTotals,
    pub risk: RiskTotals,
    pub rules: Vec<RuleExplanation>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Default)]
pub struct HandlingTotals {
    pub quarantine_bytes: u64,
    pub official_cleanup_bytes: u64,
    pub report_only_bytes: u64,
    pub partial_bytes: u64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Default)]
pub struct RiskTotals {
    pub safe_bytes: u64,
    pub review_bytes: u64,
    pub dangerous_bytes: u64,
    pub system_bytes: u64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RuleExplanation {
    pub rule_id: String,
    pub rule_name: String,
    pub observed_bytes: u64,
    pub partial_bytes: u64,
    pub total_size_bytes: u64,
    pub handling_mode: HandlingMode,
    pub risk: RiskCode,
    pub path_group_summary: PathGroupSummary,
    pub path_groups: Vec<PathGroup>,
    pub rationale: RationaleEvidence,
    pub provenance: RuleProvenance,
    pub recoverability: RecoverabilityEvidence,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum HandlingMode {
    Quarantine,
    OfficialManual,
    ReportOnly,
    Unknown,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum RiskCode {
    Safe,
    Review,
    Dangerous,
    System,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PathGroupSummary {
    pub total_path_groups: usize,
    pub included_path_groups: usize,
    pub omitted_path_groups: usize,
    pub omitted_bytes: u64,
    pub limit: usize,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PathGroup {
    pub path: PathEvidence,
    pub observed_bytes: u64,
    pub partial_bytes: u64,
    pub total_size_bytes: u64,
    pub finding_count: usize,
    pub partial: bool,
    pub partial_reasons: Vec<String>,
    pub volume: Option<VolumeReference>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PathEvidence {
    pub raw_path: String,
    pub display_path: String,
    pub disclosure: PathDisclosure,
    pub sensitivity: String,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum PathDisclosure {
    RawLocalPath,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct VolumeReference {
    pub name: String,
    pub mount_point: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RationaleEvidence {
    pub reason: String,
    pub warnings: Vec<String>,
    pub partial_reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RuleProvenance {
    pub source: RuleSourceEvidence,
    pub detector_evidence: Vec<String>,
    pub content_access: String,
    pub confidence: String,
    pub action_type: String,
    pub action_adapter: String,
    pub supports_dry_run: bool,
    pub supports_rollback: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RuleSourceEvidence {
    pub path: String,
    pub schema_version: u16,
    pub digest: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RecoverabilityEvidence {
    pub declared: String,
    pub kind: RecoverabilityKind,
    pub reversibility: Reversibility,
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum RecoverabilityKind {
    Redownload,
    Rebuildable,
    Unknown,
    OtherDeclared,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Reversibility {
    Supported,
    NotGuaranteed,
    Unknown,
}

#[derive(Debug, Default)]
struct CategoryAccumulator {
    observed_bytes: u64,
    partial_bytes: u64,
    handling: HandlingTotals,
    risk: RiskTotals,
    rules: BTreeMap<String, RuleAccumulator>,
}

#[derive(Debug)]
struct RuleAccumulator {
    rule_name: String,
    observed_bytes: u64,
    partial_bytes: u64,
    handling_mode: HandlingMode,
    risk: RiskCode,
    sensitivity: String,
    paths: BTreeMap<String, PathAccumulator>,
    rationale: RationaleEvidence,
    provenance: RuleProvenance,
    recoverability: RecoverabilityEvidence,
}

#[derive(Debug, Default)]
struct PathAccumulator {
    observed_bytes: u64,
    partial_bytes: u64,
    finding_count: usize,
    partial: bool,
    partial_reasons: Vec<String>,
}

pub(crate) fn build(report: &ScanReport, rules: &[Rule]) -> ExplainabilityReport {
    let rule_lookup: BTreeMap<&str, &Rule> =
        rules.iter().map(|rule| (rule.id.as_str(), rule)).collect();
    let mut categories: BTreeMap<String, CategoryAccumulator> = BTreeMap::new();

    for finding in report.findings.iter().filter(|finding| finding.exists) {
        let category_id = non_empty_or_unknown(&finding.category);
        let rule = rule_lookup.get(finding.id.as_str()).copied();
        let category = categories.entry(category_id).or_default();
        let mode = handling_mode(finding);
        let risk = risk_code(finding.risk);

        if finding.partial {
            category.partial_bytes = category.partial_bytes.saturating_add(finding.size_bytes);
            category.handling.partial_bytes = category
                .handling
                .partial_bytes
                .saturating_add(finding.size_bytes);
        } else {
            category.observed_bytes = category.observed_bytes.saturating_add(finding.size_bytes);
            add_handling_bytes(&mut category.handling, mode, finding.size_bytes);
        }
        add_risk_bytes(&mut category.risk, risk, finding.size_bytes);

        let rule_entry = category.rules.entry(finding.id.clone()).or_insert_with(|| {
            let provenance = rule
                .map(rule_provenance)
                .unwrap_or_else(|| unknown_provenance(&finding.id));
            let recoverability = rule
                .map(rule_recoverability)
                .unwrap_or_else(unknown_recoverability);
            RuleAccumulator {
                rule_name: rule
                    .map(|value| value.name.clone())
                    .unwrap_or_else(|| finding.name.clone()),
                observed_bytes: 0,
                partial_bytes: 0,
                handling_mode: mode,
                risk,
                sensitivity: rule
                    .map(|value| non_empty_or_unknown(&value.metadata.sensitivity))
                    .unwrap_or_else(|| "unknown".to_string()),
                paths: BTreeMap::new(),
                rationale: RationaleEvidence {
                    reason: rule
                        .map(|value| value.reason.clone())
                        .unwrap_or_else(|| finding.reason.clone()),
                    warnings: rule
                        .map(|value| value.warnings.clone())
                        .unwrap_or_else(|| finding.warnings.clone()),
                    partial_reasons: Vec::new(),
                },
                provenance,
                recoverability,
            }
        });

        if finding.partial {
            rule_entry.partial_bytes = rule_entry.partial_bytes.saturating_add(finding.size_bytes);
            rule_entry
                .rationale
                .partial_reasons
                .extend(finding.partial_reasons.clone());
        } else {
            rule_entry.observed_bytes =
                rule_entry.observed_bytes.saturating_add(finding.size_bytes);
        }

        let path_entry = rule_entry.paths.entry(finding.path.clone()).or_default();
        if finding.partial {
            path_entry.partial_bytes = path_entry.partial_bytes.saturating_add(finding.size_bytes);
        } else {
            path_entry.observed_bytes =
                path_entry.observed_bytes.saturating_add(finding.size_bytes);
        }
        path_entry.finding_count += 1;
        path_entry.partial |= finding.partial;
        extend_unique(&mut path_entry.partial_reasons, &finding.partial_reasons);
    }

    ExplainabilityReport {
        contract: EXPLAINABILITY_CONTRACT.to_string(),
        schema_version: EXPLAINABILITY_SCHEMA_VERSION,
        accounting: AccountingSemantics {
            byte_basis: ByteBasis::LogicalRuleMatchLowerBound,
            deduplication: DeduplicationSemantics::PerRulePathOnly,
            category_sum_matches_storage: true,
            rule_sum_matches_category: true,
            path_group_sum_matches_rule: true,
            partial_bytes_in_total_size: true,
            partial_bytes_excluded_from_handling_totals: true,
        },
        storage: StorageExplanation {
            observed_bytes: report.summary.observed_bytes,
            total_size_bytes: report.summary.total_size_bytes,
            potential_bytes: report.summary.potential_bytes,
            actionable_bytes: report.summary.actionable_bytes,
            quarantine_bytes: report.summary.quarantine_bytes,
            official_cleanup_bytes: report.summary.official_cleanup_bytes,
            report_only_bytes: report.summary.report_only_bytes,
            partial_bytes: report.summary.partial_bytes,
            reclaimable_safe_bytes: report.summary.reclaimable_safe_bytes,
            safe_bytes: report.summary.safe_bytes,
            review_bytes: report.summary.review_bytes,
            dangerous_bytes: report.summary.dangerous_bytes,
            system_bytes: report.summary.system_bytes,
        },
        evidence: EvidenceSummary {
            status: if report.summary.partial_bytes > 0 {
                EvidenceStatus::Partial
            } else {
                EvidenceStatus::Complete
            },
            partial_findings: report.summary.partial_findings,
            warnings: collect_report_warnings(report),
        },
        volumes: report.volumes.iter().map(volume_explanation).collect(),
        categories: categories
            .into_iter()
            .map(|(category_id, category)| build_category(category_id, category, &report.volumes))
            .collect(),
    }
}

fn build_category(
    category_id: String,
    category: CategoryAccumulator,
    volumes: &[Volume],
) -> CategoryExplanation {
    CategoryExplanation {
        category_name: category_id.clone(),
        category_id,
        observed_bytes: category.observed_bytes,
        partial_bytes: category.partial_bytes,
        total_size_bytes: category
            .observed_bytes
            .saturating_add(category.partial_bytes),
        handling: category.handling,
        risk: category.risk,
        rules: category
            .rules
            .into_iter()
            .map(|(rule_id, rule)| build_rule(rule_id, rule, volumes))
            .collect(),
    }
}

fn build_rule(rule_id: String, rule: RuleAccumulator, volumes: &[Volume]) -> RuleExplanation {
    let total_size_bytes = rule.observed_bytes.saturating_add(rule.partial_bytes);
    let all_groups = sorted_path_groups(rule.paths, &rule.sensitivity, volumes);
    let total_path_groups = all_groups.len();
    let omitted_path_groups = total_path_groups.saturating_sub(MAX_PATH_GROUPS_PER_RULE);
    let omitted_bytes = all_groups
        .iter()
        .skip(MAX_PATH_GROUPS_PER_RULE)
        .map(|group| group.total_size_bytes)
        .sum();
    let path_groups: Vec<_> = all_groups
        .into_iter()
        .take(MAX_PATH_GROUPS_PER_RULE)
        .collect();

    RuleExplanation {
        rule_id,
        rule_name: rule.rule_name,
        observed_bytes: rule.observed_bytes,
        partial_bytes: rule.partial_bytes,
        total_size_bytes,
        handling_mode: rule.handling_mode,
        risk: rule.risk,
        path_group_summary: PathGroupSummary {
            total_path_groups,
            included_path_groups: path_groups.len(),
            omitted_path_groups,
            omitted_bytes,
            limit: MAX_PATH_GROUPS_PER_RULE,
        },
        path_groups,
        rationale: RationaleEvidence {
            partial_reasons: dedupe(rule.rationale.partial_reasons),
            ..rule.rationale
        },
        provenance: rule.provenance,
        recoverability: rule.recoverability,
    }
}

fn sorted_path_groups(
    paths: BTreeMap<String, PathAccumulator>,
    sensitivity: &str,
    volumes: &[Volume],
) -> Vec<PathGroup> {
    let mut groups: Vec<_> = paths
        .into_iter()
        .map(|(path, group)| {
            let total_size_bytes = group.observed_bytes.saturating_add(group.partial_bytes);
            PathGroup {
                volume: matching_volume(&path, volumes),
                path: PathEvidence {
                    raw_path: path.clone(),
                    display_path: path,
                    disclosure: PathDisclosure::RawLocalPath,
                    sensitivity: sensitivity.to_string(),
                },
                observed_bytes: group.observed_bytes,
                partial_bytes: group.partial_bytes,
                total_size_bytes,
                finding_count: group.finding_count,
                partial: group.partial,
                partial_reasons: group.partial_reasons,
            }
        })
        .collect();

    groups.sort_by(|a, b| {
        b.total_size_bytes
            .cmp(&a.total_size_bytes)
            .then_with(|| a.path.raw_path.cmp(&b.path.raw_path))
    });
    groups
}

fn non_empty_or_unknown(value: &str) -> String {
    if value.trim().is_empty() {
        "unknown".to_string()
    } else {
        value.to_string()
    }
}

fn handling_mode(finding: &Finding) -> HandlingMode {
    match finding.action.as_str() {
        "quarantine" => HandlingMode::Quarantine,
        "guide" => HandlingMode::OfficialManual,
        "report-only" => HandlingMode::ReportOnly,
        _ => HandlingMode::Unknown,
    }
}

fn risk_code(risk: RiskLevel) -> RiskCode {
    match risk {
        RiskLevel::Safe => RiskCode::Safe,
        RiskLevel::Review => RiskCode::Review,
        RiskLevel::Dangerous => RiskCode::Dangerous,
        RiskLevel::System => RiskCode::System,
    }
}

fn add_handling_bytes(totals: &mut HandlingTotals, mode: HandlingMode, bytes: u64) {
    match mode {
        HandlingMode::Quarantine => {
            totals.quarantine_bytes = totals.quarantine_bytes.saturating_add(bytes)
        }
        HandlingMode::OfficialManual => {
            totals.official_cleanup_bytes = totals.official_cleanup_bytes.saturating_add(bytes)
        }
        HandlingMode::ReportOnly => {
            totals.report_only_bytes = totals.report_only_bytes.saturating_add(bytes)
        }
        HandlingMode::Unknown => {}
    }
}

fn add_risk_bytes(totals: &mut RiskTotals, risk: RiskCode, bytes: u64) {
    match risk {
        RiskCode::Safe => totals.safe_bytes = totals.safe_bytes.saturating_add(bytes),
        RiskCode::Review => totals.review_bytes = totals.review_bytes.saturating_add(bytes),
        RiskCode::Dangerous => {
            totals.dangerous_bytes = totals.dangerous_bytes.saturating_add(bytes)
        }
        RiskCode::System => totals.system_bytes = totals.system_bytes.saturating_add(bytes),
    }
}

fn rule_provenance(rule: &Rule) -> RuleProvenance {
    RuleProvenance {
        source: RuleSourceEvidence {
            path: rule.metadata.source.path.clone(),
            schema_version: rule.metadata.source.schema_version,
            digest: rule.metadata.source.digest.clone(),
        },
        detector_evidence: rule.metadata.detector.evidence.clone(),
        content_access: rule.metadata.content_access.clone(),
        confidence: rule.metadata.decision.confidence.clone(),
        action_type: rule.metadata.action.action_type.clone(),
        action_adapter: rule.metadata.action.adapter.clone(),
        supports_dry_run: rule.metadata.action.supports_dry_run,
        supports_rollback: rule.metadata.action.supports_rollback,
    }
}

fn rule_recoverability(rule: &Rule) -> RecoverabilityEvidence {
    RecoverabilityEvidence {
        declared: non_empty_or_unknown(&rule.metadata.recoverability),
        kind: recoverability_kind(&rule.metadata.recoverability),
        reversibility: if rule.metadata.action.supports_rollback {
            Reversibility::Supported
        } else {
            Reversibility::NotGuaranteed
        },
        evidence: vec![
            format!("recoverability={}", rule.metadata.recoverability),
            format!(
                "supports_rollback={}",
                rule.metadata.action.supports_rollback
            ),
        ],
    }
}

fn recoverability_kind(value: &str) -> RecoverabilityKind {
    match value {
        "redownload" => RecoverabilityKind::Redownload,
        "rebuildable" | "regenerable" | "derived" => RecoverabilityKind::Rebuildable,
        "" | "unknown" => RecoverabilityKind::Unknown,
        _ => RecoverabilityKind::OtherDeclared,
    }
}

fn unknown_provenance(rule_id: &str) -> RuleProvenance {
    RuleProvenance {
        source: RuleSourceEvidence {
            path: "unknown".to_string(),
            schema_version: 0,
            digest: "unknown".to_string(),
        },
        detector_evidence: vec![format!("rule provenance unavailable for {rule_id}")],
        content_access: "unknown".to_string(),
        confidence: "unknown".to_string(),
        action_type: "unknown".to_string(),
        action_adapter: "unknown".to_string(),
        supports_dry_run: false,
        supports_rollback: false,
    }
}

fn unknown_recoverability() -> RecoverabilityEvidence {
    RecoverabilityEvidence {
        declared: "unknown".to_string(),
        kind: RecoverabilityKind::Unknown,
        reversibility: Reversibility::Unknown,
        evidence: vec!["rule metadata unavailable".to_string()],
    }
}

fn volume_explanation(volume: &Volume) -> VolumeExplanation {
    VolumeExplanation {
        name: volume.name.clone(),
        mount_point: volume.mount_point.clone(),
        total_bytes: volume.total_bytes,
        available_bytes: volume.available_bytes,
    }
}

fn matching_volume(path: &str, volumes: &[Volume]) -> Option<VolumeReference> {
    volumes
        .iter()
        .filter(|volume| path_matches_mount(path, &volume.mount_point))
        .max_by_key(|volume| volume.mount_point.len())
        .map(|volume| VolumeReference {
            name: volume.name.clone(),
            mount_point: volume.mount_point.clone(),
        })
}

fn path_matches_mount(path: &str, mount_point: &str) -> bool {
    if mount_point.trim().is_empty() {
        return false;
    }
    let path = normalize_path(path);
    let mount = normalize_path(mount_point);
    path == mount || path.starts_with(&(mount.trim_end_matches('/').to_string() + "/"))
}

fn normalize_path(path: &str) -> String {
    path.replace('\\', "/").to_ascii_lowercase()
}

fn collect_report_warnings(report: &ScanReport) -> Vec<EvidenceWarning> {
    let mut warnings = Vec::new();
    if report.summary.partial_bytes > 0 {
        push_warning(
            &mut warnings,
            EvidenceWarningCode::PartialLowerBound,
            "partial findings are lower-bound evidence and are excluded from handling totals",
        );
    }
    for finding in &report.findings {
        for warning in &finding.warnings {
            push_warning(&mut warnings, EvidenceWarningCode::RuleWarning, warning);
        }
        for reason in &finding.partial_reasons {
            push_warning(&mut warnings, EvidenceWarningCode::PartialReason, reason);
        }
    }
    warnings
}

fn push_warning(warnings: &mut Vec<EvidenceWarning>, code: EvidenceWarningCode, message: &str) {
    if !warnings
        .iter()
        .any(|warning| warning.code == code && warning.message == message)
    {
        warnings.push(EvidenceWarning {
            code,
            message: message.to_string(),
        });
    }
}

fn extend_unique(target: &mut Vec<String>, values: &[String]) {
    for value in values {
        if !target.contains(value) {
            target.push(value.clone());
        }
    }
}

fn dedupe(values: Vec<String>) -> Vec<String> {
    let mut output = Vec::new();
    extend_unique(&mut output, &values);
    output
}

#[cfg(test)]
mod tests {
    use chrono::Local;

    use super::*;
    use crate::rules::{Cleanup, Decision, Detector, RuleAction, RuleMetadata, RuleSource};
    use crate::scanner::Summary;

    fn rule_with(
        id: &str,
        method: &str,
        risk: RiskLevel,
        recoverability: &str,
        supports_rollback: bool,
    ) -> Rule {
        Rule {
            id: id.to_string(),
            name: format!("Rule {id}"),
            category: "models".to_string(),
            platform: "cross-platform".to_string(),
            paths: vec!["C:\\AI".to_string()],
            risk,
            cleanup: Cleanup {
                method: method.to_string(),
            },
            exclusions: Vec::new(),
            reason: "metadata reason".to_string(),
            warnings: vec!["metadata warning".to_string()],
            metadata: RuleMetadata {
                schema_version: 2,
                detector: Detector {
                    paths: vec!["C:\\AI".to_string()],
                    evidence: vec!["path-pattern".to_string()],
                },
                data_kind: "model-file".to_string(),
                recoverability: recoverability.to_string(),
                sensitivity: "none".to_string(),
                default_liveness: "unknown".to_string(),
                decision: Decision {
                    risk,
                    confidence: "medium".to_string(),
                },
                action: RuleAction {
                    action_type: match method {
                        "quarantine" => "quarantine",
                        "guide" => "official-command",
                        "report-only" => "report-only",
                        _ => "unknown",
                    }
                    .to_string(),
                    adapter: "none".to_string(),
                    supports_dry_run: true,
                    supports_rollback,
                },
                content_access: "metadata-only".to_string(),
                source: RuleSource {
                    path: format!("rules/{id}.yaml"),
                    schema_version: 2,
                    digest: format!("sha256:{id}"),
                },
            },
        }
    }

    fn finding(
        id: &str,
        path: &str,
        size_bytes: u64,
        partial: bool,
        risk: RiskLevel,
        action: &str,
    ) -> Finding {
        Finding {
            id: id.to_string(),
            name: format!("Finding {id}"),
            category: "models".to_string(),
            path: path.to_string(),
            exists: true,
            size_bytes,
            partial,
            partial_reasons: if partial {
                vec!["depth limit".to_string()]
            } else {
                Vec::new()
            },
            risk,
            action: action.to_string(),
            reason: "finding reason".to_string(),
            warnings: Vec::new(),
        }
    }

    fn report(findings: Vec<Finding>, summary: Summary, volumes: Vec<Volume>) -> ScanReport {
        ScanReport {
            scan_time: Local::now(),
            policy: None,
            volumes,
            findings,
            summary,
        }
    }

    #[test]
    fn builds_grouped_contract_with_provenance_and_volume_evidence() {
        let scan = report(
            vec![finding(
                "model",
                "C:\\AI\\models",
                42,
                false,
                RiskLevel::Review,
                "report-only",
            )],
            Summary {
                observed_bytes: 42,
                total_size_bytes: 42,
                review_bytes: 42,
                report_only_bytes: 42,
                matched_paths: 1,
                ..Summary::default()
            },
            vec![Volume {
                name: "System".to_string(),
                mount_point: "C:\\".to_string(),
                total_bytes: 1000,
                available_bytes: 500,
            }],
        );

        let contract = build(
            &scan,
            &[rule_with(
                "model",
                "report-only",
                RiskLevel::Review,
                "unknown",
                false,
            )],
        );

        let rule = &contract.categories[0].rules[0];
        assert_eq!(contract.contract, EXPLAINABILITY_CONTRACT);
        assert_eq!(contract.schema_version, 1);
        assert_eq!(contract.evidence.status, EvidenceStatus::Complete);
        assert_eq!(contract.categories[0].observed_bytes, 42);
        assert_eq!(rule.path_groups[0].total_size_bytes, 42);
        assert_eq!(rule.handling_mode, HandlingMode::ReportOnly);
        assert_eq!(rule.risk, RiskCode::Review);
        assert_eq!(rule.provenance.source.digest, "sha256:model");
        assert_eq!(rule.path_groups[0].volume.as_ref().unwrap().name, "System");
        assert_eq!(
            rule.path_groups[0].path.disclosure,
            PathDisclosure::RawLocalPath
        );
        assert_eq!(
            rule.recoverability.reversibility,
            Reversibility::NotGuaranteed
        );
    }

    #[test]
    fn keeps_handling_and_risk_totals_independent_for_partial_evidence() {
        let scan = report(
            vec![
                finding(
                    "quarantine",
                    "C:\\AI\\safe",
                    5,
                    false,
                    RiskLevel::Safe,
                    "quarantine",
                ),
                finding(
                    "report-only",
                    "C:\\AI\\review",
                    7,
                    false,
                    RiskLevel::Review,
                    "report-only",
                ),
                finding(
                    "report-only",
                    "C:\\AI\\review-partial",
                    4,
                    true,
                    RiskLevel::Review,
                    "report-only",
                ),
            ],
            Summary {
                observed_bytes: 12,
                total_size_bytes: 16,
                quarantine_bytes: 5,
                report_only_bytes: 7,
                partial_bytes: 4,
                safe_bytes: 5,
                review_bytes: 11,
                partial_findings: 1,
                ..Summary::default()
            },
            Vec::new(),
        );

        let contract = build(
            &scan,
            &[
                rule_with(
                    "quarantine",
                    "quarantine",
                    RiskLevel::Safe,
                    "rebuildable",
                    true,
                ),
                rule_with(
                    "report-only",
                    "report-only",
                    RiskLevel::Review,
                    "unknown",
                    false,
                ),
            ],
        );
        let category = &contract.categories[0];

        assert_eq!(category.observed_bytes, 12);
        assert_eq!(category.partial_bytes, 4);
        assert_eq!(category.total_size_bytes, 16);
        assert_eq!(category.handling.quarantine_bytes, 5);
        assert_eq!(category.handling.report_only_bytes, 7);
        assert_eq!(category.handling.partial_bytes, 4);
        assert_eq!(category.risk.safe_bytes, 5);
        assert_eq!(category.risk.review_bytes, 11);
        assert_eq!(category.rules[0].handling_mode, HandlingMode::Quarantine);
        assert_eq!(category.rules[1].handling_mode, HandlingMode::ReportOnly);
        assert_eq!(contract.evidence.status, EvidenceStatus::Partial);
    }

    #[test]
    fn review_guide_items_remain_official_manual_not_quarantine() {
        let scan = report(
            vec![finding(
                "guide-rule",
                "C:\\AI\\cache",
                9,
                false,
                RiskLevel::Review,
                "guide",
            )],
            Summary {
                observed_bytes: 9,
                total_size_bytes: 9,
                official_cleanup_bytes: 9,
                review_bytes: 9,
                ..Summary::default()
            },
            Vec::new(),
        );

        let contract = build(
            &scan,
            &[rule_with(
                "guide-rule",
                "guide",
                RiskLevel::Review,
                "redownload",
                false,
            )],
        );

        let category = &contract.categories[0];
        assert_eq!(category.handling.official_cleanup_bytes, 9);
        assert_eq!(category.risk.review_bytes, 9);
        assert_eq!(
            category.rules[0].handling_mode,
            HandlingMode::OfficialManual
        );
        assert_eq!(
            category.rules[0].recoverability.kind,
            RecoverabilityKind::Redownload
        );
        assert_eq!(
            category.rules[0].recoverability.reversibility,
            Reversibility::NotGuaranteed
        );
    }

    #[test]
    fn unknown_rule_metadata_keeps_unknown_recoverability_and_provenance() {
        let scan = report(
            vec![finding(
                "unknown-rule",
                "C:\\AI\\unknown",
                10,
                true,
                RiskLevel::Review,
                "quarantine",
            )],
            Summary {
                total_size_bytes: 10,
                partial_bytes: 10,
                partial_findings: 1,
                review_bytes: 10,
                ..Summary::default()
            },
            Vec::new(),
        );

        let contract = build(&scan, &[]);
        let rule = &contract.categories[0].rules[0];

        assert_eq!(contract.storage.observed_bytes, 0);
        assert_eq!(contract.categories[0].partial_bytes, 10);
        assert_eq!(rule.path_groups[0].partial_bytes, 10);
        assert!(rule.path_groups[0].partial);
        assert_eq!(rule.provenance.source.path, "unknown");
        assert_eq!(rule.recoverability.kind, RecoverabilityKind::Unknown);
        assert_eq!(rule.recoverability.reversibility, Reversibility::Unknown);
    }

    #[test]
    fn groups_multiple_findings_by_rule_path_and_reports_byte_accounting() {
        let scan = report(
            vec![
                finding(
                    "same-path",
                    "C:\\AI\\target",
                    3,
                    false,
                    RiskLevel::Safe,
                    "quarantine",
                ),
                finding(
                    "same-path",
                    "C:\\AI\\target",
                    4,
                    false,
                    RiskLevel::Safe,
                    "quarantine",
                ),
            ],
            Summary {
                observed_bytes: 7,
                total_size_bytes: 7,
                quarantine_bytes: 7,
                safe_bytes: 7,
                ..Summary::default()
            },
            Vec::new(),
        );

        let contract = build(
            &scan,
            &[rule_with(
                "same-path",
                "quarantine",
                RiskLevel::Safe,
                "rebuildable",
                true,
            )],
        );
        let rule = &contract.categories[0].rules[0];

        assert!(contract.accounting.category_sum_matches_storage);
        assert!(contract.accounting.rule_sum_matches_category);
        assert!(contract.accounting.path_group_sum_matches_rule);
        assert_eq!(rule.observed_bytes, 7);
        assert_eq!(rule.path_group_summary.total_path_groups, 1);
        assert_eq!(rule.path_groups[0].finding_count, 2);
        assert_eq!(rule.path_groups[0].observed_bytes, 7);
    }

    #[test]
    fn bounds_path_groups_and_reports_omitted_bytes() {
        let findings: Vec<_> = (0..55)
            .map(|index| {
                finding(
                    "many-paths",
                    &format!("C:\\AI\\path-{index:02}"),
                    index + 1,
                    false,
                    RiskLevel::Review,
                    "report-only",
                )
            })
            .collect();
        let total: u64 = (1..=55).sum();
        let omitted: u64 = (1..=5).sum();
        let scan = report(
            findings,
            Summary {
                observed_bytes: total,
                total_size_bytes: total,
                report_only_bytes: total,
                review_bytes: total,
                ..Summary::default()
            },
            Vec::new(),
        );

        let contract = build(
            &scan,
            &[rule_with(
                "many-paths",
                "report-only",
                RiskLevel::Review,
                "unknown",
                false,
            )],
        );
        let summary = &contract.categories[0].rules[0].path_group_summary;

        assert_eq!(summary.total_path_groups, 55);
        assert_eq!(summary.included_path_groups, MAX_PATH_GROUPS_PER_RULE);
        assert_eq!(summary.omitted_path_groups, 5);
        assert_eq!(summary.omitted_bytes, omitted);
    }

    #[test]
    fn volume_mapping_uses_longest_matching_mount_and_keeps_unknown_unknown() {
        let volumes = vec![
            Volume {
                name: "System".to_string(),
                mount_point: "C:\\".to_string(),
                total_bytes: 100,
                available_bytes: 50,
            },
            Volume {
                name: "Nested".to_string(),
                mount_point: "C:\\AI".to_string(),
                total_bytes: 80,
                available_bytes: 40,
            },
        ];

        assert_eq!(
            matching_volume("C:\\AI\\models", &volumes).unwrap().name,
            "Nested"
        );
        assert_eq!(
            matching_volume("c:/other", &volumes).unwrap().name,
            "System"
        );
        assert!(matching_volume("D:\\AI", &volumes).is_none());
        assert!(!path_matches_mount("C:\\AI2", "C:\\AI"));
        assert!(!path_matches_mount("C:\\AI", ""));
    }
}
