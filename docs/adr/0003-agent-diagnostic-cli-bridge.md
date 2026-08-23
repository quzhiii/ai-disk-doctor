# ADR 0003: Agent Diagnostic CLI Bridge

- Status: Accepted
- Date: 2026-08-23
- Milestone: M1D

## Context

The Integration lane needs a stable, scan-free compatibility probe and a bounded no-snapshot explainability diagnostic. Core already owns scan semantics, explainability-v1, policy, risk, handling, provenance, recoverability, partial/unknown evidence, and snapshot persistence.

## Decision

Add two additive JSON-only commands:

- `aidisk capabilities --json` publishes the compiled Core version and the implemented machine contracts without scanning.
- `aidisk explain --json [--category CATEGORY] [--snapshot save|skip]` calls the existing `application::run_explainable_scan` boundary and embeds its unchanged `explainability-v1` payload.

`--snapshot skip` maps directly to `SnapshotPersistence::Skip`. No alternate persistence behavior or fallback is introduced. The Agent subset intentionally has no arbitrary root, rules, policy, reports, shell, executable, or mutation controls.

The CLI uses the existing structured JSON error convention: one error envelope on stderr, empty stdout, and stable coarse error types.

For macOS volume matching, Core uses the native mounted-volume case-sensitivity capability when available. Unknown capability preserves case and therefore may leave a volume unmatched. This is preferred over false-positive volume evidence. Windows, longest mount, and boundary semantics remain unchanged.

## Consequences

- Integration can determine compatibility without parsing human help or launching a scan.
- Integration can request a diagnostic that provably does not persist scan history.
- Core remains the only owner of evidence semantics and bounded output.
- The embedded explainability contract is not forked for Agents.
- macOS matching is conservative when native capability evidence is absent.

## Rejected Alternatives

- Reusing `scan --json` and reconstructing explainability in Integration: rejected because it duplicates Core semantics.
- Lowercasing every macOS Unix path: rejected because case-sensitive APFS and external volumes exist.
- Inferring case mode from filesystem type: rejected because formats such as APFS support both modes.
- Parsing `--help` as the long-term compatibility protocol: rejected because it is fragile and human-oriented.
