# Cross-Platform Governance Verification Checklist

Manual verification checklist for cron, launchd, and systemd timer governance adapters on real Linux/macOS environments. Each platform section includes prerequisite checks, step-by-step commands, expected output, and a results table.

Before starting, ensure the environment has a valid Rust toolchain, the `aidisk` crate builds (`cargo build` from `aidisk/`), and at least one scan snapshot exists under `aidisk/.aidisk/reports/`.

---

## Platform 1: Linux / cron

### 1.1 Prerequisites

| # | Check | Command | Expected |
|---|-------|---------|----------|
| 1 | Rust toolchain | `cargo --version` | Version printed (>= 1.70) |
| 2 | jq installed | `jq --version` | Version printed |
| 3 | bash available | `bash --version` | Version printed |
| 4 | crontab available | `crontab -l 2>&1 \| head -1` | Either crontab entries or "no crontab" |
| 5 | aidisk builds | `cd aidisk && cargo build` | Compiles without errors |

### 1.2 Steps

#### Step 1: Register

```bash
./scripts/governance/cron/register-governance-cron.sh --schedule "* * * * *"
```

**Expected output:**

```
Successfully registered cron task 'aidisk-governance' with schedule: * * * * *
```

**Verify:**

```bash
crontab -l | grep aidisk-governance
```

Should show a line containing `aidisk-governance` with the schedule and path to `run-governance.sh`.

#### Step 2: Show

```bash
./scripts/governance/cron/show-governance-cron.sh
```

**Expected output:**

```
Task Name: aidisk-governance
Status: Registered
Schedule: * * * * * cd /path/to/repo && bash .../run-governance.sh --notifier-adapter local-file # aidisk-governance

Note: cron does not track last/next run times. Use system logs to check execution history.
```

#### Step 3: Test-Run

```bash
./scripts/governance/cron/test-run-governance-cron.sh
```

**Expected output:**

```
Starting governance task 'aidisk-governance' now...
... (governance pipeline output: scan, anomaly, event, dedup, notify) ...
Task execution completed.
```

**Verify artifacts:**

```bash
ls .aidisk/governance/
```

Should contain `governance-event.json`, `latest-scan.json`, `latest-anomaly.json`, `latest-anomaly.md`.

#### Step 4: Feishu Delivery

```bash
export FEISHU_WEBHOOK_URL="<test_webhook_url>"
./scripts/governance/run-governance.sh --notifier-adapter feishu
```

**Expected:** Governance runs, Feishu adapter sends a text message to the configured bot. No `feishu-failure.json` is written.

> Note: `FEISHU_WEBHOOK_URL` must be exported in the environment. It is never passed on the command line and never written to failure artifacts.

#### Step 5: Dedup Verification

Run governance twice with the same snapshot data:

```bash
./scripts/governance/run-governance.sh
./scripts/governance/run-governance.sh
```

**Expected:** The first run delivers (or writes the event normally). The second run detects the same event hash and writes `dedup-skipped.json`:

```bash
cat .aidisk/governance/dedup-skipped.json
```

Content shape:

```json
{
  "reason": "duplicate event detected",
  "generated_at": "2026-...",
  "event_hash": "...",
  "governance_event_path": ".aidisk/governance/governance-event.json"
}
```

#### Step 6: Retry Verification

Simulate a bad webhook URL to trigger delivery failure and retry exhaustion:

```bash
./scripts/governance/run-governance.sh --notifier-adapter webhook --webhook-url https://127.0.0.1:1/nonexistent --webhook-timeout-seconds 2
```

**Expected:** The retry wrapper attempts delivery 3 times (default `--max-retries 3`, 60s delay between attempts). After all attempts fail, `retry-failure.json` is written:

```bash
cat .aidisk/governance/retry-failure.json
```

Content shape:

```json
{
  "reason": "all notify retries exhausted",
  "max_retries": 3,
  "retry_delay": "60",
  "adapter": "webhook",
  "governance_event_path": ".aidisk/governance/governance-event.json",
  "failed_at": "2026-..."
}
```

#### Step 7: Unregister

```bash
./scripts/governance/cron/unregister-governance-cron.sh
```

**Expected output:**

```
Successfully unregistered cron task 'aidisk-governance'
```

**Verify:**

```bash
crontab -l | grep aidisk-governance
```

Should return no results.

### 1.3 Results

| Date | Tester | Environment | Outcome | Notes |
|------|--------|-------------|---------|-------|
|      |        |             |         |       |

---

## Platform 2: macOS / launchd

### 2.1 Prerequisites

| # | Check | Command | Expected |
|---|-------|---------|----------|
| 1 | Rust toolchain | `cargo --version` | Version printed (>= 1.70) |
| 2 | jq installed | `jq --version` | Version printed |
| 3 | bash available | `bash --version` | Version printed |
| 4 | launchctl available | `launchctl version` | Version printed |
| 5 | aidisk builds | `cd aidisk && cargo build` | Compiles without errors |

### 2.2 Steps

#### Step 1: Register

```bash
./scripts/governance/launchd/register-governance-launchd.sh --schedule-hour 9 --schedule-minute 0
```

**Expected output:**

```
Successfully registered launchd task 'aidisk-governance'
Plist file: /Users/<user>/Library/LaunchAgents/com.aidisk.aidisk-governance.plist
Schedule: Daily at 9:00
```

**Verify:**

```bash
launchctl list | grep aidisk-governance
```

Should show the agent as loaded.

#### Step 2: Show

```bash
./scripts/governance/launchd/show-governance-launchd.sh
```

**Expected output:**

```
Task Name: aidisk-governance
Label: com.aidisk.aidisk-governance
Status: Loaded

-   0   com.aidisk.aidisk-governance
```

(Exit code 0 in the `launchctl list` output means the last run succeeded.)

#### Step 3: Test-Run

```bash
./scripts/governance/launchd/test-run-governance-launchd.sh
```

**Expected output:**

```
Starting governance task 'aidisk-governance' now...
... (governance pipeline output) ...
Task execution completed.
```

**Verify artifacts:**

```bash
ls .aidisk/governance/
```

Should contain `governance-event.json`, `latest-scan.json`, `latest-anomaly.json`, `latest-anomaly.md`.

#### Step 4: Feishu Delivery

```bash
export FEISHU_WEBHOOK_URL="<test_webhook_url>"
./scripts/governance/run-governance.sh --notifier-adapter feishu
```

**Expected:** Governance runs, Feishu adapter sends a text message. No `feishu-failure.json`.

#### Step 5: Dedup Verification

Run governance twice:

```bash
./scripts/governance/run-governance.sh
./scripts/governance/run-governance.sh
```

**Expected:** Second run writes `dedup-skipped.json` in `.aidisk/governance/` with reason `"duplicate event detected"`.

#### Step 6: Retry Verification

```bash
./scripts/governance/run-governance.sh --notifier-adapter webhook --webhook-url https://127.0.0.1:1/nonexistent --webhook-timeout-seconds 2
```

**Expected:** `retry-failure.json` written in `.aidisk/governance/` with `"reason": "all notify retries exhausted"` after 3 attempts.

#### Step 7: Unregister

```bash
./scripts/governance/launchd/unregister-governance-launchd.sh
```

**Expected output:**

```
Successfully unregistered launchd task 'aidisk-governance'
```

**Verify:**

```bash
launchctl list | grep aidisk-governance
```

Should return no results. The plist file at `~/Library/LaunchAgents/com.aidisk.aidisk-governance.plist` should be removed.

### 2.3 Results

| Date | Tester | macOS Version | Outcome | Notes |
|------|--------|---------------|---------|-------|
|      |        |               |         |       |

---

## Platform 3: Linux / systemd timer

### 3.1 Prerequisites

| # | Check | Command | Expected |
|---|-------|---------|----------|
| 1 | Rust toolchain | `cargo --version` | Version printed (>= 1.70) |
| 2 | jq installed | `jq --version` | Version printed |
| 3 | bash available | `bash --version` | Version printed |
| 4 | systemctl --user | `systemctl --user status 2>&1 \| head -1` | No error about session bus |
| 5 | linger enabled | `loginctl show-user $USER \| grep Linger` | `Linger=yes` |
| 6 | aidisk builds | `cd aidisk && cargo build` | Compiles without errors |

> If `Linger=no`, run `sudo loginctl enable-linger $USER`. Without linger, `systemd --user` services stop on logout.

### 3.2 Steps

#### Step 1: Register

```bash
./scripts/governance/systemd/register-governance-systemd.sh
```

**Expected output:**

```
Successfully registered systemd timer 'aidisk-governance'
Service file: /home/<user>/.config/systemd/user/aidisk-governance.service
Timer file: /home/<user>/.config/systemd/user/aidisk-governance.timer
Schedule: *-*-* 09:00:00
```

**Verify:**

```bash
systemctl --user is-enabled aidisk-governance.timer
systemctl --user is-active aidisk-governance.timer
```

Both should output the appropriate status.

#### Step 2: Show

```bash
./scripts/governance/systemd/show-governance-systemd.sh
```

**Expected output:**

```
=== Timer Status ===
● aidisk-governance.timer - Timer for AI Disk Doctor Governance (aidisk-governance)
   Loaded: loaded (...)
   Active: active (waiting) since ...
  Trigger: ...

=== Service Status ===
● aidisk-governance.service - AI Disk Doctor Governance (aidisk-governance)
   Loaded: loaded (...)
   Active: inactive (dead) ...

=== Next Scheduled Run ===
NEXT                        LEFT          LAST                        PASSED  UNIT
...                         ...           ...                         ...     aidisk-governance.timer
```

#### Step 3: Test-Run

```bash
./scripts/governance/systemd/test-run-governance-systemd.sh
```

**Expected output:**

```
Starting governance task 'aidisk-governance' now...
... (governance pipeline output) ...
Task execution completed.
```

**Verify artifacts:**

```bash
ls .aidisk/governance/
```

Should contain the full set of governance artifacts.

#### Step 4: Feishu Delivery

```bash
export FEISHU_WEBHOOK_URL="<test_webhook_url>"
./scripts/governance/run-governance.sh --notifier-adapter feishu
```

**Expected:** Governance runs, Feishu adapter sends a text message. No `feishu-failure.json`.

#### Step 5: Dedup Verification

Run governance twice:

```bash
./scripts/governance/run-governance.sh
./scripts/governance/run-governance.sh
```

**Expected:** Second run writes `dedup-skipped.json` in `.aidisk/governance/` with reason `"duplicate event detected"`.

#### Step 6: Retry Verification

```bash
./scripts/governance/run-governance.sh --notifier-adapter webhook --webhook-url https://127.0.0.1:1/nonexistent --webhook-timeout-seconds 2
```

**Expected:** `retry-failure.json` written in `.aidisk/governance/` with `"reason": "all notify retries exhausted"` after 3 attempts.

#### Step 7: Unregister

```bash
./scripts/governance/systemd/unregister-governance-systemd.sh
```

**Expected output:**

```
Successfully unregistered systemd timer 'aidisk-governance'
```

**Verify:**

```bash
systemctl --user status aidisk-governance.timer 2>&1
```

Should report that the unit is not found.

### 3.3 Results

| Date | Tester | Distro | Outcome | Notes |
|------|--------|--------|---------|-------|
|      |        |        |         |       |

---

## Cross-Platform Common Verification

The following checks apply identically across all platforms. Run them after any successful `test-run` execution.

### Governance Pipeline Artifacts

| Artifact | Path | Expected Content |
|----------|------|-----------------|
| Governance event | `.aidisk/governance/governance-event.json` | Valid JSON with `event_type`, `headline`, `summary_markdown`, `anomaly_summary` |
| Dedup-skipped | `.aidisk/governance/dedup-skipped.json` | Present after second identical run |
| Retry-failure | `.aidisk/governance/retry-failure.json` | Present after bad webhook URL exhausts retries |
| Feishu-failure | `.aidisk/governance/feishu-failure.json` | Present if Feishu delivery fails (never contains webhook URL) |
| Webhook-failure | `.aidisk/governance/webhook-failure.json` | Present if generic webhook delivery fails |
| Dedup hashes | `.aidisk/governance/dedup/` | One file per unique event hash |

### Governance Event Types

Run `jq .event_type .aidisk/governance/governance-event.json` and verify the value is one of:

| Event Type | Meaning |
|------------|---------|
| `anomaly_found` | One or more paths exceeded growth thresholds |
| `pending_history` | Not enough snapshots (fewer than 2) |
| `no_anomaly` | No paths exceeded thresholds |

### Cleanup After Verification

After completing all checks, clean up:

1. Unregister the scheduler (Step 7 in each platform section).
2. Remove governance artifacts if desired: `rm -rf .aidisk/governance/`

---

## Verification Summary

| Platform | Register | Show | Test-Run | Feishu | Dedup | Retry | Unregister |
|----------|----------|------|----------|--------|-------|-------|------------|
| Linux / cron | | | | | | | |
| macOS / launchd | | | | | | | |
| Linux / systemd | | | | | | | |

**How to fill:** Mark each cell with `PASS`, `FAIL`, or `N/A`. For `FAIL`, add details in the platform-specific results table.
