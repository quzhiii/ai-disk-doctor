use std::path::PathBuf;

use chrono::{DateTime, Local};
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::explainability::{
    ByteBasis, EvidenceStatus, HandlingMode, PathEvidence, RationaleEvidence,
    RecoverabilityEvidence, RiskCode, RuleProvenance, EXPLAINABILITY_CONTRACT,
    EXPLAINABILITY_SCHEMA_VERSION,
};

pub const ACTION_PROPOSAL_CONTRACT: &str = "action-proposal-v1";
pub const ACTION_PROPOSAL_SCHEMA_VERSION: u16 = 1;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ActionProposalSet {
    pub contract: String,
    pub schema_version: u16,
    pub source_scan: ScanReference,
    pub safety: ProposalSafety,
    pub scope: ProposalSetScope,
    pub summary: ProposalSummary,
    pub proposals: Vec<ActionProposal>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ScanReference {
    pub scan_time: String,
    pub snapshot_path: Option<PathBuf>,
    pub evidence_contract: String,
    pub evidence_schema_version: u16,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub struct ProposalSafety {
    pub read_only: bool,
    pub human_preview_required: bool,
    pub mutation_authorized: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Default)]
pub struct ProposalSetScope {
    pub bounded: bool,
    pub total_rules: usize,
    pub included_path_groups: usize,
    pub omitted_path_groups: usize,
    pub omitted_bytes: u64,
    pub partial_proposals: usize,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Default)]
pub struct ProposalSummary {
    pub total_proposals: usize,
    pub eligible: usize,
    pub review_required: usize,
    pub unknown: usize,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ActionProposal {
    pub proposal_id: String,
    pub finding_reference: FindingReference,
    pub proposal: ProposalDecision,
    pub evidence_refs: EvidenceReferences,
    pub impact: ImpactEstimate,
    pub risk: RiskEvidence,
    pub recoverability: RecoverabilityEvidence,
    pub scope: ProposalScope,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct FindingReference {
    pub category_id: String,
    pub rule_id: String,
    pub path_group_index: usize,
    pub path: PathEvidence,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ProposalDecision {
    pub action_type: ProposalActionType,
    pub handling_mode: HandlingMode,
    pub eligibility: ProposalEligibility,
    pub rationale: ProposalRationale,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ProposalActionType {
    QuarantineCandidate,
    OfficialManual,
    ReviewOnly,
    Unknown,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProposalEligibility {
    Eligible,
    ReviewRequired,
    Unknown,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ProposalRationale {
    pub code: ProposalReasonCode,
    pub blockers: Vec<ProposalBlocker>,
    pub rule_evidence: RationaleEvidence,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProposalReasonCode {
    CandidateForHumanPreview,
    OfficialManualReview,
    InformationalOnly,
    IncompleteEvidence,
    UnknownHandling,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub struct ProposalBlocker {
    pub code: ProposalBlockerCode,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProposalBlockerCode {
    PartialEvidence,
    IncompleteScan,
    RiskRequiresReview,
    SensitivityRequiresReview,
    RecoverabilityUnknown,
    RollbackNotDeclared,
    ProvenanceUnavailable,
    InformationalOnly,
    HandlingUnknown,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct EvidenceReferences {
    pub contract: String,
    pub schema_version: u16,
    pub category_id: String,
    pub rule_id: String,
    pub rule_provenance: RuleProvenance,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ImpactEstimate {
    pub logical_bytes_lower_bound: u64,
    pub estimated_reclaim_bytes: Option<u64>,
    pub byte_basis: ByteBasis,
    pub confidence: ImpactConfidence,
    pub non_guarantees: Vec<ImpactNonGuarantee>,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ImpactConfidence {
    LowerBound,
    Incomplete,
    Unavailable,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ImpactNonGuarantee {
    PhysicalBytes,
    CrossRuleDeduplication,
    ExecutionNotPerformed,
    PreviewRequired,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RiskEvidence {
    pub value: RiskCode,
    pub state: EvidenceState,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceState {
    Known,
    Incomplete,
    Unknown,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ProposalScope {
    pub bounded: bool,
    pub partial: bool,
    pub partial_reasons: Vec<String>,
    pub omitted_path_groups: usize,
    pub omitted_bytes: u64,
}

pub(crate) fn build(
    explainability: &crate::explainability::ExplainabilityReport,
    scan_time: &DateTime<Local>,
    snapshot_path: Option<PathBuf>,
) -> ActionProposalSet {
    let mut proposals = Vec::new();
    let mut scope = ProposalSetScope {
        bounded: true,
        total_rules: 0,
        ..ProposalSetScope::default()
    };

    for category in &explainability.categories {
        for rule in &category.rules {
            scope.total_rules += 1;
            scope.included_path_groups += rule.path_groups.len();
            scope.omitted_path_groups += rule.path_group_summary.omitted_path_groups;
            scope.omitted_bytes = scope
                .omitted_bytes
                .saturating_add(rule.path_group_summary.omitted_bytes);

            for (path_group_index, path_group) in rule.path_groups.iter().enumerate() {
                let incomplete_scan = explainability.evidence.status == EvidenceStatus::Partial;
                let decision = decision_for(
                    rule,
                    &path_group.path.sensitivity,
                    path_group.partial,
                    incomplete_scan,
                );
                let mut partial_reasons = path_group.partial_reasons.clone();
                if incomplete_scan
                    && !partial_reasons
                        .iter()
                        .any(|reason| reason == "incomplete scan evidence")
                {
                    partial_reasons.push("incomplete scan evidence".to_string());
                }
                let proposal_scope = ProposalScope {
                    bounded: true,
                    partial: path_group.partial || incomplete_scan,
                    partial_reasons,
                    omitted_path_groups: rule.path_group_summary.omitted_path_groups,
                    omitted_bytes: rule.path_group_summary.omitted_bytes,
                };
                if path_group.partial || incomplete_scan {
                    scope.partial_proposals += 1;
                }

                let eligible = decision.eligibility == ProposalEligibility::Eligible;
                let impact = ImpactEstimate {
                    logical_bytes_lower_bound: path_group.total_size_bytes,
                    estimated_reclaim_bytes: eligible.then_some(path_group.total_size_bytes),
                    byte_basis: explainability.accounting.byte_basis,
                    confidence: if path_group.partial || incomplete_scan {
                        ImpactConfidence::Incomplete
                    } else if eligible {
                        ImpactConfidence::LowerBound
                    } else {
                        ImpactConfidence::Unavailable
                    },
                    non_guarantees: vec![
                        ImpactNonGuarantee::PhysicalBytes,
                        ImpactNonGuarantee::CrossRuleDeduplication,
                        ImpactNonGuarantee::ExecutionNotPerformed,
                        ImpactNonGuarantee::PreviewRequired,
                    ],
                };

                proposals.push(ActionProposal {
                    proposal_id: proposal_id(
                        &category.category_id,
                        &rule.rule_id,
                        &path_group.path.raw_path,
                        &rule.provenance.source.digest,
                        path_group.total_size_bytes,
                        path_group.partial,
                    ),
                    finding_reference: FindingReference {
                        category_id: category.category_id.clone(),
                        rule_id: rule.rule_id.clone(),
                        path_group_index,
                        path: path_group.path.clone(),
                    },
                    proposal: decision,
                    evidence_refs: EvidenceReferences {
                        contract: EXPLAINABILITY_CONTRACT.to_string(),
                        schema_version: EXPLAINABILITY_SCHEMA_VERSION,
                        category_id: category.category_id.clone(),
                        rule_id: rule.rule_id.clone(),
                        rule_provenance: rule.provenance.clone(),
                    },
                    impact,
                    risk: RiskEvidence {
                        value: rule.risk,
                        state: if path_group.partial || incomplete_scan {
                            EvidenceState::Incomplete
                        } else {
                            EvidenceState::Known
                        },
                    },
                    recoverability: rule.recoverability.clone(),
                    scope: proposal_scope,
                });
            }
        }
    }

    let summary = ProposalSummary {
        total_proposals: proposals.len(),
        eligible: proposals
            .iter()
            .filter(|proposal| proposal.proposal.eligibility == ProposalEligibility::Eligible)
            .count(),
        review_required: proposals
            .iter()
            .filter(|proposal| proposal.proposal.eligibility == ProposalEligibility::ReviewRequired)
            .count(),
        unknown: proposals
            .iter()
            .filter(|proposal| proposal.proposal.eligibility == ProposalEligibility::Unknown)
            .count(),
    };

    ActionProposalSet {
        contract: ACTION_PROPOSAL_CONTRACT.to_string(),
        schema_version: ACTION_PROPOSAL_SCHEMA_VERSION,
        source_scan: ScanReference {
            scan_time: scan_time.to_rfc3339(),
            snapshot_path,
            evidence_contract: EXPLAINABILITY_CONTRACT.to_string(),
            evidence_schema_version: EXPLAINABILITY_SCHEMA_VERSION,
        },
        safety: ProposalSafety {
            read_only: true,
            human_preview_required: true,
            mutation_authorized: false,
        },
        scope,
        summary,
        proposals,
    }
}

fn decision_for(
    rule: &crate::explainability::RuleExplanation,
    sensitivity: &str,
    path_partial: bool,
    incomplete_scan: bool,
) -> ProposalDecision {
    let action_type = match rule.handling_mode {
        HandlingMode::Quarantine => ProposalActionType::QuarantineCandidate,
        HandlingMode::OfficialManual => ProposalActionType::OfficialManual,
        HandlingMode::ReportOnly => ProposalActionType::ReviewOnly,
        HandlingMode::Unknown => ProposalActionType::Unknown,
    };
    let mut blockers = Vec::new();

    if path_partial {
        blockers.push(ProposalBlocker {
            code: ProposalBlockerCode::PartialEvidence,
        });
    }
    if incomplete_scan {
        blockers.push(ProposalBlocker {
            code: ProposalBlockerCode::IncompleteScan,
        });
    }

    let provenance_available = rule.provenance.source.path != "unknown"
        && rule.provenance.source.digest != "unknown"
        && !rule.provenance.source.digest.is_empty();
    if !provenance_available {
        blockers.push(ProposalBlocker {
            code: ProposalBlockerCode::ProvenanceUnavailable,
        });
    }

    let recoverability_supported =
        rule.recoverability.reversibility == crate::explainability::Reversibility::Supported;
    if !recoverability_supported {
        blockers.push(ProposalBlocker {
            code: if rule.recoverability.reversibility
                == crate::explainability::Reversibility::Unknown
            {
                ProposalBlockerCode::RecoverabilityUnknown
            } else {
                ProposalBlockerCode::RollbackNotDeclared
            },
        });
    }

    let sensitivity_requires_review = sensitivity != "none";
    if sensitivity_requires_review {
        blockers.push(ProposalBlocker {
            code: ProposalBlockerCode::SensitivityRequiresReview,
        });
    }
    if rule.risk != RiskCode::Safe {
        blockers.push(ProposalBlocker {
            code: ProposalBlockerCode::RiskRequiresReview,
        });
    }

    let (eligibility, code) = match action_type {
        ProposalActionType::Unknown => {
            blockers.push(ProposalBlocker {
                code: ProposalBlockerCode::HandlingUnknown,
            });
            (
                ProposalEligibility::Unknown,
                ProposalReasonCode::UnknownHandling,
            )
        }
        ProposalActionType::QuarantineCandidate if path_partial || incomplete_scan => (
            ProposalEligibility::Unknown,
            ProposalReasonCode::IncompleteEvidence,
        ),
        ProposalActionType::QuarantineCandidate
            if rule.risk == RiskCode::Safe
                && rule.recoverability.reversibility
                    == crate::explainability::Reversibility::Supported
                && provenance_available
                && !sensitivity_requires_review
                && blockers.is_empty() =>
        {
            (
                ProposalEligibility::Eligible,
                ProposalReasonCode::CandidateForHumanPreview,
            )
        }
        ProposalActionType::OfficialManual => (
            ProposalEligibility::ReviewRequired,
            ProposalReasonCode::OfficialManualReview,
        ),
        ProposalActionType::ReviewOnly => {
            blockers.push(ProposalBlocker {
                code: ProposalBlockerCode::InformationalOnly,
            });
            (
                ProposalEligibility::ReviewRequired,
                ProposalReasonCode::InformationalOnly,
            )
        }
        ProposalActionType::QuarantineCandidate => (
            ProposalEligibility::ReviewRequired,
            ProposalReasonCode::CandidateForHumanPreview,
        ),
    };

    ProposalDecision {
        action_type,
        handling_mode: rule.handling_mode,
        eligibility,
        rationale: ProposalRationale {
            code,
            blockers,
            rule_evidence: rule.rationale.clone(),
        },
    }
}

fn proposal_id(
    category_id: &str,
    rule_id: &str,
    path: &str,
    rule_digest: &str,
    bytes: u64,
    partial: bool,
) -> String {
    let mut digest = Sha256::new();
    for value in [
        ACTION_PROPOSAL_CONTRACT,
        &ACTION_PROPOSAL_SCHEMA_VERSION.to_string(),
        category_id,
        rule_id,
        path,
        rule_digest,
        &bytes.to_string(),
        &partial.to_string(),
    ] {
        digest.update(value.as_bytes());
        digest.update([0]);
    }
    format!("proposal:sha256:{:x}", digest.finalize())
}

#[cfg(test)]
mod tests {
    use chrono::Local;
    use serde_json::Value;

    use super::*;
    use crate::explainability::{
        AccountingSemantics, CategoryExplanation, DeduplicationSemantics, EvidenceSummary,
        HandlingTotals, PathDisclosure, PathGroup, PathGroupSummary, RiskTotals, RuleExplanation,
        RuleSourceEvidence, StorageExplanation, VolumeExplanation,
    };

    fn explainability(
        partial: bool,
        sensitivity: &str,
    ) -> crate::explainability::ExplainabilityReport {
        crate::explainability::ExplainabilityReport {
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
                observed_bytes: 42,
                total_size_bytes: 42,
                potential_bytes: 42,
                actionable_bytes: 42,
                quarantine_bytes: 42,
                official_cleanup_bytes: 0,
                report_only_bytes: 0,
                partial_bytes: 0,
                reclaimable_safe_bytes: 42,
                safe_bytes: 42,
                review_bytes: 0,
                dangerous_bytes: 0,
                system_bytes: 0,
            },
            evidence: EvidenceSummary {
                status: if partial {
                    EvidenceStatus::Partial
                } else {
                    EvidenceStatus::Complete
                },
                partial_findings: usize::from(partial),
                warnings: Vec::new(),
            },
            volumes: vec![VolumeExplanation {
                name: "System".to_string(),
                mount_point: "C:\\".to_string(),
                total_bytes: 100,
                available_bytes: 58,
            }],
            categories: vec![CategoryExplanation {
                category_id: "models".to_string(),
                category_name: "models".to_string(),
                observed_bytes: 42,
                partial_bytes: 0,
                total_size_bytes: 42,
                handling: HandlingTotals {
                    quarantine_bytes: 42,
                    ..HandlingTotals::default()
                },
                risk: RiskTotals {
                    safe_bytes: 42,
                    ..RiskTotals::default()
                },
                rules: vec![RuleExplanation {
                    rule_id: "model-cache".to_string(),
                    rule_name: "Model cache".to_string(),
                    observed_bytes: 42,
                    partial_bytes: 0,
                    total_size_bytes: 42,
                    handling_mode: HandlingMode::Quarantine,
                    risk: RiskCode::Safe,
                    path_group_summary: PathGroupSummary {
                        total_path_groups: 1,
                        included_path_groups: 1,
                        omitted_path_groups: 0,
                        omitted_bytes: 0,
                        limit: 50,
                    },
                    path_groups: vec![PathGroup {
                        path: PathEvidence {
                            raw_path: "C:\\AI\\models".to_string(),
                            display_path: "C:\\AI\\models".to_string(),
                            disclosure: PathDisclosure::RawLocalPath,
                            sensitivity: sensitivity.to_string(),
                        },
                        observed_bytes: 42,
                        partial_bytes: 0,
                        total_size_bytes: 42,
                        finding_count: 1,
                        partial,
                        partial_reasons: if partial {
                            vec!["depth limit".to_string()]
                        } else {
                            Vec::new()
                        },
                        volume: None,
                    }],
                    rationale: RationaleEvidence {
                        reason: "review model cache".to_string(),
                        warnings: vec!["human review".to_string()],
                        partial_reasons: Vec::new(),
                    },
                    provenance: RuleProvenance {
                        source: RuleSourceEvidence {
                            path: "rules/model-cache.yaml".to_string(),
                            schema_version: 2,
                            digest: "sha256:model-cache".to_string(),
                        },
                        detector_evidence: vec!["path-pattern".to_string()],
                        content_access: "metadata-only".to_string(),
                        confidence: "high".to_string(),
                        action_type: "quarantine".to_string(),
                        action_adapter: "none".to_string(),
                        supports_dry_run: true,
                        supports_rollback: true,
                    },
                    recoverability: RecoverabilityEvidence {
                        declared: "rebuildable".to_string(),
                        kind: crate::explainability::RecoverabilityKind::Rebuildable,
                        reversibility: crate::explainability::Reversibility::Supported,
                        evidence: vec!["supports_rollback=true".to_string()],
                    },
                }],
            }],
        }
    }

    #[test]
    fn serializes_stable_read_only_contract_shape() {
        let set = build(&explainability(false, "none"), &Local::now(), None);
        let value: Value = serde_json::to_value(set).expect("proposal set should serialize");
        assert_eq!(value["contract"], ACTION_PROPOSAL_CONTRACT);
        assert_eq!(value["schema_version"], ACTION_PROPOSAL_SCHEMA_VERSION);
        assert_eq!(value["safety"]["read_only"], true);
        assert_eq!(value["safety"]["mutation_authorized"], false);
        assert!(value.get("command").is_none());
        assert!(value.to_string().contains("quarantine-candidate"));
        assert_no_execution_fields(&value);
    }

    fn assert_no_execution_fields(value: &Value) {
        match value {
            Value::Object(object) => {
                for key in object.keys() {
                    assert!(
                        !matches!(
                            key.as_str(),
                            "approval"
                                | "approved_action"
                                | "command"
                                | "delete"
                                | "destination"
                                | "execute"
                                | "execution_command"
                                | "quarantine_root"
                                | "restore"
                        ),
                        "proposal contains execution field {key}"
                    );
                }
                for child in object.values() {
                    assert_no_execution_fields(child);
                }
            }
            Value::Array(values) => {
                for child in values {
                    assert_no_execution_fields(child);
                }
            }
            _ => {}
        }
    }

    #[test]
    fn references_existing_evidence_and_preserves_lower_bound_semantics() {
        let set = build(&explainability(false, "none"), &Local::now(), None);
        let proposal = &set.proposals[0];
        assert_eq!(proposal.evidence_refs.contract, EXPLAINABILITY_CONTRACT);
        assert_eq!(proposal.finding_reference.rule_id, "model-cache");
        assert_eq!(proposal.impact.logical_bytes_lower_bound, 42);
        assert_eq!(proposal.impact.estimated_reclaim_bytes, Some(42));
        assert_eq!(proposal.proposal.eligibility, ProposalEligibility::Eligible);
        assert!(proposal
            .impact
            .non_guarantees
            .contains(&ImpactNonGuarantee::ExecutionNotPerformed));
    }

    #[test]
    fn partial_evidence_is_unknown_and_not_an_estimate() {
        let set = build(&explainability(true, "none"), &Local::now(), None);
        let proposal = &set.proposals[0];
        assert_eq!(proposal.proposal.eligibility, ProposalEligibility::Unknown);
        assert_eq!(proposal.impact.estimated_reclaim_bytes, None);
        assert_eq!(proposal.risk.state, EvidenceState::Incomplete);
        assert!(proposal
            .proposal
            .rationale
            .blockers
            .iter()
            .any(|blocker| blocker.code == ProposalBlockerCode::IncompleteScan));
    }

    #[test]
    fn non_none_sensitivity_requires_review() {
        let set = build(
            &explainability(false, "potentially-private"),
            &Local::now(),
            None,
        );
        let proposal = &set.proposals[0];
        assert_eq!(
            proposal.proposal.eligibility,
            ProposalEligibility::ReviewRequired
        );
        assert!(proposal
            .proposal
            .rationale
            .blockers
            .iter()
            .any(|blocker| blocker.code == ProposalBlockerCode::SensitivityRequiresReview));
    }
}
