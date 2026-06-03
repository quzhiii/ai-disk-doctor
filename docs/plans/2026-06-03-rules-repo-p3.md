# Rules Repo P3 Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Complete the P3 roadmap milestone by adding `--rules-repo` so `aidisk` can load local or HTTPS git-hosted community rule repositories safely.

**Architecture:** Add a `rules_repo` module that resolves a user-provided source into a concrete rules directory. Local paths are loaded directly, preferring `<repo>/rules` when present. HTTPS git URLs are validated, cloned into `.aidisk/rules-repos/<stable-id>`, and then resolved the same way. CLI commands keep `--rules-dir` for direct directory overrides and add `--rules-repo` for repository-style sources.

**Tech Stack:** Rust, Clap CLI, std::process for `git clone`, serde_yaml rule loading, existing PowerShell wrapper scripts.

---

### Task 1: Rules Repo Resolution Tests

**Files:**
- Create: `aidisk/src/rules_repo.rs`
- Modify: `aidisk/src/main.rs`
- Test: `aidisk/src/rules_repo.rs`

**Step 1: Write failing tests**

Add tests for:

```rust
resolve_rules_repo(local_repo_with_rules_subdir, cache_root) -> local_repo/rules
resolve_rules_repo(local_repo_with_yaml_at_root, cache_root) -> local_repo
validate_rules_repo_url("https://github.com/example/rules.git") -> ok
validate_rules_repo_url("http://example.com/rules.git") -> err
validate_rules_repo_url("file:///C:/secret") -> err
```

**Step 2: Run RED**

Run: `cargo test rules_repo::tests -- --nocapture`

Expected: failure because the module/functions do not exist.

**Step 3: Implement local resolution and URL validation**

Create:

```rust
pub fn default_rules_repo_cache_root() -> PathBuf
pub fn resolve_rules_repo(source: &str, cache_root: &Path) -> Result<PathBuf>
pub fn validate_rules_repo_url(source: &str) -> Result<()> 
```

For local paths, accept existing directories only. Prefer `rules/` if it exists, otherwise use the directory itself. For URLs, allow only `https://` sources and reject localhost/private/file/http schemes.

**Step 4: Verify GREEN**

Run: `cargo test rules_repo::tests -- --nocapture`

Expected: tests pass.

### Task 2: Remote Clone Support And CLI Wiring

**Files:**
- Modify: `aidisk/src/rules_repo.rs`
- Modify: `aidisk/src/main.rs`

**Step 1: Add clone support**

For HTTPS URLs, derive a stable cache directory name from a sanitized URL string, clone with:

```rust
git clone --depth 1 <url> <cache-dir>
```

If the cache directory already exists, reuse it for this milestone. Do not auto-update or pull yet.

**Step 2: Add `--rules-repo` to commands**

Add `rules_repo: Option<String>` to `scan`, `plan`, `clean`, and `doctor`.

Create a helper:

```rust
fn resolve_rules_dir(rules_dir: Option<PathBuf>, rules_repo: Option<String>) -> Result<PathBuf>
```

`--rules-dir` wins over `--rules-repo`. If neither is present, use bundled rules.

**Step 3: Verify command help**

Run: `cargo run -- scan --help`

Expected: help includes `--rules-repo`.

### Task 3: Docs, Wrappers, And Verification

**Files:**
- Modify: `README.md`
- Modify: `docs/execution-plan.md`
- Modify: `skills/windows-ai-space-manager/SKILL.md`
- Modify: `skills/windows-ai-space-manager/references/workflow.md`
- Modify: `skills/windows-ai-space-manager/scripts/run-scan.ps1`
- Modify: `skills/windows-ai-space-manager/scripts/run-plan.ps1`
- Modify: `skills/windows-ai-space-manager/scripts/run-clean-dry-run.ps1`
- Modify: `skills/windows-ai-space-manager/scripts/run-clean.ps1`
- Modify: `skills/windows-ai-space-manager/scripts/run-doctor.ps1`
- Modify: `aidisk/tests/skill_artifacts.rs`

**Step 1: Update wrappers**

Add optional `[string]$RulesRepo` to wrappers that invoke commands with rule loading, and pass `--rules-repo` when set.

**Step 2: Update docs and artifact test**

Document examples:

```powershell
cargo run -- scan --rules-repo "C:\path\to\rules-repo" --json
cargo run -- scan --rules-repo "https://github.com/example/windows-ai-space-rules.git" --json
```

Add artifact assertions that `SKILL.md` and wrappers mention `RulesRepo`.

**Step 3: Full verification**

Run:

```bash
cargo test
cargo run -- scan --rules-repo tests/fixtures/community-rules --json
pwsh -NoProfile -File "..\skills\windows-ai-space-manager\scripts\run-scan.ps1" -RulesRepo "tests/fixtures/community-rules" -Json
```

Expected: all tests pass; local rules repo scan succeeds.

**Step 4: Commit**

Run:

```bash
git add aidisk/src/rules_repo.rs aidisk/src/main.rs aidisk/tests/skill_artifacts.rs aidisk/tests/fixtures/community-rules README.md docs/execution-plan.md skills/windows-ai-space-manager/SKILL.md skills/windows-ai-space-manager/references/workflow.md skills/windows-ai-space-manager/scripts/run-scan.ps1 skills/windows-ai-space-manager/scripts/run-plan.ps1 skills/windows-ai-space-manager/scripts/run-clean-dry-run.ps1 skills/windows-ai-space-manager/scripts/run-clean.ps1 skills/windows-ai-space-manager/scripts/run-doctor.ps1 docs/plans/2026-06-03-rules-repo-p3.md
git commit -m "add rules repo support for community rule sets"
```
