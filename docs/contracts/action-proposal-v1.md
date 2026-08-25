# Action Proposal Contract v1

Status: additive Core contract for M2A.1. Proposal generation is read-only; M2 implementation has not started.

## Purpose

`action-proposal-v1` expresses what action could be considered next for bounded Core evidence. It stops before human preview, approval, quarantine, deletion, restore, or any other filesystem mutation. The contract is decision evidence only; it does not create an executable action candidate.

The lifecycle is:

```text
Finding -> Explainability -> Action Proposal -> Human Preview -> Future Approved Action -> Future Execution
```

An Action Proposal is not an instruction, approval, execution plan, reclaim guarantee, or risk-free decision.

## Application API

The public read-only Core application boundary exposes:

```rust
aidisk::application::run_action_proposals(request)
aidisk::application::run_action_proposals_with_progress(request, on_progress)
```

Both functions reuse the existing scan and explainability boundary. They do not call planner execution, cleaner, quarantine, restore, shell, network, telemetry, or cloud code. Existing `ScanResult`, `ExplainableScanResult`, `scan`, `explainability-v1`, and history APIs are unchanged.

The result is an additive `ActionProposalResult` containing the existing scan and explainability results plus an `ActionProposalSet`.

## Top-Level Shape

```text
ActionProposalSet {
  contract: "action-proposal-v1"
  schema_version: 1
  source_scan
  safety
  scope
  summary
  proposals[]
}
```

`safety` always declares:

```text
read_only: true
human_preview_required: true
mutation_authorized: false
```

## Proposal Shape

Each bounded path group produces one proposal:

```text
ActionProposal {
  proposal_id
  finding_reference { category_id, rule_id, path_group_index, path }
  proposal {
    action_type
    eligibility
    rationale { code, blockers[], rule_evidence }
  }
  evidence_refs { contract, schema_version, category_id, rule_id, rule_provenance }
  impact {
    logical_bytes_lower_bound
    estimated_reclaim_bytes
    byte_basis
    confidence
    non_guarantees[]
  }
  risk { value, state }
  recoverability
  scope { bounded, partial, partial_reasons, omitted_path_groups, omitted_bytes }
}
```

### Action Type

| Value | Meaning |
|---|---|
| `quarantine-candidate` | Existing Core rule handling indicates quarantine could be considered after a future human preview and policy preflight. It does not authorize quarantine. |
| `official-manual` | The rule points to an official/manual review path. Core does not invoke it. |
| `review-only` | Evidence is informational and not an executable action proposal. |
| `unknown` | Core cannot safely map the handling mode. |

### Eligibility

| Value | Meaning |
|---|---|
| `eligible` | The bounded evidence satisfies the current proposal gate for a future human preview. It is not approval, an executable candidate, or execution authorization. |
| `review_required` | A person must review the evidence or a policy-sensitive boundary before any future preview. |
| `unknown` | Incomplete or unavailable evidence prevents a safe eligibility decision. |

The contract never emits `safe_delete`, `risk_free`, or equivalent semantics.

### Current Eligibility Gate

An existing `quarantine` handling mode can be `eligible` only when all of these are true:

- risk is `safe`;
- path evidence is complete;
- scan evidence is complete;
- rule provenance is available;
- rule metadata declares rollback support;
- path sensitivity is `none`;
- the rule handling is known.

Any partial or incomplete scan evidence produces `unknown`, not an estimate. Review, official/manual, report-only, sensitive, unknown, or non-rollback evidence produces `review_required` or `unknown` as appropriate.

## Evidence And Scope

`evidence_refs` points back to `explainability-v1`, which remains the evidence source of truth. The proposal does not duplicate or reinterpret scanner logic, rule evaluation, risk calculation, provenance, or recoverability metadata.

The proposal preserves:

- logical-rule-match-lower-bound byte basis;
- path-group boundedness;
- omitted path-group count and omitted bytes;
- partial path reasons;
- independent risk and handling dimensions;
- rule rationale and provenance;
- declared recoverability and reversibility evidence.

`estimated_reclaim_bytes` is present only for an eligible bounded lower-bound candidate. It is not guaranteed reclaim, physical disk recovery, cross-rule-deduplicated space, or an execution result. All proposals state that execution has not been performed and a future human preview is required.

## Ownership And Safety

Core owns proposal identity, eligibility semantics, rationale, evidence references, risk boundary, and recoverability evidence.

Desktop owns future presentation and human confirmation UX. Desktop must render typed Core data and must not invent action decisions, risk, recoverability, reclaim values, destinations, or execution commands.

Agent/MCP may request or explain proposal information in a future adapter. It must not approve or execute mutation.

This contract does not add:

- delete, quarantine, or restore execution;
- action approval or bearer tokens;
- destination paths or arbitrary filesystem verbs;
- shell, elevated permissions, network, telemetry, cloud, auth, or billing;
- a second planner, cleaner, scanner, risk model, or recovery engine.

## Future Quarantine Relationship

If a future milestone adds a public Core Action Preview or quarantine boundary, it must reference this proposal by `proposal_id`, evidence fingerprint/provenance, policy state, and bounded scope. A future preview must revalidate freshness and policy before any future approved action. This contract itself cannot be executed and does not promise that quarantine or restore will be available.

## Compatibility

- `action-proposal-v1` is separate from and additive to `explainability-v1`.
- Existing public `ScanResult` compatibility is unchanged.
- Existing scan, explainability, CLI, history, cleaner, quarantine, restore, and rules behavior is unchanged.
- Consumers must detect the new API/capability explicitly and must not infer it from `handling_mode` or human help text.
- Later incompatible changes require a new contract version.

## Non-Guarantees

This contract does not guarantee physical reclaim, exhaustive filesystem coverage, recoverability, preview freshness, approval, mutation, quarantine, restore, or application-level recovery.
