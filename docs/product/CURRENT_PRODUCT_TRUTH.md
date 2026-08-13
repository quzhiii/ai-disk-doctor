# Current Product Truth

Date: 2026-08-13
Reality audit baseline: M1A PR #14 after read-only Core application boundary implementation, based on `master` at `9dd102abbb2908894103a3c1f2fc3f7fa50fad87`

This document separates current repository facts from product decisions and future hypotheses. It is the first place to check before making capability claims.

## Claim Labels

- Current repo fact: verified in this repository during the M0 and M1A audits.
- External-source fact: verified from an external source; retain the source URL in the relevant doc.
- Product decision: accepted direction for future work, not necessarily implemented.
- Hypothesis / to validate: product or commercial belief requiring user evidence.

## Current Repo Facts

- Current repo fact: `aidisk/Cargo.toml` defines package `aidisk` at version `1.7.0` with description `Cross-platform AI disk space diagnostics and governance CLI`.
- Current repo fact: `aidisk/src/main.rs` is now a thin binary entrypoint; internal CLI command orchestration for `scan`, `plan`, `clean`, `restore`, `diff`, `anomaly`, `doctor`, `rules`, `models`, and `visualize` lives in `aidisk/src/cli.rs`.
- Current repo fact: `aidisk/src/lib.rs` exposes a public `aidisk::application` read-only Core application boundary for scan, AI asset inventory, and history metadata while keeping mutation modules internal.
- Current repo fact: the Rust Core includes scanner, planner, cleaner/quarantine/restore, doctor, model inventory, diff, anomaly, history, reporter, rules, rules repo, policy, HTML visualization, CLI orchestration, and read-only application-boundary modules.
- Current repo fact: built-in rule coverage lives in `aidisk/rules/` and includes AI agents, coding agents, IDEs, CLIs, caches, models, Hugging Face, Docker, WSL, Playwright, browser, dev-cache, and sensitive-sample rules.
- Current repo fact: `scan`, `plan`, `doctor`, `models inventory`, and `models adapters` are read-oriented by default; `clean` and `restore` require explicit execution flags for real mutation.
- Current repo fact: `cleaner.rs` implements quarantine root containment checks, rename plus copy/verify/remove fallback, versioned execution reports, journals, and conflict-safe restore.
- Current repo fact: `history.rs`, `diff.rs`, and `anomaly.rs` support local scan snapshots, latest-pair diffs, and threshold-based growth anomaly reports.
- Current repo fact: `.github/workflows/ci.yml` runs `cargo test` on Windows, Ubuntu, and macOS; the Windows job also runs `cargo run --quiet -- rules lint --json`.
- Current repo fact: `.github/workflows/release-artifacts.yml` defines six target release artifacts and packages binary, README, changelog, licenses, rules, config, report schema, checksum, SBOM, and provenance.
- Current repo fact: no Tauri, Electron, native GUI, billing, auth, cloud sync, account, SaaS, or telemetry dependency is present in the current repository.
- Current repo fact: no root `AGENTS.md` existed before M0.

## Reality Matrix

| Claim | Repo evidence | Status |
|---|---|---|
| v1.7.0 is the current release baseline | `README.md`, `README.zh-CN.md`, `CHANGELOG.md`, `aidisk/Cargo.toml` | true |
| Cross-platform CLI/Core is implemented | CI on Windows/Ubuntu/macOS and release artifact matrix | true |
| Scanner, planner, cleaner, restore, doctor, diff, anomaly, history, model inventory, reporter, visualize are implemented | `aidisk/src/*.rs` modules, internal CLI orchestration, and application boundary wiring | true |
| Quarantine/restore are recoverable and journaled | `aidisk/src/cleaner.rs` execution and restore reports | true |
| Model asset intelligence has a metadata-only foundation | `aidisk/src/model_inventory.rs`; `models inventory` and `models adapters` | true |
| Desktop product exists | no desktop crate, no Tauri dependency, no native GUI dependency | planned only |
| Recovery Intelligence exists as a full project/session recovery layer | partial metadata fields and quarantine/history exist; no project-level recovery coverage model | partial |
| Commercial Pro, billing, entitlement, or license-key code exists | no auth/billing/license-key modules or dependencies | planned only |
| Docs consistently use AI Disk Doctor rather than Windows AI Space naming | README and Cargo mostly current; release notes, old plans, and some runtime output strings retain old naming | stale/contradictory |
| License state is consistently dual MIT/Apache everywhere | root license files, README, and CONTRIBUTING say dual; Cargo package field says `MIT` | stale/contradictory |

## Architecture Gap Analysis

| Future concept | Current support | Gap |
|---|---|---|
| One execution truth for CLI and Desktop | CLI/Core owns execution today; `aidisk::application` now provides a read-only shared boundary for scan, AI asset inventory, and history | mutation-side shared services for clean/quarantine/restore remain deferred |
| Reclaim Confidence | model inventory has `reclaim_confidence`; planner uses risk/action/policy | scanner/planner cleanup candidates do not yet expose a unified reclaim confidence field |
| Recovery Value | quarantine restore metadata, history snapshots, model rollback metadata | no general recovery-value axis, project coverage model, or recovery impact calculation |
| Beginner-friendly explanations | README, doctor recommendations, dashboard labels | future Desktop requires plain-language explanation fields across all item types |
| Recovery coverage | quarantine index/journal, scan history, latest diff | no Git/Agent snapshot/session coverage map yet |
| Commercial Desktop boundary | release artifacts and CLI/Core exist | no approved repo/process boundary, entitlement model, or legal decision |

## M0 / M1A Reality Conclusions

- Product decision: keep the public Core centered on local-first CLI/Core capabilities and make future Desktop an adapter over shared Core logic.
- Product decision: use the M1A `aidisk::application` boundary for read-only scan, AI asset inventory, and history consumers; keep mutation-side shared boundaries deferred.
- Product decision: preserve all current safety invariants; do not add cleanup behavior in M0.
- Product decision: document licensing/commercial paths but do not change license files, contribution license language, or Cargo metadata in M0.
- Product decision: treat old roadmap/strategy/plan files as historical/superseded rather than deleting them.
- Hypothesis / to validate: users may pay for continuous workspace health, richer recovery coverage UX, and retention automation after Desktop proves value.
