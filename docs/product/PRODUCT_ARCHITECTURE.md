# Product Architecture

## Current Architecture

Current repo fact: AI Disk Doctor is currently implemented as a Rust CLI/Core crate in `aidisk/`.

```text
User / AI Agent
       |
       v
   aidisk CLI
       |
       +-- Internal CLI orchestration (`src/cli.rs`)
       +-- Read-only application boundary (`aidisk::application`)
       |       +-- Scan / scan progress
       |       +-- AI asset inventory
       |       +-- History snapshot metadata
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

Current repo fact: M1A adds a public read-only Core application boundary for scan, AI asset inventory, and history metadata. It does not implement Desktop, Tauri, Desktop packaging, or mutation-side application services.

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

Current repo fact: `aidisk/src/main.rs` is now a thin binary entrypoint, CLI orchestration lives in internal `aidisk/src/cli.rs`, and `aidisk::application` provides the M1A read-only shared boundary for scan, AI asset inventory, and history metadata.

Current remaining gap: mutation-side shared application services for clean/quarantine/restore are not established. Future milestones must still avoid Desktop-specific cleanup, restore, risk, or path-policy logic.

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
