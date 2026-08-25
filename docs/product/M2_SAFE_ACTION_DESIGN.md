# M2 Safe Action Design

Status: Design gate only. M2 implementation has not started, and no M2 runtime implementation is authorized by this document.

Baseline: Core `cac502f73c39f1b5de13bab3e4de86a5c29684fc` after M1D Agent Contract Bridge.

## 1. Design Goal

M1 explains why AI/developer storage exists. M2 should answer what a person can safely do about it without turning AI Disk Doctor into an autonomous deletion system.

The product invariant is:

```text
Evidence
    -> Recommendation
    -> Preview
    -> Human confirmation
    -> Scoped action
    -> Journal
    -> Verification
    -> Rollback / Restore
```

The model, Agent, or Desktop may help explain and propose. A human must approve mutation. Core remains the execution authority and the owner of all safety-critical semantics.

## 2. Current Reality Audit

Current Core facts:

- `aidisk::application` exposes read-only scan, explainability, inventory, and history boundaries.
- `ExplainableScanResult` and `explainability-v1` already provide evidence, accounting, risk, handling, provenance, recoverability evidence, partial/unknown state, bounded path groups, and omission accounting.
- Planner semantics already distinguish executable candidates from report-only, guide, partial, sensitive, active, and blocked items.
- The existing internal/CLI cleaner path supports dry-run planning, quarantine-root containment, rename-first execution, copy/verify/remove fallback, execution reports, journal stages, quarantine indexes, and conflict-safe restore; this is not a public M2 application boundary.
- The existing internal/CLI restore path refuses to overwrite an existing destination by default and can run as a dry-run; M2 has not added Desktop or shared mutation services.
- Mutation-side shared application services are not yet established. The public application boundary is currently read-only.
- Recovery Intelligence is not a general project/session backup engine. Quarantine and restore metadata are the current recovery mechanisms.

Current gaps relevant to M2:

- No stable public mutation-side application API for Desktop and CLI to share.
- Existing cleaner records operational outcomes, but future action UX needs a stronger proposal/approval identity and preflight snapshot model.
- Reclaim confidence and recovery value are not universal first-class fields across all findings.
- Existing path-derived quarantine destinations need an explicit collision, identity, and metadata design before becoming a polished user-facing Action Center.

## 3. Action Model

### 3.1 Finding

A finding is Core-owned evidence from scan and explainability. It is not an instruction to mutate.

Minimum conceptual identity:

```text
FindingRef
  scan_id or scan_time
  rule_id
  path identity / disclosed path
  finding fingerprint
  evidence status
```

The fingerprint must be derived from stable evidence and the rule source digest, not from mutable UI labels. A finding can become stale when the source path, metadata, rule digest, policy, or scan evidence changes.

### 3.2 Recommendation

A recommendation is a Core-owned interpretation of what action mechanisms are allowed for a finding under current policy.

It must include:

- recommendation ID and source finding fingerprint
- recommendation kind: quarantine, official/manual, report-only, or no-action
- risk and handling dimensions separately
- reclaim estimate and confidence
- recovery value or recovery evidence status
- reason and warnings
- policy snapshot and rule provenance
- explicit blockers
- expiry/staleness basis

Recommendation is not approval. It must never mean “safe to delete.” The preferred executable recommendation is “quarantine this bounded path under the Core quarantine policy,” subject to preflight and human confirmation.

### 3.3 Action Preview

An Action Preview is a frozen, human-readable and machine-readable plan generated from one or more recommendations.

It must show:

- exact source path or bounded path group
- expected destination/quarantine location
- estimated bytes and whether they are lower-bound/logical
- item count and directory/file scope
- risk, handling, recovery, and uncertainty labels
- rule/provenance/rationale evidence
- what will not happen
- policy and safety blockers
- preflight requirements
- rollback route
- preview ID, Core version, contract version, and creation time

The preview must be invalidated if the source evidence, policy, rule digest, destination, or target metadata changes materially. Preview creation itself is read-only.

### 3.4 User Approval

Approval is a Desktop or human CLI event, never an Agent authorization.

Approval must bind to:

- preview ID
- exact item set and destination root
- action mechanism
- policy/rule/evidence fingerprints
- explicit confirmation timestamp
- optional typed confirmation for high-risk or sensitive-adjacent operations

Approval must not be reusable for a different path, changed preview, different quarantine root, or later scan. There is no “approve all future findings” in M2.

### 3.5 Execution

Execution is Core-owned and scoped to the approved preview. Core must re-run mutation preflight immediately before touching the filesystem.

Execution must:

- enforce quarantine-root containment
- reject source paths already inside the quarantine root
- reject destinations nested inside sources
- reject missing, changed, recently active, sensitive, partial, or stale items according to policy
- use the approved destination and no arbitrary user-supplied operation
- create journal metadata before mutation
- prefer atomic rename when safe
- use copy/verify/remove only when rename is unavailable and policy allows it
- report each item independently

No shell, arbitrary filesystem verb, direct permanent deletion, or Agent-triggered mutation is part of M2.

### 3.6 Verification

Verification is Core-owned. It must prove the expected postcondition for each item, not merely report that an OS call returned success.

For rename:

- source no longer exists
- destination exists within the approved root
- destination identity/type is consistent with the source preflight

For copy fallback:

- source and destination file/directory counts match
- byte counts match for the declared logical basis
- destination is present and source removal succeeds only after verification
- failed verification removes the incomplete destination where safe

The result must distinguish planned, quarantined, verified, restore-planned, restored, skipped, and failed stages.

### 3.7 Restore

Restore is Core-owned, explicit, conflict-safe, and never overwrites an existing destination by default.

Restore must be available from the journal/index even if the original path no longer exists. It must refuse or require a new explicit human decision for:

- existing destination
- changed destination parent
- invalid or tampered index
- missing quarantine copy
- incomplete or unverified quarantine result

Restore is not a guarantee that applications will accept the restored state. The UI must show that the content was restored to the original path, not that application-level recovery was proven.

## 4. Ownership Boundary

### Core owns

- Finding identity and evidence
- Rule and policy semantics
- Risk, handling, reclaim estimate, and future confidence fields
- Recommendation eligibility and blockers
- Action proposal/plan schema
- Destination containment and path policy
- Mutation preflight
- Quarantine execution
- Copy/verify/remove or rename semantics
- Journaling, index, integrity metadata, and execution stages
- Restore and conflict behavior
- Structured errors and machine-readable outcomes
- One execution truth for CLI and Desktop

### Desktop owns

- Finding detail presentation
- Progressive explanation and comparison
- Action preview presentation
- Human confirmation interaction
- Progress rendering and retry guidance
- Recovery Center navigation and restore initiation UI
- Local-only activity presentation
- Entitlement presentation if later approved

Desktop must not compute risk, reinterpret recoverability, choose arbitrary destinations, create a second planner, execute filesystem operations, or bypass Core preflight.

### Agent/MCP owns in a future milestone

- Read capabilities
- Read explainability evidence
- Request a bounded proposal/preview
- Present rationale to a human

Agent/MCP must not approve or directly execute mutation. An Agent request must produce a proposal or preview that requires a human-owned approval path.

## 5. Candidate Core Contracts

These are design candidates only, not approved implementation schemas.

### 5.1 `ActionProposal`

Purpose: explain why a finding is or is not eligible for a specific action.

```text
ActionProposal {
  contract: "action-proposal-v1"
  schema_version: 1
  proposal_id
  finding_ref
  recommendation_kind
  eligibility: eligible | blocked | review-required | unknown
  blockers[]
  risk
  handling_mode
  reclaim { bytes, basis, confidence }
  recovery { value, evidence, non_guarantees }
  provenance
  policy_snapshot
  generated_at
}
```

Ownership: Core. Compatibility: additive and separately versioned from `explainability-v1`; it references evidence rather than copying or changing its semantics.

Safety guarantee: proposal generation is read-only and never authorizes mutation.

### 5.2 `ActionPlan` / `ActionPreview`

Purpose: freeze a bounded set of proposals into an exact plan for human review.

```text
ActionPreview {
  contract: "action-preview-v1"
  schema_version: 1
  preview_id
  source_scan_or_proposal_ids[]
  action: quarantine | official-manual | no-action
  quarantine_root_ref
  entries[]
  totals
  preflight_requirements[]
  blockers[]
  rollback { available, index_kind, restore_command }
  evidence_fingerprint
  policy_fingerprint
  expires_or_stale_when
  generated_at
}
```

Each entry needs source identity, destination identity, expected type, expected metadata/stat basis, partial/active status, and a stable item fingerprint. The preview must be bounded and must preserve omitted/unknown accounting.

Ownership: Core creates and validates; Desktop renders and requests approval.

Safety guarantee: preview is immutable evidence, not execution. A preview cannot be executed after preflight invalidation.

### 5.3 `ApprovalToken` or `ApprovedAction`

Purpose: bind explicit human consent to exactly one preview.

This should not be a bearer token that an Agent can obtain. The first implementation may keep approval as a local Core/CLI input record rather than a cryptographic token, provided it binds all preview fingerprints and is single-use.

```text
ApprovedAction {
  contract: "approved-action-v1"
  schema_version: 1
  preview_id
  item_fingerprints[]
  action
  quarantine_root
  approved_by: human
  approved_at
  approval_context
}
```

Open question: whether the approval record belongs in a local journal, is passed through a short-lived CLI invocation, or is represented by a signed Core-generated capability. Do not select bearer tokens or Agent-issued approval without a separate threat review.

### 5.4 `QuarantinePlan`

Current Core has an internal plan with source and destination paths. Future public shape should add:

- stable plan/entry IDs
- source identity and preflight metadata
- destination root identity
- collision policy
- logical byte basis
- expected file/directory counts
- integrity algorithm and digest availability
- action and recovery declarations
- policy/rule/evidence fingerprints

The plan must not permit arbitrary copy, move, delete, or shell verbs. It represents only the Core-approved quarantine mechanism.

### 5.5 `RestoreRecord`

Current execution and restore reports already provide a foundation. Future public shape should include:

- original and quarantine identities
- source/destination fingerprints at quarantine time
- integrity verification result
- journal/index references
- restore attempts and stages
- conflict outcome
- whether the quarantine copy remains available
- explicit non-guarantees

Restore records are evidence of a Core operation, not a guarantee of application-level recovery.

## 6. Quarantine Architecture

### 6.1 Storage layout

The quarantine root must be explicitly selected by Core policy or an approved human Desktop setting. It must not be an arbitrary path supplied by an Agent.

Conceptual layout:

```text
<quarantine-root>/
  .aidisk/
    quarantine-index-<operation>.json
    quarantine-journal-<operation>.log
    quarantine-log-<operation>.log
  <operation-or-entry-id>/
    <bounded-source-relative-identity>
```

The exact layout is an implementation decision after this gate. It must prevent source/destination nesting, collisions, and accidental re-ingestion as ordinary source evidence.

### 6.2 Journal requirements

The journal must be append-oriented and durable enough to explain crashes. It should record:

- operation and entry IDs
- source/destination paths and stable fingerprints
- rule/policy/evidence fingerprints
- preflight result
- planned, renaming/copying, copied, verified, source-removing, quarantined, failed stages
- timestamps
- error class and bounded message
- recovery state

Journal writes should be flushed before irreversible source removal. A journal write failure must fail closed before mutation whenever possible.

### 6.3 Metadata and integrity

Before mutation, Core should capture metadata sufficient for conflict and verification decisions without reading content merely for classification:

- path/type
- size and file/directory counts
- modification metadata where available
- filesystem/volume reference where known
- source and destination containment identities
- content digest only when required by the chosen verification guarantee and explicitly bounded

For directories, an implementation must define whether digesting all content is required, optional, or too expensive. M2 should not claim cryptographic integrity if it only compares counts and bytes.

### 6.4 Failure matrix

| Failure | Required behavior | Recovery state |
|---|---|---|
| Move/rename fails | Log the failure; use copy/verify/remove only if the approved plan allows fallback; otherwise leave source untouched | `rename-failed` or fallback stage |
| Disk becomes full during copy | Stop; remove incomplete destination when safe; never remove source before verification | `partial-copy` / source preserved |
| Verification fails | Remove incomplete destination when safe; keep source; journal mismatch | `verification-failed` / source preserved |
| Source removal fails after verified copy | Keep verified destination and source; do not pretend the operation is a clean move; surface duplicate-space state | `source-remove-failed` |
| Original path changes after preview | Invalidate preview or skip entry at preflight; require a new scan/preview | `stale-preflight` |
| Quarantined file is edited | Mark restore as restoring the edited quarantine copy, not the original bytes; retain before/after metadata if available | `quarantine-mutated` warning |
| Destination already exists | Do not overwrite; skip and require explicit conflict resolution | `conflict` |
| Quarantine root unavailable | Abort before mutation; no fallback to an unapproved location | `destination-unavailable` |
| Journal/index write fails | Fail closed before source removal; preserve source; report incomplete operation | `journal-failed` |
| Process crashes | Recover from journal on next startup; never infer success solely from a missing process | `unknown` until reconciled |

### 6.5 Disk-full and rollback boundary

Quarantine is not transactional across arbitrary filesystems. Atomic rename may be unavailable across volumes, and copy fallback can fail after partial progress. The architecture must therefore provide per-entry outcomes and restore capability rather than claiming whole-operation atomicity.

The UI must show:

- completed entries
- untouched entries
- partially copied entries
- duplicate-space or source-remove failures
- exact restore availability

## 7. Desktop UX

The Desktop is human-first and must make “why,” “what,” “where,” “risk,” “recovery,” and “what happens next” visible before approval.

### 7.1 Finding detail

Show:

- category, rule, path group, bytes, volume/unknown
- evidence completeness and partial reasons
- risk and handling as separate axes
- provenance, rationale, warnings
- recoverability evidence and non-guarantees
- current recommendation status

Actions are not shown as a single green “clean” decision. Use explicit labels such as `Quarantine candidate`, `Official/manual`, `Report only`, `Blocked`, and `Unknown`.

### 7.2 Action preview

Show a bounded list of exact entries:

- original path
- quarantine destination
- estimated logical bytes
- item type/count
- reason for inclusion
- blockers and preflight checks
- expected reclaim only after successful verification
- restore route

Do not hide omitted groups. Do not imply that the preview is already an action.

### 7.3 Confirmation

Confirmation must be a separate step with:

- explicit action verb: “Move to quarantine,” not “Clean”
- exact count and paths or path-group expansion
- destination root
- “nothing is permanently deleted” only when true for the selected action
- warning if source removal, duplicate space, or restore limitations exist
- typed confirmation or an additional acknowledgement for high-risk cases
- no Agent-generated approval

The confirm control must be disabled if preview staleness or preflight invalidation is detected.

### 7.4 Execution progress

Show per-entry stages, not only a spinner:

- preparing
- preflight passed/blocked
- moving or copying
- verifying
- quarantined
- skipped/failed

Allow the user to stop starting new entries at a safe boundary. Do not promise rollback of an in-flight filesystem primitive.

### 7.5 Success result

Separate:

- quarantined and verified
- skipped before mutation
- failed without source mutation
- source-remove failure with duplicate copies
- reclaim observed/expected
- restore available/unavailable

Provide journal/index location through a local affordance, not by uploading it. Explain that quarantine is reversible by Core restore but application-level recovery is not guaranteed.

### 7.6 Restore history

Recovery Center should show:

- operation time and source path
- quarantine location
- original evidence and rule
- integrity/verification state
- whether the quarantine copy was edited
- restore preview and destination conflict state
- restore result and journal stages

Restore must remain explicit, conflict-safe, and human initiated.

## 8. Agent / MCP Boundary

Future MCP may:

- query `capabilities`
- query `explain`
- request a bounded action proposal
- request an action preview
- summarize evidence and non-guarantees for a human

Future MCP must not:

- approve an action
- choose an arbitrary root or destination
- call clean, quarantine, delete, or restore execution directly
- pass shell commands or arbitrary filesystem operations
- turn a recommendation into an action without a human confirmation event

The safe protocol is:

```text
MCP read evidence
  -> MCP requests proposal/preview
  -> Desktop shows preview
  -> human approves in Desktop/approved Core path
  -> Core executes scoped action
  -> MCP may read the resulting journal/report
```

## 9. Core Contract Compatibility

M2 should not modify `explainability-v1` to add action semantics. Action contracts should be additive and reference finding/provenance fingerprints.

Compatibility requirements:

- Existing scan, plan, clean, quarantine, restore, history, and M1D CLI behavior remains backward-compatible.
- Existing `QuarantinePlan`, `ExecutionReport`, and `RestoreReport` should be evolved only through additive schema versions or new public application DTOs.
- A plan created under one Core/policy/rule revision must not silently execute under another.
- Unknown, partial, active, sensitive, credential-adjacent, or stale evidence fails closed.
- New contract consumers must be able to detect support through structured capabilities rather than help-text parsing.

## 10. Commercial Boundary

This section is a product hypothesis/design boundary, not an approved pricing decision.

### Free / Core-safe baseline

- scan
- explainability
- bounded recommendations and previews where the open Core supports them
- safe quarantine execution and restore should not be paywalled in a way that traps user data or removes recovery access

The safety capability required to recover from an action should remain available to the user who can perform that action. Exact packaging requires owner/legal review.

### Possible Pro value

- richer human Action Center UX
- historical quarantine/recovery views and search
- continuous local monitoring
- retention policy simulation and scheduled review
- broader Recovery Center visualization
- multi-workspace organization

Pro must not be the only place a user can restore their own quarantined data.

### Possible Enterprise value

- organization policy distribution
- approved rule/policy bundles
- governance reports and audit export
- administrative controls and local compliance workflows

Enterprise must not introduce cloud processing of user content by default, telemetry, or autonomous mutation. It remains a later validation target.

### Deferred

- billing, auth, entitlement, licensing keys, multi-device sync, and cloud services require explicit later milestones and legal/product approval.

## 11. Risks

| Risk | Why it matters | Design response |
|---|---|---|
| “Quarantine” becomes perceived deletion | Users may approve without understanding recovery limits | Use explicit move language, preview, journal, restore path, and non-guarantees |
| Stale preview mutates changed data | Scan evidence may no longer describe the path | Bind approval to fingerprints and rerun preflight |
| False-safe recommendation | AI/developer data may be active or recovery-critical | Separate risk/recovery, fail closed on unknown/partial/active evidence |
| Partial copy consumes space | Disk full or interruption can leave duplicate/incomplete data | Verify before source removal; per-entry result; cleanup incomplete destination safely |
| Quarantine is edited | Restore may not restore original bytes | Record mutation warning and distinguish content identity |
| Cross-volume move is not atomic | Rename may fall back to copy | Make atomicity a declared guarantee per entry, never a blanket claim |
| Desktop/Core divergence | Two action engines create inconsistent safety | Core-owned mutation application boundary and typed DTOs |
| Agent overreach | Model pressure can bypass consent | no Agent approval/execution capability |
| Commercial lock-in | Users may be trapped without restore | keep recovery safety capabilities accessible; no safety paywall without owner decision |

## 12. Open Questions For Owner Approval

1. Should the first M2 executable action expose only quarantine, excluding official/manual and any direct deletion path?
2. Should mutation-side application APIs be added to the public Core before Desktop implementation, or first stabilized as internal CLI/Core services with typed DTO tests?
3. What is the approved default quarantine root and how should cross-volume roots be selected?
4. Which integrity guarantee is required for directories: metadata/stat verification, per-file digest, or a declared tiered model?
5. Should approvals be short-lived records, single-use local files, or a Core-generated signed capability?
6. What user-visible behavior is required when source removal fails after successful verification?
7. What is the minimum recovery metadata required before a recommendation can be executable?
8. Which quarantine history and continuous monitoring capabilities, if any, are commercial Pro candidates?
9. Which governance features are sufficiently validated for a later Enterprise milestone?
10. What macOS, Windows, Linux, network-volume, and removable-volume matrix must block M2 release?

## 13. Gate Outcome

This document authorizes no implementation. It establishes the design direction and identifies contracts, safety guarantees, UX states, and unresolved decisions for Product owner approval.

M2 implementation must not begin until the owner accepts the action scope, quarantine guarantees, approval boundary, and commercial boundary questions above.
