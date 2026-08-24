# Documentation Index

This index is the supersession map for current, historical, implementation, and release documentation.

## Current Authoritative Docs

| Document | Purpose |
|---|---|
| `../AGENTS.md` | persistent coding-agent constitution |
| `product/CURRENT_PRODUCT_TRUTH.md` | current repo reality and capability matrix |
| `product/VISION.md` | product direction |
| `product/USER_PROBLEM_MODEL.md` | users, jobs, and problem model |
| `product/PRODUCT_ARCHITECTURE.md` | architecture target and one-execution-truth rule |
| `product/UX_DIRECTION.md` | future human UX direction |
| `product/SAFETY_AND_RECOVERY.md` | safety and recovery principles |
| `product/LICENSING_AND_COMMERCIAL_ARCHITECTURE.md` | licensing and commercial planning audit |
| `product/ROADMAP_2026_H2.md` | proposed current roadmap after M0 acceptance |
| `product/COLLABORATION_PROTOCOL.md` | Web ChatGPT <-> Local Agent handoff process |
| `product/M2_SAFE_ACTION_DESIGN.md` | M2 safe action architecture design gate; implementation not authorized |
| `adr/0001-desktop-architecture.md` | Desktop architecture ADR/spike |
| `adr/0002-m1a-readonly-core-application-boundary.md` | M1A read-only Core application boundary implementation note |
| `adr/0003-agent-diagnostic-cli-bridge.md` | M1D Agent diagnostic CLI bridge decision |
| `adr/0004-safe-action-architecture.md` | M2 safe action architecture design gate |
| `adr/0005-action-proposal-contract.md` | M2A.1 additive read-only Action Proposal contract |
| `contracts/explainability-v1.md` | stable read-only downstream explainability contract |
| `contracts/agent-diagnostic-cli-v1.md` | stable bounded Agent diagnostic and capability CLI contract |
| `contracts/action-proposal-v1.md` | stable read-only downstream Action Proposal contract |

## Implementation / Reference Docs That Remain Useful

| Document | Status |
|---|---|
| `architecture.md` | current implementation architecture reference; some old naming remains historical |
| `rules-spec.md` | rule format reference |
| `report-schema.md` | report schema reference |
| `risk-model.md` | historical/current risk model seed; superseded for product strategy by `product/SAFETY_AND_RECOVERY.md` |
| `governance-manual.md` | local governance user manual |
| `notifier-adapters.md` | notifier adapter reference |
| `trusted-distribution.md` | release artifact and distribution reference |
| `cross-platform-verification.md` | manual cross-platform verification checklist |
| `windows-ai-storage-map.md` | historical Windows path reference; not the current product roadmap |

## Historical / Superseded Strategy And Plan Docs

These files are preserved for traceability. Do not use them as the current roadmap unless a new brief explicitly says so.

| Document | Supersession status |
|---|---|
| `AI_DISK_DOCTOR_CORRECTED_ROADMAP_2026-07.md` | superseded by `product/ROADMAP_2026_H2.md` after M0 merge; retained as v1.7/v1.8 planning history |
| `ai-disk-doctor-strategy-and-roadmap.md` | superseded by current `docs/product/` docs after M0 merge; retained as strategy/market-analysis history |
| `execution-plan.md` | historical implementation log through v1.7.0; not the current product roadmap |
| `product-plan.md` | early product plan stub; superseded by `product/` docs |
| `../windows-ai-space-manager-project-plan.md` | original Windows-first project plan; historical only |
| `plans/*.md` | milestone-specific historical implementation plans |

## Release Notes

`release-notes/v*.md` are historical release records and should not be rewritten for strategy changes. Add new release notes only during explicit release-readiness milestones.

## M0 Rule

M0 is documentation and architecture only. It must not change runtime behavior, version, license files, Desktop dependencies, detectors, rules, billing, auth, telemetry, SaaS, or cleanup/restore semantics.
