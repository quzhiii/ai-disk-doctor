# Doctor V2 Roadmap And Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Turn `aidisk doctor` from a scan topic summary into a diagnostic workflow that explains why large AI-related directories are large, what is inside them, and which actions are safe versus review-only.

**Architecture:** Keep `scan` as the broad rule-based inventory and make `doctor` the deeper analysis layer. Doctor V2 should reuse scan findings for discovery, then add bounded subdirectory breakdowns, data-driven recommendations, and optional external tool probes without changing cleanup semantics.

**Tech Stack:** Rust CLI, existing rule scanner and policy loader, bounded WalkDir traversal, serde JSON/Markdown reporting, optional Windows command probes for Docker/WSL/Ollama.

---

## Product Diagnosis

The current `doctor` is useful as a safe summary, but it mostly repeats `scan` with fixed recommendations. The latest local run showed why this is limiting: Docker, WSL, Ollama, HuggingFace, and Playwright were small or absent, while the real space consumers were AI agent directories such as `.gemini`, `.claude`, `opencode`, and `.codex`.

Doctor V2 should therefore optimize for incremental value over `scan`:

- Identify the largest AI workspace/tool roots from scan results, including agent roots, AI IDE/CLI state, installed app roots, runtime caches, installers, and test/evaluation artifacts.
- Drill into those roots with a bounded child-directory size breakdown.
- Explain whether each large child looks like cache, session history, logs, model artifacts, browser/runtime downloads, or unknown data.
- Generate recommendations from observed data, not from static topic text.
- Keep destructive cleanup out of doctor; doctor can recommend `plan`, `clean --dry-run`, official tool commands, or manual review.

## Roadmap

| Priority | Track | Outcome | Why Now |
|---|---|---|---|
| P0 | AI Agent Doctor Topic | Add `doctor --agents` and include it in default doctor runs | The biggest observed consumers are agent directories, not the current specialty topics |
| P0 | AI Tooling Coverage | Cover newly installed AI agents, IDEs, CLIs, installed app roots, runtime caches, installers, and generated test/evaluation artifacts | AI storage bloat is moving faster than Docker/WSL/model-only rules |
| P0 | Bounded Subdirectory Breakdown | For existing large findings, show top child directories/files by size | This is the core diagnostic gap: users need to know what is inside `.gemini` or `.claude` |
| P1 | Data-Driven Recommendations | Tailor advice based on `exists`, size, risk, action, and top children | Avoid unhelpful advice such as recommending cache cleanup for a 1-byte placeholder |
| P1 | Tool Presence Detection | Detect whether Docker, WSL, Ollama, and Playwright are installed or active | Missing tools should be reported as skipped/not detected, not as ambiguous empty findings |
| P2 | Optional External Probes | Add opt-in probes such as `docker system df`, `wsl --list`, and `ollama list` | External command data is valuable but should not block the local filesystem-first MVP |
| P2 | Growth-Aware Doctor | Use `.aidisk/reports` and `diff --latest` to highlight fast-growing findings | Doctor should answer both "what is large" and "what is growing" |
| P3 | Dynamic Topic Registry | Generate doctor topics from rule categories plus topic metadata | Community rules should not require hardcoded doctor switches for every category |

## Recommended Sequencing

Start with a filesystem-first Doctor V2. Do not begin with external commands; they add platform variance, command availability problems, and parsing complexity before the product has solved the obvious gap.

The first implementation slice should be:

1. Add agent rules/categories if the default rules do not already cover `.gemini`, `.claude`, `.codex`, and `opencode` consistently.
2. Add `doctor --agents`, default it on when no specific doctor flags are passed, and group findings from AI agent categories/known ids.
3. Add a reusable bounded breakdown helper that returns the top N direct children under a finding path.
4. Render the breakdown in JSON and Markdown/Text doctor output.
5. Replace static-only recommendations with data-aware recommendations for empty, tiny, moderate, and large findings.

## Non-Goals For The First Slice

- Do not execute cleanup from doctor.
- Do not delete or mutate agent directories.
- Do not parse proprietary internal formats for Gemini, Claude, Codex, or opencode.
- Do not introduce a database for doctor output.
- Do not call Docker/WSL/Ollama commands until the filesystem-first doctor is useful.

## Acceptance Criteria

Doctor V2 is acceptable when these are true:

- `aidisk doctor --agents --markdown` identifies existing `.gemini`, `.claude`, `.codex`, and `opencode` roots when rules match them.
- Each existing agent root above a configured threshold includes a top-child breakdown, sorted by size descending.
- Missing tools or missing paths produce clear `not-detected` / `no-rules` status and are summarized instead of expanded in Markdown/Text output.
- Tiny caches produce `no action needed` style recommendations instead of generic cleanup advice.
- Large review-only directories explain that doctor is informational and should be followed by `plan`, `diff --latest`, official tool commands, or manual review.
- JSON output remains structured enough for the skill wrapper or agent to extract topic name, status, total size, top children, all missing findings, and recommendation text.
- Unit tests cover empty topic, missing path, tiny existing path, large agent root, and child sorting.
- Existing `scan`, `plan --safe-only`, `doctor --docker`, `doctor --wsl`, `doctor --ollama`, `doctor --playwright`, and `doctor --huggingface` behavior remains compatible unless intentionally changed in a documented CLI migration.

## Implementation Plan

### Task 1: Agent Topic Discovery

**Files:**
- Modify: `aidisk/src/doctor.rs`
- Modify: `aidisk/src/main.rs`
- Check: `aidisk/rules/*.yaml`
- Test: `aidisk/src/doctor.rs`

**Step 1: Write failing tests**

Add a doctor unit test with findings for `.gemini`, `.claude`, `.codex`, and `opencode`. Assert `build_doctor` returns an `agents` topic when the new option is enabled and summarizes existing total bytes.

**Step 2: Add CLI option**

Add `--agents` to `Command::Doctor` and `DoctorOptions`. Include it in the default no-flags doctor set.

**Step 3: Implement minimal topic grouping**

Group findings by known agent ids/categories first. If rules need adjustment, prefer small rule metadata changes over broad path guessing in doctor.

**Step 4: Verify**

Run: `cargo test doctor::tests -- --nocapture`

Expected: new and existing doctor tests pass.

### Task 2: Bounded Breakdown Helper

**Files:**
- Modify: `aidisk/src/doctor.rs`
- Test: `aidisk/src/doctor.rs`

**Step 1: Write failing tests**

Use a temp directory with several child folders/files. Assert the helper returns only top N direct children sorted by size descending and ignores unreadable traversal errors.

**Step 2: Add report structs**

Add a small structured field, for example `breakdown: Vec<DoctorBreakdownItem>`, to `DoctorFinding` or topic-level details.

**Step 3: Implement bounded traversal**

Limit depth and item count. Reuse the same conservative approach as scan: no symlink following, saturating size addition, skip entries that cannot be read.

**Step 4: Verify**

Run: `cargo test doctor::tests -- --nocapture`

Expected: breakdown tests pass without changing scan tests.

### Task 3: Data-Driven Recommendations

**Files:**
- Modify: `aidisk/src/doctor.rs`
- Test: `aidisk/src/doctor.rs`

**Step 1: Write failing tests**

Add cases for missing path, tiny existing path, large existing path, and large agent root with cache-like child names.

**Step 2: Replace generic recommendation enrichment**

Make recommendations depend on observed `existing_count`, `total_bytes`, finding risk/action, and breakdown names. Keep topic-specific official-command advice where it is useful.

**Step 3: Verify**

Run: `cargo test doctor::tests -- --nocapture`

Expected: recommendation tests pass and old topic tests still pass.

### Task 4: Reporting And Docs

**Files:**
- Modify: `aidisk/src/reporter.rs`
- Modify: `README.md`
- Modify: `docs/execution-plan.md`
- Modify: `skills/windows-ai-space-manager/SKILL.md`
- Modify: `skills/windows-ai-space-manager/scripts/run-doctor.ps1`
- Test: `aidisk/tests/skill_artifacts.rs`

**Step 1: Update rendering**

Ensure text/Markdown doctor output shows topic summary, finding size, and top child breakdown without overwhelming the user.

**Step 2: Update wrapper and workflow**

Expose `-Agents` in the PowerShell wrapper and document that default doctor includes agents.

**Step 3: Verify**

Run: `cargo test`

Expected: all tests pass.

Run: `cargo run -- doctor --agents --markdown`

Expected: agent topic renders successfully with breakdown where matching paths exist.

### Task 5: Topic Status And Missing-Path Compression

**Files:**
- Modify: `aidisk/src/doctor.rs`
- Modify: `aidisk/src/reporter.rs`
- Test: `aidisk/src/doctor.rs`
- Test: `aidisk/src/reporter.rs`

**Step 1: Write failing tests**

Add tests that assert doctor topics expose `status` as `active`, `not-detected`, or `no-rules`, and that Markdown/Text output summarizes missing findings as `Not detected: N` instead of expanding every placeholder path.

**Step 2: Implement status and compression**

Derive status from scan findings: no matching rules -> `no-rules`; matching rules but no existing paths -> `not-detected`; at least one existing path -> `active`. Keep JSON complete, but make Text/Markdown focus on existing findings plus missing count.

**Step 3: Verify**

Run: `cargo test doctor::tests reporter::tests -- --nocapture`

Expected: status and output compression tests pass.

## Future Tracks

Tool probes should come after the first Doctor V2 slice. Add them behind explicit flags such as `--probe-tools` so default doctor stays fast, deterministic, and safe.

Growth-aware doctor can reuse `history::latest_scan_pair` and `diff::build_diff`, but it should be introduced as a separate task after the breakdown output is stable. The most useful UX is likely `doctor --agents --latest`, showing both top current consumers and recent growth.
