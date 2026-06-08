<div align="center">

# AI Disk Doctor

[![Version](https://img.shields.io/badge/version-1.2.0-blue?style=for-the-badge)](./CHANGELOG.md)
[![Rust](https://img.shields.io/badge/rust-1.78%2B-orange?style=for-the-badge)](https://rustup.rs/)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-green?style=for-the-badge)](./LICENSE-MIT)
[![Platform](https://img.shields.io/badge/platform-Windows-lightgrey?style=for-the-badge)]()

[中文](./README.zh-CN.md) · [Changelog](./CHANGELOG.md) · [Contributing](./CONTRIBUTING.md)

**AI-era disk space diagnostics and governance for Windows.**

Identify, analyze, and safely reclaim storage consumed by AI tools, browsers, and development environments—without guessing what's safe to delete.

</div>

---

## Table of Contents

[Motivation](#motivation) · [Overview](#overview) · [What's New](#whats-new) · [Key Features](#key-features) · [Installation](#installation) · [Quick Start](#quick-start) · [Command Reference](#command-reference) · [Safety First](#safety-first) · [Architecture](#architecture) · [Troubleshooting](#troubleshooting) · [Contributing](#contributing) · [License](#license)

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

**Current release:** v1.2.0

For detailed architecture and design decisions, see [`docs/architecture.md`](./docs/architecture.md).

---

## What's New

### v1.2.0

Phase 7 expands coverage and discovery while keeping the same conservative cleanup posture:

- **Large Files Discovery** — `scan --large-files --min-size 500MB` discovers the largest files and directories under a root path with no classification or cleanup suggestions.
- **Developer artifact coverage** — Built-in rules now detect common regenerable artifacts such as `node_modules`, Rust `target/`, Gradle caches, Python `__pycache__`, `dist/`, `.next`, and `.turbo`.
- **Cross-platform rule paths** — rules now expand Unix `~/` home directory paths alongside Windows `%VAR%` tokens, with linux/macOS paths added for Ollama, Hugging Face, and Docker.
- **Structured JSON errors** — `--json` command failures now write a single error object to stderr and keep stdout empty for consumers.
- **Operability metadata** — rule-driven `scan`, `plan`, and `doctor` now surface the active `policy snapshot`; when traversal is incomplete, text/markdown outputs mark sizes as `(partial)` and explain them as `best-effort, not exact`; and rule-driven `scan --policy` supports explicit policy selection.

Full notes: [`CHANGELOG.md`](./CHANGELOG.md) · [`docs/release-notes/v1.2.0.md`](./docs/release-notes/v1.2.0.md).

### v1.1.0

Doctor V2 improves AI-era diagnostics while preserving the conservative, read-only default posture:

- **AI tooling diagnostics** — `doctor --agents` covers AI agent roots, AI IDE/CLI state, runtime caches, installers, installed app roots, and test artifacts
- **Child breakdowns** — Active doctor findings show the largest direct children so oversized roots are easier to interpret
- **Opt-in probes** — `--probe-tools` can add Docker, WSL, and Ollama command probes without running external commands by default
- **Growth-aware doctor** — `doctor --latest` appends the newest scan snapshot growth context, with `--reports-dir` for custom history locations
- **Registry-driven topics** — Built-in doctor topics now share an internal `DoctorTopicSpec` registry for defaults, matching, recommendations, and probe metadata

Full notes: [`CHANGELOG.md`](./CHANGELOG.md) · [`docs/release-notes/v1.1.0.md`](./docs/release-notes/v1.1.0.md).

### v1.0.0

The first stable release brings the complete local workflow:

- **Complete command set** — `scan`, `plan`, `clean`, `restore`, `doctor`, and `diff --latest`
- **Community rules** — Load custom rule repositories via `--rules-repo` (local path or HTTPS git URL)
- **Quarantine pattern** — Move files to archive folders with full restore capability
- **Historical diff** — Compare scan snapshots over time to track space growth
- **Agent-friendly output** — JSON and Markdown outputs for both humans and AI agents
- **PowerShell wrappers** — Ready-to-use agent skill scripts in `skills/`

Full notes: [`CHANGELOG.md`](./CHANGELOG.md) · [`docs/release-notes/v1.0.0.md`](./docs/release-notes/v1.0.0.md).

---

## Key Features

| Capability | What it does |
|-----------|-------------|
| **Intelligent Scanning** | Discover space usage across AI models (Ollama, Hugging Face), AI IDEs/CLIs, browsers, Docker, WSL, Playwright, installers, test artifacts, and dev artifacts |
| **Developer Artifact Coverage** | Detect common regenerable artifacts such as `node_modules`, Rust `target/`, Gradle caches, Python `__pycache__`, `dist/`, `.next`, and `.turbo` |
| **Cross-Platform Rule Paths** | Expand Unix `~/` home paths and keep Windows `%VAR%` expansion so Ollama, Hugging Face, and Docker rules work across Windows, Linux, and macOS path layouts |
| **Rule-Driven Classification** | Every path evaluated against YAML rules with risk levels: `safe`, `careful`, `dangerous`. No hardcoded paths. |
| **Dry-Run by Default** | All destructive operations preview changes before touching disk. Explicit `--yes` required for real action. |
| **Quarantine Pattern** | Move files to designated archive folder instead of deleting. Full restore with conflict detection. |
| **Specialized Diagnostics** | `doctor` command provides targeted analysis for AI agents, AI IDEs/CLIs, installers, test artifacts, Docker, WSL, Ollama, Playwright, and Hugging Face |
| **Registry-Driven Doctor Topics** | Built-in doctor topics keep their existing flags while sharing one internal registry for topic names, defaults, matching logic, recommendations, and optional probes |
| **Historical Diff** | Compare scan snapshots to answer "what grew?" and track cleanup effectiveness |
| **Growth Anomaly Detection** | Detect paths whose growth exceeds both absolute and relative thresholds for local scheduled governance |
| **Community Rules** | Load custom rule repositories via `--rules-repo` (local directory or HTTPS git URL) |
| **Agent-Friendly Output** | JSON and Markdown outputs designed for both human reading and AI agent consumption |
| **Operability Metadata** | Reports include the active policy snapshot and mark partial sizes as `best-effort, not exact` when depth limits or unreadable descendants prevent complete traversal |
| **Cross-Disk Safety** | Quarantine handles cross-drive moves with copy+delete fallback when rename fails |

---

## Installation

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
```

The governance script keeps the workflow read-only: it runs `scan`, reuses scan snapshots, and emits anomaly artifacts locally. On the first run, if history does not yet contain two snapshots, it writes a pending note instead of failing.

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
