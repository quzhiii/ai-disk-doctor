# AI Disk Doctor

![Version](https://img.shields.io/badge/version-1.0.0-blue)
![Rust](https://img.shields.io/badge/rust-1.78%2B-orange)
![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-green)
![Platform](https://img.shields.io/badge/platform-Windows-lightgrey)

[中文](./README.zh-CN.md)

> **AI-era disk space diagnostics and governance for Windows.**
>
> Identify, analyze, and safely reclaim storage consumed by AI tools, browsers, and development environments—without guessing what's safe to delete.

---

## Table of Contents

- [Features](#features)
- [Architecture](#architecture)
- [Tech Stack](#tech-stack)
- [Quick Start](#quick-start)
- [Command Reference](#command-reference)
- [Safety First](#safety-first)
- [Screenshots](#screenshots)
- [Roadmap](#roadmap)
- [Contributing](#contributing)
- [License](#license)

---

## Features

- **Intelligent Scanning** — Discover space hogs across AI models (Ollama, Hugging Face), browsers, Docker, WSL, Playwright, and general development artifacts
- **Rule-Driven Classification** — Every path is evaluated against YAML rules with risk levels: `safe`, `careful`, `dangerous`. No hardcoded magic paths.
- **Dry-Run by Default** — All destructive operations preview changes before touching your disk. No surprises.
- **Quarantine Pattern** — Move files to a designated archive folder instead of deleting. Full restore capability with conflict detection.
- **Specialized Diagnostics** — `doctor` command provides targeted analysis for Docker, WSL, Ollama, Playwright, and Hugging Face with actionable recommendations.
- **Historical Diff** — Compare scan snapshots over time to answer "what grew?" and track cleanup effectiveness.
- **Community Rules** — Load custom rule repositories via `--rules-repo` (local path or HTTPS git URL).
- **Agent-Friendly Output** — JSON and Markdown outputs designed for both human reading and AI agent consumption.

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

### Design Principles

1. **Conservative by Default** — Unknown paths are reported, never touched.
2. **Safety-First Execution** — All mutation commands default to dry-run. Explicit `--yes` required for real action.
3. **Quarantine Over Delete** — Files are moved to a user-specified archive, not deleted. Restore available via index.
4. **Rule-Driven Everything** — Path recognition, risk classification, and policy enforcement come from external YAML rules, not baked-in code.
5. **Agent-Ready Interface** — Structured output formats enable integration with AI agents and automation workflows.

---

## Tech Stack

| Component | Technology |
|-----------|-----------|
| CLI Framework | Rust + `clap` v4 |
| Configuration | YAML (`serde_yaml`) |
| File System | `walkdir` + `sysinfo` |
| Output Formats | JSON (`serde_json`) + Markdown |
| Agent Integration | PowerShell wrapper scripts |
| Testing | Built-in Rust test framework |

---

## Quick Start

### Prerequisites

- Windows 10/11
- Rust 1.78+ ([install via rustup](https://rustup.rs/))

### Installation

```bash
# Clone the repository
git clone https://github.com/quzhiii/ai-disk-doctor.git
cd ai-disk-doctor/aidisk

# Build release binary
cargo build --release

# Run from target directory
./target/release/aidisk.exe --help
```

### Your First Scan

```powershell
# Scan everything and output JSON
cargo run -- scan --json

# Scan specific category
cargo run -- scan --category browser-cache --json

# Generate Markdown report
cargo run -- scan --markdown
```

### Generate a Cleanup Plan

```powershell
# Safe items only, dry-run
cargo run -- plan --safe-only --json

# Include careful items, skip recently modified
cargo run -- plan --json --skip-modified-within-minutes 30
```

### Execute Safe Cleanup

```powershell
# Preview quarantine plan
cargo run -- clean --dry-run --safe-only --quarantine-root "F:\archives"

# Execute quarantine (requires --yes)
cargo run -- clean --yes --safe-only --quarantine-root "F:\archives"
```

### Restore from Quarantine

```powershell
# Preview restore
cargo run -- restore --dry-run --index "F:\archives\.aidisk\quarantine-index-YYYYMMDD-HHMMSS.json"

# Execute restore
cargo run -- restore --yes --index "F:\archives\.aidisk\quarantine-index-YYYYMMDD-HHMMSS.json"
```

### Run Diagnostics

```powershell
# Full system diagnosis
cargo run -- doctor --markdown

# Specific topics
cargo run -- doctor --docker --json
cargo run -- doctor --wsl --ollama --markdown
cargo run -- doctor --playwright --huggingface --markdown
```

### Compare Snapshots

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

## Screenshots

*Screenshots will be added post-release. Run the commands above to see output samples.*

---

## Roadmap

### v1.0 ✅ Current
- Core scan/plan/clean/restore/doctor/diff commands
- Rule-driven classification
- Quarantine pattern with restore
- Community rules repository support
- PowerShell agent wrappers

### v1.1 (Planned)
- [ ] Real-time monitoring (if community demand exists)
- [ ] Scheduled cleanup jobs
- [ ] GUI companion app
- [ ] Additional platform support (macOS, Linux)

See [CHANGELOG.md](./CHANGELOG.md) for detailed version history.

---

## Contributing

We welcome contributions! Please see our [Contributing Guide](./CONTRIBUTING.md) (coming soon) for details on:

- Reporting issues
- Suggesting new rules
- Submitting pull requests
- Adding new diagnostic topics

---

## License

This project is dual-licensed under:

- **MIT License** — See [LICENSE-MIT](./LICENSE-MIT)
- **Apache License 2.0** — See [LICENSE-APACHE](./LICENSE-APACHE)

You may choose either license at your option.

---

## Acknowledgments

Built with Rust and designed for the AI era. Special thanks to the Rust community for excellent crates like `clap`, `walkdir`, `serde`, and `sysinfo`.

---

**Made with ❤️ for cleaner disks and clearer minds.**
