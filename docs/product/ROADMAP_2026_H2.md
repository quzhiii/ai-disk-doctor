# Roadmap 2026 H2

This roadmap becomes the authoritative milestone order only after the M0 PR is accepted and merged. Until then, it is the proposed product reset roadmap.

## M0 - Product Reset And Repo Foundation

Goal: create a reliable source of truth for product, architecture, safety, commercial boundaries, and Web ChatGPT <-> Local Agent collaboration.

Implementation: documentation and architecture only.

Gate: Draft PR reviewed by Web ChatGPT before merge.

Constraints:

- no runtime behavior change
- no Desktop dependency
- no version bump
- no license change
- no new detectors
- no billing, auth, SaaS, telemetry, or cloud sync

## M1A - Core Application Boundary: Read-Only Understand

Target release family: candidate for v1.8.0 or later after M0 acceptance.

Goal: define the minimum shared read-only application-service boundary needed for a Desktop or other UI to ask Core for scan, AI asset inventory, and history outputs without duplicating CLI orchestration logic.

Candidate scope:

- read-only scan application service
- read-only AI asset inventory service
- read-only history/snapshot access service
- stable request/response/error shape for these read-only flows
- CLI and future Desktop both call or preserve the same Core execution truth

Out of scope:

- cleanup, quarantine, restore, or policy-gate refactoring beyond what is needed to keep read-only flows separated
- Desktop shell implementation
- Tauri or GUI dependency additions

Validation question: can read-only UI consumers reuse Core outputs without creating a second scanner, asset inventory, or history engine?

## M1B - Desktop Alpha: Understand

Target release family: candidate for v1.8.0 or later after M0 acceptance.

Goal: a non-expert can install/open AI Disk Doctor, scan the machine, and understand AI/developer storage without destructive action.

Candidate scope:

- Desktop shell
- Overview
- scan invocation
- AI Assets
- basic model view
- plain-language explanations
- read-only Activity from existing snapshots/history where feasible

Safety: read-only. This milestone should use the M1A read-only boundary for scan, asset inventory, and history. It does not require cleanup, quarantine, or restore refactoring to be complete before a read-only Desktop Alpha.

Validation question: can a beginner understand what is taking space and why?

## M2 - Safe Clean + Restore UX

Goal: turn existing planner/quarantine/restore capabilities into a human-reviewable Desktop workflow.

Candidate scope:

- cleanup candidate review
- why/evidence/risk explanation
- reclaim estimates
- quarantine action
- history
- restore
- conflict UX

Safety gate: Desktop may not introduce a second mutation implementation.

Validation question: can users reclaim space confidently without fear of losing work?

## M3 - Recovery Intelligence + Agent Asset Lifecycle

Goal: understand supported Agent/project recovery assets and their storage cost.

Initial candidates:

- OpenCode
- Claude
- Gemini
- Git
- AI Disk Doctor quarantine

Scope: metadata-only first.

Candidate outputs:

- recovery coverage
- snapshot/checkpoint/session metadata inventory
- storage consumed by recovery assets
- project relation
- age/recency evidence
- retention recommendations

Validation question: do users care enough about recovery visibility to return to the product?

## M4 - Continuous AI Workspace Health / Pro Beta

Goal: validate recurring value.

Candidate scope:

- scheduled scans
- growth timeline
- anomaly explanations
- tray/menu presence
- notifications
- retention recommendations
- continuous recovery-coverage checks
- Pro packaging experiment

Validation question: would users pay annually for continuous AI-workspace health?

## Later - Recovery Vault / Team

Recovery Vault: only after evidence that native Git/Agent recovery is insufficient for important non-Git work.

Team: only after design partners demonstrate demand for policy, audit, device health, deployment, and fleet governance.

## Release Discipline

Every milestone follows:

```text
Brief -> Branch -> Draft PR -> Tests/Evidence -> Web Review -> PASS / CONDITIONAL PASS / BLOCK -> Merge -> Next Brief
```

Do not parallelize product milestones without explicit approval.
