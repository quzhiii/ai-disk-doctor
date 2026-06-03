# Changelog

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
