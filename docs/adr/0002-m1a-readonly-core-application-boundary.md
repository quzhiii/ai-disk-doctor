# ADR 0002: M1A Read-Only Core Application Boundary

Status: Proposed
Date: 2026-08-12
Scope: M1A implementation note. No Desktop runtime, Tauri dependency, mutation refactor, license change, or version bump is added.

## Context

M0 established that future Desktop work must reuse the existing CLI/Core execution truth instead of creating another scanner, model inventory engine, history reader, cleaner, restore engine, policy gate, or risk model.

Reality audit for M1A found:

- `aidisk src/main.rs` was binary-oriented and directly orchestrated command parsing, scan, policy/rules loading, snapshot persistence, model inventory, history latest-pair discovery, rendering, and mutation commands.
- General scan depends on rules, optional rules repo resolution, policy loading, scanner execution, report policy snapshots, optional progress, and existing `.aidisk/reports` snapshot persistence.
- Model inventory already has a read-only `build_inventory` domain function and specialized adapters for Ollama, Hugging Face, LM Studio, and generic model files.
- History had snapshot save and latest-pair discovery, but no reusable list/latest metadata shape for UI consumers.
- Clean, quarantine, restore, and planner mutation behavior remain separate and safety-critical.

## Decision

M1A adds a library crate boundary and a focused `application` module for read-only consumers:

```text
UI / CLI / future Desktop
        |
        v
aidisk::application
        |
        +-- run_scan / run_scan_with_progress
        +-- inventory_assets
        +-- read_history
        |
        v
existing rules / policy / scanner / model_inventory / history modules
```

The binary entrypoint is now intentionally thin: `src/main.rs` delegates to `aidisk::run_from_env()`. Existing CLI orchestration moved into `src/cli.rs`, which lets the CLI and the new application boundary share the same library module/type universe without making the CLI module part of the public typed application surface.

## Application Surface

### Scan

`application::ScanRequest` includes:

- rules directory or rules repo source
- optional category filter
- optional policy path
- caller-provided default rules and policy paths
- optional reports directory
- explicit `SnapshotPersistence::{Save, Skip}`

`application::run_scan` and `application::run_scan_with_progress` reuse existing rule loading, policy loading, scanner execution, and policy snapshot behavior. They return the existing compatible `ScanReport` plus optional snapshot path metadata.

Snapshot persistence is explicit at the application boundary. CLI `aidisk scan` continues to request `SnapshotPersistence::Save`, preserving current snapshot/history behavior. Future read-only UI consumers can request `SnapshotPersistence::Skip` when they need scan computation without writing an AI Disk Doctor-owned snapshot.

### AI Asset Inventory

`application::AssetInventoryRequest` includes root, tool, max depth, and stale cutoff. `application::inventory_assets` delegates to existing `model_inventory::build_inventory` and returns the existing `ModelInventoryReport` shape. It does not add providers or reimplement Ollama, Hugging Face, LM Studio, or generic model detection.

The application-facing tool enum is `ApplicationInventoryTool`, so external consumers do not need to depend directly on internal model inventory enum paths.

### History

`application::HistoryRequest` accepts an optional reports directory. `application::read_history` returns:

- explicit reports directory
- sorted scan snapshot metadata
- latest snapshot metadata
- latest pair metadata when at least two snapshots exist

This is intentionally smaller than a full Activity Timeline. It is enough for a future Desktop to list scan snapshots and locate latest before/after pairs without duplicating history discovery.

## Visibility

`aidisk::application` is the public read-only application surface. Internal modules such as `scanner`, `rules`, `policy`, `rules_repo`, `history`, and `model_inventory` remain crate-private in `lib.rs` except for types re-exported through `application` as needed.

Mutation modules remain internal and are not exposed through the new application boundary:

- cleaner / quarantine / restore
- planner mutation policy gates
- real filesystem mutation execution

## Compatibility

M1A is intended to preserve existing CLI behavior:

- `aidisk scan` still saves a snapshot by default.
- `aidisk scan --json` keeps the existing JSON report shape.
- `aidisk models inventory` and `aidisk models inventory --json` keep existing output behavior.
- `aidisk diff --latest`, `anomaly --latest`, and `doctor --latest` continue to use scan snapshot history.
- No clean, restore, planner, reporter schema, safety, or exit-contract change is intended.

## Safety

The application boundary is read-only with respect to user/workspace content. The only write option in this boundary is explicit AI Disk Doctor-owned snapshot persistence under the reports directory, preserving current scan history behavior while allowing future consumers to skip it.

M1A does not add:

- Desktop UI
- Tauri or JavaScript/frontend dependencies
- cleanup, quarantine, or restore application services
- new provider/session/content parsing
- Recovery Intelligence
- billing, auth, telemetry, SaaS, cloud sync, license keys, or license transition

## Deferred

- M1B Desktop Alpha shell and UI.
- Tauri dependency decision and packaging.
- Mutation application boundary for clean/quarantine/restore, likely around M2.
- Recovery Intelligence and Agent recovery asset lifecycle parsing.
- Commercial Desktop repository and licensing implementation decisions.
