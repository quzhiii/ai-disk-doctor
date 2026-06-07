# P7 Large Files Discovery Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a lightweight read-only `scan --large-files --min-size <SIZE>` command that discovers large files and directories without classification or cleanup suggestions.

**Architecture:** New `scanner::scan_large_files` function walks a root directory, collects entries above a size threshold, and produces a standalone `LargeFilesReport` struct. Reporter renders JSON and text. No rules, no risk classification, no cleanup suggestions. The new CLI flag attaches to existing `scan` subcommand for discoverability but produces a completely separate output contract.

**Tech Stack:** Rust 2021, Cargo, `walkdir`, `serde`, `serde_json`, `clap`, CLI integration tests with `std::process::Command`, `tempfile`.

---

## Guardrails

- Do not change existing `ScanReport`, scanner rules path, planner, cleaner, or doctor.
- `LargeFilesReport` is a separate struct; JSON schema is independent of existing scan.
- Do not add classification, risk levels, or cleanup recommendations.
- `--large-files` without `--min-size` defaults to 500 MB.
- `--large-files` is mutually exclusive with `--category`, `--rules-dir`, `--rules-repo`? No — for simplicity, `--large-files` overrides them; other scan flags are silently ignored.
- Keep `--json` / `--markdown` / `--text` format support.
- Use strict TDD: write failing tests first, verify RED, implement minimal GREEN, re-run verification.

## CLI Contract

```
aidisk scan --large-files [--min-size <SIZE>] [--json|--markdown|--text] [--root <PATH>]
```

- `--min-size` accepts human-readable values (e.g. `500MB`, `1GB`, `100MB`). Default: 500 MB.
- `--root` defaults to `%USERPROFILE%`.

## JSON Output Schema

```json
{
  "scan_root": "C:\\Users\\demo",
  "min_size": "500MB",
  "min_size_bytes": 524288000,
  "scan_time": "2026-06-06T12:00:00+08:00",
  "entries": [
    {
      "path": "C:\\Users\\demo\\AppData\\Local\\Docker\\wsl\\disk\\docker_data.vhdx",
      "size_bytes": 15000000000,
      "is_directory": false
    }
  ]
}
```

## Milestones

### Milestone 0: Plan And Baseline

**Files:**
- Create: `docs/plans/2026-06-06-p7-large-files-discovery.md`

**Verification command:**

```powershell
cargo test
```

Run from `aidisk`.

**Expected result:** Full suite passes before implementation starts.

**Commit point:** Commit the plan after baseline verification.

### Milestone A: Scanner And Unit Tests

**Tasks:** 1

**Verification command:**

```powershell
cargo test scanner::tests::scan_large_files
```

Run from `aidisk`.

**Expected result:** Scanner unit tests pass.

**Commit point:** Commit after Task 1 passes.

### Milestone B: CLI, Reporter, And Integration Tests

**Tasks:** 2

**Verification command:**

```powershell
cargo test --test large_files_cli
cargo test
```

Run from `aidisk`.

**Expected result:** CLI integration tests pass and full suite passes.

**Commit point:** Commit after Task 2 passes.

### Milestone C: Docs And Final Verification

**Tasks:** 3

**Verification command:**

```powershell
cargo test --test large_files_cli
cargo test
```

Run from `aidisk`.

**Expected result:** Full suite passes.

**Commit point:** Commit after Task 3 passes.

---

## Task 1: Add Scanner Function And Unit Tests

**Files:**
- Modify: `aidisk/src/scanner.rs`

**Step 1: Write the failing test**

Append this test to `aidisk/src/scanner.rs` tests:

```rust
#[test]
fn scan_large_files_discovers_entries_above_threshold() {
    let temp = tempdir().expect("tempdir should exist");
    let root = temp.path();
    fs::create_dir_all(root.join("big-dir")).expect("big dir should exist");
    fs::write(root.join("small.txt"), vec![0_u8; 100]).expect("small file should write");
    fs::write(root.join("big-dir").join("large.bin"), vec![0_u8; 1000])
        .expect("large file should write");
    fs::write(root.join("big-dir").join("tiny.bin"), vec![0_u8; 10])
        .expect("tiny file should write");

    let report = super::scan_large_files(root, 500).expect("scan should succeed");

    assert_eq!(report.scan_root, root.display().to_string());
    assert_eq!(report.min_size_bytes, 500);
    assert!(report.entries.len() >= 1, "should find at least one entry above 500 bytes");

    // The big-dir directory itself is >500 bytes
    let big_dir = report.entries.iter().find(|e| e.path.ends_with("big-dir"));
    assert!(big_dir.is_some(), "should find big-dir directory");
    assert!(big_dir.unwrap().is_directory);

    // large.bin inside big-dir is 1000 bytes > threshold
    let large_file = report.entries.iter().find(|e| e.path.ends_with("large.bin"));
    assert!(large_file.is_some(), "should find large.bin");
    assert!(!large_file.unwrap().is_directory);

    // small.txt (100 bytes) should NOT appear
    assert!(!report.entries.iter().any(|e| e.path.ends_with("small.txt")));
    // tiny.bin (10 bytes) should NOT appear
    assert!(!report.entries.iter().any(|e| e.path.ends_with("tiny.bin")));
}
```

**Step 2: Run test to verify it fails**

Run:

```powershell
cargo test scan_large_files_discovers_entries_above_threshold
```

Expected: FAIL because `scan_large_files` does not exist.

**Step 3: Write minimal scanner implementation**

Add to `aidisk/src/scanner.rs`:

```rust
#[derive(Debug, Serialize)]
pub struct LargeFilesReport {
    pub scan_root: String,
    pub min_size: String,
    pub min_size_bytes: u64,
    pub scan_time: DateTime<Local>,
    pub entries: Vec<LargeFileEntry>,
}

#[derive(Debug, Serialize)]
pub struct LargeFileEntry {
    pub path: String,
    pub size_bytes: u64,
    pub is_directory: bool,
}

pub fn scan_large_files(root: &Path, min_size_bytes: u64) -> Result<LargeFilesReport> {
    let mut entries = Vec::new();

    for entry in WalkDir::new(root).follow_links(false).max_depth(20) {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        let metadata = match entry.metadata() {
            Ok(metadata) => metadata,
            Err(_) => continue,
        };

        let size_bytes = if metadata.is_dir() {
            compute_size(entry.path(), 20).unwrap_or(0)
        } else {
            metadata.len()
        };

        if size_bytes >= min_size_bytes {
            entries.push(LargeFileEntry {
                path: entry.path().display().to_string(),
                size_bytes,
                is_directory: metadata.is_dir(),
            });
        }
    }

    entries.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));

    Ok(LargeFilesReport {
        scan_root: root.display().to_string(),
        min_size: human_bytes(min_size_bytes),
        min_size_bytes,
        scan_time: Local::now(),
        entries,
    })
}
```

**Step 4: Run test to verify it passes**

Run:

```powershell
cargo test scan_large_files_discovers_entries_above_threshold
```

Expected: PASS.

**Step 5: Commit**

After fresh verification passes, commit:

```powershell
git add aidisk/src/scanner.rs
git commit -m "feat: add scan_large_files scanner"
```

---

## Task 2: Add CLI, Reporter, And Integration Tests

**Files:**
- Modify: `aidisk/src/main.rs`
- Modify: `aidisk/src/reporter.rs`
- Create: `aidisk/tests/large_files_cli.rs`

**Step 1: Write the failing integration test**

Create `aidisk/tests/large_files_cli.rs`:

```rust
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use tempfile::tempdir;

fn aidisk_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_aidisk"))
}

#[test]
fn scan_large_files_json_outputs_parseable_report() {
    let temp = tempdir().expect("tempdir should exist");
    let root = temp.path();
    fs::create_dir_all(root.join("big")).expect("big dir should exist");
    fs::write(root.join("big").join("large.bin"), vec![0_u8; 600])
        .expect("large file should write");

    let output = Command::new(aidisk_bin())
        .args([
            "scan",
            "--large-files",
            "--min-size",
            "500",
            "--root",
            root.to_str().unwrap(),
            "--json",
        ])
        .output()
        .expect("scan --large-files should run");

    assert!(
        output.status.success(),
        "scan --large-files should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let parsed: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("stdout should be parseable JSON");

    assert_eq!(parsed["scan_root"], root.display().to_string());
    assert_eq!(parsed["min_size_bytes"], 500);
    assert_eq!(parsed["entries"].as_array().unwrap().len(), 1);
    assert_eq!(parsed["entries"][0]["is_directory"], true);
}

#[test]
fn scan_large_files_filters_below_min_size() {
    let temp = tempdir().expect("tempdir should exist");
    let root = temp.path();
    fs::write(root.join("small.txt"), vec![0_u8; 10]).expect("small file should write");

    let output = Command::new(aidisk_bin())
        .args([
            "scan",
            "--large-files",
            "--min-size",
            "500",
            "--root",
            root.to_str().unwrap(),
            "--json",
        ])
        .output()
        .expect("scan --large-files should run");

    assert!(output.status.success());
    let parsed: serde_json::Value = serde_json::from_slice(&output.stdout)
        .expect("stdout should be parseable JSON");

    assert!(
        parsed["entries"].as_array().unwrap().is_empty(),
        "no entries should appear below threshold"
    );
}
```

**Step 2: Run test to verify it fails**

Run:

```powershell
cargo test --test large_files_cli
```

Expected: FAIL because `--large-files` flag does not exist or causes clap error.

**Step 3: Add CLI flags**

In `aidisk/src/main.rs`, add to the `Command::Scan` variant:

```rust
#[arg(long)]
large_files: bool,
#[arg(long, default_value_t = parse_size::parse_size("500MB").unwrap())]
min_size: u64,
#[arg(long)]
root: Option<PathBuf>,
```

The `min_size` default is 500 MB in bytes. For a simpler approach without a parse_size crate, accept raw bytes integer with a default of 524288000 (500 MB).

```rust
#[arg(long)]
large_files: bool,
#[arg(long, default_value_t = 524_288_000)]
min_size: u64,
#[arg(long)]
root: Option<PathBuf>,
```

In the `Command::Scan` match arm, add before the existing scan logic:

```rust
if large_files {
    let root = root.unwrap_or_else(large_files_default_root);
    let report = scanner::scan_large_files(&root, min_size)?;
    println!("{}", reporter::render_large_files(&report, effective_format)?);
    return Ok(());
}
```

Add:

```rust
fn large_files_default_root() -> PathBuf {
    std::env::var_os("USERPROFILE")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("C:\\"))
}
```

**Step 4: Add reporter**

Add to `aidisk/src/reporter.rs`:

```rust
use crate::scanner::LargeFilesReport;

pub fn render_large_files(report: &LargeFilesReport, format: OutputFormat) -> Result<String> {
    let output = match format {
        OutputFormat::Json => serde_json::to_string_pretty(report)?,
        OutputFormat::Markdown => render_large_files_markdown(report),
        OutputFormat::Text => render_large_files_text(report),
    };

    Ok(output)
}

fn render_large_files_text(report: &LargeFilesReport) -> String {
    let mut lines = vec![
        "Large Files Discovery".to_string(),
        format!("Scan Root: {}", report.scan_root),
        format!("Min Size: {}", report.min_size),
        format!("Scan Time: {}", report.scan_time),
        format!("Entries: {}", report.entries.len()),
        String::new(),
    ];

    for entry in &report.entries {
        lines.push(format!(
            "{} {} {}",
            if entry.is_directory { "[DIR]" } else { "[FILE]" },
            human_bytes(entry.size_bytes),
            entry.path
        ));
    }

    lines.join("\n")
}

fn render_large_files_markdown(report: &LargeFilesReport) -> String {
    let mut lines = vec![
        "# Large Files Discovery".to_string(),
        String::new(),
        format!("- Scan Root: `{}`", report.scan_root),
        format!("- Min Size: {}", report.min_size),
        format!("- Scan Time: {}", report.scan_time),
        format!("- Entries: {}", report.entries.len()),
        String::new(),
        "| Type | Size | Path |".to_string(),
        "|---|---|---|".to_string(),
    ];

    for entry in &report.entries {
        lines.push(format!(
            "| {} | {} | `{}` |",
            if entry.is_directory { "DIR" } else { "FILE" },
            human_bytes(entry.size_bytes),
            entry.path
        ));
    }

    lines.join("\n")
}
```

**Step 5: Run integration tests**

Run:

```powershell
cargo test --test large_files_cli
```

Expected: PASS.

**Step 6: Run full suite**

Run:

```powershell
cargo test
```

Expected: PASS.

**Step 7: Commit**

After fresh verification passes, commit:

```powershell
git add aidisk/src/main.rs aidisk/src/reporter.rs aidisk/tests/large_files_cli.rs
git commit -m "feat: add scan --large-files cli and reporter"
```

---

## Task 3: Document Large Files Feature

**Files:**
- Modify: `README.md`
- Modify: `README.zh-CN.md`
- Modify: `CHANGELOG.md`
- Modify: `docs/execution-plan.md`

**Step 1: Update docs**

In `README.md`, add to the key features table or What's New section:

```
### What's New (Unreleased)

- **Large Files Discovery** — `scan --large-files --min-size 500MB` discovers the largest files and directories under a root path with no classification or cleanup suggestions.
```

In `README.md`, add to the Command Reference table:

```
| `scan --large-files` | Discover largest files and directories | `--min-size`, `--root`, `--json`, `--markdown` |
```

In `README.zh-CN.md`, add Chinese equivalents.

In `CHANGELOG.md`, add under `## Unreleased`:

```markdown
- Added `scan --large-files --min-size <SIZE>` for lightweight large file and directory discovery without classification or cleanup suggestions.
```

In `docs/execution-plan.md`, mark Phase 7 P2 as completed.

**Step 2: Run full verification**

Run:

```powershell
cargo test --test large_files_cli
cargo test
```

Expected: PASS.

**Step 3: Commit**

After fresh verification passes, commit:

```powershell
git add README.md README.zh-CN.md CHANGELOG.md docs/execution-plan.md
git commit -m "docs: document large files discovery feature"
```

---

## Final Verification

Run from `aidisk`:

```powershell
cargo test
cargo build --release
cargo run -- scan --large-files --min-size 500MB --json
```

Expected results:

- Full tests pass.
- Release build succeeds.
- `scan --large-files --json` emits parseable JSON with the expected schema.

## Open Questions

None. This slice intentionally avoids classification, cleanup suggestions, and cross-platform path expansion (Phase 7 P3).
