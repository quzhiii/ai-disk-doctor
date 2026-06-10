# v1.4.0 Cross-Platform Governance Release Readiness Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Promote the completed cross-platform scheduler adapters and Unix governance entrypoint into a fully verified v1.4.0 release-ready state.

**Architecture:** Keep v1.4.0 as a release-hardening slice, not a new product-feature slice. The implementation updates release artifacts, documentation, version metadata, and smoke verification while preserving the existing Rust governance core, stable `governance-event.json` contract, and scheduler-first boundary. Concrete IM notifier adapters remain out of scope.

**Tech Stack:** Rust/Cargo, Markdown release artifacts, PowerShell release smoke script, Bash governance scripts, existing release artifact tests.

---

## Scope

- Formalize the current Unreleased cross-platform governance work as v1.4.0.
- Update tests first so missing release artifacts fail before documentation/version changes.
- Update README, README.zh-CN, CHANGELOG, release notes, roadmap, and version metadata.
- Run targeted release artifact tests and full Rust test suite before completion.

## Non-Goals

- Do not add Feishu / Slack / WeChat / DingTalk notifier adapters.
- Do not change Rust anomaly behavior or `governance-event.json` schema.
- Do not introduce a background daemon.
- Do not add destructive cleanup automation to governance.

---

### Task 1: Add v1.4.0 Release Artifact Test

**Files:**
- Modify: `aidisk/tests/release_artifacts.rs`

**Step 1: Write the failing test**

Add this test after `changelog_and_release_notes_cover_v1_3_scope`:

```rust
#[test]
fn changelog_readmes_and_release_notes_cover_v1_4_scope() {
    let changelog = read_repo_file("CHANGELOG.md");
    let release_notes = read_repo_file("docs/release-notes/v1.4.0.md");
    let readme = read_repo_file("README.md");
    let readme_zh = read_repo_file("README.zh-CN.md");
    let roadmap = read_repo_file("docs/execution-plan.md");
    let cargo_toml = read_repo_file("aidisk/Cargo.toml");
    let cargo_lock = read_repo_file("aidisk/Cargo.lock");

    let required_terms = [
        "Cross-Platform Scheduled Governance",
        "cron",
        "launchd",
        "systemd timer",
        "run-governance.sh",
        "run-governance.ps1",
        "governance-event.json",
        "generic webhook",
        "bash",
        "jq",
        "curl",
        "no background daemon",
        "notifier adapter",
    ];

    assert!(changelog.contains("## 1.4.0"));
    assert!(release_notes.contains("# Windows AI Space Manager v1.4.0"));
    assert!(release_notes.contains("## Test Plan"));
    assert!(release_notes.contains("## Safety Boundaries"));
    assert!(release_notes.contains("## Known Limits"));
    assert!(readme.contains("v1.4.0"));
    assert!(readme_zh.contains("v1.4.0"));
    assert!(roadmap.contains("Phase 12 status: Completed"));
    assert!(cargo_toml.contains("version = \"1.4.0\""));
    assert!(cargo_lock.contains("name = \"aidisk\"\nversion = \"1.4.0\""));

    for term in required_terms {
        assert!(changelog.contains(term), "CHANGELOG.md should mention {term}");
        assert!(release_notes.contains(term), "release notes should mention {term}");
        assert!(readme.contains(term), "README.md should mention {term}");
    }

    assert!(
        readme_zh.contains("cron")
            && readme_zh.contains("launchd")
            && readme_zh.contains("systemd timer")
            && readme_zh.contains("run-governance.sh"),
        "Chinese README should cover v1.4.0 scheduler and Unix governance entrypoint"
    );
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test changelog_readmes_and_release_notes_cover_v1_4_scope`

Expected: FAIL because `docs/release-notes/v1.4.0.md` does not exist and version metadata is still `1.3.0`.

**Step 3: Commit**

```bash
git add aidisk/tests/release_artifacts.rs
git commit -m "test: add v1.4.0 release artifact coverage"
```

---

### Task 2: Add v1.4.0 Release Notes

**Files:**
- Create: `docs/release-notes/v1.4.0.md`

**Step 1: Create release notes**

Create a Markdown file with:

```markdown
# Windows AI Space Manager v1.4.0

v1.4.0 packages Cross-Platform Scheduled Governance. It extends the v1.3.0 Windows Task Scheduler governance loop to cron, launchd, and systemd timer, and adds `run-governance.sh` as the Unix-like governance entrypoint.

## What Changed

- Added cron scheduler helpers under `scripts/governance/cron/` for register, show, unregister, and test-run workflows.
- Added macOS launchd helpers under `scripts/governance/launchd/` using `.plist` and `launchctl`.
- Added Linux systemd timer helpers under `scripts/governance/systemd/` using `.service` and `.timer` units.
- Added `scripts/governance/run-governance.sh` for Unix-like scan -> anomaly -> governance-event.json -> generic webhook workflows.
- Preserved the Windows `run-governance.ps1` workflow and stable `governance-event.json` contract.

## Test Plan

- `cargo test --test release_artifacts`
- `cargo test --all`
- Static release artifact coverage verifies `run-governance.ps1`, `run-governance.sh`, cron, launchd, and systemd timer scripts.

## Safety Boundaries

- Cross-Platform Scheduled Governance does not perform cleanup.
- Scheduler adapters do not run as a background daemon.
- Generic webhook remains the delivery boundary; concrete notifier adapter expansion is reserved for later.
- `run-governance.sh` requires `bash`, `jq`, `curl` for webhook delivery, and `cargo` for local governance runs.

## Known Limits

- cron does not expose first-class last/next run metadata like Windows Task Scheduler or systemd timer.
- launchd and systemd timer scripts are static contract-tested in this repository; full native scheduler execution should be verified on macOS and Linux hosts.
- Concrete IM notifier adapter support such as Feishu, Slack, WeChat, DingTalk, or email is not included in v1.4.0.
```

**Step 2: Run test**

Run: `cargo test changelog_readmes_and_release_notes_cover_v1_4_scope`

Expected: still FAIL because README, CHANGELOG, roadmap, and version metadata are not updated yet.

**Step 3: Commit**

```bash
git add docs/release-notes/v1.4.0.md
git commit -m "docs: add v1.4.0 release notes"
```

---

### Task 3: Promote CHANGELOG and Roadmap to v1.4.0

**Files:**
- Modify: `CHANGELOG.md`
- Modify: `docs/execution-plan.md`

**Step 1: Update CHANGELOG**

Change `## Unreleased` to empty and add `## 1.4.0` with existing cross-platform governance entries.

Required terms: `Cross-Platform Scheduled Governance`, `cron`, `launchd`, `systemd timer`, `run-governance.sh`, `run-governance.ps1`, `governance-event.json`, `generic webhook`, `bash`, `jq`, `curl`, `no background daemon`, `notifier adapter`.

**Step 2: Update roadmap**

In `docs/execution-plan.md`:
- Change `Unreleased / v1.4.0 candidate` to `v1.4.0`.
- Change `Phase 12 status: Recommended next` to `Phase 12 status: Completed`.
- Replace progress estimate with completion notes: README/release notes/version/test artifacts completed.

**Step 3: Run test**

Run: `cargo test changelog_readmes_and_release_notes_cover_v1_4_scope`

Expected: still FAIL because README and version metadata are not updated yet.

**Step 4: Commit**

```bash
git add CHANGELOG.md docs/execution-plan.md
git commit -m "docs: promote cross-platform governance to v1.4.0 roadmap"
```

---

### Task 4: Update README and README.zh-CN for v1.4.0

**Files:**
- Modify: `README.md`
- Modify: `README.zh-CN.md`

**Step 1: Update English README**

- Version badge: `1.4.0`.
- Current release: `v1.4.0`.
- Add a new `### v1.4.0` What's New section before v1.3.0.
- Add Key Feature row for Cross-Platform Scheduled Governance.
- Mention cron / launchd / systemd timer, `run-governance.sh`, `run-governance.ps1`, `governance-event.json`, generic webhook, no background daemon, notifier adapter boundary, and dependency words `bash`, `jq`, `curl`.
- Link to `docs/release-notes/v1.4.0.md`.

**Step 2: Update Chinese README**

- Version badge: `1.4.0`.
- 当前版本：`v1.4.0`.
- Add a new `### v1.4.0` 最新动态 section before v1.3.0.
- Add 核心特性 row for 跨平台定时治理.
- Mention cron / launchd / systemd timer and `run-governance.sh`.
- Link to `docs/release-notes/v1.4.0.md`.

**Step 3: Run test**

Run: `cargo test changelog_readmes_and_release_notes_cover_v1_4_scope`

Expected: still FAIL because Cargo version metadata is not updated yet.

**Step 4: Commit**

```bash
git add README.md README.zh-CN.md
git commit -m "docs: update READMEs for v1.4.0 governance release"
```

---

### Task 5: Bump Crate Version to v1.4.0

**Files:**
- Modify: `aidisk/Cargo.toml`
- Modify: `aidisk/Cargo.lock`

**Step 1: Update Cargo.toml**

Change package version from `1.3.0` to `1.4.0`.

**Step 2: Update Cargo.lock**

Run: `cargo update -p aidisk --precise 1.4.0` from `aidisk/` if it works, otherwise run `cargo check` after editing `Cargo.toml` so Cargo.lock refreshes the package version.

**Step 3: Run test**

Run: `cargo test changelog_readmes_and_release_notes_cover_v1_4_scope`

Expected: PASS.

**Step 4: Commit**

```bash
git add aidisk/Cargo.toml aidisk/Cargo.lock
git commit -m "chore: bump aidisk version to v1.4.0"
```

---

### Task 6: Final Release Verification

**Files:**
- Test only

**Step 1: Run release artifact tests**

Run: `cargo test --test release_artifacts`

Expected: PASS.

**Step 2: Run full test suite**

Run: `cargo test --all`

Expected: PASS.

**Step 3: Inspect git status and logs**

Run:

```bash
git status --short --branch
git log --oneline -10
```

**Step 4: Commit any final roadmap-only cleanup if needed**

Only if verification exposed docs drift.

---

## Execution Notes

- Use TDD strictly for release artifact coverage.
- Keep v1.4.0 focused on cross-platform scheduled governance release readiness.
- Do not add concrete IM notifier adapters.
- Do not modify Rust governance behavior beyond version metadata.
- Commit after each task.
