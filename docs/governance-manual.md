# Governance User Manual

AI Disk Doctor cross-platform scheduled governance guide. Covers Windows Task Scheduler, cron, launchd, and systemd timer registration, notifier adapter configuration, deduplication, retry, and troubleshooting.

## Prerequisites

### All Platforms

| Dependency | Purpose | Install |
|------------|---------|---------|
| `cargo` | Build and run `aidisk` CLI | [rustup](https://rustup.rs/) |
| `jq` | Process `governance-event.json` | `choco install jq` (Windows), `brew install jq` (macOS), `apt install jq` / `dnf install jq` (Linux) |

### Notifier Delivery

| Dependency | Purpose | Install |
|------------|---------|---------|
| `curl` | Webhook and Feishu delivery | Included on macOS/Linux; `choco install curl` on Windows |

### Platform-Specific Scheduler Tools

| Platform | Scheduler | Required |
|----------|-----------|----------|
| Windows | Task Scheduler | Built-in (`powershell.exe`) |
| Linux | cron | Built-in (`crontab`) |
| macOS | launchd | Built-in (`launchctl`) |
| Linux (systemd) | systemd timer | Built-in (`systemctl --user`) |

## Quick Start

### Windows Task Scheduler

```powershell
# Register a daily governance task at 09:00
.\scripts\governance\register-governance-task.ps1 -DailyAt "09:00"

# Register with webhook delivery
.\scripts\governance\register-governance-task.ps1 -DailyAt "09:00" -NotifierAdapter webhook -WebhookUrl https://example.test/webhook

# Show the registered task
.\scripts\governance\show-governance-task.ps1

# Trigger a test run immediately
.\scripts\governance\test-run-governance-task.ps1

# Unregister the task
.\scripts\governance\unregister-governance-task.ps1
```

The task is registered under the name "AI Disk Doctor Governance" and calls `run-governance.ps1`. Use `Get-ScheduledTask` to inspect and `Start-ScheduledTask` to trigger.

### cron (Linux)

```bash
# Register a daily cron job at 09:00
./scripts/governance/cron/register-governance-cron.sh --schedule "0 9 * * *"

# Register with Feishu delivery
FEISHU_WEBHOOK_URL="https://example.test/feishu-webhook" \
  ./scripts/governance/cron/register-governance-cron.sh --schedule "0 9 * * *" --notifier-adapter feishu

# Show the registered cron entry
./scripts/governance/cron/show-governance-cron.sh

# Trigger a test run immediately
./scripts/governance/cron/test-run-governance-cron.sh

# Remove the cron entry
./scripts/governance/cron/unregister-governance-cron.sh
```

The cron adapter adds a `crontab` entry tagged `aidisk-governance`. The `test-run` script calls `run-governance.sh` directly without touching the cron schedule.

### launchd (macOS)

```bash
# Register a daily launchd agent at 09:00
./scripts/governance/launchd/register-governance-launchd.sh --schedule-hour 9 --schedule-minute 0

# Register with webhook delivery
./scripts/governance/launchd/register-governance-launchd.sh --schedule-hour 9 --schedule-minute 0 --notifier-adapter webhook --webhook-url https://example.test/webhook

# Show the registered agent
./scripts/governance/launchd/show-governance-launchd.sh

# Trigger a test run immediately
./scripts/governance/launchd/test-run-governance-launchd.sh

# Unload and remove the agent
./scripts/governance/launchd/unregister-governance-launchd.sh
```

The launchd adapter creates a user agent plist in `~/Library/LaunchAgents/` with a `StartCalendarInterval`. Use `launchctl list | grep aidisk` to verify.

### systemd timer (Linux)

```bash
# Register a daily systemd user timer at 09:00
./scripts/governance/systemd/register-governance-systemd.sh --schedule "*-*-* 09:00:00"

# Register with Feishu delivery
FEISHU_WEBHOOK_URL="https://example.test/feishu-webhook" \
  ./scripts/governance/systemd/register-governance-systemd.sh --schedule "*-*-* 09:00:00" --notifier-adapter feishu

# Show the registered timer and service
./scripts/governance/systemd/show-governance-systemd.sh

# Trigger a test run immediately
./scripts/governance/systemd/test-run-governance-systemd.sh

# Stop, disable, and remove the timer
./scripts/governance/systemd/unregister-governance-systemd.sh
```

The systemd adapter creates a user service and timer under `~/.config/systemd/user/`. Requires `systemctl --user` with linger enabled for persistent scheduling.

## Governance Flow

The governance pipeline runs in this order:

```
scan → anomaly → governance-event.json → dedup → notify
```

### Step 1: Scan

`run-governance.ps1` (Windows) or `run-governance.sh` (Unix) calls `aidisk scan --json` to produce a fresh scan snapshot. The snapshot is saved under `.aidisk/reports/`.

### Step 2: Anomaly Detection

`aidisk anomaly --latest` compares the two most recent snapshots using dual thresholds (absolute + relative growth). It outputs structured findings with paths, growth bytes, and growth percentages.

### Step 3: Event Packaging

The governance script wraps the anomaly result into a stable `governance-event.json` envelope. The event carries one of three types:

| Event Type | Meaning |
|------------|---------|
| `anomaly_found` | One or more paths exceeded growth thresholds |
| `pending_history` | Not enough snapshots (fewer than 2) |
| `no_anomaly` | No paths exceeded thresholds |

Each event includes `headline`, `summary_markdown`, `top_anomaly_path`, and `top_anomaly_growth_bytes` for message-friendly delivery.

### Step 4: Deduplication

Before delivery, `dedup-governance-event.sh` computes an `event_hash` from key fields in the governance event. If the same hash already exists in the dedup directory, the event is skipped and `dedup-skipped.json` is written. This prevents duplicate notifications when the scheduler triggers and no new growth has occurred.

### Step 5: Notification

The event is passed to `retry-governance-notify.sh`, which wraps `send-governance-event.sh` (the notifier dispatcher). The dispatcher routes to the configured adapter: `local-file`, `webhook` (generic), or `feishu`.

## Notifier Adapters

### local-file

No external delivery. The governance event remains in local artifacts only. This is the default adapter when no notifier is configured.

Use when you want to inspect governance artifacts manually or build your own delivery pipeline on top of `governance-event.json`.

### webhook (generic)

Posts the full `governance-event.json` payload to a generic HTTP endpoint.

```bash
./scripts/governance/run-governance.sh \
  --notifier-adapter webhook \
  --webhook-url https://example.test/webhook \
  --webhook-timeout-seconds 30
```

On failure, writes `webhook-failure.json` with delivery status, timing, and error context (no secrets).

### Feishu

Posts a formatted text message to a Feishu group bot using the `FEISHU_WEBHOOK_URL` environment variable.

```bash
export FEISHU_WEBHOOK_URL="https://example.test/feishu-webhook"

./scripts/governance/run-governance.sh --notifier-adapter feishu
```

Or use the dispatcher directly:

```bash
./scripts/governance/send-governance-event.sh \
  --adapter feishu \
  --event-path .aidisk/governance/governance-event.json \
  --output-dir .aidisk/governance
```

On failure, writes `feishu-failure.json` with timing, adapter, and event path context — but never includes the resolved webhook URL. Secrets must always come from the environment, not from command-line arguments.

## Reliability

### Idempotency (Dedup)

The `dedup-governance-event.sh` script prevents duplicate delivery of the same governance event:

- Computes a hash from key event fields (`event_type`, `headline`, `top_anomaly_path`, `top_anomaly_growth_bytes`, `anomaly_count`)
- Checks the hash against stored hashes in the dedup directory (`.aidisk/governance/dedup/`)
- If found: writes `dedup-skipped.json` and exits — no notification is sent
- If new: stores the hash and allows delivery to proceed

This is especially useful for cron/launchd/systemd schedules where the scheduler fires regularly but actual disk growth may not have changed between runs.

### Retry

The `retry-governance-notify.sh` wrapper retries failed notifier delivery:

- Default: 3 retries with 60-second linear delay between attempts
- Calls `send-governance-event.sh` with the configured adapter
- On success: exits 0 immediately
- After all retries exhausted: writes `retry-failure.json` with retry count, timings, and adapter name

```bash
./scripts/governance/retry-governance-notify.sh \
  --max-retries 5 \
  --retry-delay 30 \
  --adapter feishu \
  --event-path .aidisk/governance/governance-event.json \
  --output-dir .aidisk/governance
```

### Failure Artifacts

All failure artifacts are saved under the output directory for later diagnosis:

| Artifact | Trigger | Contents |
|----------|---------|----------|
| `webhook-failure.json` | Generic webhook delivery failed | Status, timing, error context |
| `feishu-failure.json` | Feishu delivery failed | Timing, adapter, event path (no webhook URL) |
| `retry-failure.json` | All retry attempts exhausted | Retry count, timings, adapter name |
| `dedup-skipped.json` | Event already delivered | Skip reason, event hash, timestamp |

## Templates

Feishu messages are built from a template file to allow customization of message format.

### Default Template

The default template is at `scripts/governance/templates/feishu-governance.tmpl`. It renders the governance event `headline` and `summary_markdown` into a Feishu text message.

### Custom Template

Create a custom template file and pass it to the Feishu adapter:

```bash
./scripts/governance/notifiers/feishu.sh \
  --event-path .aidisk/governance/governance-event.json \
  --output-dir .aidisk/governance \
  --template ./my-custom-feishu-template.tmpl
```

Template variables available:

| Variable | Description |
|----------|-------------|
| `${headline}` | One-line summary (e.g. "2 paths grew abnormally (+3.5 GB)") |
| `${summary_markdown}` | Markdown-formatted anomaly summary |

## Troubleshooting

### cron: No output, no notification

**Symptom:** Cron entry appears in `crontab -l` but governance never runs.

**Check:**
```bash
# Verify cron daemon is running
systemctl status cron   # Debian/Ubuntu
systemctl status crond  # RHEL/Fedora

# Check cron logs
grep CRON /var/log/syslog | grep aidisk
```

**Common causes:** Missing `PATH` in crontab (cron has minimal environment). The `register-governance-cron.sh` script sets `PATH` explicitly in the crontab entry. If you edit the entry manually, ensure `cargo` and `jq` are on the `PATH`.

### launchd: Agent not running

**Symptom:** `launchctl list | grep aidisk` shows status `1` or no entry.

**Check:**
```bash
# Verify plist is valid
plutil ~/Library/LaunchAgents/com.aidisk.governance.plist

# Check system log for launchd errors
log show --predicate 'subsystem == "com.apple.launchd"' --last 1h | grep aidisk
```

**Common causes:** Missing execute permissions on `run-governance.sh`, or `cargo` not on user `PATH`. Ensure `~/.cargo/bin` is in your shell profile. LaunchAgents inherit the user environment differently than interactive shells.

### systemd --user: Timer doesn't persist after logout

**Symptom:** Timer works with `systemctl --user start` but stops after logout.

**Fix:**
```bash
# Enable linger so user services persist after logout
sudo loginctl enable-linger $USER

# Verify
loginctl show-user $USER | grep Linger
```

Without `enable-linger`, `systemd --user` services stop when the user's last session ends.

### Feishu webhook: No response

**Symptom:** `feishu-failure.json` is written but no message appears in Feishu.

**Check:**
```bash
# Verify the webhook URL is correct
echo $FEISHU_WEBHOOK_URL

# Test the webhook manually
curl -X POST "$FEISHU_WEBHOOK_URL" \
  -H "Content-Type: application/json" \
  -d '{"msg_type":"text","content":{"text":"test"}}'
```

**Common causes:** Expired or invalid webhook URL, network restrictions blocking outbound HTTPS, or the bot has been removed from the group.

### Windows Task Scheduler: Task doesn't run

**Symptom:** Task exists but never executes.

**Check:**
```powershell
# View task history
Get-ScheduledTask -TaskName "AI Disk Doctor Governance" | Get-ScheduledTaskInfo

# Check Task Scheduler event log
Get-WinEvent -LogName "Microsoft-Windows-TaskScheduler/Operational" -MaxEvents 20 | Where-Object {$_.Message -match "aidisk"}
```

**Common causes:** PowerShell execution policy blocking scripts. Run `Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser` if needed.

### Dedup: Events always marked as duplicate

**Symptom:** All governance events produce `dedup-skipped.json` even when new growth exists.

**Check:**
```bash
# Inspect the dedup directory
ls .aidisk/governance/dedup/

# Remove stale hashes
rm .aidisk/governance/dedup/*
```

If the same anomaly set is detected repeatedly (no new growth), dedup correctly prevents duplicate notifications. This is expected behavior for environments where disk usage is stable.

### General: `jq: command not found`

Ensure `jq` is installed and available on `PATH`. See [Prerequisites](#prerequisites) for platform-specific install commands.
