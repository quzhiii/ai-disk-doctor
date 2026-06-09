# Phase 10 Cross-Platform Scheduler Adapters Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Extend the local governance scheduler surface from Windows Task Scheduler to cron, launchd, and systemd timer while keeping the existing governance event contract and Rust anomaly core unchanged.

**Architecture:** Keep scheduler adapters script-level and platform-specific. Reuse `run-governance.ps1` as the existing Windows entrypoint and add matching platform launcher/register/show/unregister/test-run flows for cron, launchd, and systemd timer. Keep this slice scheduler-first; concrete notifier adapter expansion stays later and continues to depend on the stable governance event contract.

**Tech Stack:** PowerShell, shell scripts, platform scheduler config files, existing governance artifact contract, Rust CLI unchanged.

---

## Scope

- Add cross-platform scheduler adapters for `cron`, `launchd`, and `systemd timer`.
- Keep Windows Task Scheduler as the existing reference implementation.
- Preserve `run-governance.ps1`, `governance-event.json`, and the current local governance artifact contract.

## Scheduler-First Decision

This slice is explicitly **scheduler-first**.

- Phase 10 prioritizes cross-platform scheduler adapters before notifier adapter expansion.
- Concrete notifier adapter work remains later because it introduces secrets, platform APIs, delivery retry semantics, and configuration coupling.
- The generic webhook and stable local governance event payload remain the abstraction boundary for future notifier adapters.

## Acceptance Criteria

- A Phase 10 roadmap entry exists in `docs/execution-plan.md`.
- The roadmap explicitly states `scheduler-first` and places notifier adapter expansion after scheduler adapters.
- The plan names `cron`, `launchd`, and `systemd timer` as first-class targets.
- The plan preserves the existing governance entrypoint and mentions `run-governance.ps1` as the current Windows reference workflow.
- No requirement in this slice modifies the Rust anomaly engine or binds the project to a concrete notifier adapter.

## Non-Goals

- Do not add a concrete notifier adapter in this slice.
- Do not move governance delivery logic into the Rust core.
- Do not add a daemon or background resident service.
- Do not redesign the governance event schema.

## Recommended Sequencing

1. Define the shared scheduler adapter contract: register, show, unregister, and immediate test run.
2. Map the contract onto cron.
3. Map the contract onto launchd.
4. Map the contract onto systemd timer.
5. Update docs and artifact tests.

## Future Follow-Up

- Notifier adapter expansion after scheduler coverage is stable.
- Concrete platform delivery options such as Feishu, Slack, or WeCom only after the scheduler layer is complete and the generic webhook path remains stable.
