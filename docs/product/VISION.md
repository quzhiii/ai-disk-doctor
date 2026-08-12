# Product Vision

## Product Definition

- Product decision: AI Disk Doctor is a local-first storage, recovery, and workspace-health product for machines increasingly used with AI tools.
- Current repo fact: the current implementation is a Rust CLI/Core plus local dashboard, rules, scripts, release workflows, and Skill wrappers.
- Product decision: future user-facing surfaces may include Desktop, but the Core remains the authority for safety and execution.

## Core Insight

AI tools create two opposing local data classes:

| Class | Meaning | Product implication |
|---|---|---|
| Reclaimable data | caches, stale artifacts, rebuildable dependencies, installers, temporary runtime assets | candidates for safe planning, quarantine, official cleanup, or manual cleanup |
| Recovery-critical data | snapshots, checkpoints, sessions, local history, quarantine records, project state | protect by default and explain the recovery tradeoff |

Product decision: the product must reason about both reclaim confidence and recovery value. Bytes reclaimed alone is not a sufficient success metric.

## User Questions

AI Disk Doctor should help users answer:

1. What is taking space?
2. What is this data?
3. What can I safely reclaim and what is the cost?
4. What recovery capability would I lose?
5. What is growing or becoming unhealthy over time?

## Differentiation

- AI-aware storage semantics.
- Relationship-aware model, cache, session, and project metadata.
- Recovery-aware safety decisions.
- Local-first privacy posture.
- Metadata-first classification by default.
- Explainable and reversible workflows.
- Agent-ready structured outputs.
- Cross-platform Core and release discipline.

## Product Boundary

### Near-Term In Scope

- AI/developer storage diagnosis.
- Safe reclaim planning.
- Model asset intelligence.
- Agent/project asset inventory.
- Recovery intelligence as metadata and capability detection.
- Quarantine and restore.
- History, diff, activity, and anomaly intelligence.
- Human-friendly Desktop experience after M0 acceptance.

### Near-Term Out of Scope

- Generic app uninstaller.
- Generic system optimizer.
- Broad CPU/GPU/network system monitor.
- Backup replacement.
- Source-code or prompt indexing.
- Autonomous destructive cleanup.
- Enterprise control plane.
- Cloud-first architecture.

## Product Thesis

- Hypothesis / to validate: cleaning is acquisition; understanding, recovery, and continuous workspace health are retention.
- Product decision: future milestones should validate this thesis through the smallest safe product slice, not by widening cleanup paths indiscriminately.
