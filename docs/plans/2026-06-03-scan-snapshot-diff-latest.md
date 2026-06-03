# Scan Snapshot Diff Latest Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make history comparison usable end-to-end by automatically saving scan snapshots and adding `aidisk diff --latest` to compare the newest two snapshots.

**Architecture:** Add a small `history` module responsible for report persistence and latest-pair discovery under `.aidisk/reports`. Keep scan output unchanged while writing a pretty JSON snapshot as a side effect. Extend the existing `diff` command with `--latest` and optional `--reports-dir`, reusing the existing `diff::build_diff` implementation.

**Tech Stack:** Rust, Clap CLI, serde_json, chrono timestamps, existing PowerShell skill wrapper scripts.

---

### Task 1: Scan Snapshot Persistence

**Files:**
- Create: `aidisk/src/history.rs`
- Modify: `aidisk/src/main.rs`
- Test: `aidisk/src/history.rs`

**Step 1: Write failing tests**

Add tests for `history::save_scan_snapshot(report, reports_dir)` that assert:

```rust
let path = save_scan_snapshot(&report, temp.path()).expect("snapshot should save");
assert!(path.file_name().unwrap().to_string_lossy().starts_with("scan-"));
assert_eq!(path.extension().unwrap(), "json");
let parsed: serde_json::Value = serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap();
assert!(parsed.get("findings").is_some());
```

Also add a test that saving twice creates two distinct paths.

**Step 2: Run targeted tests to verify RED**

Run: `cargo test history::tests -- --nocapture`

Expected: compile failure or test failure because `history` does not exist yet.

**Step 3: Implement minimal history module**

Create:

```rust
pub fn default_reports_dir() -> PathBuf
pub fn save_scan_snapshot(report: &ScanReport, reports_dir: &Path) -> Result<PathBuf>
```

Use `Local::now().format("%Y%m%d-%H%M%S-%3f")` for filenames, create the directory, serialize with `serde_json::to_string_pretty`, and write `scan-<timestamp>.json`.

**Step 4: Wire scan command**

In `Command::Scan`, after `scanner::scan(...)`, call `history::save_scan_snapshot(&report, &history::default_reports_dir())?;` before printing output.

**Step 5: Verify GREEN**

Run: `cargo test history::tests -- --nocapture`

Expected: history tests pass.

### Task 2: Diff Latest Pair Discovery

**Files:**
- Modify: `aidisk/src/history.rs`
- Modify: `aidisk/src/main.rs`
- Test: `aidisk/src/history.rs`

**Step 1: Write failing tests**

Add tests for `history::latest_scan_pair(reports_dir)` that create three `scan-*.json` files and assert the newest two sorted by filename are returned as `(before, after)`.

Also assert an error is returned when fewer than two snapshots exist.

**Step 2: Run targeted tests to verify RED**

Run: `cargo test history::tests::latest -- --nocapture`

Expected: failure because latest-pair discovery is not implemented.

**Step 3: Implement latest-pair discovery**

Create:

```rust
pub fn latest_scan_pair(reports_dir: &Path) -> Result<(PathBuf, PathBuf)>
```

Read files matching `scan-*.json`, sort paths by filename ascending, require at least two, and return the last two as `(before, after)`.

**Step 4: Extend CLI**

Update `Command::Diff` to accept:

```rust
#[arg(long)] latest: bool,
#[arg(long)] reports_dir: Option<PathBuf>,
#[arg(long)] before: Option<PathBuf>,
#[arg(long)] after: Option<PathBuf>,
```

If `latest` is true, resolve `(before, after)` from `reports_dir.unwrap_or_else(history::default_reports_dir)`. If `latest` is false, require both `--before` and `--after`.

**Step 5: Verify GREEN**

Run: `cargo test history::tests -- --nocapture`

Expected: all history tests pass.

### Task 3: Documentation, Skill, And Smoke Verification

**Files:**
- Modify: `README.md`
- Modify: `docs/execution-plan.md`
- Modify: `skills/windows-ai-space-manager/SKILL.md`
- Modify: `skills/windows-ai-space-manager/scripts/run-diff.ps1`
- Modify: `aidisk/tests/skill_artifacts.rs`

**Step 1: Update wrapper and docs**

Add `-Latest` and optional `-ReportsDir` to `run-diff.ps1`, while preserving explicit `-Before` / `-After` support.

Update README and skill workflow to prefer:

```powershell
cargo run -- scan --json
cargo run -- scan --json
cargo run -- diff --latest --markdown
```

Keep the explicit before/after example as the deterministic fixture example.

**Step 2: Update artifact tests**

Extend `skill_artifacts.rs` to assert `run-diff.ps1` includes `Latest` and `ReportsDir`, and that `SKILL.md` mentions `diff --latest`.

**Step 3: Run full verification**

Run: `cargo test`

Expected: all tests pass.

Run: `cargo run -- scan --json`

Expected: scan output still prints JSON and `.aidisk/reports/scan-*.json` is created.

Run: `cargo run -- diff --latest --markdown`

Expected: markdown diff output comparing the two newest snapshots, or a clear error if fewer than two snapshots exist.

Run: `pwsh -NoProfile -File "..\skills\windows-ai-space-manager\scripts\run-diff.ps1" -Latest -Markdown`

Expected: wrapper successfully calls `diff --latest`.

**Step 4: Commit**

Run:

```bash
git add aidisk/src/history.rs aidisk/src/main.rs aidisk/tests/skill_artifacts.rs README.md docs/execution-plan.md skills/windows-ai-space-manager/SKILL.md skills/windows-ai-space-manager/scripts/run-diff.ps1 docs/plans/2026-06-03-scan-snapshot-diff-latest.md
git commit -m "add scan snapshots and diff latest workflow"
```
