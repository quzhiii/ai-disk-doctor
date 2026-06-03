# Skill Artifact Validation Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add automated validation for the Windows AI Space Manager skill artifacts so workflow docs, references, scripts, and examples stay aligned with the Rust CLI.

**Architecture:** Keep validation as Rust integration tests under `aidisk/tests/` so normal `cargo test` checks both CLI code and skill artifacts. The tests read repository files through paths relative to the `aidisk` crate, assert documented coverage, and leave runtime script parsing to a PowerShell verification command.

**Tech Stack:** Rust integration tests, Markdown artifact checks, PowerShell syntax checks, existing `cargo test` workflow.

---

### Task 1: Add Failing Skill Artifact Tests

**Files:**
- Create: `aidisk/tests/skill_artifacts.rs`

**Step 1: Write tests before changing docs**

Add tests that assert:

```rust
#[test]
fn skill_response_style_includes_all_next_step_commands() {
    assert!(skill.contains("scan / plan / clean / restore / doctor / diff"));
}

#[test]
fn risk_cheatsheet_covers_execution_and_restore_statuses() {
    for status in ["moved", "planned", "restored", "skipped-active", "skipped-conflict", "skipped-locked", "failed"] {
        assert!(risk.contains(status));
    }
}

#[test]
fn category_map_covers_rule_categories() {
    for category in categories_from_rule_yaml() {
        assert!(category_map.contains(&format!("`{category}`")));
    }
}
```

Also validate every `scripts/*.ps1` file is listed in `SKILL.md`, every listed script exists, and `run-diff.ps1` is referenced in `references/workflow.md`.

**Step 2: Run tests to verify they fail**

Run: `cargo test skill_artifacts -- --nocapture`

Expected: failures for missing `diff` in response style, incomplete risk statuses, and incomplete category coverage.

### Task 2: Fix Skill Artifacts Minimally

**Files:**
- Modify: `skills/windows-ai-space-manager/SKILL.md`
- Modify: `skills/windows-ai-space-manager/references/workflow.md`
- Modify: `skills/windows-ai-space-manager/references/risk-cheatsheet.md`
- Modify: `skills/windows-ai-space-manager/references/category-map.md`

**Step 1: Update only the missing coverage**

Add `diff` to response style next-step choices. Add trigger phrases for historical growth questions. Expand the risk cheatsheet with execution and restore statuses. Expand the category map to cover every category currently present in `aidisk/rules/*.yaml`.

**Step 2: Run artifact tests**

Run: `cargo test skill_artifacts -- --nocapture`

Expected: all artifact tests pass.

### Task 3: Verify Scripts And Commit

**Files:**
- Test: `skills/windows-ai-space-manager/scripts/*.ps1`

**Step 1: Parse every wrapper script with PowerShell**

Run: `pwsh -NoProfile -Command "$files = Get-ChildItem -LiteralPath '..\skills\windows-ai-space-manager\scripts' -Filter '*.ps1'; foreach ($file in $files) { $null = [System.Management.Automation.Language.Parser]::ParseFile($file.FullName, [ref]$null, [ref]$errors); if ($errors.Count) { throw \"Parse failed: $($file.Name)\" } }; \"parsed $($files.Count) scripts\""`

Expected: `parsed 7 scripts`.

**Step 2: Run full tests**

Run: `cargo test`

Expected: all tests pass.

**Step 3: Commit**

Run:

```bash
git add aidisk/tests/skill_artifacts.rs skills/windows-ai-space-manager/SKILL.md skills/windows-ai-space-manager/references/workflow.md skills/windows-ai-space-manager/references/risk-cheatsheet.md skills/windows-ai-space-manager/references/category-map.md docs/plans/2026-06-03-skill-artifact-validation.md
git commit -m "validate windows space manager skill artifacts"
```
