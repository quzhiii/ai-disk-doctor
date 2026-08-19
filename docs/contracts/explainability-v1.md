# Explainability Contract v1

`explainability-v1` is the stable, read-only downstream contract for presenting local scan evidence. It is produced by the public Core application boundary through `run_explainable_scan` and `run_explainable_scan_with_progress`.

It is intentionally not a serialized `ScanReport`. `ScanReport` remains the CLI report and snapshot representation. Consumers should use this contract for progressive drill-down and should not derive action eligibility, risk, recoverability, or provenance from untyped report fields.

## Compatibility

- `contract` is exactly `explainability-v1`.
- `schema_version` is `1`.
- The existing public `ScanResult` struct is unchanged for Rust SemVer compatibility. Adding a public field to that all-public constructible struct would be a breaking API change for external struct literals.
- Explainability is exposed through the new additive `ExplainableScanResult` wrapper and `run_explainable_scan*` functions.
- Additive fields may be added in later v1-compatible releases.
- Existing fields and their semantics will not be renamed or changed without a new contract version.
- All byte counts are unsigned bytes, not formatted display strings.

## Top-Level Shape

```text
ExplainabilityReport {
  contract,
  schema_version,
  accounting,
  storage,
  evidence,
  volumes,
  categories
}
```

Core owns machine-stable codes and structured evidence. Desktop and other consumers own user-facing localization.

## Accounting Semantics

`accounting.byte_basis = logical-rule-match-lower-bound` means bytes are logical bytes associated with rule/path matches. They are not guaranteed to be physical disk blocks and are not de-duplicated across independent rules.

`accounting.deduplication = per-rule-path-only` means the scanner deduplicates equivalent paths within a single rule, but separate rules may overlap. The hierarchy is internally reconcilable for this report:

- Sum of `categories[].observed_bytes` equals `storage.observed_bytes`.
- Sum of `categories[].partial_bytes` equals `storage.partial_bytes`.
- Within each category, sum of rule bytes equals category bytes.
- Within each rule, included plus omitted path-group bytes equals rule bytes.

`partial_bytes_in_total_size = true` means `total_size_bytes` is a lower-bound total including partial observations. `partial_bytes_excluded_from_handling_totals = true` means partial observations do not enter quarantine, official/manual, or report-only handling totals.

## Storage, Handling, And Risk

`storage` preserves the existing two-axis model:

- Storage totals: `observed_bytes`, `total_size_bytes`, `partial_bytes`.
- Handling totals: `quarantine_bytes`, `official_cleanup_bytes`, `report_only_bytes`, and `partial_bytes`.
- Risk totals: `safe_bytes`, `review_bytes`, `dangerous_bytes`, `system_bytes`.

Handling and risk are independent. A finding may be `risk = review` and `handling_mode = report-only` at the same time. Consumers must not convert these fields into fake mutually exclusive buckets such as priority/review/protected unless a future Core policy contract explicitly defines that model.

`handling_mode` values are:

| Value | Meaning |
|---|---|
| `quarantine` | The rule reports aidisk-managed quarantine as its handling mode. This contract does not authorize or execute it. |
| `official-manual` | The rule reports an official/manual cleanup flow. This contract does not invoke that flow. |
| `report-only` | Visibility evidence only; it never enters executable cleanup planning. |
| `unknown` | The Core cannot safely map the action. |

`risk` values are `safe`, `review`, `dangerous`, or `system`. Risk remains descriptive, not an action authorization. Risk totals include partial findings so users can see lower-bound risk distribution; handling totals exclude them.

## Evidence State

`evidence.status` is `complete` when the scan has no partial bytes, otherwise `partial`.

`evidence.partial_findings` is the number of partial findings. `evidence.warnings[]` contains stable warning codes plus language-neutral source text from rules or scanner partial reasons.

Consumers must retain `partial` and `unknown` labels. They must not infer complete discovery, action eligibility, or recoverability from missing evidence.

## Progressive Drill-Down

The ordered data hierarchy is:

```text
categories[] -> rules[] -> path_groups[]
```

Each category aggregates complete `observed_bytes`, lower-bound `partial_bytes`, `total_size_bytes`, handling totals, and risk totals. `category_id` is the rule category. `category_name` currently equals that stable identifier.

Each rule provides observed/partial/total bytes, one display-neutral `handling_mode`, one `risk`, bounded local path groups, rationale, provenance, and recoverability evidence.

Each rule includes `path_group_summary`:

- `total_path_groups`: all path groups under the rule.
- `included_path_groups`: path groups present in `path_groups[]`.
- `omitted_path_groups`: path groups omitted by the bounded output limit.
- `omitted_bytes`: total bytes represented by omitted groups.
- `limit`: current inclusion limit.

Current `path_groups[]` are sorted by descending `total_size_bytes`, then path, and limited to the largest 50 groups per rule. Consumers that need exhaustive evidence should request or use a future paged contract rather than relying on raw scan report dumps.

## Path Privacy

Path evidence is explicit:

```text
PathEvidence {
  raw_path,
  display_path,
  disclosure,
  sensitivity
}
```

`disclosure = raw-local-path` means Core is exposing local path evidence produced by the scanner. `display_path` currently mirrors `raw_path`; consumers may mask or collapse it for UI while retaining traceability. `sensitivity` comes from rule metadata when available and is `unknown` otherwise.

The contract does not read file contents, prompts, transcripts, source files, documents, tokens, cookies, credentials, model payloads, or arbitrary user content to improve explanations. It aggregates scanner metadata and loaded rule metadata only.

## Volume Mapping

`volumes[]` mirrors scanner volume metadata. Each path group may include a `volume` reference when Core can match the path to a known mount point. Matching normalizes path separators and case, uses the longest matching mount point, and requires either exact match or a path boundary after the mount. Empty mount points do not match. If no volume matches, `volume` is `null`; unknown remains unknown.

## Rationale And Provenance

`rationale` provides the rule's local `reason`, `warnings`, and per-rule `partial_reasons`.

`provenance` provides stable local rule evidence:

- source path, schema version, and digest;
- detector evidence identifiers;
- declared content-access policy;
- decision confidence;
- action type and adapter;
- dry-run and rollback capability declarations.

This is rule metadata, not UI-generated rationale and not a mutation record. If a finding cannot be matched to a loaded rule, provenance fields are emitted as `unknown`; the contract must not fabricate a rule source or safety claim.

## Recoverability

Recoverability is evidence-only and safety-critical.

`recoverability.declared` is the loaded rule's declared recoverability metadata. `kind` normalizes only a small set of known declarations: `redownload`, `rebuildable`, `unknown`, or `other-declared`. `reversibility` is:

| Value | Meaning |
|---|---|
| `supported` | The rule metadata declares rollback support. This does not execute or guarantee a restore. |
| `not-guaranteed` | The rule metadata does not declare rollback support. |
| `unknown` | Rule metadata was unavailable. |

The contract must not be presented as `safe to recover`, `fully recoverable`, or equivalent without additional evidence from a future recovery policy. `redownload` means the rule declares a plausible redownload path, not that every local asset has been verified recoverable.

## Read-Only Boundary

Creating this report aggregates an already produced scan and loaded rules in memory. It does not call planning, cleaning, quarantine, restore, external tools, arbitrary shell commands, network, telemetry, or cloud services. It does not grant frontend filesystem access or mutate scanned paths.

## Non-Guarantees

- The contract does not authorize cleanup.
- The contract does not execute cleanup.
- The contract does not guarantee physical-byte de-duplication across rules.
- The contract does not guarantee recoverability.
- The contract does not provide exhaustive path lists beyond the current bounded path groups.
- The contract does not localize final user-facing copy.
