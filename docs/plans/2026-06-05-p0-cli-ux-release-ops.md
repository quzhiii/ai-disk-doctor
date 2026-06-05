# P0 CLI UX And Release Ops Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Ship the next smallest release-ops and CLI-UX slice: optimized release builds, minimal GitHub Actions coverage, interactive scan progress, and clearer text/markdown summaries for `scan` and `doctor` without changing JSON schema.

**Architecture:** Keep data structures stable and put the UX changes at the CLI/rendering boundary. `scanner` gains an optional progress callback that is only wired by `main` for interactive, non-JSON runs; `reporter` gains text/markdown executive summary and bar rendering helpers while JSON continues to serialize the existing structs unchanged.

**Tech Stack:** Rust 2021, Cargo, `clap`, `serde`, `serde_json`, `walkdir`, `indicatif`, `console`, `tempfile`, GitHub Actions.

---

## Guardrails

- Do not implement MCP server code.
- Do not add Windows-specific low-level scanning APIs.
- Do not expose public `--topic` CLI.
- Do not externalize topic metadata.
- Do not add TUI, duplicate detection, or idempotency token work.
- Preserve JSON schema for `scan` and `doctor`; new summary/bar UX applies only to text/markdown output.
- Progress indicators must be disabled for `--json`, non-TTY, CI, and tests.
- Use strict TDD for production code: write failing tests first, verify RED, implement the smallest GREEN change, then re-run verification.
- Commit only after a fresh verification command has passed for the checkpoint.

## Batch Strategy

Batch A covers release ergonomics and CI because it is configuration-heavy, low coupling, and independently verifiable. Batch B covers interactive and rendering UX because it touches runtime behavior and should be tested separately to protect JSON consumers.

## Milestones

### Milestone 0: Plan And Baseline

**Files:**
- Create: `docs/plans/2026-06-05-p0-cli-ux-release-ops.md`

**Verification command:**

```powershell
cd aidisk
cargo build
cargo test
```

**Expected result:** Build succeeds and the current test suite passes before implementation starts.

**Commit point:** Commit the plan after baseline verification if the working tree only contains this plan.

### Milestone A: Release Ops

**Tasks:** 1-2

**Verification command:**

```powershell
cd aidisk
cargo test release_artifacts
cargo build --release
cargo test
```

**Expected result:** Release profile tests pass, release binary builds, and the full Rust test suite passes.

**Commit point:** Commit after Tasks 1-2 pass.

### Milestone B: CLI UX

**Tasks:** 3-5

**Verification command:**

```powershell
cd aidisk
cargo test reporter::tests
cargo test scanner::tests
cargo test doctor_cli
cargo test
```

**Expected result:** Reporter, scanner, CLI integration, and full suite pass; JSON output tests still parse and do not require new fields.

**Commit point:** Commit after Tasks 3-5 pass.

---

## Task 1: Add Release Profile

**Files:**
- Modify: `aidisk/Cargo.toml`
- Modify: `aidisk/tests/release_artifacts.rs`

**Step 1: Write the failing test**

Add this test to `aidisk/tests/release_artifacts.rs`:

```rust
#[test]
fn cargo_toml_defines_release_profile_for_distributable_binary() {
    let cargo_toml = read_repo_file("aidisk/Cargo.toml");

    assert!(cargo_toml.contains("[profile.release]"));
    assert!(cargo_toml.contains("lto = \"thin\""));
    assert!(cargo_toml.contains("strip = \"symbols\""));
    assert!(cargo_toml.contains("codegen-units = 1"));
    assert!(cargo_toml.contains("opt-level = \"z\""));
}
```

**Step 2: Run test to verify it fails**

Run:

```powershell
cd aidisk
cargo test cargo_toml_defines_release_profile_for_distributable_binary
```

Expected: FAIL because `[profile.release]` does not exist yet.

**Step 3: Write minimal implementation**

Append this to `aidisk/Cargo.toml`:

```toml
[profile.release]
lto = "thin"
strip = "symbols"
codegen-units = 1
opt-level = "z"
```

Rationale to preserve in the implementation summary:

- `lto = "thin"`: improves release binary optimization with lower build cost than full LTO.
- `strip = "symbols"`: removes symbols from distributed binaries to reduce artifact size.
- `codegen-units = 1`: improves optimization quality for release artifacts.
- `opt-level = "z"`: prioritizes small distributable binaries over maximum runtime speed, fitting a CLI release artifact.

**Step 4: Run test to verify it passes**

Run:

```powershell
cd aidisk
cargo test cargo_toml_defines_release_profile_for_distributable_binary
```

Expected: PASS.

**Step 5: Verify release build**

Run:

```powershell
cd aidisk
cargo build --release
```

Expected: PASS and `target/release/aidisk.exe` exists on Windows.

**Step 6: Commit**

After fresh verification passes, commit:

```powershell
git add aidisk/Cargo.toml aidisk/tests/release_artifacts.rs
git commit -m "build: tune release profile"
```

---

## Task 2: Add Minimal CI And Release Artifact Workflow

**Files:**
- Create: `.github/workflows/ci.yml`
- Create: `.github/workflows/release-artifacts.yml`
- Modify: `aidisk/tests/release_artifacts.rs`

**Step 1: Write the failing test**

Add this test to `aidisk/tests/release_artifacts.rs`:

```rust
#[test]
fn github_actions_run_tests_and_build_windows_release_artifact() {
    let ci = read_repo_file(".github/workflows/ci.yml");
    let release = read_repo_file(".github/workflows/release-artifacts.yml");

    assert!(ci.contains("cargo test"));
    assert!(ci.contains("working-directory: aidisk"));
    assert!(release.contains("cargo build --release"));
    assert!(release.contains("windows-latest"));
    assert!(release.contains("aidisk.exe"));
    assert!(release.contains("actions/upload-artifact"));
}
```

**Step 2: Run test to verify it fails**

Run:

```powershell
cd aidisk
cargo test github_actions_run_tests_and_build_windows_release_artifact
```

Expected: FAIL because `.github/workflows/ci.yml` and `.github/workflows/release-artifacts.yml` do not exist.

**Step 3: Write minimal implementation**

Create `.github/workflows/ci.yml`:

```yaml
name: CI

on:
  push:
    branches: [master]
  pull_request:

jobs:
  test:
    runs-on: windows-latest
    defaults:
      run:
        working-directory: aidisk
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Run tests
        run: cargo test
```

Create `.github/workflows/release-artifacts.yml`:

```yaml
name: Release Artifacts

on:
  push:
    tags:
      - "v*.*.*"
  workflow_dispatch:

jobs:
  windows:
    runs-on: windows-latest
    defaults:
      run:
        working-directory: aidisk
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Build release binary
        run: cargo build --release
      - name: Upload aidisk.exe
        uses: actions/upload-artifact@v4
        with:
          name: aidisk-windows-x86_64
          path: aidisk/target/release/aidisk.exe
```

Artifact strategy:

- Scope is intentionally Windows-only because the project currently documents Windows as the supported platform.
- CI is always-on for push/PR via `cargo test`.
- Release artifacts build on version tags and manual dispatch, uploading only `aidisk.exe` as a minimal usable binary.

**Step 4: Run test to verify it passes**

Run:

```powershell
cd aidisk
cargo test github_actions_run_tests_and_build_windows_release_artifact
```

Expected: PASS.

**Step 5: Run Batch A verification**

Run:

```powershell
cd aidisk
cargo test release_artifacts
cargo build --release
cargo test
```

Expected: PASS.

**Step 6: Commit**

After fresh verification passes, commit:

```powershell
git add .github/workflows/ci.yml .github/workflows/release-artifacts.yml aidisk/tests/release_artifacts.rs
git commit -m "ci: add tests and release artifact build"
```

---

## Task 3: Add Scan Progress Hook Without Polluting JSON Or Non-TTY Output

**Files:**
- Modify: `aidisk/Cargo.toml`
- Modify: `aidisk/src/scanner.rs`
- Modify: `aidisk/src/main.rs`
- Test: `aidisk/src/scanner.rs`

**Step 1: Write the failing scanner test**

Add this test to `aidisk/src/scanner.rs` tests:

```rust
#[test]
fn scan_with_progress_reports_rule_steps() {
    let temp = tempdir().expect("tempdir should exist");
    let first = temp.path().join("first-cache");
    let second = temp.path().join("second-cache");
    fs::create_dir_all(&first).expect("first dir should exist");
    fs::create_dir_all(&second).expect("second dir should exist");

    let rules = vec![
        crate::rules::Rule {
            id: "first".to_string(),
            name: "First".to_string(),
            category: "test".to_string(),
            platform: "windows".to_string(),
            paths: vec![first.display().to_string()],
            risk: RiskLevel::Safe,
            cleanup: crate::rules::CleanupRule {
                method: "quarantine".to_string(),
            },
            reason: "first".to_string(),
            warnings: Vec::new(),
        },
        crate::rules::Rule {
            id: "second".to_string(),
            name: "Second".to_string(),
            category: "test".to_string(),
            platform: "windows".to_string(),
            paths: vec![second.display().to_string()],
            risk: RiskLevel::Review,
            cleanup: crate::rules::CleanupRule {
                method: "report-only".to_string(),
            },
            reason: "second".to_string(),
            warnings: Vec::new(),
        },
    ];
    let mut events = Vec::new();

    let report = super::scan_with_progress(&rules, 20, |event| {
        events.push((event.current, event.total, event.rule_id.to_string()));
    })
    .expect("scan should succeed");

    assert_eq!(report.summary.total_rules, 2);
    assert_eq!(events, vec![(1, 2, "first".to_string()), (2, 2, "second".to_string())]);
}
```

**Step 2: Run test to verify it fails**

Run:

```powershell
cd aidisk
cargo test scan_with_progress_reports_rule_steps
```

Expected: FAIL because `scan_with_progress` and its event type do not exist.

**Step 3: Write minimal scanner implementation**

In `aidisk/src/scanner.rs`:

```rust
pub struct ScanProgressEvent<'a> {
    pub current: usize,
    pub total: usize,
    pub rule_id: &'a str,
}

pub fn scan(rules: &[Rule], max_scan_depth: usize) -> Result<ScanReport> {
    scan_with_progress(rules, max_scan_depth, |_| {})
}

pub fn scan_with_progress<F>(
    rules: &[Rule],
    max_scan_depth: usize,
    mut on_progress: F,
) -> Result<ScanReport>
where
    F: FnMut(ScanProgressEvent<'_>),
{
    // existing scan body, calling on_progress once per rule before/after rule work.
}
```

Use the existing scan body and emit one event per rule using 1-based `current` and `rules.len()` as `total`. Do not change `ScanReport`, `Summary`, `Finding`, or JSON fields.

**Step 4: Run scanner test to verify it passes**

Run:

```powershell
cd aidisk
cargo test scan_with_progress_reports_rule_steps
```

Expected: PASS.

**Step 5: Write CLI progress wiring test indirectly through release artifacts**

Add this test to `aidisk/tests/release_artifacts.rs`:

```rust
#[test]
fn cargo_toml_includes_progress_terminal_dependencies() {
    let cargo_toml = read_repo_file("aidisk/Cargo.toml");

    assert!(cargo_toml.contains("indicatif"));
    assert!(cargo_toml.contains("console"));
}
```

**Step 6: Run dependency test to verify it fails**

Run:

```powershell
cd aidisk
cargo test cargo_toml_includes_progress_terminal_dependencies
```

Expected: FAIL because `indicatif` and `console` are not dependencies yet.

**Step 7: Wire minimal progress implementation in CLI**

Update `aidisk/Cargo.toml` dependencies:

```toml
console = "0.15"
indicatif = "0.17"
```

Update `aidisk/src/main.rs`:

```rust
fn progress_enabled(format: OutputFormat) -> bool {
    format != OutputFormat::Json
        && std::env::var_os("CI").is_none()
        && console::Term::stderr().is_term()
}
```

For `scan`, `plan`, `clean`, and `doctor` scan calls, keep the normal `scanner::scan` path unless progress is enabled. When enabled, create an `indicatif::ProgressBar` on stderr and call `scanner::scan_with_progress`, updating message/position. Finish and clear the progress bar before printing the report.

Minimum behavior:

- `--json` never creates progress.
- Non-TTY/CI/tests do not create progress.
- Progress writes to stderr only.
- JSON stdout remains valid because report printing is unchanged.

**Step 8: Run tests to verify progress work passes**

Run:

```powershell
cd aidisk
cargo test cargo_toml_includes_progress_terminal_dependencies
cargo test scanner::tests
cargo test doctor_cli
```

Expected: PASS. `doctor_cli` JSON parsing still succeeds, proving progress does not pollute JSON stdout in tests.

**Step 9: Commit**

After fresh verification passes, commit:

```powershell
git add aidisk/Cargo.toml aidisk/Cargo.lock aidisk/src/main.rs aidisk/src/scanner.rs aidisk/tests/release_artifacts.rs
git commit -m "feat: show scan progress in interactive terminals"
```

---

## Task 4: Add Scan Executive Summary And Unicode Risk Bars

**Files:**
- Modify: `aidisk/src/reporter.rs`

**Step 1: Write failing reporter tests**

Add these tests to `aidisk/src/reporter.rs` tests:

```rust
#[test]
fn scan_markdown_renders_executive_summary_and_unicode_risk_bars() {
    let report = crate::scanner::ScanReport {
        scan_time: Local::now(),
        volumes: Vec::new(),
        findings: Vec::new(),
        summary: crate::scanner::Summary {
            total_rules: 3,
            matched_paths: 2,
            total_size_bytes: 10,
            safe_bytes: 6,
            review_bytes: 3,
            dangerous_bytes: 1,
            system_bytes: 0,
            top_findings: Vec::new(),
            reclaimable_safe_bytes: 6,
        },
    };

    let output = super::render(&report, OutputFormat::Markdown).expect("scan markdown should render");

    assert!(output.contains("## Executive Summary"));
    assert!(output.contains("Reclaimable now: 6 B"));
    assert!(output.contains("Risk Distribution"));
    assert!(output.contains("SAFE"));
    assert!(output.contains("REVIEW"));
    assert!(output.contains("DANGEROUS"));
    assert!(output.contains('█'));
}

#[test]
fn scan_json_does_not_gain_executive_summary_text() {
    let report = crate::scanner::ScanReport {
        scan_time: Local::now(),
        volumes: Vec::new(),
        findings: Vec::new(),
        summary: crate::scanner::Summary::default(),
    };

    let output = super::render(&report, OutputFormat::Json).expect("scan json should render");

    assert!(!output.contains("Executive Summary"));
    assert!(!output.contains('█'));
}
```

**Step 2: Run tests to verify they fail**

Run:

```powershell
cd aidisk
cargo test scan_markdown_renders_executive_summary_and_unicode_risk_bars scan_json_does_not_gain_executive_summary_text
```

Expected: First test FAILS because the markdown summary and bars do not exist. If the JSON test passes immediately, keep it as a guardrail because the behavior already exists and protects JSON stability.

**Step 3: Write minimal reporter implementation**

In `aidisk/src/reporter.rs`, add small helpers near `human_bytes`:

```rust
fn risk_bar(value: u64, total: u64, width: usize) -> String {
    if total == 0 || value == 0 {
        return "░".repeat(width);
    }
    let filled = (((value as f64 / total as f64) * width as f64).round() as usize)
        .clamp(1, width);
    format!("{}{}", "█".repeat(filled), "░".repeat(width - filled))
}
```

Add a `render_scan_executive_summary_lines(report: &ScanReport, markdown: bool) -> Vec<String>` helper that returns:

- `## Executive Summary` for markdown or `Executive Summary:` for text.
- `Reclaimable now: <safe bytes>`.
- `Needs review: <review bytes>`.
- `High risk/system protected: <dangerous + system bytes>`.
- A risk distribution block using `risk_bar` for Safe, Review, Dangerous, and System.

Insert this helper after the header block and before Top Findings in `render_text` and `render_markdown`. Do not call it from JSON.

**Step 4: Run tests to verify they pass**

Run:

```powershell
cd aidisk
cargo test scan_markdown_renders_executive_summary_and_unicode_risk_bars scan_json_does_not_gain_executive_summary_text
```

Expected: PASS.

**Step 5: Run reporter tests**

Run:

```powershell
cd aidisk
cargo test reporter::tests
```

Expected: PASS.

**Step 6: Commit**

After fresh verification passes, commit:

```powershell
git add aidisk/src/reporter.rs
git commit -m "feat: summarize scan risk in text outputs"
```

---

## Task 5: Add Doctor Executive Summary And Topic Bars

**Files:**
- Modify: `aidisk/src/reporter.rs`

**Step 1: Write failing reporter tests**

Add these tests to `aidisk/src/reporter.rs` tests:

```rust
#[test]
fn doctor_markdown_renders_executive_summary_and_topic_bars() {
    let report = DoctorReport {
        generated_at: Local::now(),
        policy_summary: "test policy".to_string(),
        latest_diff: None,
        topics: vec![
            DoctorTopic {
                name: "agents".to_string(),
                status: "active".to_string(),
                summary: "2 matching items".to_string(),
                findings: vec![DoctorFinding {
                    id: "agent".to_string(),
                    path: "C:\\Users\\demo\\.claude".to_string(),
                    exists: true,
                    size_bytes: 6,
                    risk: "review".to_string(),
                    action: "report-only".to_string(),
                    reason: "agent state".to_string(),
                    breakdown: Vec::new(),
                }],
                recommendations: Vec::new(),
                probes: Vec::new(),
            },
            DoctorTopic {
                name: "docker".to_string(),
                status: "not-detected".to_string(),
                summary: "not found".to_string(),
                findings: Vec::new(),
                recommendations: Vec::new(),
                probes: Vec::new(),
            },
        ],
    };

    let output = render_doctor(&report, OutputFormat::Markdown).expect("doctor markdown should render");

    assert!(output.contains("## Executive Summary"));
    assert!(output.contains("Active topics: 1"));
    assert!(output.contains("Not detected topics: 1"));
    assert!(output.contains("Topic Size Distribution"));
    assert!(output.contains("agents"));
    assert!(output.contains('█'));
}

#[test]
fn doctor_json_does_not_gain_executive_summary_text() {
    let report = DoctorReport {
        generated_at: Local::now(),
        policy_summary: "test policy".to_string(),
        latest_diff: None,
        topics: Vec::new(),
    };

    let output = render_doctor(&report, OutputFormat::Json).expect("doctor json should render");

    assert!(!output.contains("Executive Summary"));
    assert!(!output.contains('█'));
}
```

**Step 2: Run tests to verify they fail**

Run:

```powershell
cd aidisk
cargo test doctor_markdown_renders_executive_summary_and_topic_bars doctor_json_does_not_gain_executive_summary_text
```

Expected: First test FAILS because the doctor executive summary and topic bars do not exist. JSON guardrail may pass immediately.

**Step 3: Write minimal reporter implementation**

In `aidisk/src/reporter.rs`, add `render_doctor_executive_summary_lines(report: &DoctorReport, markdown: bool) -> Vec<String>`:

- Counts `active`, `not-detected`, and `no-rules` topics from `topic.status`.
- Sums existing finding sizes per topic.
- Uses the same Unicode bar helper for topic size distribution.
- Adds markdown section `## Executive Summary` or text section `Executive Summary:`.

Insert it immediately after generated-at/policy lines in `render_doctor_text` and `render_doctor_markdown`. Do not call it from JSON.

Color strategy:

- Do not embed ANSI color in reporter output by default; this keeps markdown and redirected text clean.
- Treat risk emphasis as textual labels plus Unicode bars.
- If terminal colors are added later, gate them in `main` only for interactive terminals.

Unicode fallback strategy:

- This slice uses Unicode bars in text/markdown as requested.
- Do not add terminal width or encoding negotiation in this round; terminal width adaptation is P3.
- Keep bar width fixed and small so output remains readable when copied into markdown.

**Step 4: Run tests to verify they pass**

Run:

```powershell
cd aidisk
cargo test doctor_markdown_renders_executive_summary_and_topic_bars doctor_json_does_not_gain_executive_summary_text
```

Expected: PASS.

**Step 5: Run Batch B verification**

Run:

```powershell
cd aidisk
cargo test reporter::tests
cargo test scanner::tests
cargo test doctor_cli
cargo test
```

Expected: PASS.

**Step 6: Commit**

After fresh verification passes, commit:

```powershell
git add aidisk/src/reporter.rs
git commit -m "feat: summarize doctor topics in text outputs"
```

---

## Final Verification

Run from `aidisk`:

```powershell
cargo test
cargo build --release
cargo run -- scan --rules-repo "tests/fixtures/community-rules" --json
cargo run -- scan --rules-repo "tests/fixtures/community-rules" --markdown
cargo run -- doctor --markdown
```

Expected results:

- `cargo test` passes.
- `cargo build --release` passes.
- `scan --json` emits parseable JSON without executive-summary text or Unicode bars as added fields.
- `scan --markdown` includes `Executive Summary` and Unicode bars.
- `doctor --markdown` includes `Executive Summary` and topic distribution bars.

## Open Questions

None. This plan intentionally chooses the narrower path: Windows-only release artifacts, no JSON schema changes, no terminal width adaptation, no ANSI color injection in rendered markdown/text, and no performance or MCP implementation work in this slice.
