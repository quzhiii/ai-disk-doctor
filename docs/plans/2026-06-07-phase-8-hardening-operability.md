# Phase 8 Hardening And Operability Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Improve post-`v1.2.0` trust and operability by making active policy/scan limits visible in reports and by marking partial scan results instead of silently presenting truncated size data as complete.

**Architecture:** Keep the current rule-driven scanner, planner, and doctor flow intact. Add small, additive report metadata for policy visibility and partial-scan signaling rather than redesigning the command model. Reuse existing reporter test style and keep JSON changes additive so text/markdown and machine consumers stay aligned.

**Tech Stack:** Rust CLI, serde JSON/Markdown reporters, existing `policy.rs` config loader, `WalkDir`, current `scanner.rs` / `planner.rs` / `doctor.rs` / `reporter.rs` architecture.

---

### Task 1: Add Structured Policy Snapshot Metadata

**Files:**
- Modify: `aidisk/src/policy.rs`
- Modify: `aidisk/src/scanner.rs`
- Modify: `aidisk/src/planner.rs`
- Modify: `aidisk/src/doctor.rs`
- Modify: `aidisk/src/main.rs`
- Test: `aidisk/src/reporter.rs`

**Step 1: Write the failing reporter tests**

Add tests in `aidisk/src/reporter.rs` that construct scan/plan/doctor reports with a structured policy snapshot and assert:
- Text output shows current sensitive markers, allowed actions, skip-modified window, and max scan depth.
- Markdown output shows the same information.
- JSON serialization includes the new additive metadata fields.

**Step 2: Run the targeted tests to verify RED**

Run: `cargo test reporter::tests -- --nocapture`

Expected: FAIL because scan/plan reports do not yet expose structured policy metadata.

**Step 3: Add minimal shared policy snapshot types**

In `aidisk/src/policy.rs`, add a small serializable snapshot struct, for example:
- `PolicySnapshot`
- `PlannerPolicySnapshot`

Include only already-supported settings:
- `sensitive_markers`
- `allow_actions`
- `skip_modified_within_minutes`
- `max_scan_depth`

Add a helper like `Policy::snapshot()` so call sites do not reformat fields manually.

**Step 4: Thread the snapshot into reports**

Add additive fields to:
- `scanner::ScanReport`
- `planner::PlanReport`
- `doctor::DoctorReport`

For `doctor`, replace the existing free-form `policy_summary: String` with a structured snapshot plus, if needed, a small renderer helper for text/markdown.

**Step 5: Populate report metadata in `main.rs` / builders**

Do the smallest correct thing:
- attach `policy.snapshot()` to scan output before render/save
- attach the same snapshot to `PlanReport`
- use the snapshot in `build_doctor`

**Step 6: Re-run the targeted tests to verify GREEN**

Run: `cargo test reporter::tests -- --nocapture`

Expected: PASS.

### Task 2: Mark Partial Scan Results When Sizes Are Truncated

**Files:**
- Modify: `aidisk/src/scanner.rs`
- Modify: `aidisk/src/planner.rs`
- Modify: `aidisk/src/doctor.rs`
- Modify: `aidisk/src/reporter.rs`
- Test: `aidisk/src/scanner.rs`
- Test: `aidisk/src/reporter.rs`

**Step 1: Write failing scanner tests**

Add scanner unit tests for two cases:
- A directory tree deeper than `max_scan_depth` should mark the finding as partial.
- A traversal error / unreadable descendant should mark the finding as partial while still returning the best-effort size.

Keep tests local and deterministic with temp directories. If permission-denied is awkward on Windows, use a helper path or traversal shim already consistent with the codebase’s current test style.

**Step 2: Verify RED**

Run: `cargo test scanner::tests -- --nocapture`

Expected: FAIL because findings currently expose only `size_bytes` and silently truncate depth/errors.

**Step 3: Add additive partial metadata**

Add small additive fields such as:
- `Finding.partial: bool`
- `Finding.partial_reasons: Vec<String>` or one concise `partial_reason`
- `Summary.partial_findings: usize`

Keep the model minimal. Do not add a large diagnostics subsystem.

**Step 4: Change size computation to return best-effort metadata**

Refactor `compute_size` to return a small struct rather than bare `u64`, for example:
- `ComputedSize { size_bytes, partial, partial_reasons }`

Mark partial when:
- traversal hits a `WalkDir` error
- metadata for a descendant cannot be read
- a directory is encountered at the configured max depth boundary, meaning deeper children were intentionally not traversed

Continue to skip unreadable descendants rather than failing the whole scan.

**Step 5: Propagate partial metadata through planner/doctor/reporter**

Ensure:
- `plan` candidates/skipped output preserves partial status where relevant
- `doctor` surfaces partial findings as informational warnings, not hard failures
- text/markdown clearly label partial size results

**Step 6: Verify GREEN**

Run: `cargo test scanner::tests reporter::tests -- --nocapture`

Expected: PASS.

### Task 3: Make Text And Markdown Outputs Explicitly Explain Limits

**Files:**
- Modify: `aidisk/src/reporter.rs`
- Modify: `aidisk/src/doctor.rs`
- Test: `aidisk/src/reporter.rs`
- Test: `aidisk/src/doctor.rs`

**Step 1: Write failing rendering tests**

Add tests that assert:
- scan text/markdown summary includes current max depth and partial finding count
- plan text/markdown includes current policy snapshot and partial warnings for candidates/skipped items
- doctor text/markdown explains when active findings are partial rather than exact

**Step 2: Verify RED**

Run: `cargo test reporter::tests doctor::tests -- --nocapture`

Expected: FAIL.

**Step 3: Implement minimal rendering updates**

Keep rendering compact:
- add a short policy block near existing headers
- add a short line like `Partial Findings: N`
- annotate only affected rows/findings instead of flooding output

Do not degrade JSON completeness, and do not expand every missing path again.

**Step 4: Verify GREEN**

Run: `cargo test reporter::tests doctor::tests -- --nocapture`

Expected: PASS.

### Task 4: Update Roadmap And Public Docs

**Files:**
- Modify: `docs/execution-plan.md`
- Modify: `README.md`
- Modify: `README.zh-CN.md`
- Test: `aidisk/tests/readme_artifacts.rs`

**Step 1: Write the failing docs assertions if needed**

Only add tests if required to lock a new release-facing statement. Keep changes minimal.

**Step 2: Update roadmap text**

Add Phase 8 with the current recommended sequencing:
- P0: policy visibility in reports
- P0: partial scan signaling
- P1: operability / CI maintenance follow-through

**Step 3: Update README language only if user-facing behavior changed enough to mention**

Keep edits concise. Do not invent a new release section unless the work is actually shipped.

**Step 4: Verify**

Run: `cargo test --test readme_artifacts`

Expected: PASS.

### Task 5: Full Verification And Handoff

**Files:**
- Verify only

**Step 1: Run focused tests**

Run:
- `cargo test scanner::tests reporter::tests doctor::tests -- --nocapture`
- `cargo test --test release_artifacts`
- `cargo test --test readme_artifacts`

**Step 2: Run the full suite**

Run:
- `cargo test`

**Step 3: Inspect final diff**

Run:
- `git status --short`
- `git diff -- aidisk/src/policy.rs aidisk/src/scanner.rs aidisk/src/planner.rs aidisk/src/doctor.rs aidisk/src/reporter.rs aidisk/src/main.rs docs/execution-plan.md README.md README.zh-CN.md`

**Step 4: Prepare branch completion**

When implementation is done and verified, use `superpowers:finishing-a-development-branch` to decide whether to merge, PR, or keep the branch.
