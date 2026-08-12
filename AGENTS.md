# AI Disk Doctor Agent Constitution

This file is the persistent instruction layer for coding agents working in this repository. It is intentionally concise and applies unless a later milestone brief explicitly narrows scope.

## 1. Product Direction

- Product decision: AI Disk Doctor is a local-first storage, recovery, and AI-workspace health product for machines used with AI tools.
- Current repo fact: the public repository currently contains the `aidisk` Rust CLI/Core, local dashboard generation, rules, scripts, release workflows, and Skill wrappers.
- Product decision: the CLI/Core remains the execution authority for scan, plan, safety policy, quarantine, restore, history, diff, anomaly, reporting, rules, and model inventory.
- Product decision: a future Desktop must reuse the same execution truth instead of creating another cleanup engine.

## 2. Target Users

- AI power users and vibe coders using multiple AI agents, IDEs, model runtimes, and local tools.
- Non-specialist builders in product, design, research, operations, content, and support roles who need plain-language storage and recovery explanations.
- Developers and independent builders who want safe, structured, agent-readable storage diagnostics.
- Teams are a later validation target, not a current implementation target.

## 3. Core Principles

- Read first; mutate only after explicit consent.
- Prefer quarantine and restore over irreversible deletion.
- Treat recovery value separately from reclaim confidence.
- Keep user data, prompts, source, credentials, and paths local by default.
- Expose evidence, policy, action source, and uncertainty in structured outputs.
- Unknown, partial, active, or credential-adjacent data fails closed.
- Safety capabilities must not be paywalled in a way that traps users.

## 4. Must Do

- Audit repository reality before changing docs, code, rules, or release behavior.
- Preserve current CLI behavior unless the milestone explicitly authorizes runtime changes.
- Update tests and docs with any user-visible behavior change.
- Keep JSON and Markdown outputs stable, explicit, and agent-readable.
- Run relevant quality gates from `aidisk/`: `cargo fmt -- --check`, `cargo clippy --all-targets --all-features -- -D warnings` when compatible, and `cargo test`.
- Explain any skipped quality gate, platform limitation, or pre-existing blocker in the PR.

## 5. Must Never Do

- Do not bypass dry-run and `--yes` requirements for cleanup or restore.
- Do not add a second cleaner, scanner, restore engine, or risk model for GUI/Desktop work.
- Do not classify unknown models, sessions, snapshots, prompts, source, or recovery data as auto-cleanable.
- Do not read prompt, transcript, source-code, document, token, cookie, or credential contents merely to classify storage.
- Do not add SaaS, account, telemetry, billing, licensing-key, or cloud-sync features without an explicit milestone.
- Do not change MIT/Apache license files, product version, or release semantics without an explicit release/legal brief.
- Do not add Desktop dependencies, Tauri, or GUI runtime packages during foundation-only milestones.

## 6. Safety-Critical Areas

Treat changes to these areas as high risk and request line-by-line review in the PR:

- `aidisk/src/planner.rs`
- `aidisk/src/cleaner.rs`
- `aidisk/src/scanner.rs`
- `aidisk/src/rules.rs`
- `aidisk/src/model_inventory.rs`
- `aidisk/config/policy.yaml`
- `aidisk/rules/*.yaml`
- quarantine path containment, restore, copy/verify/remove, real deletion, privilege escalation, and Agent recovery asset cleanup

## 7. Repository Map

- `aidisk/` - Rust CLI/Core crate.
- `aidisk/src/main.rs` - CLI entrypoint and command wiring.
- `aidisk/src/scanner.rs` - rule-driven scans, path sizing, snapshots, summary metrics.
- `aidisk/src/planner.rs` - dry-run cleanup planning and policy gates.
- `aidisk/src/cleaner.rs` - quarantine, journal, and restore execution.
- `aidisk/src/doctor.rs` - diagnostic topics and recommendations.
- `aidisk/src/model_inventory.rs` - read-only model inventory and adapter capability reports.
- `aidisk/src/diff.rs`, `aidisk/src/anomaly.rs`, `aidisk/src/history.rs` - growth history, diff, and anomaly reporting.
- `aidisk/src/reporter.rs`, `aidisk/src/visualize.rs` - text/Markdown/JSON and HTML dashboard outputs.
- `aidisk/rules/` - built-in rule packs.
- `docs/product/` - current product, safety, roadmap, architecture, licensing, and collaboration source of truth.
- `docs/adr/` - architecture decision records.
- `docs/release-notes/` - immutable historical release records.
- `docs/plans/` - historical implementation plans.
- `scripts/governance/` - local governance and notifier scripts.
- `skills/windows-ai-space-manager/` - current Skill wrapper and agent workflow docs.

## 8. Test / Quality Gates

- Required for normal Rust changes: `cargo fmt -- --check`, strict clippy when compatible, and `cargo test` from `aidisk/`.
- Required for rule changes: `cargo run --quiet -- rules lint --json` from `aidisk/`.
- Required for release/distribution changes: inspect `.github/workflows/ci.yml`, `.github/workflows/release-artifacts.yml`, release notes, README, and artifact tests.
- Documentation-only PRs still run Rust gates unless explicitly blocked, because docs may be asserted by tests.

## 9. Destructive-Change Review Rules

- Any PR that can affect real filesystem mutation must be labeled high-risk in the PR body.
- The PR must show the dry-run path, policy gate, mutation preflight, execution stage, rollback/restore path, and failure behavior.
- New cleanup actions require fixtures that prove sensitive, active, partial, and credential-adjacent paths are blocked.
- Restore must never overwrite existing destination paths by default.

## 10. Product Decision Filter

Before adding a feature, answer:

1. Does it solve AI/developer storage, recovery, or workspace-health pain?
2. Does AI-aware semantics create value beyond generic disk analysis?
3. Can the behavior be explained with evidence?
4. Can risky actions be previewed and recovered?
5. Does it preserve local-first/privacy defaults?
6. Can CLI and future Desktop share one execution truth?
7. Is this based on repo reality or still a hypothesis?
8. Is this the smallest milestone slice that validates the idea?

If several answers are no, defer the feature.

## 11. Source-of-Truth Documents

- `docs/README.md` - documentation index and supersession map.
- `docs/product/CURRENT_PRODUCT_TRUTH.md` - current repo reality and capability matrix.
- `docs/product/VISION.md` - product direction.
- `docs/product/USER_PROBLEM_MODEL.md` - users, jobs, and problem model.
- `docs/product/PRODUCT_ARCHITECTURE.md` - target architecture and one-execution-truth model.
- `docs/product/UX_DIRECTION.md` - future human UX direction.
- `docs/product/SAFETY_AND_RECOVERY.md` - safety and recovery principles.
- `docs/product/LICENSING_AND_COMMERCIAL_ARCHITECTURE.md` - licensing and commercial boundary planning.
- `docs/product/ROADMAP_2026_H2.md` - current roadmap after M0 acceptance.
- `docs/product/COLLABORATION_PROTOCOL.md` - Web ChatGPT <-> Local Agent process.
- `docs/adr/0001-desktop-architecture.md` - Desktop architecture ADR/spike.

## 12. Web <-> Local Agent PR Handoff Protocol

- Work one milestone only.
- Start from a feature branch.
- Audit current branch, HEAD, docs, code, tests, workflows, licenses, and relevant agent instructions before editing.
- Keep changes minimal and reviewable.
- Open a Draft PR, do not merge.
- PR body must include objective, baseline SHA, head SHA, files changed, reality audit, decisions, safety impact, exact test results, known limitations, deferred work, and owner questions.
- Stop after the Draft PR and wait for Web ChatGPT acceptance before starting the next milestone.
