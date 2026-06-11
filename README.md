<div align="center">

# AI Disk Doctor

[![Version](https://img.shields.io/badge/version-1.6.0-blue?style=for-the-badge)](./CHANGELOG.md)
[![Rust](https://img.shields.io/badge/rust-1.78%2B-orange?style=for-the-badge)](https://rustup.rs/)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-green?style=for-the-badge)](./LICENSE-MIT)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20Linux%20%7C%20macOS-lightgrey?style=for-the-badge)]()

[中文](./README.zh-CN.md) · [Changelog](./CHANGELOG.md) · [Contributing](./CONTRIBUTING.md)

**AI-era disk space diagnostics and governance.**

Identify, analyze, and safely reclaim storage consumed by AI tools, browsers, and development environments—without guessing what's safe to delete.

</div>

---

## Table of Contents

[Motivation](#motivation) · [Overview](#overview) · [Key Features](#key-features) · [Why aidisk vs Manual Cleanup](#why-aidisk-vs-manual-cleanup) · [What's New](#whats-new) · [Installation](#installation) · [Quick Start](#quick-start) · [Command Reference](#command-reference) · [Safety First](#safety-first) · [Architecture](#architecture) · [Troubleshooting](#troubleshooting) · [Contributing](#contributing) · [License](#license)

---

## Motivation

AI tools have become essential to modern development, but they come with a hidden cost: **massive disk space consumption**.

- **Ollama** models weigh 4–70 GB each
- **Hugging Face** caches accumulate silently in `%USERPROFILE%\.cache`
- **Docker Desktop** images and **WSL** distros eat tens of gigabytes
- **Playwright** browser binaries install per-project
- **AI IDE/CLI caches, installers, test reports, and dev tool artifacts** pile up over months

Existing disk cleaners treat all files the same. They either delete too aggressively or leave AI-specific bloat untouched. **AI Disk Doctor** was born from a simple observation: *AI-era storage bloat has different patterns, different risks, and deserves a different tool.*

We believe cleanup should be **transparent** (you see exactly what will happen), **reversible** (quarantine, not delete), and **rule-driven** (no hardcoded magic paths). Every path is evaluated against YAML rules with explicit risk levels—so you never have to guess what's safe.

---

## Overview

AI Disk Doctor is a **rule-driven, safety-first** disk space diagnostic tool built for the AI era. It discovers space hogs across AI model caches, browser data, Docker images, WSL distros, and development artifacts—then helps you clean up with confidence.

The default posture is **conservative**: scan and report first, dry-run second, quarantine third—never delete directly. All destructive operations preview changes before touching your disk. Explicit `--yes` is required for any real action.

**Current release:** v1.6.0

For detailed architecture and design decisions, see [`docs/architecture.md`](./docs/architecture.md).

---

## Key Features

| Capability | What it does |
|-----------|-------------|
| **Intelligent Scanning** | Discover space usage across AI models, IDEs, CLIs, browsers, Docker, WSL, and dev artifacts |
| **AI-Aware Rules** | 25 YAML rules covering 200+ paths: Claude, Codex, Gemini, Ollama, LM Studio, MCP servers, CUDA, etc. |
| **Visual Dashboard** | `visualize --html` generates interactive HTML dashboard with bilingual support, category filtering, safe reclaim checklist |
| **AI Footprint Report** | `doctor --ai-footprint` aggregates all AI findings across 10 categories with actionable recommendations |
| **Cross-Platform** | Windows, Linux, macOS with platform-native paths for all AI tools |
| **Rule-Driven Classification** | 25 rules with risk levels: `safe`, `review`, `dangerous`. No hardcoded paths. |
| **Dry-Run by Default** | Preview all changes before touching disk. `--yes` required for real action. |
| **Quarantine Pattern** | Archive instead of delete. Full restore with conflict detection. |
| **Scheduled Governance** | Windows Task Scheduler / cron / launchd / systemd timer with anomaly detection |
| **Historical Diff** | Track growth over time with scan snapshots |
| **Growth Anomaly Detection** | Absolute + relative threshold alerts |

---

## Why aidisk vs Manual Cleanup

| Dimension | Manual Cleanup (AI Agent / Human) | aidisk |
|-----------|----------------------------------|--------|
| **Coverage** | Misses hidden caches, model blobs, AI IDE state | 25 rules covering 200+ known AI paths |
| **Risk Assessment** | Guess-based; might delete configs or credentials | Every path rated `safe` / `review` / `dangerous` |
| **Safety** | No quarantine; deletion is permanent | Quarantine + restore; `--dry-run` before any action |
| **Traceability** | Ad-hoc; no history | Snapshot diff + governance event history |
| **Cross-Platform** | Agent behavior varies by OS | Same rules on Windows / Linux / macOS |
| **Automated Governance** | Requires manual re-execution | Scheduled via Task Scheduler / cron / launchd / systemd timer |
| **AI Tool Awareness** | Limited to known tools | 25 rules: Claude, Codex, Gemini, Ollama, LM Studio, MCP, CUDA, etc. |
| **Dashboard** | Not available | Visual HTML dashboard with bilingual support |
| **Time Cost** | 30-60 minutes per session | 5 seconds to scan; full report in seconds |

---

## What's New

### v1.6.0

- **Visual dashboard** — `aidisk visualize --html`: interactive bilingual HTML dashboard with category filtering and safe reclaim checklist
- **AI footprint** — `doctor --ai-footprint`: aggregates all AI findings across 10 categories
- **5 new AI rules** — GPU runners, coding agents, MCP servers, next-gen IDEs, CUDA/cuDNN runtime
- **Model file detection** — GGUF/SafeTensors/ONNX/MLX glob matching with `risk: safe`
- **Cross-platform rules** — 6 rules upgraded to Windows/Linux/macOS

Full notes: [`CHANGELOG.md`](./CHANGELOG.md) · [`docs/release-notes/v1.6.0.md`](./docs/release-notes/v1.6.0.md).

### v1.5.0

- **Feishu notifier** — `FEISHU_WEBHOOK_URL` environment variable for secrets
- **Governance reliability** — event dedup + configurable retry
- **Cross-platform CI** — Windows / Ubuntu / macOS
- **User manual** — [`docs/governance-manual.md`](./docs/governance-manual.md)

### v1.4.0

- **Cross-Platform Governance** — cron, launchd, systemd timer + `run-governance.sh`

### v1.3.0

- **Local Scheduled Governance** — anomaly detection + governance events + Windows Task Scheduler

### v1.2.0

- **Coverage expansion** — large file discovery, cross-platform rules, JSON errors

### v1.1.0

- **Doctor V2** — AI agent diagnostics, child breakdowns, opt-in probes

### v1.0.0

- **Complete workflow** — scan, plan, clean, restore, doctor, diff

### Option 1: Pre-built Binary (Recommended — No Rust Required)

Download the latest release binary from the [Releases page](https://github.com/quzhiii/ai-disk-doctor/releases). Extract `aidisk.exe` and place it on your PATH.

### Option 2: Build from Source (Rust Required)

**Prerequisites:**

| Requirement | Version |
|------------|---------|
| Windows | 10/11 |
| Rust | 1.78+ |

Install Rust via [rustup](https://rustup.rs/) if needed.

```bash
git clone https://github.com/quzhiii/ai-disk-doctor.git
cd ai-disk-doctor/aidisk
cargo build --release
# Binary: target/release/aidisk.exe
```

### Option 3: PowerShell Skill Wrappers (Agent Integration)

No Rust or compilation needed. The `skills/windows-ai-space-manager/scripts/` directory contains standalone PowerShell wrappers that call the CLI. If you have the pre-built binary on PATH, these work immediately:

```powershell
# Scan via PowerShell wrapper
.\skills\windows-ai-space-manager\scripts\run-scan.ps1

# Run doctor
.\skills\windows-ai-space-manager\scripts\run-doctor.ps1
```

### Development Setup

```bash
cd ai-disk-doctor/aidisk
cargo build
cargo test
```

Verify your build with the non-destructive smoke test:

```powershell
pwsh -NoProfile -File "scripts/release-smoke.ps1"
```

---

## Quick Start

### 1. Scan Your System

```powershell
# Scan everything and output JSON
aidisk scan --json

# Generate Markdown report
aidisk scan --markdown

# Scan specific category
aidisk scan --category browser-cache --json
```

### 2. Generate a Cleanup Plan

```powershell
# Safe items only, dry-run
aidisk plan --safe-only --json

# Include careful items, skip recently modified
aidisk plan --json --skip-modified-within-minutes 30
```

### 3. Execute Safe Cleanup (Quarantine)

```powershell
# Preview quarantine plan
aidisk clean --dry-run --safe-only --quarantine-root "F:\archives"

# Execute quarantine (requires --yes)
aidisk clean --yes --safe-only --quarantine-root "F:\archives"
```

### 4. Restore if Needed

```powershell
# Preview restore
aidisk restore --dry-run --index "F:\archives\.aidisk\quarantine-index-YYYYMMDD-HHMMSS.json"

# Execute restore
aidisk restore --yes --index "F:\archives\.aidisk\quarantine-index-YYYYMMDD-HHMMSS.json"
```

### 5. Run Diagnostics

```powershell
# Full system diagnosis
aidisk doctor --markdown

# Specific topics
aidisk doctor --docker --json
aidisk doctor --wsl --ollama --markdown
aidisk doctor --playwright --huggingface --markdown
aidisk doctor --agents --markdown
aidisk doctor --docker --probe-tools --markdown

# Combine current diagnosis with latest snapshot growth
aidisk doctor --agents --latest --markdown
aidisk doctor --latest --reports-dir .aidisk/reports --json
```

`doctor --agents` covers Claude, Codex, Gemini, opencode, Cursor, Windsurf, Trae, aider, Continue, installed apps, AI runtime caches, installers, and test artifacts.
Built-in doctor topics are assembled from an internal registry, so default runs and explicit topic flags use the same source of truth without changing the public CLI.
Markdown/Text doctor output focuses on active paths and summarizes missing matches as `Not detected`; JSON keeps the complete finding list for automation.
Use `--probe-tools` to opt into external command probes such as `docker system df`, `wsl --list --verbose`, and `ollama list`.
Use `--latest` to append the most recent growth summary from the newest two scan snapshots in `.aidisk/reports`; override the snapshot location with `--reports-dir` when needed.

### 6. Compare Snapshots

```powershell
# Auto-compare last two scans
aidisk diff --latest --markdown

# Compare specific snapshots
aidisk diff --before scan-20260101-120000.json --after scan-20260102-120000.json --markdown
```

### 7. Run Local Governance

```powershell
# Run one local governance cycle
.\scripts\governance\run-governance.ps1

# Keep artifacts under a custom directory
.\scripts\governance\run-governance.ps1 -OutputDir ".aidisk\governance"

# Tune anomaly thresholds
.\scripts\governance\run-governance.ps1 -MinGrowth "2GB" -MinGrowthPercent 50

# Send anomaly JSON to a generic webhook endpoint
.\scripts\governance\run-governance.ps1 -NotifierAdapter webhook -WebhookUrl https://example.test/webhook

# Tune webhook timeout for slower endpoints
.\scripts\governance\run-governance.ps1 -NotifierAdapter webhook -WebhookUrl https://example.test/webhook -WebhookTimeoutSeconds 30
```

The governance script keeps the workflow read-only: it runs `scan`, reuses scan snapshots, and emits anomaly artifacts locally. On the first run, if history does not yet contain two snapshots, it writes a pending note instead of failing. It also writes a stable `governance-event.json` envelope with one of three event types: `anomaly_found`, `pending_history`, or `no_anomaly`. The event includes message-friendly summary fields such as `headline`, `summary_markdown`, `top_anomaly_path`, and `top_anomaly_growth_bytes`. `-NotifierAdapter webhook` posts that governance event payload to a generic HTTP endpoint so future WeChat / WeCom / Feishu / Slack / Telegram / Discord adapters can share the same contract. If webhook delivery fails, the local artifacts are kept and the script writes `webhook-failure.json` with `delivery_status`, timeout, and error context for follow-up.

### 8. Register a Daily Windows Task

```powershell
# Register a daily governance run at 09:00
.\scripts\governance\register-governance-task.ps1 -DailyAt "09:00"

# Register a webhook-enabled daily run
.\scripts\governance\register-governance-task.ps1 -DailyAt "09:00" -NotifierAdapter webhook -WebhookUrl https://example.test/webhook

# Show the registered governance task
.\scripts\governance\show-governance-task.ps1

# Trigger the registered governance task immediately for a test run
.\scripts\governance\test-run-governance-task.ps1

# Unregister the governance task
.\scripts\governance\unregister-governance-task.ps1
```

The scheduler setup script only registers a Windows Task Scheduler entry that calls `run-governance.ps1`; it does not perform cleanup or delete any files. Use `test-run-governance-task.ps1` to call `Start-ScheduledTask` on the existing task when you want to verify the registered local governance chain immediately.

### 9. Register Cross-Platform Governance Schedulers

```bash
# Run one Unix-like governance cycle directly
./scripts/governance/run-governance.sh --notifier-adapter local-file

# Send a Unix governance event to a generic webhook
./scripts/governance/run-governance.sh --notifier-adapter webhook --webhook-url https://example.test/webhook

# cron: register, show, test-run, unregister
./scripts/governance/cron/register-governance-cron.sh --schedule "0 9 * * *"
./scripts/governance/cron/show-governance-cron.sh
./scripts/governance/cron/test-run-governance-cron.sh
./scripts/governance/cron/unregister-governance-cron.sh

# launchd: register, show, test-run, unregister
./scripts/governance/launchd/register-governance-launchd.sh --schedule-hour 9 --schedule-minute 0
./scripts/governance/launchd/show-governance-launchd.sh
./scripts/governance/launchd/test-run-governance-launchd.sh
./scripts/governance/launchd/unregister-governance-launchd.sh

# systemd timer: register, show, test-run, unregister
./scripts/governance/systemd/register-governance-systemd.sh --schedule "*-*-* 09:00:00"
./scripts/governance/systemd/show-governance-systemd.sh
./scripts/governance/systemd/test-run-governance-systemd.sh
./scripts/governance/systemd/unregister-governance-systemd.sh
```

Unix-like governance requires `bash`, `jq`, `curl` for webhook or Feishu delivery, and `cargo` for local runs. The cron, launchd, and systemd timer adapters only register native scheduler entries that call `run-governance.sh`; they do not add a background daemon or perform cleanup. On the current branch, Phase 13 adds a concrete Feishu notifier adapter while keeping broader adapter expansion for later.

### 10. Send Governance Events to Feishu

```bash
# Inject the Feishu webhook as an environment variable, not a command-line flag
export FEISHU_WEBHOOK_URL="https://example.test/feishu-webhook"

# Send an existing governance-event.json through the notifier dispatcher
./scripts/governance/send-governance-event.sh --adapter feishu --event-path .aidisk/governance/governance-event.json --output-dir .aidisk/governance

# Or run governance and deliver through Feishu in one step
./scripts/governance/run-governance.sh --notifier-adapter feishu
```

See [`docs/notifier-adapters.md`](./docs/notifier-adapters.md) for the Notifier Adapter Foundation, Feishu adapter, `FEISHU_WEBHOOK_URL` secrets handling, generic webhook compatibility, and `feishu-failure.json` behavior.

For comprehensive governance documentation covering all four platforms, deduplication, retry, notifier adapters, and troubleshooting, see [`docs/governance-manual.md`](./docs/governance-manual.md).

---

## Command Reference

| Command | Description | Key Flags |
|---------|-------------|-----------|
| `scan` | Discover and classify space usage | `--category`, `--rules-repo`, `--json`, `--markdown` |
| `scan --large-files` | Discover largest files and directories | `--min-size`, `--root`, `--json`, `--markdown` |
| `plan` | Generate cleanup recommendations | `--safe-only`, `--skip-modified-within-minutes` |
| `clean` | Execute quarantine or dry-run | `--dry-run`, `--yes`, `--quarantine-root`, `--safe-only` |
| `restore` | Restore quarantined files | `--dry-run`, `--yes`, `--index` |
| `doctor` | Run targeted diagnostics | `--agents`, `--docker`, `--wsl`, `--ollama`, `--playwright`, `--huggingface`, `--probe-tools`, `--latest`, `--reports-dir` |
| `diff` | Compare scan snapshots | `--latest`, `--before`, `--after` |
| `anomaly` | Detect growth anomalies from scan snapshots | `--latest`, `--before`, `--after`, `--min-growth`, `--min-growth-percent` |

### JSON Error Contract

When `--json` or `--format json` is selected and a command fails, `aidisk` writes one JSON error object to stderr and leaves stdout empty. Successful JSON reports are still written to stdout.

```json
{
  "ok": false,
  "error": {
    "type": "usage",
    "message": "restore execution requires --yes or use --dry-run",
    "command": "restore",
    "details": []
  }
}
```

---

## Safety First

### Default Behavior

- **Scan** is always read-only
- **Plan** is always read-only
- **Clean** defaults to `--dry-run`; requires `--yes` for actual changes
- **Restore** defaults to `--dry-run`; requires `--yes` for actual changes

### Quarantine Safety

- Cross-disk moves use copy+delete fallback (Windows `rename` fails across drives)
- Restore validates index structure before execution
- Conflicts (destination exists) are skipped and reported, never overwritten

### Risk Levels

| Level | Meaning | Default Action |
|-------|---------|----------------|
| `safe` | Well-known cache/temp directories | Included in `--safe-only` |
| `careful` | User data that may be needed | Requires explicit inclusion |
| `dangerous` | System-critical or irreversible | Blocked by sensitive path filter |

---

## Architecture

```text
User / AI Agent
       |
       v
  aidisk CLI
       |
       +-- Config Loader (policy.yaml)
       +-- Rules Engine (YAML rules + glob expansion)
       +-- Scanner (WalkDir with depth limits)
       +-- Planner (risk filter + sensitive path blocking)
       +-- Cleaner (quarantine / restore with cross-disk fallback)
       +-- Doctor (registry-driven topic analyzers)
       +-- Diff Engine (snapshot comparison)
       +-- Anomaly Engine (growth threshold detection)
       +-- Reporter (JSON / Markdown output)
```

For detailed architecture documentation, see [`docs/architecture.md`](./docs/architecture.md).

### Design Principles

1. **Conservative by Default** — Unknown paths are reported, never processed
2. **Safety-First Execution** — All mutation commands default to dry-run
3. **Quarantine Over Delete** — Files are moved, not deleted; recoverable via index
4. **Rule-Driven Everything** — Path recognition, risk classification, and policy enforcement come from external YAML rules
5. **Agent-Ready Interface** — Structured output formats enable AI agent integration

---

## Troubleshooting

### Where can I get a pre-built binary?

Check the [Releases](https://github.com/quzhiii/ai-disk-doctor/releases) page. If no binary is available yet, building from source requires only Rust (installed via [rustup](https://rustup.rs/) in minutes).

### Do I need Rust to use this?

**No** — Download the pre-built `aidisk.exe` from Releases. Rust is only needed if you want to build from source or contribute code.

### Can I use PowerShell without the Rust CLI?

The PowerShell wrappers in `skills/` call the `aidisk` CLI under the hood. You need the binary, but no Rust toolchain.

### Is there a Python version?

Not yet. The core engine is Rust for performance and safety. A Python binding or native Python port may come if the community wants it. Contributions welcome!

### `cargo build` fails on Windows

Ensure you have the latest stable Rust toolchain:

```bash
rustup update stable
```

If you see linker errors, install Visual Studio Build Tools with C++ workload.

### Scan finds no paths

Check that your environment variables (like `%USERPROFILE%`) are properly set. AI Disk Doctor expands these in rule patterns.

### Quarantine fails with "Access Denied"

Some directories are locked by running processes. Close browsers, Docker Desktop, or WSL before quarantine operations.

### Cross-disk quarantine is slow

When quarantining across drives (e.g., C: to F:), Windows requires copy+delete instead of a fast rename. This is expected behavior for safety.

---

## Contributing

Contributions of every kind are welcome — bug reports, new rules, documentation, and core improvements. See [`CONTRIBUTING.md`](./CONTRIBUTING.md) for:

- Development setup
- Code standards
- Adding new rules
- Pull request process

[Code of Conduct](./CODE_OF_CONDUCT.md) · [Security](./SECURITY.md) · [License](#license)

---

## Acknowledgments

- Built with Rust. Special thanks to the Rust community for crates like `clap`, `walkdir`, `serde`, and `sysinfo`.

## License

This project is dual-licensed under:

- **MIT License** — See [LICENSE-MIT](./LICENSE-MIT)
- **Apache License 2.0** — See [LICENSE-APACHE](./LICENSE-APACHE)

You may choose either license at your option.

---

<div align="center">

**Made with ❤️ for cleaner disks and clearer minds.**

</div>
