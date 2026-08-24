# ADR 0005: Action Proposal Contract

- Status: Accepted for M2A.1 additive read-only contract; M2 mutation implementation has not started
- Date: 2026-08-24
- Baseline: `dccba4f4c503eacb00a46208c66ea9b172f19516`

## Context

M1 explainability provides evidence but intentionally does not answer what a person may consider next. Desktop needs a Core-owned decision-evidence boundary without deriving action semantics from UI labels or implementing a second cleanup engine.

## Decision

Add the read-only `action-proposal-v1` contract and expose it through `aidisk::application::run_action_proposals*`.

The API composes the existing scanner and `explainability-v1` output. It emits one typed proposal per bounded path group and preserves evidence references, partial/unknown state, omitted scope, risk, handling, rationale, provenance, and recoverability evidence.

Eligibility is deliberately narrow. A quarantine candidate is `eligible` only when evidence is complete, risk is `safe`, provenance is available, sensitivity is `none`, and rollback support is declared. Other states are `review_required` or `unknown`. This is decision evidence for a future human preview, not an executable candidate or safety authorization.

## Safety Boundary

The new API:

- does not call planner execution, cleaner, quarantine, restore, shell, network, telemetry, or cloud;
- does not add action approval, destination selection, execution commands, or permissions;
- does not modify `ScanResult` or overload `explainability-v1`;
- does not calculate a guaranteed reclaim value or claim risk-free cleanup;
- does not read file contents beyond the existing metadata-only scan/rule boundary.

The result declares `read_only: true`, `human_preview_required: true`, and `mutation_authorized: false`.

## Consequences

Consumers can render Core-owned action decision evidence without inventing eligibility or risk semantics. Partial and unknown evidence fails closed for future preview consideration. A later Action Preview contract may bind to proposal identity, but no such preview or execution is included in this ADR.

Desktop M2A Preview may begin only as a presentation-only adapter after this contract is reviewed and the Desktop pins the released Core commit. Quarantine, restore, approval, and mutation remain separate future milestones.
