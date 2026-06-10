# Phase 14: Governance Reliability, Documentation, and Cross-Platform Verification

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Harden the notifier foundation with retry and idempotency, create comprehensive cross-platform user manuals, and verify scheduler + notifier workflows on real Linux/macOS environments.

**Architecture:** Keep the dispatcher/adapter boundary from Phase 13 unchanged. Add a thin retry + dedup wrapper around the dispatcher so adapters stay simple. Create a single `docs/governance-manual.md` covering Windows/cron/launchd/systemd scheduler usage and Feishu/webhook notifier delivery. Extend GitHub CI with Ubuntu and macOS runners for basic smoke verification, and document manual verification results for scheduler-specific behavior.

**Tech Stack:** Bash scripts, `jq`, `curl`, Markdown documentation, GitHub Actions, existing Rust release artifact tests.

---

## Milestone M1: Notifier Reliability Enhancements

### Task M1.1: Add Governance-Event Deduplication

**Files:**
- Create: `scripts/governance/dedup-governance-event.sh`
- Modify: `aidisk/tests/release_artifacts.rs` (add dedup contract test)

**Step 1: Write failing test**

Add `governance_event_dedup_script_covers_idempotency_contract` test that reads:
- `scripts/governance/dedup-governance-event.sh`
- `scripts/governance/send-governance-event.sh`
- `docs/governance-manual.md`

Asserts dedup script contains `--event-hash`, `--dedup-dir`, `jq`, and does NOT contain `rm -rf`.

Asserts dispatcher or `run-governance.sh` references `dedup-governance-event.sh`.

Asserts manual doc references `dedup`, `idempotency`, `event hash`.

**Step 2: Verify RED**

Run: `cargo test governance_event_dedup_script_covers_idempotency_contract`
Expected: FAIL (script not found).

**Step 3: Implement dedup script**

The script accepts `--event-path`, `--dedup-dir`, and `--output-dir`.

- Computes an `event_hash` from key governance event fields using `jq`.
- Checks if `event_hash` already exists in the dedup directory.
- If duplicate: writes `dedup-skipped.json` with skip reason and timestamp, exits 0.
- If new: stores the hash file, exits 0 with `dedup-fresh` marker.

No real secret storage, no destructive cleanup.

**Step 4: Verify GREEN**

**Step 5: Commit**

---

### Task M1.2: Add Notifier Retry Wrapper

**Files:**
- Create: `scripts/governance/retry-governance-notify.sh`
- Modify: `aidisk/tests/release_artifacts.rs` (add retry contract test)

**Step 1: Write failing test**

Add `governance_notify_retry_script_covers_contract` test that reads:
- `scripts/governance/retry-governance-notify.sh`
- `scripts/governance/send-governance-event.sh`
- `docs/governance-manual.md`

Asserts retry script contains `--max-retries`, `--retry-delay`, `--event-path`, `--adapter`, `send-governance-event.sh`, `retry-failure.json`, and does NOT contain `rm -rf`.

Asserts dispatcher or manual doc references `retry-governance-notify.sh`.

**Step 2: Verify RED**

Run: `cargo test governance_notify_retry_script_covers_contract`
Expected: FAIL.

**Step 3: Implement retry wrapper**

- Accepts `--max-retries` (default 3), `--retry-delay` (default 60s).
- Calls `send-governance-event.sh` with the given adapter and event path.
- After each failed attempt, sleeps `--retry-delay` seconds (linear, not exponential).
- After all retries exhausted, writes `retry-failure.json` with retry count, timings, adapter.
- On success, exits 0 immediately.
- Never doubles up on the adapter itself — just wraps the dispatcher.

**Step 4: Verify GREEN**

**Step 5: Commit**

---

### Task M1.3: Integrate Dedup + Retry into Governance Flow

**Files:**
- Modify: `scripts/governance/run-governance.sh`
- Modify: `aidisk/tests/release_artifacts.rs` (update Unix governance contract)

**Step 1: Update governance flow**

In `run-governance.sh`, after writing `governance-event.json` and before calling dispatcher:
- Call `dedup-governance-event.sh --event-path ... --dedup-dir ...`
- If dedup returns "skipped", exit early without delivery.
- Otherwise, call `retry-governance-notify.sh` instead of raw dispatcher.

**Step 2: Update artifact tests**

Update `unix_governance_script_is_non_destructive_and_covers_scan_anomaly_workflow` to assert `dedup-governance-event.sh` and `retry-governance-notify.sh` references in the governance script.

**Step 3: Verify all tests pass**

Run: `cargo test --all`
Expected: PASS.

**Step 4: Commit**

---

### Task M1.4: Add Message Template Support

**Files:**
- Create: `scripts/governance/templates/feishu-governance.tmpl`
- Modify: `scripts/governance/notifiers/feishu.sh` (accept `--template` argument)
- Modify: `docs/governance-manual.md`

**Step 1: Create the default template file**

A template file that contains minimal `jq`-based string interpolation for the Feishu text message body using the governance event fields (`headline`, `summary_markdown`, `top_anomaly_path`, etc.).

**Step 2: Update Feishu adapter**

Add `--template` argument (optional). If provided, read template file and use it instead of the hardcoded text format. This is a minimal template system: the file is just a text file that the adapter reads with `cat` and passes as `text` field.

**Step 3: Update artifact tests**

Add coverage that the Feishu adapter script contains `--template` and the template file is referenced in docs.

**Step 4: Verify all tests pass**

**Step 5: Commit**

---

## Milestone M2: Cross-Platform User Manual

### Task M2.1: Create the Governance User Manual

**Files:**
- Create: `docs/governance-manual.md`
- Modify: `README.md` (link to manual)
- Modify: `README.zh-CN.md` (link to manual)
- Modify: `aidisk/tests/release_artifacts.rs` (manual contract test)

**Step 1: Write failing test**

Add `governance_manual_covers_all_platforms_and_adapters` test that:
- Reads `docs/governance-manual.md`
- Asserts it contains sections for: `cron`, `launchd`, `systemd timer`, `Windows Task Scheduler`, `Feishu`, `generic webhook`, `local-file`, `FEISHU_WEBHOOK_URL`, `prerequisites`, `idempotency`, `retry`, `troubleshooting`.
- Asserts README links to `docs/governance-manual.md`.

**Step 2: Verify RED**

Run: `cargo test governance_manual_covers_all_platforms_and_adapters`
Expected: FAIL.

**Step 3: Write the manual**

Cover these sections:
1. **Prerequisites** — cargo, jq, curl, bash, platform-specific scheduler tools.
2. **Quick Start** — one-section per platform: Windows (Task Scheduler), cron, launchd, systemd timer.
3. **Governance Flow** — scan → anomaly → governance-event.json → dedup → notify.
4. **Notifier Adapters** — local-file, generic webhook, Feishu.
5. **Reliability** — idempotency (dedup), retry, retention of failure artifacts.
6. **Templates** — how to customize Feishu messages.
7. **Troubleshooting** — common issues per platform and per notifier.

**Step 4: Verify GREEN**

**Step 5: Update README links**

Add `For detailed governance documentation, see [docs/governance-manual.md].`

**Step 6: Commit**

---

### Task M2.2: Update Existing Docs for M1 Deliverables

**Files:**
- Modify: `docs/notifier-adapters.md`
- Modify: `CHANGELOG.md`
- Modify: `docs/execution-plan.md`

**Step 1: Update notifier docs**

Add sections for dedup and retry under a "Reliability" heading.

**Step 2: Update CHANGELOG**

Add Unreleased entries for M1 deliverables.

**Step 3: Update roadmap**

Add Phase 14 entry with M1/M2/M3 status tracking.

**Step 4: Commit**

---

## Milestone M3: Cross-Platform Real Environment Verification

### Task M3.1: Extend GitHub CI with Ubuntu and macOS Runners

**Files:**
- Modify: `.github/workflows/ci.yml`

**Step 1: Add Ubuntu test job**

Add a `test-ubuntu` job that:
- Runs on `ubuntu-latest`.
- Checks out repo.
- Installs Rust toolchain.
- Runs `cargo test --all` from `aidisk/`.
- Runs `bash -n scripts/governance/*.sh scripts/governance/**/*.sh` for syntax check.

**Step 2: Add macOS test job**

Add a `test-macos` job that:
- Runs on `macos-latest`.
- Same steps as Ubuntu.

**Step 3: Keep existing Windows job unchanged.**

**Step 4: Commit**

---

### Task M3.2: Create Manual Verification Checklist

**Files:**
- Create: `docs/cross-platform-verification.md`

**Step 1: Write the checklist**

Document for each platform (Linux/cron, macOS/launchd, Linux/systemd):
1. Prerequisites checklist.
2. Steps: register, show, test-run, unregister.
3. Expected outputs for each step.
4. Feishu delivery verification.
5. Dedup verification.
6. Retry verification.
7. Space for manual results (date, tester, outcome, notes).

**Step 2: Commit**

---

### Task M3.3: Final Verification and Push

**Step 1: Run all tests**

```bash
cargo test --all
```

Expected: PASS.

**Step 2: Request code review on full Phase 14 diff.**

**Step 3: Fix review findings.**

**Step 4: Commit and push.**

---

## Execution Order

```
M1.1 (dedup) → M1.2 (retry) → M1.3 (integrate) → M1.4 (templates)
    ↓
M2.1 (manual) → M2.2 (update docs)
    ↓
M3.1 (CI) → M3.2 (checklist) → M3.3 (final)
```

Each task is independently committable. Use TDD: test first, RED, implement, GREEN, commit.

## Non-Goals

- Do not add Slack / WeChat / DingTalk adapters.
- Do not add exponential backoff or jitter — keep linear retry minimal.
- Do not change Rust anomaly core.
- Do not change governance-event.json schema.
- Do not add daemon or cleanup automation.

