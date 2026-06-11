# Phase 17: AI Footprint Visual Dashboard

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add `aidisk visualize --html` command that generates a single-file Swiss Style HTML dashboard showing AI disk footprint with treemap, bar charts, and KPI tower.

**Architecture:** New Rust module `visualize.rs` reads scan/anomaly JSON and generates a self-contained HTML file with inline CSS/JS. Design follows Swiss Style principles (IKB accent, grid layout, data-driven, no shadows). No external CDN dependencies.

**Tech Stack:** Rust, serde_json, HTML/CSS generation, minimal embedded JS for charts.

---

## Task 1: Add visualize CLI command

**Files:**
- Modify: `aidisk/src/main.rs` (add `Visualize` subcommand)
- Create: `aidisk/src/visualize.rs` (module skeleton)

**Step 1: Write failing test**

Add CLI test in `aidisk/tests/visualize_cli.rs` that verifies `--help` output.

**Step 2: Implement module structure**

```rust
pub fn generate_dashboard(reports_dir: &Path, output_path: &Path, format: OutputFormat) -> Result<()>
```

**Step 3: Commit**

---

## Task 2: Implement HTML dashboard generator

**Files:**
- Modify: `aidisk/src/visualize.rs`

**Design specs (Swiss Style reference):**

- Accent color: `#1a5276` (IKB blue)
- Font: Inter 300/400/600, monospace for numbers
- Layout: Top KPI row (total footprint, safe reclaim %, tool count) → Category treemap (CSS grid) → Per-tool bar chart → Recommendations
- No gradients, no shadows, no rounded corners
- "越大越细，越小越粗" — KPI numbers in weight 200, labels in weight 600

**Dashboard sections:**
1. **Header** — "AI Disk Footprint" + generated timestamp
2. **KPI Tower** — Total AI footprint / Safe to reclaim / Tools detected
3. **Category Treemap** — Visual blocks sized by space usage
4. **Tool Bar Chart** — Horizontal bars per tool, color-coded by risk
5. **Recommendations** — Actionable suggestions

**Commit**

---

## Task 3: Integration test with real data

**Files:**
- Test: `aidisk/tests/visualize_cli.rs`

**Step 1: Add integration test** that runs `aidisk visualize --html` with test fixtures.

**Step 2: Verify HTML output** contains expected elements.

**Step 3: Run full test suite** → `cargo test --all`

**Commit**

---

## Task 4: Real-world validation + README + push

**Step 1: Build release and run** on local machine.

**Step 2: Add README entry.**

**Step 3: Commit and push.**
