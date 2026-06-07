# P7 Cross-Platform Rules Adaptation Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add Unix-style `~` path expansion and linux/macOS paths to existing AI tooling rules so the same rule files work across platforms without changing scan or planner core logic.

**Architecture:** Rename `expand_windows_path` to `expand_path`, add `~` -> `$HOME` expansion alongside existing `%VAR%` expansion (which is harmless to run on Unix), then add `linux` and `macos` path entries to existing ollama, huggingface, and docker rules.

**Tech Stack:** Rust 2021, Cargo, unit tests.

---

## Guardrails

- Do not change scanner, planner, cleaner, or reporter.
- Do not introduce platform-specific scanning branches or filtering.
- `platform` field in rules remains informational; scanning does not filter by it.
- Existing Windows paths in rules are not modified.
- Unix path expansion uses `$HOME` environment variable; fallback to `/tmp` if absent (for CI/docker).

## Milestones

### Milestone 0: Plan And Baseline

**Verification command:**

```powershell
cargo test
```

Run from `aidisk`.

### Milestone A: Path Expansion

**Tasks:** 1

**Verification command:**

```powershell
cargo test rules::tests::expand_path
```

### Milestone B: Cross-Platform Rule Paths

**Tasks:** 2

**Verification command:**

```powershell
cargo test --test scan_smoke
```

### Milestone C: Docs

**Tasks:** 3

**Verification command:**

```powershell
cargo test
```

---

## Task 1: Add Unix Path Expansion

**Files:**
- Modify: `aidisk/src/rules.rs`
- Modify: `aidisk/src/scanner.rs`

**Step 1: Write the failing test**

Add to `aidisk/src/rules.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expand_path_handles_home_tilde() {
        std::env::set_var("HOME", "/home/demo");
        let result = expand_path("~/.cache/huggingface");
        assert_eq!(result, PathBuf::from("/home/demo/.cache/huggingface"));
    }

    #[test]
    fn expand_path_preserves_tilde_without_home() {
        std::env::remove_var("HOME");
        let result = expand_path("~/unknown");
        assert_eq!(result, PathBuf::from("/tmp/unknown"));
    }

    #[test]
    fn expand_path_keeps_windows_var_expansion() {
        std::env::set_var("USERPROFILE", "C:\\Users\\demo");
        let result = expand_path("%USERPROFILE%\\.cache\\huggingface");
        assert_eq!(
            result,
            PathBuf::from("C:\\Users\\demo\\.cache\\huggingface")
        );
    }

    #[test]
    fn expand_path_preserves_unchanged_paths() {
        let result = expand_path("/usr/local/bin/tool");
        assert_eq!(result, PathBuf::from("/usr/local/bin/tool"));
    }
}
```

**Step 2: Run to verify RED**

```powershell
cargo test expand_path_handles_home_tilde
```

**Step 3: Implement minimal expand_path**

```rust
pub fn expand_path(pattern: &str) -> PathBuf {
    let mut expanded = pattern.to_owned();

    if expanded.starts_with("~/") {
        if let Ok(home) = env::var("HOME") {
            expanded = expanded.replacen("~", &home, 1);
        } else {
            expanded = expanded.replacen("~", "/tmp", 1);
        }
    }

    for (key, value) in env::vars() {
        let token = format!("%{key}%");
        if expanded.contains(&token) {
            expanded = expanded.replace(&token, &value);
        }
    }

    PathBuf::from(expanded)
}
```

Keep `expand_windows_path` as a thin wrapper calling `expand_path` for backward compat, then update the single call site in scanner.rs:

```rust
pub fn expand_windows_path(pattern: &str) -> PathBuf {
    expand_path(pattern)
}
```

**Step 4: Run tests**

```powershell
cargo test expand_path
```

**Step 5: Commit**

```powershell
git add aidisk/src/rules.rs aidisk/src/scanner.rs
git commit -m "feat: add unix path expansion"
```

---

## Task 2: Add Cross-Platform Rule Paths

**Files:**
- Modify: `aidisk/rules/models.yaml`
- Modify: `aidisk/rules/huggingface.yaml`
- Modify: `aidisk/rules/docker.yaml`
- Modify: `aidisk/tests/scan_smoke.rs`

**Step 1: Write the failing test**

Add to `aidisk/tests/scan_smoke.rs`:

```rust
#[test]
fn loads_cross_platform_rule_paths() {
    let models = fs::read_to_string("rules/models.yaml").expect("models rule should exist");
    let huggingface = fs::read_to_string("rules/huggingface.yaml").expect("huggingface rule should exist");
    let docker = fs::read_to_string("rules/docker.yaml").expect("docker rule should exist");

    assert!(models.contains("~/.ollama"), "models should include unix ollama path");
    assert!(huggingface.contains("~/.cache/huggingface"), "huggingface should include unix path");
    assert!(docker.contains("~/.docker"), "docker should include unix path");
}
```

**Step 2: Run to verify RED**

```powershell
cargo test --test scan_smoke loads_cross_platform_rule_paths
```

**Step 3: Add unix paths to existing rules**

In `aidisk/rules/models.yaml`, add to paths:
```yaml
  - "~/.ollama/models"
  - "~/.ollama/models/blobs"
```

In `aidisk/rules/huggingface.yaml`, add to paths:
```yaml
  - "~/.cache/huggingface"
  - "~/.cache/huggingface/hub"
```

In `aidisk/rules/docker.yaml`, add to paths:
```yaml
  - "~/.docker"
```

Do not change existing Windows paths, `platform` field, or any other rule metadata.

**Step 4: Run tests**

```powershell
cargo test --test scan_smoke
```

**Step 5: Commit**

```powershell
git add aidisk/rules/models.yaml aidisk/rules/huggingface.yaml aidisk/rules/docker.yaml aidisk/tests/scan_smoke.rs
git commit -m "feat: add unix paths to ai tooling rules"
```

---

## Task 3: Document Cross-Platform Adaptation

**Files:**
- Modify: `README.md`
- Modify: `README.zh-CN.md`
- Modify: `CHANGELOG.md`
- Modify: `docs/execution-plan.md`

**Step 1: Update docs**

In `README.md`, add to key features:
```markdown
- **Cross-platform path expansion** — rules now support Unix `~/` home directory paths alongside Windows `%VAR%` expansion.
```

In `README.zh-CN.md`, add Chinese equivalent.

In `CHANGELOG.md`, add under Unreleased.

In `docs/execution-plan.md`, mark Phase 7 P3 as completed and add next steps.

**Step 2: Run full verification**

```powershell
cargo test
```

**Step 3: Commit**

```powershell
git add README.md README.zh-CN.md CHANGELOG.md docs/execution-plan.md
git commit -m "docs: document cross-platform path expansion"
```
