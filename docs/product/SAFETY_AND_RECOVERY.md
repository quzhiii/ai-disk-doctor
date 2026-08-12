# Safety And Recovery

## Existing Safety Foundation

Current repo fact: the Core already implements a conservative safety model:

- scan first
- dry-run planning
- explicit `--yes` for real cleanup and restore execution
- rule and policy semantics
- sensitive path blocking
- partial finding exclusion from executable plans
- quarantine instead of direct deletion for Core-owned cleanup
- execution journal, index, and restore reports
- conflict-safe restore
- report-only and guide actions for non-executable or official-tool paths

M0 preserves these invariants and makes no runtime behavior changes.

## Conceptual Axes

### Reclaim Confidence

How confident AI Disk Doctor is that bytes can be reclaimed without unacceptable functional or user-data loss.

Current repo fact: `models inventory` exposes `reclaim_confidence`; scanner/planner broadly use `risk`, `action`, policy, partial status, and sensitive markers.

Gap: reclaim confidence is not yet a universal field across all scan/planner/clean candidates.

### Recovery Value

How valuable an asset is as a route to restore prior work or reconstruct useful state.

Current repo fact: quarantine restore, scan history, diff/anomaly history, and model rollback metadata exist.

Gap: no general recovery value axis exists yet for Agent snapshots, Git coverage, sessions, project state, or cleanup impact.

## Current Field Mapping

| Future concept | Current repo fields or behavior | Status |
|---|---|---|
| asset identity | `Finding.id`, `Finding.name`, `Finding.category`, model asset IDs | partial |
| origin/tool | rule category/name, doctor topic, model manager | partial |
| physical/logical size | scanner `size_bytes`; model logical/exclusive/shared physical size | partial |
| reclaim estimate | scanner/planner summary v2 fields | true |
| reclaim confidence | model inventory `reclaim_confidence` | partial |
| recovery value | quarantine/restore metadata and model rollback metadata | partial |
| action mechanism | `quarantine`, `report-only`, `guide`, official dry-run report-only plans | true |
| risk class | `safe`, `review`, `dangerous`, `system` | true |
| evidence | rule reason/warnings, policy snapshot, rule digest, model evidence | partial |
| last-use / age evidence | model inventory stale/last access/last modified where available | partial |
| why explanation | README, doctor recommendations, rule reason | partial |

## Recovery Capability Vocabulary

Product decision: future Recovery Intelligence should use these levels:

- Strong: multiple recent, verifiable recovery mechanisms.
- Partial: at least one mechanism exists but has scope, time, or tool limits.
- Weak: evidence is incomplete, old, or narrow.
- Unknown: insufficient metadata.
- None observed: no supported recovery mechanism detected.

These are product concepts today, not current schema fields.

## Safety Rules

1. Read first.
2. Default to no mutation.
3. Require explicit consent for real actions.
4. Fail closed on unknown, partial, active, or credential-adjacent evidence.
5. Prefer quarantine over permanent deletion.
6. Preserve restore capability.
7. Do not infer recoverability from filenames alone when authoritative metadata exists.
8. Do not read prompts, source, transcripts, documents, or credentials merely to classify storage when metadata is sufficient.
9. Never turn recovery-critical data into safe cleanup solely because it is old.
10. GUI/Desktop must not bypass CLI/Core safety rules.

## M0 Safety Impact

- Current repo fact: M0 is documentation and architecture only.
- Current repo fact: M0 does not change scanner, planner, cleaner, quarantine, restore, rules, model inventory, CLI commands, report schema, version, dependencies, license files, billing, auth, or telemetry.
