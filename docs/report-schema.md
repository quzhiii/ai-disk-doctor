# Report Schema Notes

This document records the current JSON report contracts that are most relevant to agents and downstream tooling.

## Scan Summary v2

`scan --json` includes `summary.schema_version: 2` and separates observed footprint from executable cleanup capacity.

The same summary includes `rule_sources`, with one entry per loaded rule:

| Field | Meaning |
|---|---|
| `path` | Rule file loaded for the scan. |
| `schema_version` | Normalized rule schema version (`1` for compatibility-loaded v1 rules, `2` for native v2 rules). |
| `digest` | SHA-256 digest of the source YAML, prefixed with `sha256:`. |

| Field | Meaning |
|---|---|
| `total_size_bytes` | Lower-bound total across existing findings, including partial findings. Kept for compatibility. |
| `observed_bytes` | Bytes from findings whose size was fully scanned. Partial findings are excluded. |
| `potential_bytes` | Bytes that may be reclaimable through aidisk quarantine or an official/manual cleanup flow. |
| `actionable_bytes` | Bytes that aidisk can currently put into an executable plan. |
| `quarantine_bytes` | Bytes eligible for aidisk-managed quarantine. |
| `official_cleanup_bytes` | Bytes that should be handled by an official/manual cleanup command instead of quarantine. |
| `report_only_bytes` | Bytes shown for visibility only. They do not enter executable cleanup planning. |
| `partial_bytes` | Lower-bound bytes from partial findings. Partial findings never enter real cleanup. |
| `reclaimable_safe_bytes` | Legacy compatibility field. It now means safe, non-partial quarantine-ready bytes only. |

Risk buckets (`safe_bytes`, `review_bytes`, `dangerous_bytes`, `system_bytes`) remain descriptive risk totals. They should not be interpreted as cleanup capacity.

## Plan Summary v2

`plan --json` includes `summary.schema_version: 2` and counts only executable candidates as actionable.

| Field | Meaning |
|---|---|
| `eligible_candidates` | Number of findings that entered the executable plan. |
| `reclaimable_bytes` | Legacy compatibility field. It mirrors executable/actionable candidate bytes. |
| `actionable_bytes` | Bytes in executable plan candidates. |
| `quarantine_bytes` | Bytes in candidates where `action == "quarantine"`. |
| `official_cleanup_bytes` | Existing non-partial `guide` findings that require official/manual cleanup. |
| `report_only_bytes` | Existing non-partial `report-only` findings. |
| `skipped_partial_findings` | Partial findings skipped from execution. |

`report-only`, `guide`, and `partial` findings are intentionally excluded from `candidates` even if they have positive size.

## Dashboard Semantics

The HTML dashboard's reclaim checklist is now based on quarantine-ready entries:

```text
exists == true && partial == false && action == "quarantine"
```

The dashboard must not treat `risk: safe` alone as cleanup-ready. This prevents unknown model files or report-only safe items from appearing in the executable checklist.

## Model File Rule

Generic model files (`.gguf`, `.safetensors`, `.onnx`, `.mlx`) are `risk: review` and `cleanup.method: report-only` until provenance is known. Unknown or custom models must not be shown as safe-to-clean.

## Quarantine Index v2

Quarantine execution indexes include `schema_version: 2`, per-entry `stage` fields, and recovery guidance for failed or skipped entries.

| Field | Meaning |
|---|---|
| `schema_version` | Quarantine index schema. Missing values from legacy indexes deserialize as `1`. |
| `journal_path` | Path to the append-only journal written before and after each filesystem action. |
| `results[].status` | Final result status such as `quarantined`, `skipped-active`, `partial-copy`, `verification-failed`, or `source-remove-failed`. Legacy `moved` is still accepted for restore. |
| `results[].stage` | Last recorded execution stage. Missing values from legacy indexes deserialize as `unknown`. |
| `results[].recovery` | Human-readable recovery guidance. Missing values from legacy indexes deserialize as an empty string. |

Current quarantine execution uses platform-native destination paths, validates quarantine-root containment before creating metadata, blocks source/destination nesting, tries `rename` first, and falls back to `copy -> verify file/dir/byte stats -> remove source` when rename fails.

The append-only journal records stage transitions such as `planned`, `renaming`, `copying`, `copied`, `verified`, `source-removing`, `quarantined`, `restore-planned`, `restoring`, and `restored`. Failure states leave a final stage plus recovery text so an interrupted run can be inspected before retrying.

## Model Inventory v1

`models inventory --json` is a read-only inventory report. It does not read model contents or
modify third-party indexes. The report includes:

| Field | Meaning |
|---|---|
| `assets[].logical_size_bytes` | Size of each discovered model file or cache blob. |
| `stale_after_days` | Metadata activity cutoff used for `assets[].stale`. |
| `assets[].exclusive_physical_size_bytes` | Physical bytes counted once for an asset path. |
| `assets[].shared_physical_size_bytes` | Bytes recognized as a repeated physical path and excluded from exclusive totals. |
| `assets[].state` | Conservative state such as `managed-cache`, `managed-cache-unresolved`, or `unknown-custom`. |
| `assets[].action` | Always `report-only` in the initial inventory foundation. |
| `assets[].reclaim_confidence` | Explanatory score only; it does not authorize cleanup. |
| `assets[].stale` | Metadata-only activity marker controlled by `--stale-after-days`. |
| `assets[].duplicate_logical_model` | Indicates repeated logical identity across revisions or distinct physical paths. |
| `nodes` / `edges` | Minimal in-memory provenance graph for tools, models, and revisions. |

When a small local index can be parsed without reading model contents, the inventory also adds
reference evidence:

| Field | Meaning |
|---|---|
| `assets[].state = managed-cache-referenced` | A parsed Hugging Face ref or Ollama manifest resolves to the asset or its physical blob. |
| `assets[].state = detached-revision` | A Hugging Face snapshot revision exists, but no parsed ref points to that revision. |
| `assets[].state = orphan-blob` | A parsed local index exists, but no reference points to the blob. |
| `assets[].state = incomplete-download` | The asset has a conservative incomplete-download filename marker. |
| `summary.referenced_assets` | Count of assets resolved from parsed local references. |
| `summary.detached_revision_assets` | Count of snapshot assets with no parsed Hugging Face ref. |
| `summary.orphan_blob_assets` | Count of blobs unreferenced by a successfully parsed local index. |
| `summary.incomplete_download_assets` | Count of assets with an incomplete-download marker. |
| `summary.stale_assets` | Count of assets older than the configured metadata cutoff. |
| `summary.duplicate_logical_model_assets` | Count of assets sharing a repeated logical model identity. |
| `edges` | May include `resolves-to`, `has-manifest`, `part-of`, `belongs-to`, and `references` relations. |

Index parsing is bounded to small local metadata files. Missing, unreadable, or invalid indexes do
not cause assets to be classified as orphaned or safe to reclaim. All actions remain `report-only`.

Unknown or potentially private model files remain report-only and have zero reclaim confidence.

## Model Adapter Capability v1

`models adapters --json` reports the safe adapter boundary without invoking external tools:

| Field | Meaning |
|---|---|
| `adapters[].tool` | Adapter identifier, currently `huggingface` or `ollama`. |
| `adapters[].root_exists` | Whether the selected local cache root exists. |
| `adapters[].index_present` | Whether a local refs or manifest index was found. |
| `adapters[].index_parseable` | Whether at least one bounded local index entry parsed successfully. |
| `adapters[].official_cli.probed` | Always `false` in this foundation; no external command is invoked. |
| `adapters[].plan_mode` | `metadata-only-dry-run` for the current foundation. |
| `adapters[].action` | Always `report-only`. |
| `adapters[].capabilities` | Capabilities available without mutation, plus future official CLI work. |

Missing or invalid indexes remain report-only and do not produce cleanup candidates.
