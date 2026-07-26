# Changelog

## Unreleased

- Added scan and plan summary schema v2 fields to separate observed, potential, actionable, quarantine-ready, official/manual cleanup, report-only, and partial bytes.
- Changed planner semantics so `report-only`, `guide`, and `partial` findings do not enter executable cleanup candidates.
- Updated the visual dashboard reclaim checklist to use quarantine-ready entries instead of treating all `risk: safe` entries as cleanup-ready.
- Changed generic model file detection (`.gguf`, `.safetensors`, `.onnx`, `.mlx`) to `risk: review` with `cleanup.method: report-only` so unknown or custom models are not shown as safe-to-clean.
- Added `docs/report-schema.md` to document summary v2 semantics for downstream agents and JSON consumers.
- Hardened quarantine execution with platform-native destination paths, root containment checks, source/destination nesting guards, versioned execution indexes, execution stages, and copy-verify-remove fallback for rename failures.
- Added trusted distribution foundation with six-target release artifact matrix, versioned package naming, SHA-256 checksums, Cargo metadata SBOM, provenance JSON, package smoke tests, and Homebrew/winget draft manifests.
- Updated Cargo metadata and install docs to describe AI Disk Doctor as a cross-platform CLI with checksum verification, upgrade, uninstall, SBOM, and provenance guidance.
- Added Rule Schema v2 compatibility loading and validation with separated detector, decision, and action fields; migrated model cache and model file rules while retaining v1 rule loading.
- Added `aidisk rules lint` and scan rule source metadata with SHA-256 digests for rule provenance.
- Added read-only `aidisk models inventory` for conservative model asset discovery, logical versus physical size reporting, basic shared-path accounting, custom-model detection, and provenance graph output.
- Extended model inventory with bounded Hugging Face refs/snapshot/blob and Ollama manifest/blob reference analysis, conservative detached/orphan/incomplete states, and reference evidence in the provenance graph; all actions remain report-only.
- Added metadata-only stale and duplicate logical model markers, configurable via `--stale-after-days`, with explanatory reclaim confidence that never enables cleanup.
- Added metadata-only external-drive/cold-storage candidate markers for large managed model assets with stale, duplicate, detached, or orphaned cache signals; recommendations remain report-only.
- Added metadata-only cost-aware eviction signals for expected reclaim bytes, recovery size/time bands, network/offline recovery hints, shared-blob protection, and report-only utility bands.
- Added metadata-only LM Studio support to `models inventory` and `models adapters`; LM Studio model files are reported as managed, unresolved, report-only assets and do not invoke any official cleanup CLI.
- Added `aidisk models adapters` as a read-only capability report for Hugging Face, Ollama, and LM Studio model tooling boundaries; external CLIs are not invoked unless explicitly requested and allowlisted.
- Added explicit opt-in, bounded official CLI version/help probing for `models adapters`; probing never executes cleanup or mutation commands.
- Added explicit allowlisted official adapter dry-run invocation: Hugging Face uses `hf cache prune --dry-run --cache-dir <root>` only after help confirms dry-run support, while Ollama is limited to `ollama ls` read-only listing.
- Added report-only `official_cleanup_plan` normalization for Hugging Face official dry-run output; Ollama read-only list output remains evidence-only and does not create cleanup candidates.
- Added metadata-only rollback capability to official cleanup plans; Hugging Face candidates document manual redownload recovery, while no rollback or cleanup commands are executed.

## 1.6.0

- Added `aidisk visualize --html` to generate an interactive Swiss Style HTML dashboard with bilingual Chinese/English support, category filtering, tool detail expansion, KPI tooltips, and a safe reclaim checklist.
- Added `aidisk doctor --ai-footprint` to aggregate all AI-related findings across 10 categories with actionable recommendations.
- Added 5 new AI tool rules: GPU runners, coding agents, MCP servers, next-gen AI IDEs, and CUDA/cuDNN runtime environments.
- Added model file format detection (`.gguf`, `.safetensors`, `.onnx`, `.mlx`) with `risk: safe`.
- Upgraded 6 existing rules to cross-platform format supporting Windows, Linux, and macOS.
- Fixed multi-platform YAML rule parsing to handle both `platform:` string and `platforms:` array formats.
- Expanded `release_artifacts` test suite from 26 to 29 tests.

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
