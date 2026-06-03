# V1 Release Readiness Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Prepare the project for a v1.0-ready local release by adding release documentation, a non-destructive demo smoke script, and automated artifact checks.

**Architecture:** Keep release readiness as repository artifacts, not new product logic. Add integration tests that read the repository files and verify the release package documents the current CLI surface, safety boundaries, and reproducible smoke verification.

**Tech Stack:** Rust integration tests, Markdown release docs, PowerShell smoke script, existing `cargo test` workflow.

---

### Task 1: Release Artifact Tests

**Files:**
- Create: `aidisk/tests/release_artifacts.rs`

**Step 1: Write failing tests**

Add tests that assert:

```rust
assert!(read("CHANGELOG.md").contains("## 1.0.0"));
assert!(read("docs/release-notes/v1.0.0.md").contains("## Test Plan"));
assert!(read("scripts/release-smoke.ps1").contains("cargo test"));
assert!(!read("scripts/release-smoke.ps1").contains("--yes"));
assert!(read("aidisk/Cargo.toml").contains("version = \"1.0.0\""));
```

Also assert README contains `scripts/release-smoke.ps1`, `CHANGELOG.md`, and `docs/release-notes/v1.0.0.md`.

**Step 2: Run RED**

Run: `cargo test --test release_artifacts -- --nocapture`

Expected: failure because the release artifacts are missing and the crate version is still pre-release.

### Task 2: Add Release Artifacts

**Files:**
- Create: `CHANGELOG.md`
- Create: `docs/release-notes/v1.0.0.md`
- Create: `scripts/release-smoke.ps1`
- Modify: `README.md`
- Modify: `aidisk/Cargo.toml`
- Modify: `docs/execution-plan.md`

**Step 1: Add release docs**

Document v1.0.0 scope:

- scan / plan / clean dry-run / quarantine / restore
- doctor topics
- diff latest and scan snapshots
- local and HTTPS rules repo support
- safety boundaries and known limitations

**Step 2: Add smoke script**

Create a non-destructive PowerShell script that runs:

```powershell
cargo test
cargo run -- scan --rules-repo "tests/fixtures/community-rules" --json
cargo run -- plan --safe-only --json
cargo run -- clean --dry-run --safe-only --markdown
cargo run -- doctor --markdown
cargo run -- diff --before "..\examples\diff-before.example.json" --after "..\examples\diff-after.example.json" --markdown
```

The script must not include `--yes` or real cleanup execution.

**Step 3: Verify GREEN**

Run: `cargo test --test release_artifacts -- --nocapture`

Expected: release artifact tests pass.

### Task 3: Final Verification And Commit

**Files:**
- All release artifacts above

**Step 1: Run full verification**

Run:

```bash
cargo test
pwsh -NoProfile -File "..\scripts\release-smoke.ps1"
```

Expected: all tests and non-destructive smoke commands pass.

**Step 2: Commit**

Run:

```bash
git add CHANGELOG.md docs/release-notes/v1.0.0.md scripts/release-smoke.ps1 README.md aidisk/Cargo.toml docs/execution-plan.md aidisk/tests/release_artifacts.rs docs/plans/2026-06-03-v1-release-readiness.md
git commit -m "prepare v1 release readiness artifacts"
```
