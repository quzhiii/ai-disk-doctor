# P1 Structured JSON Errors Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make `aidisk` emit stable, parseable, agent-friendly JSON errors in `--json` mode while preserving existing success JSON schemas.

**Architecture:** Keep the successful command report structs unchanged and add a narrow CLI-boundary error envelope rendered only when a command selected JSON output fails. `main` will parse the CLI, run command execution through a small `run(cli) -> Result<()>` wrapper, and on error write one JSON error document to stderr while leaving stdout empty; non-JSON failures keep the current text stderr behavior.

**Tech Stack:** Rust 2021, Cargo, `clap`, `anyhow`, `serde`, `serde_json`, CLI integration tests with `std::process::Command`, `tempfile`.

---

## Guardrails

- Do not change existing successful JSON response structs for `scan`, `plan`, `clean`, `restore`, `doctor`, or `diff` except for the explicit `clean --dry-run --json --quarantine-root` fix in Batch B.
- Do not add Scoop, benchmark, parallel scanning, MCP server work, shell completion, terminal width adaptation, topic metadata externalization, `--topic`, duplicate detection, TUI, or idempotency tokens.
- JSON errors must be a single JSON object and must be written to stderr, not stdout.
- In JSON error mode, stdout must remain empty on failure.
- In text/markdown modes, preserve the existing text error behavior as much as possible.
- Keep the error model minimal and stable: `ok`, `error.type`, `error.message`, `error.command`, `error.details`.
- Use strict TDD for production code: write failing CLI integration tests first, verify RED, implement the smallest GREEN change, then re-run verification.
- Commit only after a fresh verification command has passed for the checkpoint.

## Current Behavior From Review

- Successful JSON outputs are rendered by `aidisk/src/reporter.rs` with `serde_json::to_string_pretty(...)` and printed to stdout.
- `aidisk/src/main.rs` currently returns `anyhow::Result<()>`, so errors from `anyhow::bail!`, file I/O, rule loading, policy loading, and diff history are printed by the runtime as plain text on stderr.
- Existing CLI tests parse success JSON from stdout, especially `aidisk/tests/doctor_cli.rs`.
- Interactive progress is already gated off for `OutputFormat::Json`, so it should not pollute JSON stdout/stderr in JSON-mode tests.
- `clean --dry-run --json --quarantine-root <root>` currently prints two separate JSON documents to stdout: first `CleanReport`, then a blank line, then `QuarantinePlan`. This breaks JSON consumers and is included in Batch B as a minimal, tested fix.

## JSON Error Contract

Error JSON is emitted to stderr only when the command selected JSON output by using `--json` or `--format json`.

```json
{
  "ok": false,
  "error": {
    "type": "usage",
    "message": "restore execution requires --yes or use --dry-run",
    "command": "restore",
    "details": []
  }
}
```

Initial error types:

- `usage`: missing required action flags or command arguments detected after clap parsing.
- `input`: filesystem/config/rules/history inputs cannot be read, parsed, or resolved.
- `execution`: runtime execution errors after validation, such as quarantine/restore operation failures that bubble out.
- `internal`: serialization or unexpected errors.

Classification is intentionally string-based in this slice because most existing errors are `anyhow::Error` without typed variants. A later hardening pass can introduce typed domain errors if needed.

## Batch Strategy

Batch A creates the JSON error envelope and covers the core error paths for `scan`, `plan`, `clean`, `restore`, `diff`, and `doctor` without touching successful JSON schemas.

Batch B fixes the known double-JSON stdout bug for `clean --dry-run --json --quarantine-root`, documents the JSON error contract briefly, and performs full verification.

## Milestones

### Milestone 0: Plan And Baseline

**Files:**
- Create: `docs/plans/2026-06-05-p1-structured-json-errors.md`

**Verification command:**

```powershell
cargo test
```

Run from `aidisk`.

**Expected result:** Current test suite passes before implementation starts.

**Commit point:** Commit the plan after baseline verification if the working tree only contains this plan.

### Milestone A: Structured JSON Errors

**Tasks:** 1-3

**Verification command:**

```powershell
cargo test json_errors_cli
cargo test doctor_cli
cargo test
```

Run from `aidisk`.

**Expected result:** JSON error integration tests pass, existing doctor success JSON tests still parse stdout, and full suite passes.

**Commit point:** Commit after Tasks 1-3 pass.

### Milestone B: Single JSON Clean Dry-Run Output And Docs

**Tasks:** 4-5

**Verification command:**

```powershell
cargo test clean_json_cli
cargo test json_errors_cli
cargo test
```

Run from `aidisk`.

**Expected result:** `clean --dry-run --json --quarantine-root` emits one parseable stdout document, JSON error contract still passes, and full suite passes.

**Commit point:** Commit after Tasks 4-5 pass.

---

## Task 1: Add CLI Tests For JSON Error Contract

**Files:**
- Create: `aidisk/tests/json_errors_cli.rs`

**Step 1: Write the failing test**

Create `aidisk/tests/json_errors_cli.rs` with helper functions and these tests:

```rust
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use tempfile::tempdir;

fn aidisk_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_aidisk"))
}

fn parse_json(bytes: &[u8]) -> serde_json::Value {
    serde_json::from_slice(bytes).expect("stderr should be a single JSON document")
}

fn assert_json_error(output: &std::process::Output, command: &str) -> serde_json::Value {
    assert!(!output.status.success(), "command should fail");
    assert!(output.stdout.is_empty(), "stdout must stay empty on JSON errors: {}", String::from_utf8_lossy(&output.stdout));
    let parsed = parse_json(&output.stderr);
    assert_eq!(parsed["ok"], false);
    assert_eq!(parsed["error"]["command"], command);
    assert!(parsed["error"]["message"].as_str().expect("message should be string").len() > 0);
    assert!(parsed["error"]["type"].as_str().expect("type should be string").len() > 0);
    assert!(parsed["error"]["details"].as_array().is_some(), "details should be an array");
    parsed
}

fn write_policy(path: &Path) {
    fs::write(
        path,
        r#"sensitive_markers:
  - token
planner:
  skip_modified_within_minutes: 30
  allow_actions:
    - quarantine
    - report-only
    - guide
  max_scan_depth: 20
"#,
    )
    .expect("policy should be written");
}

#[test]
fn scan_json_error_is_parseable_and_keeps_stdout_empty() {
    let temp = tempdir().expect("tempdir should exist");
    let missing_rules = temp.path().join("missing-rules");

    let output = Command::new(aidisk_bin())
        .args(["scan", "--rules-dir", missing_rules.to_str().unwrap(), "--json"])
        .output()
        .expect("scan should run");

    let parsed = assert_json_error(&output, "scan");
    assert_eq!(parsed["error"]["type"], "input");
}

#[test]
fn plan_json_error_uses_same_contract() {
    let temp = tempdir().expect("tempdir should exist");
    let rules_dir = temp.path().join("rules");
    let missing_policy = temp.path().join("missing-policy.yaml");
    fs::create_dir_all(&rules_dir).expect("rules dir should exist");

    let output = Command::new(aidisk_bin())
        .args([
            "plan",
            "--rules-dir",
            rules_dir.to_str().unwrap(),
            "--policy",
            missing_policy.to_str().unwrap(),
            "--json",
        ])
        .output()
        .expect("plan should run");

    let parsed = assert_json_error(&output, "plan");
    assert_eq!(parsed["error"]["type"], "input");
}

#[test]
fn clean_json_error_uses_same_contract_for_usage_errors() {
    let temp = tempdir().expect("tempdir should exist");
    let rules_dir = temp.path().join("rules");
    let policy = temp.path().join("policy.yaml");
    fs::create_dir_all(&rules_dir).expect("rules dir should exist");
    write_policy(&policy);

    let output = Command::new(aidisk_bin())
        .args([
            "clean",
            "--rules-dir",
            rules_dir.to_str().unwrap(),
            "--policy",
            policy.to_str().unwrap(),
            "--json",
        ])
        .output()
        .expect("clean should run");

    let parsed = assert_json_error(&output, "clean");
    assert_eq!(parsed["error"]["type"], "usage");
    assert!(parsed["error"]["message"].as_str().unwrap().contains("--yes"));
}

#[test]
fn restore_json_error_uses_same_contract_for_usage_errors() {
    let temp = tempdir().expect("tempdir should exist");
    let index = temp.path().join("index.json");

    let output = Command::new(aidisk_bin())
        .args(["restore", "--index", index.to_str().unwrap(), "--json"])
        .output()
        .expect("restore should run");

    let parsed = assert_json_error(&output, "restore");
    assert_eq!(parsed["error"]["type"], "usage");
}

#[test]
fn diff_json_error_uses_same_contract() {
    let output = Command::new(aidisk_bin())
        .args(["diff", "--json"])
        .output()
        .expect("diff should run");

    let parsed = assert_json_error(&output, "diff");
    assert_eq!(parsed["error"]["type"], "usage");
}

#[test]
fn doctor_json_error_uses_same_contract() {
    let temp = tempdir().expect("tempdir should exist");
    let rules_dir = temp.path().join("rules");
    let reports_dir = temp.path().join("reports");
    let policy = temp.path().join("policy.yaml");
    fs::create_dir_all(&rules_dir).expect("rules dir should exist");
    fs::create_dir_all(&reports_dir).expect("reports dir should exist");
    write_policy(&policy);

    let output = Command::new(aidisk_bin())
        .args([
            "doctor",
            "--latest",
            "--reports-dir",
            reports_dir.to_str().unwrap(),
            "--rules-dir",
            rules_dir.to_str().unwrap(),
            "--policy",
            policy.to_str().unwrap(),
            "--json",
        ])
        .output()
        .expect("doctor should run");

    let parsed = assert_json_error(&output, "doctor");
    assert_eq!(parsed["error"]["type"], "input");
}
```

**Step 2: Run test to verify it fails**

Run:

```powershell
cargo test json_errors_cli
```

Expected: FAIL because stderr is still plain text from `anyhow`, not JSON.

**Step 3: Commit**

No commit yet. This is the RED step for Task 2.

---

## Task 2: Implement Minimal JSON Error Envelope

**Files:**
- Modify: `aidisk/src/main.rs`

**Step 1: Refactor `main` to preserve existing behavior**

Move the existing command `match` body into:

```rust
fn run(cli: Cli) -> Result<()> {
    match cli.command {
        // existing command arms unchanged except for JSON error plumbing
    }
    Ok(())
}
```

Change `main` to:

```rust
fn main() -> std::process::ExitCode {
    let cli = Cli::parse();
    let error_context = ErrorContext::from_command(&cli.command);

    match run(cli) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            emit_error(&error_context, &error);
            std::process::ExitCode::FAILURE
        }
    }
}
```

**Step 2: Add the minimal serializable error model**

In `aidisk/src/main.rs`, add:

```rust
#[derive(Clone, Copy)]
struct ErrorContext {
    command: &'static str,
    format: OutputFormat,
}

#[derive(serde::Serialize)]
struct JsonErrorEnvelope {
    ok: bool,
    error: JsonErrorBody,
}

#[derive(serde::Serialize)]
struct JsonErrorBody {
    #[serde(rename = "type")]
    error_type: String,
    message: String,
    command: String,
    details: Vec<String>,
}
```

Implement `ErrorContext::from_command(&Command)` by matching each command and reusing the same effective-format logic as the command arms.

**Step 3: Add rendering and classification helpers**

In `aidisk/src/main.rs`, add:

```rust
fn emit_error(context: &ErrorContext, error: &anyhow::Error) {
    if context.format == OutputFormat::Json {
        let envelope = JsonErrorEnvelope {
            ok: false,
            error: JsonErrorBody {
                error_type: classify_cli_error(error).to_string(),
                message: error.to_string(),
                command: context.command.to_string(),
                details: Vec::new(),
            },
        };
        match serde_json::to_string_pretty(&envelope) {
            Ok(output) => eprintln!("{output}"),
            Err(render_error) => eprintln!("{{\"ok\":false,\"error\":{{\"type\":\"internal\",\"message\":\"failed to render JSON error: {render_error}\",\"command\":\"{}\",\"details\":[]}}}}", context.command),
        }
    } else {
        eprintln!("Error: {error:?}");
    }
}

fn classify_cli_error(error: &anyhow::Error) -> &'static str {
    let message = error.to_string().to_ascii_lowercase();
    if message.contains("requires --yes")
        || message.contains("requires --before")
        || message.contains("requires --after")
        || message.contains("requires --quarantine-root")
    {
        return "usage";
    }
    if message.contains("failed to read")
        || message.contains("failed to parse")
        || message.contains("requires at least two scan snapshots")
        || message.contains("no such file")
        || message.contains("not found")
        || error.downcast_ref::<std::io::Error>().is_some()
    {
        return "input";
    }
    "execution"
}
```

Keep this helper private to `main.rs`; do not introduce a new public module in this slice unless needed by tests.

**Step 4: Run test to verify it passes**

Run:

```powershell
cargo test json_errors_cli
```

Expected: PASS.

**Step 5: Run existing JSON success tests**

Run:

```powershell
cargo test doctor_cli
```

Expected: PASS, proving successful JSON stdout remains parseable.

**Step 6: Commit**

No commit yet. Task 3 adds one guard test before the Batch A commit.

---

## Task 3: Add Non-JSON Error Regression Guard And Batch A Verification

**Files:**
- Modify: `aidisk/tests/json_errors_cli.rs`

**Step 1: Write the failing or guard test**

Append this test to `aidisk/tests/json_errors_cli.rs`:

```rust
#[test]
fn text_error_remains_non_json() {
    let output = Command::new(aidisk_bin())
        .args(["diff"])
        .output()
        .expect("diff should run");

    assert!(!output.status.success(), "diff should fail without --before/--after");
    assert!(output.stdout.is_empty(), "stdout should be empty on text errors");
    assert!(serde_json::from_slice::<serde_json::Value>(&output.stderr).is_err(), "text-mode stderr should not become JSON");
    assert!(String::from_utf8_lossy(&output.stderr).contains("diff requires --before"));
}
```

If this passes immediately, keep it as a regression guard because it protects existing text-mode behavior.

**Step 2: Run focused tests**

Run:

```powershell
cargo test json_errors_cli
```

Expected: PASS.

**Step 3: Run Batch A verification**

Run:

```powershell
cargo test json_errors_cli
cargo test doctor_cli
cargo test
```

Expected: PASS.

**Step 4: Commit**

After fresh verification passes, commit:

```powershell
git add aidisk/src/main.rs aidisk/tests/json_errors_cli.rs
git commit -m "feat: emit structured json errors"
```

---

## Task 4: Fix Clean Dry-Run JSON To Emit One Document With Quarantine Plan

**Files:**
- Create: `aidisk/tests/clean_json_cli.rs`
- Modify: `aidisk/src/cleaner.rs`
- Modify: `aidisk/src/reporter.rs`
- Modify: `aidisk/src/main.rs`

**Step 1: Write the failing test**

Create `aidisk/tests/clean_json_cli.rs`:

```rust
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use tempfile::tempdir;

fn aidisk_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_aidisk"))
}

fn write_policy(path: &Path) {
    fs::write(
        path,
        r#"sensitive_markers:
  - token
planner:
  skip_modified_within_minutes: 0
  allow_actions:
    - quarantine
    - report-only
    - guide
  max_scan_depth: 20
"#,
    )
    .expect("policy should be written");
}

fn write_rule(path: &Path, target: &Path) {
    let escaped = target.display().to_string().replace('\\', "\\\\");
    fs::write(
        path.join("cache.yaml"),
        format!(
            r#"id: clean-json-cache
name: Clean JSON cache
category: dev-cache
platform: windows
paths:
  - "{escaped}"
risk: safe
cleanup:
  method: quarantine
reason: test cache
"#
        ),
    )
    .expect("rule should be written");
}

#[test]
fn clean_dry_run_json_with_quarantine_root_emits_single_parseable_document() {
    let temp = tempdir().expect("tempdir should exist");
    let rules_dir = temp.path().join("rules");
    let policy = temp.path().join("policy.yaml");
    let target = temp.path().join("cache");
    let quarantine_root = temp.path().join("archive");
    fs::create_dir_all(&rules_dir).expect("rules dir should exist");
    fs::create_dir_all(&target).expect("target should exist");
    fs::write(target.join("data.bin"), vec![1_u8; 16]).expect("data should exist");
    write_policy(&policy);
    write_rule(&rules_dir, &target);

    let output = Command::new(aidisk_bin())
        .args([
            "clean",
            "--dry-run",
            "--rules-dir",
            rules_dir.to_str().unwrap(),
            "--policy",
            policy.to_str().unwrap(),
            "--quarantine-root",
            quarantine_root.to_str().unwrap(),
            "--json",
        ])
        .output()
        .expect("clean should run");

    assert!(output.status.success(), "clean should succeed: {}", String::from_utf8_lossy(&output.stderr));
    assert!(output.stderr.is_empty(), "stderr should be empty on success");
    let parsed: serde_json::Value = serde_json::from_slice(&output.stdout).expect("stdout should be one JSON document");
    assert_eq!(parsed["mode"], "dry-run");
    assert_eq!(parsed["quarantine_plan"]["root"], quarantine_root.display().to_string());
    assert_eq!(parsed["quarantine_plan"]["entries"].as_array().unwrap().len(), 1);
}
```

**Step 2: Run test to verify it fails**

Run:

```powershell
cargo test clean_dry_run_json_with_quarantine_root_emits_single_parseable_document
```

Expected: FAIL because stdout contains two JSON documents instead of one parseable document.

**Step 3: Add the minimal combined report type**

In `aidisk/src/cleaner.rs`, add:

```rust
#[derive(Debug, Serialize)]
pub struct CleanDryRunOutput {
    #[serde(flatten)]
    pub clean: CleanReport,
    pub quarantine_plan: Option<QuarantinePlan>,
}
```

This keeps existing `CleanReport` fields at the same top-level paths and only adds `quarantine_plan` when the caller requested it.

**Step 4: Add reporter helper**

In `aidisk/src/reporter.rs`, import `CleanDryRunOutput` and add:

```rust
pub fn render_clean_dry_run_output(report: &CleanDryRunOutput, format: OutputFormat) -> Result<String> {
    let output = match format {
        OutputFormat::Json => serde_json::to_string_pretty(report)?,
        OutputFormat::Markdown => render_clean_markdown(&report.clean),
        OutputFormat::Text => render_clean_text(&report.clean),
    };
    Ok(output)
}
```

Text/markdown can keep existing behavior in this slice. The bug is JSON consumer correctness.

**Step 5: Update `clean --dry-run` CLI branch**

In `aidisk/src/main.rs`, replace the current dry-run branch with:

```rust
let clean_report = cleaner::build_dry_run(&plan_report);
if effective_format == OutputFormat::Json {
    let quarantine_plan = quarantine_root
        .as_deref()
        .map(|root| cleaner::build_quarantine_plan(&plan_report, root));
    let output = cleaner::CleanDryRunOutput {
        clean: clean_report,
        quarantine_plan,
    };
    println!("{}", reporter::render_clean_dry_run_output(&output, effective_format)?);
} else {
    println!("{}", reporter::render_clean(&clean_report, effective_format)?);
    if let Some(quarantine_root) = quarantine_root {
        let quarantine_plan = cleaner::build_quarantine_plan(&plan_report, &quarantine_root);
        println!();
        println!("{}", reporter::render_quarantine_plan(&quarantine_plan, effective_format)?);
    }
}
```

**Step 6: Run test to verify it passes**

Run:

```powershell
cargo test clean_dry_run_json_with_quarantine_root_emits_single_parseable_document
```

Expected: PASS.

**Step 7: Run regression tests**

Run:

```powershell
cargo test clean_json_cli
cargo test json_errors_cli
```

Expected: PASS.

**Step 8: Commit**

No commit yet. Task 5 adds docs before the Batch B commit.

---

## Task 5: Document JSON Error Contract And Run Final Verification

**Files:**
- Modify: `README.md`
- Modify: `README.zh-CN.md`
- Modify: `CHANGELOG.md`

**Step 1: Add concise documentation**

In `README.md`, under `## Command Reference`, add a short `### JSON Error Contract` section:

```markdown
### JSON Error Contract

When `--json` or `--format json` is selected and a command fails, `aidisk` writes one JSON error object to stderr and leaves stdout empty. Successful JSON reports are still written to stdout.

```json
{
  "ok": false,
  "error": {
    "type": "usage",
    "message": "restore execution requires --yes or use --dry-run",
    "command": "restore",
    "details": []
  }
}
```
```

In `README.zh-CN.md`, add the equivalent concise Chinese section under `## 命令参考`.

In `CHANGELOG.md`, add an `## Unreleased` section:

```markdown
## Unreleased

- Added structured JSON error output for `--json` command failures. JSON-mode failures now write a single error object to stderr and keep stdout empty for consumers.
- Fixed `clean --dry-run --json --quarantine-root` to emit a single parseable JSON document instead of two consecutive JSON documents.
```

**Step 2: Run final verification**

Run:

```powershell
cargo test clean_json_cli
cargo test json_errors_cli
cargo test doctor_cli
cargo test
cargo build --release
```

Expected: PASS.

**Step 3: Commit**

After fresh verification passes, commit:

```powershell
git add aidisk/src/main.rs aidisk/src/cleaner.rs aidisk/src/reporter.rs aidisk/tests/clean_json_cli.rs README.md README.zh-CN.md CHANGELOG.md
git commit -m "fix: keep clean dry-run json single document"
```

---

## Final Verification

Run from `aidisk`:

```powershell
cargo test
cargo build --release
cargo run -- restore --index missing.json --json
cargo run -- diff --json
cargo run -- clean --dry-run --rules-repo "tests/fixtures/community-rules" --quarantine-root "$env:TEMP\aidisk-archive" --json
```

Expected results:

- `cargo test` passes.
- `cargo build --release` passes.
- `restore --json` failure exits non-zero, writes JSON to stderr, and leaves stdout empty.
- `diff --json` failure exits non-zero, writes JSON to stderr, and leaves stdout empty.
- `clean --dry-run --json --quarantine-root` succeeds and stdout is a single parseable JSON document.

## Open Questions

None. The plan chooses stderr for JSON errors because stdout is already the success JSON channel and must stay clean for parsers. The double-JSON clean dry-run issue is included because it is an existing JSON-consumer correctness bug in the same surface area and can be fixed with a minimal additive wrapper.
