# AI Disk Doctor

![Version](https://img.shields.io/badge/version-1.0.0-blue)
![Rust](https://img.shields.io/badge/rust-1.78%2B-orange)
![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-green)
![Platform](https://img.shields.io/badge/platform-Windows-lightgrey)

[中文](./README.zh-CN.md) · [Changelog](./CHANGELOG.md) · [Contributing](./CONTRIBUTING.md)

> **AI-era disk space diagnostics and governance for Windows.**
>
> Identify, analyze, and safely reclaim storage consumed by AI tools, browsers, and development environments—without guessing what's safe to delete.

---

## Table of Contents

- [Overview](#overview)
- [What's New](#whats-new)
- [Key Features](#key-features)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Command Reference](#command-reference)
- [Safety First](#safety-first)
- [Architecture](#architecture)
- [Troubleshooting](#troubleshooting)
- [Contributing](#contributing)
- [License](#license)

---

## Overview

AI Disk Doctor is a rule-driven, safety-first disk space diagnostic tool built for the AI era. It discovers space hogs across AI model caches, browser data, Docker images, WSL distros, and development artifacts—then helps you clean up with confidence.

Unlike generic disk cleaners, AI Disk Doctor is **rule-driven**: every path is evaluated against YAML rules with explicit risk levels (`safe`, `careful`, `dangerous`). No hardcoded magic paths, no guessing. The default posture is **conservative**: scan and report first, dry-run second, quarantine third—never delete directly.

**Current release:** v1.0.0

For detailed architecture and design decisions, see [`docs/architecture.md`](./docs/architecture.md).

---

## What's New

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
| **Intelligent Scanning** | Discover space usage across AI models (Ollama, Hugging Face), browsers, Docker, WSL, Playwright, and dev artifacts |
| **Rule-Driven Classification** | Every path evaluated against YAML rules with risk levels: `safe`, `careful`, `dangerous`. No hardcoded paths. |
| **Dry-Run by Default** | All destructive operations preview changes before touching disk. Explicit `--yes` required for real action. |
| **Quarantine Pattern** | Move files to designated archive folder instead of deleting. Full restore with conflict detection. |
| **Specialized Diagnostics** | `doctor` command provides targeted analysis for Docker, WSL, Ollama, Playwright, and Hugging Face |
| **Historical Diff** | Compare scan snapshots to answer "what grew?" and track cleanup effectiveness |
| **Community Rules** | Load custom rule repositories via `--rules-repo` (local directory or HTTPS git URL) |
| **Agent-Friendly Output** | JSON and Markdown outputs designed for both human reading and AI agent consumption |
| **Cross-Disk Safety** | Quarantine handles cross-drive moves with copy+delete fallback when rename fails |

---

## Installation

### Prerequisites

| Requirement | Version |
|------------|---------|
| Windows | 10/11 |
| Rust | 1.78+ |

Install Rust via [rustup](https://rustup.rs/) if you don't have it.

### From Source

```bash
# Clone the repository
git clone https://github.com/quzhiii/ai-disk-doctor.git
cd ai-disk-doctor/aidisk

# Build release binary
cargo build --release

# The binary will be at target/release/aidisk.exe
```

### Development Setup

```bash
cd ai-disk-doctor/aidisk

# Build and test
cargo build
cargo test
```

### Verify Your Build

Run the non-destructive smoke test to verify everything works:

```powershell
pwsh -NoProfile -File "scripts/release-smoke.ps1"
```

---

## Quick Start

### 1. Scan Your System

```powershell
# Scan everything and output JSON
cargo run -- scan --json

# Generate Markdown report
cargo run -- scan --markdown

# Scan specific category
cargo run -- scan --category browser-cache --json
```

### 2. Generate a Cleanup Plan

```powershell
# Safe items only, dry-run
cargo run -- plan --safe-only --json

# Include careful items, skip recently modified
cargo run -- plan --json --skip-modified-within-minutes 30
```

### 3. Execute Safe Cleanup (Quarantine)

```powershell
# Preview quarantine plan
cargo run -- clean --dry-run --safe-only --quarantine-root "F:\archives"

# Execute quarantine (requires --yes)
cargo run -- clean --yes --safe-only --quarantine-root "F:\archives"
```

### 4. Restore if Needed

```powershell
# Preview restore
cargo run -- restore --dry-run --index "F:\archives\.aidisk\quarantine-index-YYYYMMDD-HHMMSS.json"

# Execute restore
cargo run -- restore --yes --index "F:\archives\.aidisk\quarantine-index-YYYYMMDD-HHMMSS.json"
```

### 5. Run Diagnostics

```powershell
# Full system diagnosis
cargo run -- doctor --markdown

# Specific topics
cargo run -- doctor --docker --json
cargo run -- doctor --wsl --ollama --markdown
cargo run -- doctor --playwright --huggingface --markdown
```

### 6. Compare Snapshots

```powershell
# Auto-compare last two scans
cargo run -- diff --latest --markdown

# Compare specific snapshots
cargo run -- diff --before scan-20260101-120000.json --after scan-20260102-120000.json --markdown
```

---

## Command Reference

| Command | Description | Key Flags |
|---------|-------------|-----------|
| `scan` | Discover and classify space usage | `--category`, `--rules-repo`, `--json`, `--markdown` |
| `plan` | Generate cleanup recommendations | `--safe-only`, `--skip-modified-within-minutes` |
| `clean` | Execute quarantine or dry-run | `--dry-run`, `--yes`, `--quarantine-root`, `--safe-only` |
| `restore` | Restore quarantined files | `--dry-run`, `--yes`, `--index` |
| `doctor` | Run targeted diagnostics | `--docker`, `--wsl`, `--ollama`, `--playwright`, `--huggingface` |
| `diff` | Compare scan snapshots | `--latest`, `--before`, `--after` |

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
       +-- Doctor (topic-specific analyzers)
       +-- Diff Engine (snapshot comparison)
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

## License

This project is dual-licensed under:

- **MIT License** — See [LICENSE-MIT](./LICENSE-MIT)
- **Apache License 2.0** — See [LICENSE-APACHE](./LICENSE-APACHE)

You may choose either license at your option.

---

**Made with ❤️ for cleaner disks and clearer minds.**
