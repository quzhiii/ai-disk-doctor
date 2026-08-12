# Collaboration Protocol

## Roles

### Web ChatGPT - Product / Architecture / Acceptance Control Plane

Responsible for:

- product direction
- research synthesis
- milestone scope
- acceptance criteria
- architecture/safety review
- PR review
- next-brief issuance

Web ChatGPT does not accept completion based only on a Local Agent summary.

### Local Agent - Implementation Plane

Responsible for:

- repository reality audit
- technical implementation
- tests
- documentation changes within brief scope
- branch and commits
- Draft PR
- transparent reporting of uncertainty, skipped checks, and failures

### GitHub - Source Of Truth

Required evidence:

- branch
- commits
- changed files
- PR
- CI/test result
- reviewable diff

## Standard Cycle

### Step A - Brief

One milestone only. The brief should state objective, scope, non-goals, safety constraints, expected files, tests, acceptance pack, and stop condition.

### Step B - Reality Audit

Before editing, inspect current branch/head, relevant code, docs, tests, workflows, licenses, and agent instructions. If repo reality materially invalidates the brief, stop and report instead of improvising a large redesign.

### Step C - Implementation

Use a feature branch. Keep changes minimal and coherent. Do not add opportunistic features.

### Step D - Verification

Run relevant format, lint, tests, and platform checks that are realistically available. Document anything not run.

### Step E - GitHub Handoff

Open a Draft PR with:

1. Objective
2. Baseline commit
3. Head commit
4. Files changed
5. What was implemented
6. Architecture decisions
7. Safety impact
8. Tests and exact results
9. Known limitations
10. Deferred work
11. Questions requiring owner decision

### Step F - Stop

Do not start the next milestone. Return the PR URL/number and acceptance pack.

### Step G - Web Acceptance

Web ChatGPT reviews the real PR and returns PASS, CONDITIONAL PASS, or BLOCK. Only then is the next brief issued.

## Destructive-Change Rule

Any PR touching cleaner, planner execution semantics, path containment, quarantine, restore, real deletion, privilege escalation, or Agent recovery asset cleanup must be explicitly labeled high-risk and requires line-by-line safety review.

## Product Decision Filter

Before adding a feature, ask:

1. Does it solve AI/developer storage, recovery, or workspace-health pain?
2. Does AI-aware semantics create meaningful value?
3. Can the behavior be explained?
4. Can risky actions be previewed and recovered?
5. Does it preserve local-first/privacy principles?
6. Can CLI/Desktop share one execution truth?
7. Is there user evidence, or is this still a hypothesis?
8. Is this milestone the smallest safe way to validate it?
