# Agent Diagnostic CLI v1

Status: additive Core contract for M1D Agent Contract Bridge.

## Commands

Compatibility detection is intentionally scan-free:

```text
aidisk capabilities --json
```

The bounded explainability diagnostic is:

```text
aidisk explain --json [--category CATEGORY] [--snapshot save|skip]
```

`--snapshot save` is the default and preserves the normal snapshot behavior. `--snapshot skip` executes the same read-only Core scan and explainability application boundary without creating a `scan-*.json` history snapshot.

The Agent consumer uses only this fixed subset. Human commands retain their existing options, but Agent integrations must not pass arbitrary roots, rules directories, rules repositories, policy overrides, reports directories, executable paths, shell commands, or mutation controls.

## Capabilities Output

`aidisk capabilities --json` returns one JSON document on stdout:

```json
{
  "ok": true,
  "command": "capabilities",
  "contract": "agent-capabilities-v1",
  "schema_version": 1,
  "core_version": "1.7.0",
  "capabilities": {
    "explainability": {
      "contract": "explainability-v1",
      "schema_versions": [1],
      "cli_available": true,
      "snapshot_modes": ["save", "skip"],
      "bounded_path_groups": true
    }
  }
}
```

The version is compiled from `CARGO_PKG_VERSION`; no runtime capability is claimed unless the binary implements it.

## Explainability Output

The command returns an envelope whose `explainability` member is the authoritative unchanged `explainability-v1` payload from `ExplainableScanResult`:

```json
{
  "ok": true,
  "command": "explain",
  "contract": "agent-diagnostic-cli-v1",
  "schema_version": 1,
  "core_version": "1.7.0",
  "snapshot": {
    "requested": "skip",
    "persisted": false,
    "path": null
  },
  "explainability": {
    "contract": "explainability-v1",
    "schema_version": 1,
    "accounting": {},
    "storage": {},
    "evidence": {},
    "volumes": [],
    "categories": []
  }
}
```

The abbreviated objects above are illustrative. The real payload preserves all existing fields: accounting semantics, storage totals, handling totals, independent risk totals, evidence status, warnings, partial reasons, categories, rules, bounded path groups, omission counts, omitted bytes, limits, provenance, volume evidence, rationale, and recoverability evidence.

The CLI does not recompute risk, infer recoverability, create recommendations, or build another evidence tree. Category filtering is passed to the existing Core rule loader and scan boundary.

## Snapshot And Privacy Semantics

- `skip` maps directly to `SnapshotPersistence::Skip`.
- `skip` returns `snapshot.requested = "skip"`, `snapshot.persisted = false`, and `snapshot.path = null`.
- `skip` does not create the reports directory as a scan side effect.
- `save` maps directly to `SnapshotPersistence::Save` and returns the created snapshot path.
- Both modes read user/workspace metadata and paths through the existing scanner; neither executes clean, delete, quarantine, restore, shell, network, telemetry, or cloud behavior.
- The diagnostic output includes local path evidence already bounded by Core. Consumers must treat paths and rule evidence as local-sensitive data.

## Errors

With `--json`, failures are emitted as one JSON document on stderr and stdout remains empty:

```json
{
  "ok": false,
  "error": {
    "type": "usage|input|execution",
    "message": "...",
    "command": "explain|capabilities",
    "details": []
  }
}
```

`usage` identifies invalid command/argument use, `input` identifies malformed or unavailable rules/policy/filesystem inputs, and `execution` identifies a scan or internal execution failure. A capability is unavailable when the `capabilities` payload does not advertise it; unsupported future additions must not be inferred from human help text.

## Bounds And Compatibility

- `explainability-v1` remains the evidence source of truth.
- Path groups remain bounded by Core. `total_path_groups`, `included_path_groups`, `omitted_path_groups`, `omitted_bytes`, and `limit` are authoritative.
- Byte accounting remains `logical-rule-match-lower-bound` with `per-rule-path-only` deduplication.
- Partial and unknown evidence remains evidence state, not a recovery or safety guarantee.
- Handling and risk remain orthogonal.
- This is additive. Existing `scan`, `history`, `clean`, `quarantine`, `restore`, and other human CLI behavior is unchanged.
- The CLI envelope contract is separate from the embedded explainability contract so consumers can distinguish transport metadata from Core evidence.

## Volume Evidence Note

Windows drive paths remain case-insensitive. Linux and other Unix paths remain case-sensitive. On macOS, Core queries the mounted volume's case capability using the native volume attribute API. A proven case-insensitive volume is matched case-insensitively; a proven case-sensitive volume is matched case-sensitively; unavailable or invalid capability evidence fails closed and preserves case. Longest mount matching and path boundaries remain enforced.

## Non-Guarantees

This contract does not guarantee physical block accounting, exhaustive unbounded filesystem enumeration, recoverability, cleanup authorization, or mutation. It does not expose MCP, Agent Skill, transport, authentication, cloud, or telemetry behavior.
