# Phase 9 Local Scheduled Governance Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a local scheduled governance slice that detects disk growth anomalies from scan snapshots and prepares report payloads for future scheduler/notifier adapters.

**Architecture:** Keep anomaly detection in Rust so thresholds and JSON output stay testable and cross-platform. Keep scheduling and notification orchestration script-based for the first slice so Windows Task Scheduler can land quickly while leaving room for cron, launchd, systemd timers, and IM/webhook adapters later.

**Tech Stack:** Rust, Cargo, clap, serde JSON, Markdown output, PowerShell scripts.

---

### Task 1: Roadmap and plan artifacts

**Files:**
- Modify: `docs/execution-plan.md`
- Create: `docs/plans/2026-06-08-phase-9-local-scheduled-governance.md`

**Step 1: Update roadmap**

Add Phase 9 with:
- Local Scheduled Governance goal
- Rust anomaly core + script orchestration architecture
- absolute + relative growth threshold model
- scheduler adapter future: Task Scheduler, cron, launchd, systemd timer
- notifier adapter future: local file, webhook, WeChat/WeCom, Feishu, Slack, Telegram, Discord, email

**Step 2: Verify docs are present**

Run: `git diff -- docs/execution-plan.md docs/plans/2026-06-08-phase-9-local-scheduled-governance.md`

Expected: Phase 9 section and this implementation plan are visible.

### Task 2: Anomaly data model and threshold engine

**Files:**
- Create: `aidisk/src/anomaly.rs`
- Modify: `aidisk/src/main.rs`

**Step 1: Write failing unit tests**

Add tests for:
- growth must satisfy both `min_growth_bytes` and `min_growth_percent` when a prior size exists
- large newly appeared paths can alert on absolute threshold with no percent baseline
- shrinking or unchanged paths never alert

Run: `cargo test anomaly::tests`

Expected: FAIL because `anomaly` module does not exist yet.

**Step 2: Implement minimal anomaly model**

Create:
- `AnomalyThresholds`
- `AnomalyReport`
- `AnomalySummary`
- `GrowthAnomaly`
- `build_anomaly_report(diff_report, thresholds)`

Rules:
- use only positive `delta_bytes`
- if `before_bytes > 0`, require `delta_bytes >= min_growth_bytes` and growth percent >= `min_growth_percent`
- if `before_bytes == 0`, require `delta_bytes >= min_growth_bytes` and set `growth_percent = None`
- sort anomalies by descending `delta_bytes`

**Step 3: Verify unit tests**

Run: `cargo test anomaly::tests`

Expected: PASS.

### Task 3: CLI command and rendering

**Files:**
- Modify: `aidisk/src/main.rs`
- Modify: `aidisk/src/reporter.rs`
- Test: `aidisk/tests/anomaly_cli.rs`

**Step 1: Write failing CLI tests**

Add tests for:
- `aidisk anomaly --before before.json --after after.json --min-growth 1GB --min-growth-percent 30 --json`
- `aidisk anomaly --latest --reports-dir <DIR> --json`

Expected JSON:
- `summary.anomalies`
- `thresholds.min_growth_bytes`
- `anomalies[0].path`
- `anomalies[0].growth_percent`

Run: `cargo test --test anomaly_cli`
Expected: FAIL because command is missing.

**Step 2: Implement command**

Add `Command::Anomaly` with:
- `--format`, `--json`, `--markdown`
- `--latest`
- `--reports-dir`
- `--before`
- `--after`
- `--min-growth` using existing `parse_size_arg`, default `1GB`
- `--min-growth-percent`, default `30.0`

Behavior mirrors `diff` path selection.

**Step 3: Add renderers**

Add `reporter::render_anomaly`:
- JSON uses serde pretty
- Text lists summary and top anomalies
- Markdown renders an IM-friendly table

Run: `cargo test --test anomaly_cli`
Expected: PASS.

### Task 4: Governance script skeleton

**Files:**
- Create: `scripts/governance/run-governance.ps1`
- Modify: `aidisk/tests/release_artifacts.rs` or create an artifact test if appropriate

**Step 1: Write artifact test**

Assert script exists and contains:
- `cargo run -- scan --json`
- `cargo run -- anomaly --latest`
- no `clean --yes`
- no destructive commands

Run: `cargo test --test release_artifacts`
Expected: FAIL until script exists.

**Step 2: Add non-destructive script skeleton**

Script should:
- accept reports output dir
- run scan JSON
- run anomaly latest Markdown/JSON when at least two snapshots exist
- write artifacts locally
- leave notifier/webhook as a documented future adapter placeholder

Run: `cargo test --test release_artifacts`
Expected: PASS.

### Task 5: Documentation and verification

**Files:**
- Modify: `README.md`
- Modify: `README.zh-CN.md`
- Modify: `docs/architecture.md`

**Step 1: Document Phase 9 surface**

Add short sections for:
- `aidisk anomaly`
- local scheduled governance script
- current scope and future notifier adapters

**Step 2: Focused verification**

Run:
- `cargo test anomaly::tests`
- `cargo test --test anomaly_cli`
- `cargo test --test release_artifacts`

Expected: all pass.

**Step 3: Full verification**

Run: `cargo test`
Expected: all pass.

**Step 4: Commit**

Run:
- `git status --short`
- `git diff`
- `git add docs/execution-plan.md docs/plans/2026-06-08-phase-9-local-scheduled-governance.md aidisk/src/anomaly.rs aidisk/src/main.rs aidisk/src/reporter.rs aidisk/tests/anomaly_cli.rs scripts/governance/run-governance.ps1 README.md README.zh-CN.md docs/architecture.md aidisk/tests/release_artifacts.rs`
- `git commit -m "feat: add local governance anomaly detection"`

---

## Extended completion notes

Phase 9 status: Completed

The first implementation plan landed the Rust anomaly core and the initial local governance script. Follow-up commits intentionally completed the first Windows governance slice without changing the Phase 9 safety boundary.

Completed extended scope:

- `run-governance.ps1` writes a stable `governance-event.json` envelope for `anomaly_found`, `pending_history`, and `no_anomaly`.
- Governance events include message-friendly fields: `headline`, `summary_markdown`, `top_anomaly_path`, and `top_anomaly_growth_bytes`.
- The generic webhook adapter posts the governance event payload and records failed deliveries in `webhook-failure.json`.
- Windows Task Scheduler management scripts cover registration, inspection, unregistration, and immediate test runs.
- `register-governance-task.ps1` registers the scheduled local governance entry.
- `show-governance-task.ps1` displays task status and run timing.
- `unregister-governance-task.ps1` removes the task without confirmation prompts.
- `test-run-governance-task.ps1` calls `Start-ScheduledTask` on the existing task so the registered chain can be tested immediately.

Remaining product directions are intentionally outside Phase 9:

- v1.3.0 release readiness.
- Concrete platform notifier adapters after the generic adapter boundary is stable.
- Cross-platform scheduler adapters for cron, launchd, and systemd timer.
