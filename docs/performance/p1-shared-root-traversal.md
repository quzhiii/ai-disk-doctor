# P1 Shared-Root Traversal Optimization

Status: completed Core performance fix slice.

Date: 2026-08-26

## Baseline

The Performance Evidence Gate found that current Core `c6614bb78b3846c831d34828146b2d3391f3d92f` exceeded the Integration `120s` timeout for complete explain:

| Run | Command | Duration |
|---:|---|---:|
| 1 | `aidisk explain --json --snapshot skip` | `136.353s` |
| 2 | `aidisk explain --json --snapshot skip` | `134.705s` |
| 3 | `aidisk explain --json --snapshot skip` | `135.052s` |

The gate attributed runtime to scanner traversal:

| Stage | Duration |
|---|---:|
| `scanner_scan` | `~135,787ms` |
| `explainability_aggregation` | `~63ms` |
| JSON serialization/stdout | `~0ms` |

Main hotspots were repeated recursive `%USERPROFILE%\\**` traversal:

| Rule | Recursive patterns | Discovery time before |
|---|---:|---:|
| `common-dev-artifacts` | `7` | `~78,886ms` |
| `model-files` | `4` | `~46,205ms` |

## Implementation

P1 implements scanner-internal shared-root traversal for recursive glob patterns that share the same literal root. It does not change rule definitions, public contracts, explainability schema, category behavior, risk, handling, warnings, snapshot behavior, cleanup, restore, quarantine, or action proposal behavior.

Before:

```text
%USERPROFILE%\\**\\node_modules   -> enumerate USERPROFILE
%USERPROFILE%\\**\\dist           -> enumerate USERPROFILE
%USERPROFILE%\\**\\*.onnx         -> enumerate USERPROFILE
```

After:

```text
%USERPROFILE%
  -> enumerate once per scan
  -> match candidates against each original glob Pattern
  -> emit the same rule-specific logical path groups
```

The optimization is implemented in `aidisk/src/scanner.rs` with:

- recursive-pattern root detection for a single `**` segment and a literal non-glob root;
- a scan-local `SharedRecursiveRootCache` keyed by normalized root string;
- matching through `glob::Pattern::matches_path` to preserve per-pattern glob semantics;
- unchanged downstream `compute_size`, finding creation, summary accounting, risk, action, and warning handling.

This slice intentionally does not implement directory-size caching, canonical deduplication, reparse/junction pruning, parent-child accounting changes, scan budgets, deferred scanning, or root narrowing.

## Semantic Preservation

Synthetic frozen fixture comparison was run between clean baseline `c6614bb...` and the optimized branch with `USERPROFILE`, `HOME`, `APPDATA`, and `LOCALAPPDATA` redirected to the same fixture.

| Scenario | Result | Notes |
|---|---|---|
| Complete explain | PASS | Normalized hashes matched. Contract/schema/evidence/storage/category/path-group semantics preserved. |
| `dev-artifact` explain | PASS | Normalized hashes matched. |
| `ai-model` explain | PASS | Normalized hashes matched. |
| `dev-artifact` scan | PASS | Normalized hashes matched. |
| `ai-model` scan | PASS | Normalized hashes matched. |

Synthetic fixture coverage included:

```text
user/
  project-a/
    node_modules/
    dist/
  project-b/
    node_modules/
    __pycache__/
  rust-project/
    target/
  models/
    model.onnx
    model.gguf
  unrelated/
    normal.txt
```

Additional real-workspace comparison found exact equality for `dev-artifact` and `ai-model` explain/scan outputs. Complete real-workspace explain had a volatile difference in active AI-agent state (`.codex`) between long-running baseline and optimized runs; the measured drift was outside the optimized rules and is not attributed to shared-root traversal.

## Traversal Count Before / After

Regression tests assert traversal behavior with a counter rather than wall-clock timing:

| Test | Assertion |
|---|---|
| `shared_recursive_root_matches_individual_glob_results` | Shared resolver returns exactly the same matches as individual `glob` resolution while enumerating the same root once. |
| `shared_recursive_root_preserves_findings_and_accounting` | Findings, matched paths, accounting, risk, action, and partial state are preserved for a multi-pattern synthetic fixture. |
| `shared_recursive_root_is_cached_across_rules` | Two different rules sharing the same recursive root enumerate that root once across the scan. |

Before the implementation, the gate observed:

| Rule | Shared root | Recursive pattern count | Root traversal behavior before |
|---|---|---:|---|
| `common-dev-artifacts` | `%USERPROFILE%` | `7` | One full/root traversal per pattern. |
| `model-files` | `%USERPROFILE%` | `4` | One full/root traversal per pattern. |

After the implementation, tests prove that multiple patterns and multiple rules sharing one root use one shared enumeration for that root in a scan.

## Performance Before / After

Release build, same Windows environment as the evidence gate.

### Baseline

| Command | Baseline duration |
|---|---:|
| `aidisk explain --json --snapshot skip` run 1 | `136.353s` |
| `aidisk explain --json --snapshot skip` run 2 | `134.705s` |
| `aidisk explain --json --snapshot skip` run 3 | `135.052s` |
| `aidisk explain --json --snapshot skip --category dev-artifact` | `107.052s` |
| `aidisk explain --json --snapshot skip --category ai-model` | `48.781s` |

### P1 result

| Command | Duration | Evidence status | Path groups | Output size |
|---|---:|---|---:|---:|
| `aidisk explain --json --snapshot skip --category dev-artifact` | `39.948s` | `complete` | `15,282` | `44,217` bytes |
| `aidisk explain --json --snapshot skip --category ai-model` | `27.009s` | `complete` | `36` | `36,537` bytes |
| `aidisk explain --json --snapshot skip` run 1 | `44.926s` | `complete` | `15,331` | `111,970` bytes |
| `aidisk explain --json --snapshot skip` run 2 | `43.577s` | `complete` | `15,331` | `111,970` bytes |
| `aidisk explain --json --snapshot skip` run 3 | `41.334s` | `complete` | `15,331` | `111,970` bytes |

Median complete explain after P1: `43.577s`.

Gate result: **PASS / Strong performance result** because median complete explain is below `60s` and every complete run is below `120s`.

## Windows Behavior

P1 does not change Windows alias, reparse point, or junction semantics.

- Existing canonical and alias path groups remain discoverable if the underlying glob pattern sees them.
- No canonical deduplication was introduced.
- No reparse/junction pruning was introduced.
- No byte-accounting changes were introduced.

The shared traversal cache only changes how often a root is enumerated. It does not decide whether a discovered alias path should be retained or deduplicated.

## Remaining Hotspots

Measured after P1:

| Remaining area | Evidence |
|---|---|
| Directory size walking | `dev-artifact` still takes about `39.948s`; this slice intentionally did not add size caching or parent-child aggregation. |
| Windows alias/reparse duplication | Still present by design. P1 does not prune or canonical-dedupe alias path groups. |
| Active workspace volatility | Real complete explain can change between long runs because AI-agent state such as `.codex` changes while scans are running. |

Do not start P2 automatically. Directory-size cache, canonical deduplication, and reparse/junction policy remain separate optimization/review slices.

## Verification

| Command | Result |
|---|---|
| `cargo fmt -- --check` | PASS |
| `cargo test` | PASS: `168` unit tests plus integration/doc tests passed |
| `cargo run --quiet -- rules lint --json` | PASS: `26` rules linted |
| `cargo clippy --all-targets --all-features -- -D warnings` | BLOCKED by existing repo-wide clippy warnings under Rust `1.97.0`, including `anomaly.rs`, `cleaner.rs`, `doctor.rs`, `model_inventory.rs`, `rules.rs`, and `visualize.rs`. Scanner-specific new clone-on-copy / len-zero / sort lints surfaced by this branch were fixed. |

CI workflow commands in `.github/workflows/ci.yml` are `cargo test` and `cargo run --quiet -- rules lint --json`; both pass locally on Windows for this branch.

## Gate Result

**PASS**.

Semantic preservation passed on frozen fixtures; all complete explain performance runs were under `120s`; median complete explain was `43.577s`, which is below the `60s` strong-pass threshold.
