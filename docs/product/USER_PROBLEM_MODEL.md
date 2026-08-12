# User And Problem Model

## Primary Users

| User | Need |
|---|---|
| AI power users / vibe coders | understand storage created by AI agents, AI IDEs, local models, runtimes, and build artifacts |
| Non-specialist builders | plain-language answers about what can be removed and what must be protected |
| Developers and independent builders | stable CLI/JSON, evidence, policy, and safe automation hooks |
| Teams | later policy, audit, fleet health, and deployment controls after individual PMF evidence |

## Problem Categories

| Problem | User symptom | Current repo support | Gap |
|---|---|---|---|
| Invisible AI storage growth | disk suddenly fills up | scan, doctor, dashboard, diff, anomaly | attribution is still path/rule-centric |
| Meaning gap | user sees large AI folders but does not know what they mean | risk/action/reason fields, doctor recommendations, model inventory | plain-language explanations are not yet universal |
| Deletion risk | cleanup can break tools or lose state | dry-run, policy gates, quarantine, restore, sensitive markers | recovery-value axis is not universal |
| Recovery uncertainty | latest Agent output is worse; user wants earlier state | quarantine restore, scan history, model rollback metadata | no Git/Agent/project recovery coverage model yet |
| No lifecycle view | same storage grows again after cleanup | snapshots, diff, anomaly, governance scripts | Desktop Activity/Recovery UX not implemented |

## User Literacy Levels

| Level | Required output style |
|---|---|
| Beginner | what this is, why it exists, whether it is safe, what happens if removed, whether it can be recovered |
| Practitioner | size, age evidence, origin tool, rule, action, risk, policy, confidence, and action mechanism |
| Expert / Agent | stable JSON, exact paths when requested, evidence, policy, confidence, deterministic dry-run/execution behavior |

Product decision: one underlying truth model should serve all three levels. GUI/Desktop must not define separate safety semantics.

## Storage-Recovery Matrix

| | Low Recovery Value | High Recovery Value |
|---|---|---|
| High Reclaim Confidence | strong cleanup candidate | retention-policy candidate; explain tradeoff |
| Low Reclaim Confidence | review, official-tool, or unknown | protect by default |

Current repo fact: risk, action, report-only, guide, partial, and quarantine metadata exist today. Product decision: future work should add explicit recovery value and recovery impact semantics before exposing richer cleanup/recovery UX.

## Required Future Explanation Fields

Future GUI/Desktop cleanup or recovery items should aim to expose:

- human label
- origin/tool
- asset type
- physical and logical size where available
- reclaimable estimate
- reclaim confidence
- recovery value
- recovery/rebuild method
- evidence
- last-use or age evidence when reliable
- reference state when known
- risk class
- recommended action
- action mechanism: quarantine, official CLI, report-only, protect, archive, or manual review
- plain-language why explanation
