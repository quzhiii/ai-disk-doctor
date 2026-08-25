# ADR 0004: Safe Action Architecture

- Status: Proposed design gate; M2 implementation has not started
- Date: 2026-08-24
- Milestone: M2 Safe Action Design

## Context

M1 established read-only explainability and a stable Agent diagnostic bridge. The product now needs a design for safe action without becoming an autonomous deletion engine. Current Core already contains internal planner, cleaner, quarantine, journal, index, and restore behavior, but its public application boundary remains read-only and Desktop mutation services do not yet exist.

The safety model requires a human-controlled, recoverable sequence:

```text
evidence -> recommendation -> preview -> human approval -> scoped execution -> journal -> verification -> restore
```

## Decision

This ADR proposes the following architecture for owner approval:

1. Core remains the sole owner of finding identity, recommendations, policy, risk, handling, recovery evidence, action plans, preflight, quarantine, verification, journaling, and restore.
2. Desktop remains a human-first adapter that renders evidence and previews, collects explicit human approval, displays progress/results, and starts a scoped Core operation. Desktop must not implement a second planner, risk model, cleaner, path policy, or restore engine.
3. Future MCP/Agent interfaces may read evidence and request proposals/previews but must not approve or directly execute mutation.
4. M2 should initially expose quarantine-first actions only. Permanent deletion and autonomous cleanup are excluded.
5. Action contracts must be additive and separate from `explainability-v1`. Candidate contracts are `ActionProposal`, `ActionPreview`/`ActionPlan`, `ApprovedAction`, `QuarantinePlan`, and `RestoreRecord`.
6. Every approved action is bound to a preview, item fingerprints, rule/policy/evidence fingerprints, destination root, and a human approval event. Core reruns mutation preflight immediately before mutation.
7. Quarantine uses containment checks, rename-first where safe, copy/verify/remove fallback when approved, append journal stages, per-entry outcomes, and conflict-safe restore. No whole-operation atomicity is promised across volumes.
8. Unknown, partial, active, sensitive, credential-adjacent, or stale evidence fails closed.

## Proposed Flow

```text
Core scan/explainability
        |
        v
Core ActionProposal
        |
        v
Core ActionPreview / bounded plan
        |
        +--> Desktop finding detail and preview
        |          |
        |          v
        |     Human approval
        |          |
        +----------+
                   v
          Core preflight + scoped execution
                   |
                   v
          journal + verification report
                   |
                   v
          Desktop result / Recovery Center
                   |
                   v
             explicit restore
```

## Contract Direction

The action contract family must preserve:

- source finding and evidence fingerprints
- rule and policy provenance
- risk and handling as separate fields
- reclaim basis and confidence
- recovery evidence and non-guarantees
- exact bounded entries and destination identity
- omitted/unknown accounting
- preflight requirements and blockers
- journal/index references
- verification and restore outcomes

`explainability-v1` remains the evidence contract. It must not be redesigned into an action schema.

## Quarantine Guarantees

The architecture promises only guarantees it can verify per entry:

- destination stays inside the approved quarantine root
- source is not already within that root
- destination is not nested inside source
- source is preserved if copy or verification fails
- destination conflicts do not overwrite existing paths
- source removal occurs only after successful copy verification
- journal stages explain planned, copied, verified, quarantined, failed, and restore states
- restore does not overwrite existing destinations by default

The architecture does not promise:

- whole-operation atomicity across filesystems
- application-level recovery after restore
- original bytes after a user edits quarantine content
- automatic resolution of disk-full or changed-path conflicts
- safe action for unknown or incomplete evidence

## Alternatives Rejected

### Desktop-owned cleanup engine

Rejected. It creates a second execution truth and risks divergence in policy, path containment, risk, and restore semantics.

### Agent-approved mutation

Rejected. Model output is not human consent, and Agent input must remain a narrow read/propose boundary.

### Direct permanent deletion as M2 MVP

Rejected. It removes the preferred rollback path and is inconsistent with quarantine-first product principles.

### Lower confidence thresholds to increase automation

Rejected. Unknown, partial, active, and recovery-adjacent evidence must fail closed rather than be converted into actionability.

### Paywall restore or safety recovery

Rejected as a default product direction. A user must not be trapped because recovery is reserved for a commercial tier.

## Consequences

Positive:

- One Core execution truth for CLI, Desktop, and future Agent integrations.
- Clear separation between evidence, recommendation, approval, execution, and recovery.
- Per-entry failures remain explainable and recoverable.
- M1 explainability remains compatible and reusable.

Costs and risks:

- A public mutation-side Core application boundary is required before Desktop implementation.
- Preview staleness and approval binding add contract and testing complexity.
- Cross-volume quarantine cannot claim transactionality.
- Directory integrity guarantees may require a declared tiered verification model.
- Recovery history and integrity metadata require durable local storage and migration policy.

## Implementation Preconditions

No implementation should start until Product approves:

- quarantine-only M2 scope
- human approval and Agent prohibition
- public Core mutation application boundary direction
- action contract versioning strategy
- destination-root policy
- integrity and disk-full guarantees
- conflict and edited-quarantine behavior
- free/Pro/Enterprise boundary hypotheses
- cross-platform action test matrix

## Status

This ADR is a design gate only. It changes no code, runtime behavior, CLI contract, Desktop behavior, licensing, or commercial entitlement. Product owner approval is required before M2 implementation begins.
