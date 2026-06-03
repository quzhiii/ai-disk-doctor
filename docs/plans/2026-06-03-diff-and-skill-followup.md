# Diff And Skill Follow-Up Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Harden `aidisk diff` so scan-history comparisons stop misreporting non-existent placeholder paths, then expose the command cleanly through docs and the Windows skill workflow.

**Architecture:** Keep the fix inside `aidisk/src/diff.rs` by teaching snapshot comparison about the scanner's `exists` field instead of inferring existence from `size_bytes == 0`. Preserve the current CLI surface, then add lightweight example snapshots and a PowerShell wrapper so the skill can drive the new command the same way it already drives `scan`, `plan`, `clean`, `restore`, and `doctor`.

**Tech Stack:** Rust, Clap CLI, serde/serde_json, PowerShell wrapper scripts, Markdown docs.

---

### Task 1: Reproduce The Diff Bug With Tests

**Files:**
- Modify: `aidisk/src/diff.rs`
- Test: `aidisk/src/diff.rs`

**Step 1: Write the failing tests**

Add unit tests that write minimal scan JSON snapshots to a temp directory and assert:

```rust
#[test]
fn ignores_paths_missing_in_both_snapshots() {
    assert_eq!(report.summary.appeared, 0);
    assert!(report.changes.is_empty());
}

#[test]
fn preserves_zero_byte_existing_paths() {
    assert_eq!(report.summary.appeared, 1);
    assert_eq!(report.changes[0].change, "appeared");
}
```

The first test must encode the root-cause case from the smoke run: the same path appears in both scan files with `exists: false` and `size_bytes: 0`, and `build_diff` must not mark it as `appeared`.

The second test must prove why `exists` matters: a real zero-byte path with `exists: true` in the `after` snapshot still counts as `appeared`.

**Step 2: Run the targeted tests to verify they fail**

Run: `cargo test diff::tests -- --nocapture`

Expected: at least one failure showing the current implementation misclassifies the placeholder path.

### Task 2: Fix Diff Snapshot Semantics

**Files:**
- Modify: `aidisk/src/diff.rs`
- Test: `aidisk/src/diff.rs`

**Step 1: Write the minimal implementation**

Update the internal snapshot struct and comparison logic so diff decisions are based on path plus `exists` state:

```rust
#[derive(Debug, Deserialize)]
struct ScanFinding {
    path: String,
    exists: bool,
    size_bytes: u64,
}
```

Then compare the union of paths from `before` and `after` with these rules:

```rust
match (before_exists, after_exists) {
    (false, false) => {}
    (false, true) => appeared,
    (true, false) => disappeared,
    (true, true) => compare sizes for grew/shrunk,
}
```

Do not add new CLI flags or new report types. Keep the fix local to `diff.rs` unless tests force a smaller shared helper.

**Step 2: Run the targeted tests to verify they pass**

Run: `cargo test diff::tests -- --nocapture`

Expected: all new diff tests pass.

### Task 3: Document And Expose The Diff Workflow

**Files:**
- Modify: `README.md`
- Modify: `docs/execution-plan.md`
- Modify: `skills/windows-ai-space-manager/SKILL.md`
- Modify: `skills/windows-ai-space-manager/references/workflow.md`
- Create: `skills/windows-ai-space-manager/scripts/run-diff.ps1`
- Create: `examples/diff-before.example.json`
- Create: `examples/diff-after.example.json`

**Step 1: Add minimal docs and script coverage**

Update README quick-start and workflow sections with one concrete `aidisk diff` example that uses the new example JSON files. Mark P2 as complete in `docs/execution-plan.md` and move the next focus to Phase 5 / Phase 6 follow-up.

Add a PowerShell wrapper consistent with the existing scripts:

```powershell
param(
    [Parameter(Mandatory = $true)][string]$Before,
    [Parameter(Mandatory = $true)][string]$After,
    [switch]$Json,
    [switch]$Markdown
)
```

Update the skill workflow so `diff` is described as the follow-up step after two `scan` runs when the user asks what grew over time.

**Step 2: Run full verification**

Run: `cargo test`

Expected: all tests pass.

Run: `cargo run -- diff --before ..\examples\diff-before.example.json --after ..\examples\diff-after.example.json --markdown`

Expected: markdown output showing at least one `appeared`, one `grew`, and one `disappeared` example without false-positive placeholder paths.

**Step 3: Commit the stable milestone**

Run:

```bash
git add aidisk/src/diff.rs README.md docs/execution-plan.md skills/windows-ai-space-manager/SKILL.md skills/windows-ai-space-manager/references/workflow.md skills/windows-ai-space-manager/scripts/run-diff.ps1 examples/diff-before.example.json examples/diff-after.example.json docs/plans/2026-06-03-diff-and-skill-followup.md
git commit -m "harden diff comparison and document workflow"
```
