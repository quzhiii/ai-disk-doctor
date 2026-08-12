# ADR 0001: Desktop Architecture Direction

Status: Proposed  
Date: 2026-08-12  
Scope: M0 architecture spike only; no Desktop runtime dependency is added.

## Context

Current repo fact: AI Disk Doctor is a Rust CLI/Core at v1.7.0 with scanner, planner, cleaner/quarantine/restore, doctor, model inventory, diff, anomaly, history, reporter, visualization, rules, release workflows, and Skill wrappers.

Product decision: future Desktop must provide a human-first UI while preserving one execution truth in the Core. It must not duplicate cleanup, restore, risk, or policy logic.

External-source facts reviewed:

- Tauri 2 official docs describe desktop apps built with Rust tools plus HTML in a WebView and message passing between WebView and Rust backend: `https://v2.tauri.app/concept/architecture/`.
- Tauri 2 distribution docs cover platform-specific installers, bundling, signing, Windows/macOS/Linux distribution paths, and updater support: `https://v2.tauri.app/distribute/` and `https://v2.tauri.app/plugin/updater/`.
- Tauri itself is licensed MIT or Apache-2.0 per official architecture docs: `https://v2.tauri.app/concept/architecture/`.

## Decision Drivers

- Fit with current Rust crate/module structure.
- Required refactor before Desktop can invoke Core safely.
- Windows/macOS/Linux packaging.
- UI capability and accessibility.
- One-execution-truth requirement.
- Security boundary for filesystem operations.
- Testability.
- Signing and update path.
- Binary/runtime footprint.
- Future public Core plus proprietary Desktop separation.
- License compatibility.
- CI/release complexity.

## Options

### Option A - Tauri 2 Integrated With Shared Rust Domain Services

Summary: a Tauri Desktop shell calls shared Rust application/domain services extracted from the current CLI/Core.

Pros:

- Strong fit with current Rust Core.
- Web UI can support a richer beginner-first UX faster than pure Rust-native widgets.
- Uses OS WebView rather than bundling a full browser engine.
- Official docs cover Windows/macOS/Linux app distribution, signing, and updater paths.
- MIT/Apache-2.0 framework licensing appears compatible with the current repository posture, subject to full dependency review.

Cons:

- Requires frontend toolchain and Desktop packaging complexity.
- Requires a Core service extraction before any safe mutation UX.
- IPC and file-system permissions must be tightly scoped.
- Proprietary Desktop separation is easier if Desktop lives outside the public Core repo or uses a stable Core contract.

Assessment: best default Desktop direction after a shared Core service boundary exists.

### Option B - Rust-Native GUI Approach

Summary: build a native Rust GUI using a Rust UI framework.

Pros:

- Single-language development path.
- Can link directly against Core domain services.
- Potentially simpler dependency story than a web frontend.

Cons:

- UI ecosystem, accessibility, layout, and design iteration may be weaker or slower for a rich consumer Desktop.
- Packaging/signing/updater story must be assembled separately.
- Contributor pool may be smaller for product UX work.
- Commercial Desktop polish may require more custom engineering.

Assessment: useful fallback for a very narrow tray/status app, but less attractive for a full beginner-first Desktop.

### Option C - Separate Desktop Process Using Stable CLI/IPC/Core Contract

Summary: Desktop is a separate proprietary process or app that invokes a stable public Core executable, local IPC, or library contract.

Pros:

- Strongest public Core vs proprietary Desktop separation.
- Reduces accidental license/contribution boundary confusion.
- Desktop can be developed in a different stack while Core remains Rust.
- CLI executable contract can be tested independently.

Cons:

- Requires a stable machine-readable contract and version negotiation.
- Process/IPC error handling, cancellation, progress, and permissions become product architecture work.
- If built only by shelling out to CLI commands, advanced UX may be constrained until Core APIs are exposed.

Assessment: strongest fallback if commercial/license boundaries are complex, and a likely distribution boundary even if the Desktop implementation uses Tauri.

## Comparison

| Driver | Option A: Tauri 2 | Option B: Rust-native GUI | Option C: Separate process/contract |
|---|---|---|---|
| Current code fit | high after service extraction | medium-high after service extraction | high if CLI/JSON contract is formalized |
| Refactor required | extract shared domain/application services | extract shared domain/application services | formalize CLI/IPC contract and progress/error model |
| Packaging | built-in Tauri path | framework-specific/manual | independent per chosen Desktop stack |
| UI/accessibility | strong web UI ecosystem | variable by framework | depends on chosen Desktop stack |
| One execution truth | good if calls shared services | good if calls shared services | good if Core executable/API remains sole authority |
| Security boundary | needs scoped IPC/capabilities | direct process access; must design carefully | strongest boundary if Core process owns mutations |
| Testing | Rust unit tests plus frontend/e2e | Rust-heavy tests plus GUI testing | contract tests plus Desktop e2e |
| Signing/update | Tauri docs and updater plugin path exist | assembled separately | depends on Desktop stack |
| Runtime footprint | uses OS WebView; no bundled browser engine | potentially small | depends on Desktop stack |
| OSS/proprietary split | manageable, especially separate repo | manageable but linked boundary needs review | strongest separation |
| License risk | Tauri MIT/Apache-2.0; still review dependencies | framework-specific | mostly contract/distribution review |
| CI/release complexity | moderate-high | moderate | moderate-high due contract matrix |

## Recommendation

Recommendation: pursue Option A, Tauri 2 integrated with shared Rust domain/application services, for Desktop Alpha after M0 acceptance and after a dedicated Core service-boundary milestone.

Fallback: use Option C as the commercial and license boundary if the future Desktop needs a stronger public Core / proprietary product separation. A Tauri Desktop can still use Option C by invoking a stable Core process rather than directly linking Core internals.

## Required Pre-Desktop Work

Before M1 Desktop can safely implement scan/plan/clean/restore flows:

- extract shared application/domain services from CLI orchestration in `main.rs`
- define stable progress, cancellation, error, policy, and report contracts
- define allowed Desktop request types and mutation preflight gates
- preserve JSON/report schema stability for CLI and agents
- add tests proving Desktop adapters cannot bypass Core safety gates

## Consequences

- M0 adds no Tauri, GUI, JavaScript, updater, or Desktop package dependency.
- M0 does not refactor runtime code.
- Future Desktop work must be scoped as a separate milestone.
- Safety review remains mandatory before Desktop can perform any real cleanup or restore action.
