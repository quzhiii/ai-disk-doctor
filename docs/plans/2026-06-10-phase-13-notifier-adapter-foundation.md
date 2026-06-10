# Phase 13 Notifier Adapter Foundation Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a secure notifier adapter foundation and the first concrete Feishu adapter without changing the governance event contract.

**Architecture:** Keep `governance-event.json` as the stable producer/consumer boundary. Add a script-level dispatcher that can deliver an existing governance event through `local-file`, `webhook`, or `feishu`, and add a Feishu-specific adapter that reads `FEISHU_WEBHOOK_URL` from the environment instead of command-line arguments. `run-governance.sh` remains the orchestration entrypoint and delegates Feishu delivery to the dispatcher.

**Tech Stack:** Bash scripts, `jq`, `curl`, existing Rust release artifact tests, Markdown documentation.

---

## Scope

- Add static release artifact coverage for notifier adapter contract and Feishu safety boundaries.
- Add `scripts/governance/send-governance-event.sh` dispatcher.
- Add `scripts/governance/notifiers/feishu.sh` concrete adapter.
- Update `run-governance.sh` to support `--notifier-adapter feishu`.
- Document secrets handling and usage in `docs/notifier-adapters.md`, README, CHANGELOG, and roadmap.

## Non-Goals

- Do not add Slack, WeChat, DingTalk, email, retry queues, or daemon delivery.
- Do not store webhook secrets in source, command examples, or failure artifacts.
- Do not change Rust anomaly logic or `governance-event.json` schema.
- Do not add cleanup automation.

---

### Task 1: Add Notifier Adapter Artifact Test

**Files:**
- Modify: `aidisk/tests/release_artifacts.rs`

**Step 1: Write failing test**

Add `notifier_adapter_foundation_covers_feishu_contract` that reads:

- `docs/notifier-adapters.md`
- `scripts/governance/send-governance-event.sh`
- `scripts/governance/notifiers/feishu.sh`
- `scripts/governance/run-governance.sh`
- `CHANGELOG.md`
- `docs/execution-plan.md`

Assert the dispatcher includes `--adapter`, `--event-path`, `--output-dir`, `local-file`, `webhook`, `feishu`, `send_feishu_event`, and `FEISHU_WEBHOOK_URL`.

Assert the Feishu adapter includes `FEISHU_WEBHOOK_URL`, `curl`, `Content-Type: application/json`, `msg_type`, `summary_markdown`, `governance-event.json`, `feishu-failure.json`, and does not include `rm -rf`, `clean --yes`, or a hardcoded Feishu hook URL.

Assert `run-governance.sh` mentions `feishu` and `send-governance-event.sh`.

Assert docs mention `Notifier Adapter Foundation`, `Feishu`, `FEISHU_WEBHOOK_URL`, `secrets`, `generic webhook`, `governance-event.json`, and `feishu-failure.json`.

**Step 2: Verify RED**

Run: `cargo test notifier_adapter_foundation_covers_feishu_contract`

Expected: FAIL because docs and scripts do not exist.

**Step 3: Commit**

Commit: `test: add notifier adapter foundation coverage`

---

### Task 2: Add Notifier Dispatcher and Feishu Adapter

**Files:**
- Create: `scripts/governance/send-governance-event.sh`
- Create: `scripts/governance/notifiers/feishu.sh`

**Step 1: Implement dispatcher**

The dispatcher accepts `--adapter`, `--event-path`, `--output-dir`, `--webhook-url`, and `--webhook-timeout-seconds`.

- `local-file`: validates event file and exits 0.
- `webhook`: posts raw `governance-event.json` to `--webhook-url`, writes/updates delivery fields and `webhook-failure.json` on failure.
- `feishu`: calls `scripts/governance/notifiers/feishu.sh` and requires `FEISHU_WEBHOOK_URL` from the environment.

**Step 2: Implement Feishu adapter**

The adapter accepts `--event-path`, `--output-dir`, and `--webhook-timeout-seconds`.

- Reads webhook URL only from `FEISHU_WEBHOOK_URL`.
- Builds Feishu text payload from `headline` and `summary_markdown` using `jq`.
- Posts JSON with `curl` and `Content-Type: application/json`.
- On success, adds `delivery_status: delivered` and `notifier_adapter: feishu` to event file.
- On failure, writes `feishu-failure.json` without storing the webhook URL.

**Step 3: Verify GREEN for new test after docs are added later**

Run target test after Task 3 and Task 4.

**Step 4: Commit**

Commit: `feat(governance): add notifier dispatcher and feishu adapter`

---

### Task 3: Wire Feishu into run-governance.sh

**Files:**
- Modify: `scripts/governance/run-governance.sh`

**Step 1: Add Feishu case**

In `send_notifier_event`, add an `elif [[ "$NOTIFIER_ADAPTER" == "feishu" ]]` branch that delegates to `send-governance-event.sh --adapter feishu --event-path "$GOVERNANCE_EVENT_PATH" --output-dir "$RESOLVED_OUTPUT_DIR" --webhook-timeout-seconds "$WEBHOOK_TIMEOUT_SECONDS"`.

Do not add a `--feishu-url` CLI flag.

**Step 2: Verify script contract**

Run: `cargo test unix_governance_script_is_non_destructive_and_covers_scan_anomaly_workflow`

Expected: PASS.

**Step 3: Commit**

Commit: `feat(governance): wire feishu notifier into unix governance`

---

### Task 4: Add Notifier Adapter Documentation and Roadmap

**Files:**
- Create: `docs/notifier-adapters.md`
- Modify: `CHANGELOG.md`
- Modify: `docs/execution-plan.md`
- Modify: `README.md`
- Modify: `README.zh-CN.md`

**Step 1: Add docs**

Document:

- Notifier Adapter Foundation
- Stable `governance-event.json` boundary
- `local-file`, `webhook`, and `Feishu`
- `FEISHU_WEBHOOK_URL` secrets handling
- `feishu-failure.json`
- Examples using environment variables, not inline secrets

**Step 2: Update roadmap**

Add Phase 13 with status Completed and keep later Slack/WeChat/DingTalk adapters as future work.

**Step 3: Update README/CHANGELOG**

Add concise Unreleased notes and usage examples.

**Step 4: Run target test**

Run: `cargo test notifier_adapter_foundation_covers_feishu_contract`

Expected: PASS.

**Step 5: Commit**

Commit: `docs: document notifier adapter foundation`

---

### Task 5: Final Verification and Review

**Files:**
- Test only

**Step 1: Run release artifact tests**

Run: `cargo test --test release_artifacts`

Expected: PASS.

**Step 2: Run full test suite**

Run: `cargo test --all`

Expected: PASS.

**Step 3: Request code/security review**

Review for secrets leakage, command injection, artifact consistency, and roadmap drift.

**Step 4: Fix review findings and re-run verification**

Only push after fresh verification passes.

---

## Security Rules

- Feishu webhook URL must come from `FEISHU_WEBHOOK_URL`.
- Never write `FEISHU_WEBHOOK_URL` or the resolved URL into `feishu-failure.json`.
- Never include a real Feishu URL in docs or tests.
- Quote all paths and shell variables.
- Keep failure artifacts useful but non-secret.
