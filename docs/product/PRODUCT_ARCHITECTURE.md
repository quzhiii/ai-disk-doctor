# Product Architecture

## Current Architecture

Current repo fact: AI Disk Doctor is currently implemented as a Rust CLI/Core crate in `aidisk/`.

```text
User / AI Agent
       |
       v
   aidisk CLI
       |
       +-- Policy and config loader
       +-- Rules engine
       +-- Scanner
       +-- Planner
       +-- Cleaner / quarantine / restore
       +-- Doctor
       +-- Model inventory / adapters
       +-- History / diff / anomaly
       +-- Reporter / visual dashboard
```

## Target Product Structure

Product decision:

```text
                    AI Disk Doctor
                          |
              Shared domain / contracts
                          |
              AI Disk Doctor Core
                    Rust / OSS
                          |
       +------------------+------------------+
       |                                     |
      CLI                            Desktop product
developer / Agent                     human-first UI
```

The Desktop must be an adapter over shared Core/domain services. It must not create its own scanner, planner, cleaner, restore engine, risk model, or path policy.

## Core Responsibilities

The open Core should own:

- rules and policy semantics
- scanning and findings
- model/AI asset inventory
- planning and risk classification
- mutation authorization
- quarantine and restore
- action journal/history
- diff and anomaly detection
- structured output
- safety invariants

## Desktop Responsibilities

Future Desktop may own:

- onboarding and navigation
- plain-language explanations
- storage map and review UX
- Activity and Recovery Center UX
- notifications/tray presence
- commercial entitlement UX
- Pro-only automation or continuous management experiences

Product decision: exact open/proprietary boundaries remain deferred, but the architecture must support a public Core plus a commercial Desktop.

## One Execution Truth

Bad target:

```text
CLI cleaner logic
Desktop cleaner logic
```

Preferred target:

```text
Shared application/domain service
        |              |
       CLI          Desktop adapter
```

Current gap: `aidisk/src/main.rs` wires command-specific orchestration directly. Future milestones should extract shared application/domain services before any Desktop starts invoking cleanup or restore flows. M0 does not perform that refactor.

## Recovery Intelligence Layer

Product decision: Recovery Intelligence starts as metadata and capability detection, not a backup engine.

It may eventually answer:

- Is Git present?
- Does a supported Agent expose snapshots, checkpoints, or sessions?
- How much space do recovery assets consume?
- How recent are recoverable states?
- Is recovery coverage strong, partial, weak, unknown, or absent?
- What would a cleanup remove from recovery capability?

Current gap: the repo has quarantine restore, scan history, diff/anomaly, model rollback metadata, and report-only model intelligence, but no general project-level recovery coverage model.

## Privacy Boundary

- Product decision: no upload of user paths, content, prompts, or source code by default.
- Product decision: metadata-only inspection for Agent/session/model assets unless a user explicitly opts in to deeper inspection in a future milestone.
- Current repo fact: M0 adds no telemetry, account, SaaS, or cloud sync behavior.
