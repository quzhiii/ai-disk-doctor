# P7 Rule Coverage Expansion Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Expand built-in rule coverage for common development artifacts so existing `scan` and `plan` workflows immediately catch high-volume developer machine bloat.

**Architecture:** Add rule-only coverage through a new `dev-artifacts` rules file, repository fixtures, and documentation. Do not change scanner, planner, cleaner, reporter, or JSON schemas; the existing YAML rules loader and category-map artifact tests should enforce consistency.

**Tech Stack:** Rust 2021, Cargo integration tests, YAML rule files, repository fixtures, Markdown docs.

---

## Guardrails

- Do not change scan/planner core logic.
- Do not add `scan --large-files` in this slice; that is Phase 7 P2.
- Do not add Linux/macOS path expansion in this slice; that is Phase 7 P3.
- Do not introduce automatic cleanup execution. New rules should use conservative existing actions.
- Keep successful JSON schemas unchanged.
- Use strict TDD: write failing tests first, verify RED, implement minimal rules/docs, then verify GREEN.

## Coverage Scope

This slice adds Windows/developer-machine rules for:

- `node_modules`
- Rust `target/`
- Gradle `.gradle`
- Python `__pycache__`
- build output `dist/`
- Next.js `.next/`
- Turborepo `.turbo/`

Recommended classification:

- Category: `dev-artifact`
- Risk: `safe`
- Cleanup method: `quarantine`
- Reason: generated dependency/build/test cache artifacts that can usually be regenerated

## Milestones

### Milestone 0: Plan And Baseline

**Files:**
- Create: `docs/plans/2026-06-05-p7-rule-coverage-expansion.md`

**Verification command:**

```powershell
cargo test
```

Run from `aidisk`.

**Expected result:** Baseline full suite passes before implementation starts.

**Commit point:** Commit the plan after baseline verification if only the plan changed.

### Milestone A: Rule File And Loader Coverage

**Tasks:** 1

**Verification command:**

```powershell
cargo test --test scan_smoke loads_common_dev_artifact_rule_yaml
```

Run from `aidisk`.

**Expected result:** New rule file exists and contains the planned development artifact patterns.

**Commit point:** Commit after Task 1 passes.

### Milestone B: Fixtures And Category Map

**Tasks:** 2

**Verification command:**

```powershell
cargo test --test scan_category repository_fixtures_cover_common_dev_artifacts
cargo test --test skill_artifacts category_map_covers_rule_categories
```

Run from `aidisk`.

**Expected result:** Repository fixtures cover new artifact shapes and category map includes `dev-artifact`.

**Commit point:** Commit after Task 2 passes.

### Milestone C: Docs And Full Verification

**Tasks:** 3

**Verification command:**

```powershell
cargo test --test scan_smoke
cargo test --test scan_category
cargo test --test skill_artifacts
cargo test
```

Run from `aidisk`.

**Expected result:** Focused tests and full suite pass.

**Commit point:** Commit after Task 3 passes.

---

## Task 1: Add Common Development Artifact Rule File

**Files:**
- Create: `aidisk/rules/dev-artifacts.yaml`
- Modify: `aidisk/tests/scan_smoke.rs`

**Step 1: Write the failing test**

Add this test to `aidisk/tests/scan_smoke.rs`:

```rust
#[test]
fn loads_common_dev_artifact_rule_yaml() {
    let content = fs::read_to_string("rules/dev-artifacts.yaml")
        .expect("dev artifact rule should exist");

    for term in [
        "node_modules",
        "target",
        ".gradle",
        "__pycache__",
        "dist",
        ".next",
        ".turbo",
        "category: dev-artifact",
    ] {
        assert!(content.contains(term), "dev artifact rules should include {term}");
    }
}
```

**Step 2: Run test to verify it fails**

Run:

```powershell
cargo test --test scan_smoke loads_common_dev_artifact_rule_yaml
```

Expected: FAIL because `rules/dev-artifacts.yaml` does not exist yet.

**Step 3: Create minimal rule file**

Create `aidisk/rules/dev-artifacts.yaml`:

```yaml
id: common-dev-artifacts
name: Common development artifacts
category: dev-artifact
platform: windows
paths:
  - "%USERPROFILE%\\**\\node_modules"
  - "%USERPROFILE%\\**\\target"
  - "%USERPROFILE%\\**\\.gradle"
  - "%USERPROFILE%\\**\\__pycache__"
  - "%USERPROFILE%\\**\\dist"
  - "%USERPROFILE%\\**\\.next"
  - "%USERPROFILE%\\**\\.turbo"
risk: safe
cleanup:
  method: quarantine
reason: "Generated dependency, build, and framework cache artifacts can often be recreated. Review project context before cleanup."
warnings:
  - "Some projects may rely on generated artifacts for offline work; quarantine first and restore if needed."
```

Rationale:

- One rule file is enough for this slice; no core code changes.
- `risk: safe` makes the rule eligible for `plan --safe-only`.
- `quarantine` preserves reversibility.

**Step 4: Run test to verify it passes**

Run:

```powershell
cargo test --test scan_smoke loads_common_dev_artifact_rule_yaml
```

Expected: PASS.

**Step 5: Commit**

After fresh verification passes, commit:

```powershell
git add aidisk/rules/dev-artifacts.yaml aidisk/tests/scan_smoke.rs
git commit -m "feat: add common dev artifact rules"
```

---

## Task 2: Add Fixtures And Category Map Coverage

**Files:**
- Modify: `aidisk/tests/scan_category.rs`
- Create fixture directories/files under `aidisk/tests/fixtures/windows-user/projects/dev-artifacts-demo/`
- Modify: `skills/windows-ai-space-manager/references/category-map.md`

**Step 1: Write the failing fixture test**

Add this test to `aidisk/tests/scan_category.rs`:

```rust
#[test]
fn repository_fixtures_cover_common_dev_artifacts() {
    let root = Path::new("tests/fixtures/windows-user/projects/dev-artifacts-demo");

    assert!(root.join("node_modules/pkg/index.js").exists());
    assert!(root.join("target/debug/app.exe").exists());
    assert!(root.join(".gradle/caches/modules-2/module.bin").exists());
    assert!(root.join("src/__pycache__/module.pyc").exists());
    assert!(root.join("dist/app.js").exists());
    assert!(root.join(".next/cache/build.bin").exists());
    assert!(root.join(".turbo/cache/hash.bin").exists());
}
```

**Step 2: Run test to verify it fails**

Run:

```powershell
cargo test --test scan_category repository_fixtures_cover_common_dev_artifacts
```

Expected: FAIL because fixture files do not exist yet.

**Step 3: Create minimal fixture files**

Create these files with tiny placeholder content:

```text
aidisk/tests/fixtures/windows-user/projects/dev-artifacts-demo/node_modules/pkg/index.js
aidisk/tests/fixtures/windows-user/projects/dev-artifacts-demo/target/debug/app.exe
aidisk/tests/fixtures/windows-user/projects/dev-artifacts-demo/.gradle/caches/modules-2/module.bin
aidisk/tests/fixtures/windows-user/projects/dev-artifacts-demo/src/__pycache__/module.pyc
aidisk/tests/fixtures/windows-user/projects/dev-artifacts-demo/dist/app.js
aidisk/tests/fixtures/windows-user/projects/dev-artifacts-demo/.next/cache/build.bin
aidisk/tests/fixtures/windows-user/projects/dev-artifacts-demo/.turbo/cache/hash.bin
```

Use simple text like `fixture` or `binary fixture`; no large files are needed.

**Step 4: Update category map**

Add `dev-artifact` to `skills/windows-ai-space-manager/references/category-map.md` with a concise description:

```markdown
- `dev-artifact`: Regenerable development artifacts such as `node_modules`, Rust `target/`, Gradle caches, Python `__pycache__`, and web build caches.
```

**Step 5: Run focused tests**

Run:

```powershell
cargo test --test scan_category repository_fixtures_cover_common_dev_artifacts
cargo test --test skill_artifacts category_map_covers_rule_categories
```

Expected: PASS.

**Step 6: Commit**

After fresh verification passes, commit:

```powershell
git add aidisk/tests/scan_category.rs aidisk/tests/fixtures/windows-user/projects/dev-artifacts-demo skills/windows-ai-space-manager/references/category-map.md
git commit -m "test: cover common dev artifact fixtures"
```

---

## Task 3: Document Rule Coverage And Verify Full Suite

**Files:**
- Modify: `README.md`
- Modify: `README.zh-CN.md`
- Modify: `CHANGELOG.md`
- Modify: `docs/execution-plan.md`

**Step 1: Update docs**

In `README.md`, update the key features or overview text to mention common developer artifacts:

```markdown
- **Developer artifact coverage** — detects common regenerable artifacts such as `node_modules`, Rust `target/`, Gradle caches, Python `__pycache__`, `dist/`, `.next/`, and `.turbo`.
```

In `README.zh-CN.md`, add the equivalent Chinese bullet:

```markdown
- **开发产物覆盖** — 识别 `node_modules`、Rust `target/`、Gradle 缓存、Python `__pycache__`、`dist/`、`.next/`、`.turbo` 等常见可再生成产物。
```

In `CHANGELOG.md`, add under `## Unreleased`:

```markdown
- Added built-in rules for common development artifacts including `node_modules`, Rust `target/`, Gradle caches, Python `__pycache__`, web `dist/`, `.next`, and `.turbo` caches.
```

In `docs/execution-plan.md`, mark Phase 7 P1 as completed or partially completed:

```markdown
| P1 | 扩大规则覆盖面 | Completed: 内置规则覆盖 ... |
```

**Step 2: Run focused tests**

Run:

```powershell
cargo test --test scan_smoke
cargo test --test scan_category
cargo test --test skill_artifacts
```

Expected: PASS.

**Step 3: Run full verification**

Run:

```powershell
cargo test
```

Expected: PASS.

**Step 4: Commit**

After fresh verification passes, commit:

```powershell
git add README.md README.zh-CN.md CHANGELOG.md docs/execution-plan.md
git commit -m "docs: document dev artifact coverage"
```

---

## Final Verification

Run from `aidisk`:

```powershell
cargo test
cargo run -- scan --json
cargo run -- plan --safe-only --json
```

Expected results:

- Full tests pass.
- `scan --json` remains parseable.
- `plan --safe-only --json` remains parseable and can include `dev-artifact` findings when matching paths exist.

## Open Questions

None for this slice. `scan --large-files` and cross-platform path support remain explicit later roadmap items.
