# Changelog

## 1.5.0

- Added Cross-Platform Governance with Feishu notifier delivery, governance reliability (dedup + retry), a comprehensive user manual, and cross-platform CI.
- Added Notifier Adapter Foundation with `send-governance-event.sh` dispatcher for `local-file`, generic webhook, and Feishu adapters.
- Added Feishu delivery via `scripts/governance/notifiers/feishu.sh` using the `FEISHU_WEBHOOK_URL` environment variable for secrets and writing `feishu-failure.json` without storing webhook URLs.
- Added `scripts/governance/dedup-governance-event.sh` for governance event idempotency via event hash deduplication.
- Added `scripts/governance/retry-governance-notify.sh` with configurable `--max-retries` (default 3) and `--retry-delay` (default 60s) for notifier reliability.
- Added `scripts/governance/templates/feishu-governance.tmpl` for customizable Feishu message templates.
- Added `docs/governance-manual.md` covering all four scheduler platforms, notifier adapters, governance reliability, and troubleshooting.
- Added `docs/cross-platform-verification.md` for manual cross-platform verification.
- Extended GitHub CI to Ubuntu and macOS runners alongside Windows.
- Governance scheduling and delivery do not perform cleanup, do not run as a daemon, and use no background daemon.
- Concrete notifier adapter expansion beyond Feishu remains out of scope for this release.

## 1.4.0

- Added Cross-Platform Scheduled Governance for cron, launchd, and systemd timer.
- Added `scripts/governance/run-governance.sh`, a Unix governance entrypoint matching the Windows `run-governance.ps1` scan -> anomaly -> governance event -> notifier workflow.
- Added release artifact coverage for `run-governance.sh`, including scan/anomaly workflow markers, `governance-event.json`, generic webhook fields, and non-destructive safety checks.
- Added `scripts/governance/cron/` with register, show, unregister, and test-run scripts for cron-based scheduling.
- Added `scripts/governance/launchd/` with register, show, unregister, and test-run scripts for macOS launchd scheduling.
- Added `scripts/governance/systemd/` with register, show, unregister, and test-run scripts for systemd timer scheduling.
- Documented Unix governance dependencies: bash, jq, curl, and cargo.
- All scheduler adapters follow unified contract: register/show/unregister/test-run operations.
- Scheduler adapters remain script-level, platform-specific, with no background daemon or cleanup automation.
- Concrete notifier adapter expansion remains out of scope for this release.

## 1.3.0

- Added Local Scheduled Governance for detecting scan snapshot growth anomalies without cleanup automation.
- Added `aidisk anomaly` with `--latest` and explicit `--before` / `--after` workflows using absolute + relative growth thresholds.
- Added `scripts/governance/run-governance.ps1` for the local governance chain: scan, anomaly, and report artifact generation.
- Added stable `governance-event.json` payloads with `anomaly_found`, `pending_history`, and `no_anomaly` event types.
- Added message-friendly governance fields including `headline`, `summary_markdown`, `top_anomaly_path`, and `top_anomaly_growth_bytes`.
- Added generic webhook delivery for governance events, plus `webhook-failure.json` for failed delivery context.
- Added Windows Task Scheduler helpers: `register-governance-task.ps1`, `show-governance-task.ps1`, `unregister-governance-task.ps1`, and `test-run-governance-task.ps1`.
- Added `Start-ScheduledTask` test-run support so a registered governance task can be triggered immediately.
- Governance scheduling does not perform cleanup, does not run as a daemon, and does not bind to a single IM platform.

## 1.2.0

- Added `scan --large-files --min-size 500MB` for lightweight large file and directory discovery without classification or cleanup suggestions.
- Added built-in rules for common development artifacts including `node_modules`, Rust `target/`, Gradle caches, Python `__pycache__`, web `dist/`, `.next`, and `.turbo` caches.
- Added structured JSON error output for `--json` command failures. JSON-mode failures now write a single error object to stderr and keep stdout empty for consumers.
- Fixed `clean --dry-run --json --quarantine-root` to emit a single parseable JSON document instead of two consecutive JSON documents.
- Added cross-platform `~/` home directory path expansion alongside existing Windows `%VAR%` expansion for rules.
- Added linux/macOS paths to ollama, huggingface, and docker rules.
- Added operability metadata so rule-driven `scan`, `plan`, and `doctor` reports carry the active `policy snapshot`; when traversal is incomplete, text/markdown outputs mark sizes as `(partial)` and explain them as `best-effort, not exact` in the accompanying warning.
- Added `scan --policy <PATH>` for explicit policy selection during rule-driven read-only scans while keeping built-in defaults available when the default policy file is absent.

## 1.1.0

Doctor V2 release for AI tooling diagnostics and growth-aware topic analysis.

Included:

- `doctor --agents` for Claude, Codex, Gemini, opencode, AI IDE/CLI state, installed app roots, runtime caches, installers, and test artifacts.
- Bounded child breakdowns for active doctor findings so large AI roots show the biggest direct children.
- Data-driven doctor recommendations that account for missing paths, tiny placeholders, large roots, and cache-like children.
- `doctor --probe-tools` for opt-in Docker, WSL, and Ollama external probes without changing default read-only behavior.
- `doctor --latest` and `--reports-dir` for appending recent scan snapshot growth context to doctor output.
- Internal `DoctorTopicSpec` topic registry that centralizes built-in doctor topic names, defaults, matchers, recommendations, and probe metadata while keeping existing public flags unchanged.

Safety boundaries:

- Doctor remains read-only and never performs cleanup.
- External probes only run when `--probe-tools` is explicitly provided.
- `doctor --latest` only reads existing scan snapshots and keeps JSON output structured.
- The topic registry is code-side only in this release; no public `--topic` selector or external topic metadata format is introduced.

## 1.0.0

Initial v1 release-ready build for Windows AI Space Manager.

Included:

- `scan` for rule-driven Windows AI/storage discovery, volume summaries, top findings, and automatic scan snapshots.
- `plan` for safe-only dry-run planning with sensitive-path blocking and recently-modified filtering.
- `clean` dry-run and quarantine workflows with execution logs and recovery indexes.
- `restore` dry-run and execution from quarantine indexes, including conflict-safe `skipped-conflict` handling.
- `doctor` topic diagnostics for Docker, WSL, Ollama, Hugging Face, and Playwright storage patterns.
- `diff --latest` for comparing the newest two scan snapshots and explicit `--before` / `--after` diff support.
- `--rules-repo` for local or HTTPS community rule repositories.
- Skill integration with PowerShell wrappers and artifact tests.

Safety boundaries:

- Real cleanup still requires explicit `clean --yes` plus a quarantine root.
- Unknown or dangerous paths are reported or guided, not blindly removed.
- Community rules are loaded only from local directories or HTTPS git URLs.
