# Doctor V2 Dynamic Topic Registry Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace the current hardcoded doctor topic assembly with a registry-driven implementation while preserving the existing CLI flags, JSON shape, probe behavior, and default read-only semantics.

**Architecture:** Introduce a single `DoctorTopicSpec` registry in `aidisk/src/doctor.rs` that owns topic name, default enablement, matching logic, base recommendations, and optional probe metadata. Keep `main.rs` as the CLI front door, but move topic selection/defaulting to registry-aware helpers so default doctor runs and explicit flags are derived from the same source of truth. Preserve `DoctorReport`, `DoctorTopic`, and reporter output so downstream skills and automation do not need to change.

**Tech Stack:** Rust CLI, `clap`, existing `ScanReport` / `Finding` model, serde JSON/Markdown/Text reporting, existing doctor probe runner and CLI integration tests.

---

## Scope For This Slice

- Keep existing flags: `--docker`, `--wsl`, `--ollama`, `--playwright`, `--huggingface`, `--agents`
- Keep existing doctor output schema and formatting behavior
- Keep `--probe-tools` opt-in only
- Keep `doctor --latest` / `--reports-dir` behavior unchanged
- Remove the hardcoded topic-building `if options.* { ... }` ladder from `build_doctor_with_probe_runner`

## Non-Goals For This Slice

- Do not add a new public `--topic <name>` CLI yet
- Do not externalize registry metadata into YAML yet
- Do not change cleanup semantics or let doctor mutate files
- Do not redesign reporter output unless tests prove it is necessary

## Acceptance Criteria

- Default doctor topic enablement is derived from registry metadata instead of duplicated `main.rs` booleans
- `build_doctor` builds topics by iterating a registry, not by hardcoded per-topic branches
- Existing explicit flags still work exactly as before
- Probe commands are still only attached to topics that define them, and still only when `--probe-tools` is set
- Existing reporter tests and CLI tests continue to pass without schema changes
- New tests prove topic selection and default enablement come from registry metadata

## Implementation Notes

- Use a code-side registry first. This is the smallest correct P3 slice and removes most current duplication without committing to an external metadata format too early.
- Prefer function pointers and static slices over trait objects or heap allocations.
- Keep the registry local to `doctor.rs` unless a second file becomes clearly necessary.
- Treat the `agents` topic as one registry topic that matches multiple AI-tooling categories.

## Task 1: Add Registry Types And Selection Helpers

**Files:**
- Modify: `aidisk/src/doctor.rs`
- Test: `aidisk/src/doctor.rs`

**Step 1: Write the failing tests**

Add unit tests that describe the registry contract before changing production code.

Example test cases to add:

```rust
#[test]
fn topic_registry_includes_existing_builtin_topics() {
    let names = doctor_topic_specs()
        .iter()
        .map(|spec| spec.name)
        .collect::<Vec<_>>();

    assert_eq!(
        names,
        vec!["docker", "wsl", "ollama", "huggingface", "playwright", "agents"]
    );
}

#[test]
fn topic_registry_marks_default_topics() {
    let defaults = doctor_topic_specs()
        .iter()
        .filter(|spec| spec.default_enabled)
        .map(|spec| spec.name)
        .collect::<Vec<_>>();

    assert!(defaults.contains(&"agents"));
    assert!(defaults.contains(&"docker"));
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test doctor::tests::topic_registry -- --nocapture`

Expected: FAIL because registry helpers do not exist yet.

**Step 3: Add minimal registry types**

Add internal types near `DoctorOptions` in `aidisk/src/doctor.rs`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DoctorTopicKey {
    Docker,
    Wsl,
    Ollama,
    HuggingFace,
    Playwright,
    Agents,
}

struct DoctorTopicSpec {
    key: DoctorTopicKey,
    name: &'static str,
    default_enabled: bool,
    matcher: fn(&Finding) -> bool,
    recommendations: &'static [&'static str],
    probe: Option<DoctorProbeSpec>,
}

struct DoctorProbeSpec {
    name: &'static str,
    program: &'static str,
    args: &'static [&'static str],
}
```

Also add helpers:

- `fn doctor_topic_specs() -> &'static [DoctorTopicSpec]`
- `fn doctor_topic_enabled(options: DoctorOptions, key: DoctorTopicKey) -> bool`
- `fn doctor_topic_defaults(options: &mut DoctorOptions)` or equivalent helper that turns on default topics from registry metadata

**Step 4: Run the focused tests**

Run: `cargo test doctor::tests::topic_registry -- --nocapture`

Expected: PASS.

**Step 5: Commit**

```bash
git add aidisk/src/doctor.rs
git commit -m "refactor: add doctor topic registry scaffolding"
```

## Task 2: Build Doctor Topics By Iterating Registry Specs

**Files:**
- Modify: `aidisk/src/doctor.rs`
- Test: `aidisk/src/doctor.rs`

**Step 1: Write the failing tests**

Add tests that ensure topic creation now comes from registry iteration rather than bespoke branches.

Example test cases:

```rust
#[test]
fn build_doctor_uses_registry_for_selected_topics() {
    let doctor = build_doctor(
        &empty_scan(),
        DoctorOptions {
            docker: false,
            wsl: false,
            ollama: false,
            playwright: false,
            huggingface: false,
            agents: true,
            probe_tools: false,
        },
        &test_policy(),
    );

    assert_eq!(doctor.topics.len(), 1);
    assert_eq!(doctor.topics[0].name, "agents");
}

#[test]
fn build_doctor_registry_preserves_topic_order() {
    let doctor = build_doctor(
        &empty_scan(),
        DoctorOptions {
            docker: true,
            wsl: true,
            ollama: true,
            playwright: true,
            huggingface: true,
            agents: true,
            probe_tools: false,
        },
        &test_policy(),
    );

    let names = doctor.topics.iter().map(|topic| topic.name.as_str()).collect::<Vec<_>>();
    assert_eq!(names, vec!["docker", "wsl", "ollama", "huggingface", "playwright", "agents"]);
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test doctor::tests::build_doctor_registry -- --nocapture`

Expected: FAIL before the builder is refactored.

**Step 3: Refactor topic building to iterate specs**

Replace the current `if options.docker { ... }` / `if options.wsl { ... }` ladder in `build_doctor_with_probe_runner` with:

```rust
let topics = doctor_topic_specs()
    .iter()
    .filter(|spec| doctor_topic_enabled(options, spec.key))
    .map(|spec| build_topic_from_spec(spec, scan_report, options.probe_tools, probe_runner))
    .collect::<Vec<_>>();
```

Add helper:

```rust
fn build_topic_from_spec<F>(
    spec: &DoctorTopicSpec,
    scan_report: &ScanReport,
    probe_tools: bool,
    probe_runner: &F,
) -> DoctorTopic
where
    F: Fn(&str, &[&str]) -> ProbeCommandResult,
```

That helper should:

- call `build_topic(spec.name, scan_report, spec.matcher, ...)`
- convert `&'static [&'static str]` recommendations into `Vec<String>`
- attach probes only when `probe_tools` is true and `spec.probe` exists

**Step 4: Replace name-based probe dispatch**

Move probe metadata into the registry and remove `topic_probe_command(name: &str)` if it becomes unused.

**Step 5: Run focused tests**

Run: `cargo test doctor::tests -- --nocapture`

Expected: existing doctor tests still pass, plus new registry tests.

**Step 6: Commit**

```bash
git add aidisk/src/doctor.rs
git commit -m "refactor: build doctor topics from registry"
```

## Task 3: Derive Default Doctor Selection From Registry Metadata

**Files:**
- Modify: `aidisk/src/main.rs`
- Modify: `aidisk/src/doctor.rs`
- Test: `aidisk/tests/doctor_cli.rs`
- Test: `aidisk/src/doctor.rs`

**Step 1: Write the failing tests**

Add one unit test for default topic enablement and one CLI-level test that proves default doctor still includes the built-in registry defaults.

Example unit test:

```rust
#[test]
fn apply_default_topics_uses_registry_metadata() {
    let mut options = DoctorOptions::default_disabled();
    apply_default_doctor_topics(&mut options);

    assert!(options.docker);
    assert!(options.agents);
}
```

Example CLI test shape in `aidisk/tests/doctor_cli.rs`:

```rust
#[test]
fn doctor_without_topic_flags_uses_registry_defaults() {
    // Arrange temp rules, policy, and minimal snapshots if needed.
    // Run `aidisk doctor --json` without explicit topic flags.
    // Assert that `topics` contains built-in defaults such as docker + agents.
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test apply_default_topics_uses_registry_metadata -- --nocapture`

Expected: FAIL because default selection is still hardcoded in `main.rs`.

**Step 3: Implement registry-driven defaulting**

In `main.rs`, replace:

```rust
if !(docker || wsl || ollama || playwright || huggingface || agents) {
    docker = true;
    wsl = true;
    ollama = true;
    playwright = true;
    huggingface = true;
    agents = true;
}
```

with a helper call such as:

```rust
let mut doctor_options = doctor::DoctorOptions { ... };
doctor::apply_default_topics_if_none_selected(&mut doctor_options);
```

The helper should inspect registry metadata instead of duplicating names in `main.rs`.

**Step 4: Run focused tests**

Run: `cargo test doctor_cli doctor::tests -- --nocapture`

Expected: PASS, and current CLI behavior remains unchanged.

**Step 5: Commit**

```bash
git add aidisk/src/main.rs aidisk/src/doctor.rs aidisk/tests/doctor_cli.rs
git commit -m "refactor: derive doctor defaults from registry"
```

## Task 4: Final Verification And Lightweight Docs Sync

**Files:**
- Modify: `docs/execution-plan.md`
- Modify: `docs/plans/2026-06-03-doctor-v2-roadmap.md`
- Optional Modify: `skills/windows-ai-space-manager/references/category-map.md`
- Test: `aidisk/tests/skill_artifacts.rs` (only if docs/skill wording changes)

**Step 1: Update roadmap status notes**

Mark that Doctor V2 P3 now has registry scaffolding in place, while externalized topic metadata remains a future extension.

**Step 2: Keep docs minimal**

Do not promise new public flags or external metadata yet. Only document the architectural milestone if the code is merged in this slice.

**Step 3: Run the full suite**

Run: `cargo test`

Expected: all tests pass.

**Step 4: Run CLI smoke tests**

Run:

```bash
cargo run -- doctor --agents --markdown
cargo run -- doctor --docker --probe-tools --markdown
cargo run -- doctor --agents --latest --json
```

Expected:

- agents topic still renders
- probe output still appears only with `--probe-tools`
- latest diff still renders with unchanged JSON shape

**Step 5: Commit**

```bash
git add docs/execution-plan.md docs/plans/2026-06-03-doctor-v2-roadmap.md
git commit -m "docs: note doctor topic registry milestone"
```

## Risks To Watch

- Do not accidentally change topic names; reporter tests and skills depend on them.
- Do not move recommendation generation into the registry if it duplicates existing `enrich_recommendations` behavior.
- Do not reintroduce probe behavior on default doctor runs without `--probe-tools`.
- Do not over-generalize into dynamic external metadata in this slice; keep P3 minimal and shippable.

## Suggested Commit Sequence

1. `refactor: add doctor topic registry scaffolding`
2. `refactor: build doctor topics from registry`
3. `refactor: derive doctor defaults from registry`
4. `docs: note doctor topic registry milestone`
