# v1.2.0 Release Prep Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Prepare the repository for the `v1.2.0` release by promoting current unreleased Phase 7 work into formal release artifacts and syncing versioned documentation.

**Architecture:** Keep the existing release workflow intact. Update release metadata in one pass: crate version, changelog, README/README.zh-CN, release notes, smoke script, and artifact tests. Avoid product behavior changes unless they are required to keep the release contract accurate.

**Tech Stack:** Rust, Cargo, Markdown docs, PowerShell release smoke script, GitHub Actions release artifacts workflow.

---

### Task 1: Add the release plan and choose release target

**Files:**
- Create: `docs/plans/2026-06-07-v1-2-0-release-prep.md`

**Step 1: Record the target release**

Target release: `v1.2.0`

Reasoning:
- Current shipped release is `v1.1.0`
- Phase 7 adds new user-facing capability (`scan --large-files`)
- Rule coverage and cross-platform path handling expand supported behavior without breaking existing commands

**Step 2: Keep release scope minimal**

Include only:
- structured JSON errors
- dev artifact coverage
- large files discovery
- cross-platform rule path support

Do not add new product features during release prep.

### Task 2: Promote unreleased work into the v1.2.0 changelog and notes

**Files:**
- Modify: `CHANGELOG.md`
- Create: `docs/release-notes/v1.2.0.md`

**Step 1: Write release notes structure**

Sections:
- `# Windows AI Space Manager v1.2.0`
- `## Summary`
- `## Included Workflows`
- `## Safety Boundaries`
- `## Test Plan`
- `## Known Limits`

**Step 2: Promote changelog entries**

Move current `## Unreleased` items into a new `## 1.2.0` section and leave `## Unreleased` empty or ready for future work.

**Step 3: Capture release-specific workflows**

Required topics:
- `scan --large-files --min-size 500MB`
- dev artifact rule coverage (`node_modules`, `target/`, `.gradle`, `__pycache__`, `dist/`, `.next`, `.turbo`)
- structured JSON error contract
- cross-platform `~/` and `%VAR%` rule path handling for Ollama, Hugging Face, and Docker

### Task 3: Sync public version references

**Files:**
- Modify: `aidisk/Cargo.toml`
- Modify: `README.md`
- Modify: `README.zh-CN.md`
- Modify: `docs/execution-plan.md`

**Step 1: Bump crate version**

Change:
- `aidisk/Cargo.toml`: `version = "1.2.0"`

**Step 2: Update README release metadata**

Update:
- badge version
- current release line
- add `### v1.2.0` section above `v1.1.0`
- point “Full notes” / “完整说明” to `docs/release-notes/v1.2.0.md`

**Step 3: Keep prior release history intact**

Do not remove `v1.1.0` or `v1.0.0` sections.

**Step 4: Refresh release readiness note**

Update `docs/execution-plan.md` so release readiness references `v1.2.0` artifacts instead of only `v1.1.0`.

### Task 4: Update release validation artifacts

**Files:**
- Modify: `scripts/release-smoke.ps1`
- Modify: `aidisk/tests/release_artifacts.rs`

**Step 1: Extend release smoke coverage**

Add non-destructive checks for the new release surface:
- `cargo run -- scan --large-files --min-size 500MB --json`

Optionally keep existing commands unchanged and append the new one.

**Step 2: Update release artifact tests**

Add/adjust assertions for:
- `docs/release-notes/v1.2.0.md`
- `version = "1.2.0"`
- README references to `v1.2.0`
- changelog/release notes terms for Phase 7 scope

### Task 5: Refresh Cargo lockfile via verification

**Files:**
- Modify indirectly: `aidisk/Cargo.lock`

**Step 1: Regenerate lock metadata naturally**

Run:
- `cargo test`

Expected:
- lockfile records root crate version `1.2.0`

### Task 6: Verify the release prep end-to-end

**Files:**
- Verify only

**Step 1: Run release-focused tests**

Run:
- `cargo test --test release_artifacts`
- `cargo test --test readme_artifacts`

**Step 2: Run full suite**

Run:
- `cargo test`

**Step 3: Run smoke script**

Run from repo root:
- `pwsh -File .\scripts\release-smoke.ps1`

**Step 4: Inspect final diff**

Run:
- `git status --short`
- `git diff -- CHANGELOG.md README.md README.zh-CN.md docs/release-notes/v1.2.0.md docs/execution-plan.md aidisk/Cargo.toml aidisk/Cargo.lock scripts/release-smoke.ps1 aidisk/tests/release_artifacts.rs`

### Task 7: Prepare release handoff

**Files:**
- No file changes required unless gaps are found

**Step 1: Summarize what is ready**

Include:
- version target
- verified commands and outcomes
- files changed
- next publish actions: commit, tag `v1.2.0`, push tag, watch release artifact workflow
