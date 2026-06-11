use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use chrono::Local;
use serde::Deserialize;

struct ToolEntry {
    category: String,
    tool_name: String,
    #[allow(dead_code)]
    path: String,
    size_bytes: u64,
    risk: String,
    exists: bool,
}

pub fn generate_dashboard(reports_dir: &Path, output: &Path) -> Result<()> {
    let entries: Vec<ToolEntry> = collect_tool_data(reports_dir)?;
    let html = build_dashboard_html(&entries);
    fs::write(output, html)
        .with_context(|| format!("failed to write dashboard to {}", output.display()))?;
    println!("Dashboard written to {}", output.display());
    Ok(())
}

#[derive(Deserialize)]
struct ScanSnapshot {
    #[serde(default)]
    findings: Vec<FindingSnapshot>,
}

#[derive(Deserialize)]
struct FindingSnapshot {
    #[allow(dead_code)]
    id: String,
    name: String,
    category: String,
    path: String,
    exists: bool,
    size_bytes: u64,
    risk: String,
}

fn collect_tool_data(reports_dir: &Path) -> Result<Vec<ToolEntry>> {
    let mut snapshots = Vec::new();

    if reports_dir.exists() {
        for entry in fs::read_dir(reports_dir)? {
            let entry = entry?;
            let path = entry.path();
            let file_name = path
                .file_name()
                .and_then(|v| v.to_str())
                .unwrap_or("");
            if file_name.starts_with("scan-") && file_name.ends_with(".json") {
                snapshots.push(path);
            }
        }
    }

    snapshots.sort_by(|a, b| a.file_name().cmp(&b.file_name()));

    let latest = snapshots
        .pop()
        .ok_or_else(|| anyhow::anyhow!("no scan snapshots found in {}", reports_dir.display()))?;

    let content =
        fs::read_to_string(&latest).with_context(|| format!("failed to read {}", latest.display()))?;
    let snapshot: ScanSnapshot = serde_json::from_str(&content)
        .with_context(|| format!("failed to parse {}", latest.display()))?;

    let entries: Vec<ToolEntry> = snapshot
        .findings
        .into_iter()
        .map(|f| ToolEntry {
            category: f.category,
            tool_name: f.name,
            path: f.path,
            size_bytes: f.size_bytes,
            risk: f.risk,
            exists: f.exists,
        })
        .collect();

    Ok(entries)
}

fn build_dashboard_html(entries: &[ToolEntry]) -> String {
    let generated = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let kpi_total = format_size(entries.iter().map(|e| e.size_bytes).sum());
    let kpi_safe = format_size(
        entries
            .iter()
            .filter(|e| e.risk == "safe" && e.exists)
            .map(|e| e.size_bytes)
            .sum(),
    );
    let kpi_tools = entries.len();

    let categories_html = build_category_treemap(entries);
    let bar_chart_html = build_bar_chart(entries);
    let recommendations_html = build_recommendations(entries);

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>AI Disk Footprint</title>
<style>
:root {{
  --ink: #1a1a1a;
  --paper: #ffffff;
  --accent: #1a5276;
  --accent-light: #e8f0f6;
  --muted: #888888;
  --safe: #1e8449;
  --review: #d4ac0d;
  --dangerous: #c0392b;
  --system: #5b5b5b;
  --sp-xs: 4px;
  --sp-s: 8px;
  --sp-m: 16px;
  --sp-l: 24px;
  --sp-xl: 40px;
  --sp-xxl: 64px;
}}

*, *::before, *::after {{
  box-sizing: border-box;
  margin: 0;
  padding: 0;
}}

body {{
  font-family: Inter, system-ui, -apple-system, sans-serif;
  color: var(--ink);
  background: var(--paper);
  max-width: 1200px;
  margin: 0 auto;
  padding: var(--sp-xxl) var(--sp-xl);
  line-height: 1.5;
}}

header {{
  margin-bottom: var(--sp-xxl);
}}

header h1 {{
  font-size: min(5vw, 42px);
  font-weight: 200;
  color: var(--ink);
  letter-spacing: -0.02em;
  margin-bottom: var(--sp-xs);
}}

header time {{
  font-size: 13px;
  font-weight: 500;
  color: var(--muted);
  text-transform: uppercase;
  letter-spacing: 0.06em;
}}

.kpi-row {{
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: var(--sp-xl);
  margin-bottom: var(--sp-xxl);
}}

.kpi-card {{
  border-top: 2px solid var(--accent-light);
  padding-top: var(--sp-m);
}}

.kpi-value {{
  font-size: min(8vw, 96px);
  font-weight: 200;
  color: var(--accent);
  line-height: 1;
  margin-bottom: var(--sp-xs);
}}

.kpi-label {{
  font-size: 14px;
  font-weight: 600;
  text-transform: uppercase;
  color: var(--muted);
  letter-spacing: 0.06em;
}}

.section-title {{
  font-size: min(3vw, 24px);
  font-weight: 300;
  color: var(--ink);
  margin-bottom: var(--sp-l);
  padding-bottom: var(--sp-s);
  border-bottom: 1px solid var(--accent-light);
}}

.treemap {{
  display: flex;
  gap: var(--sp-s);
  margin-bottom: var(--sp-xxl);
  min-height: 120px;
}}

.treemap-block {{
  display: flex;
  flex-direction: column;
  justify-content: center;
  padding: var(--sp-m);
  border-left: 3px solid transparent;
}}

.treemap-block.safe {{
  background: #eafaf1;
  border-left-color: var(--safe);
}}

.treemap-block.review {{
  background: #fef9e7;
  border-left-color: var(--review);
}}

.treemap-block.dangerous {{
  background: #fdedec;
  border-left-color: var(--dangerous);
}}

.treemap-block.system {{
  background: #f4f4f4;
  border-left-color: var(--system);
}}

.treemap-block-name {{
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: var(--muted);
  margin-bottom: var(--sp-xs);
}}

.treemap-block-size {{
  font-size: min(3vw, 32px);
  font-weight: 200;
  color: var(--ink);
  line-height: 1;
}}

.bar-chart {{
  margin-bottom: var(--sp-xxl);
}}

.bar-row {{
  display: grid;
  grid-template-columns: 200px 1fr 80px;
  gap: var(--sp-m);
  align-items: center;
  padding: var(--sp-s) 0;
  border-bottom: 1px solid #f0f0f0;
}}

.bar-row:last-child {{
  border-bottom: none;
}}

.bar-label {{
  font-size: 13px;
  font-weight: 500;
  color: var(--ink);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}}

.bar-track {{
  height: 16px;
  background: var(--accent-light);
}}

.bar-fill {{
  height: 100%;
  background: var(--accent);
  transition: none;
}}

.bar-fill.safe {{
  background: var(--safe);
}}

.bar-fill.review {{
  background: var(--review);
}}

.bar-fill.dangerous {{
  background: var(--dangerous);
}}

.bar-fill.system {{
  background: var(--system);
}}

.bar-size {{
  font-size: 13px;
  font-weight: 500;
  color: var(--muted);
  text-align: right;
}}

.recommendations {{
  margin-bottom: var(--sp-xxl);
}}

.rec-list {{
  list-style: none;
}}

.rec-item {{
  padding: var(--sp-m);
  margin-bottom: var(--sp-s);
  background: #fafafa;
  border-left: 3px solid var(--accent);
  font-size: 14px;
  line-height: 1.7;
}}

.rec-item strong {{
  font-weight: 600;
}}
</style>
</head>
<body>
<header>
  <h1>AI Disk Footprint</h1>
  <time>Generated: {generated}</time>
</header>
<section class="kpi-row">
  <div class="kpi-card">
    <div class="kpi-value">{kpi_total}</div>
    <div class="kpi-label">Total AI Footprint</div>
  </div>
  <div class="kpi-card">
    <div class="kpi-value">{kpi_safe}</div>
    <div class="kpi-label">Safe to Reclaim</div>
  </div>
  <div class="kpi-card">
    <div class="kpi-value">{kpi_tools}</div>
    <div class="kpi-label">Tools Detected</div>
  </div>
</section>
<section>
  <div class="section-title">By Category</div>
  <div class="treemap">
    {categories_html}
  </div>
</section>
<section>
  <div class="section-title">Tool Breakdown</div>
  <div class="bar-chart">
    {bar_chart_html}
  </div>
</section>
<section>
  <div class="section-title">Recommendations</div>
  <ul class="rec-list">
    {recommendations_html}
  </ul>
</section>
</body>
</html>"#
    )
}

fn build_category_treemap(entries: &[ToolEntry]) -> String {
    use std::collections::BTreeMap;

    let mut cats: BTreeMap<String, (u64, String)> = BTreeMap::new();
    for e in entries {
        let entry = cats.entry(e.category.clone()).or_insert((0, String::new()));
        entry.0 = entry.0.saturating_add(e.size_bytes);
        entry.1 = pick_category_risk(&entry.1, &e.risk);
    }

    if cats.is_empty() {
        return String::from(
            "<div class=\"treemap-block system\" style=\"flex:1\"><div class=\"treemap-block-name\">No data</div><div class=\"treemap-block-size\">0 B</div></div>"
        );
    }

    let total_bytes: u64 = cats.values().map(|(s, _)| *s).sum();

    cats.into_iter()
        .map(|(name, (size, risk))| {
            let flex = if total_bytes > 0 {
                ((size as f64 / total_bytes as f64) * 100.0).max(3.0)
            } else {
                1.0
            };
            format!(
                "<div class=\"treemap-block {risk}\" style=\"flex:{flex:.0}\"><div class=\"treemap-block-name\">{name}</div><div class=\"treemap-block-size\">{}</div></div>",
                format_size(size)
            )
        })
        .collect::<Vec<_>>()
        .join("\n    ")
}

fn pick_category_risk(current: &str, risk: &str) -> String {
    fn rank(r: &str) -> u8 {
        match r {
            "dangerous" => 3,
            "review" => 2,
            "safe" => 1,
            _ => 0,
        }
    }
    if rank(risk) > rank(current) {
        risk.to_string()
    } else {
        current.to_string()
    }
}

fn build_bar_chart(entries: &[ToolEntry]) -> String {
    let max_size = entries.iter().map(|e| e.size_bytes).max().unwrap_or(1).max(1);

    let mut rows = String::new();
    let mut shown: Vec<(&ToolEntry, f64)> = entries
        .iter()
        .filter(|e| e.exists && e.size_bytes > 0)
        .map(|e| {
            let pct = (e.size_bytes as f64 / max_size as f64 * 100.0).round().max(1.0);
            (e, pct)
        })
        .collect();
    shown.sort_by(|a, b| b.0.size_bytes.cmp(&a.0.size_bytes));

    let max_display = 20;
    for (e, pct) in shown.iter().take(max_display) {
        rows.push_str(&format!(
            r#"<div class="bar-row"><span class="bar-label">{name}</span><div class="bar-track"><div class="bar-fill {risk}" style="width:{pct}%"></div></div><span class="bar-size">{size}</span></div>
"#,
            name = html_escape(&e.tool_name),
            risk = e.risk,
            pct = pct,
            size = format_size(e.size_bytes),
        ));
    }

    if shown.len() > max_display {
        rows.push_str(&format!(
            r#"<div class="bar-row"><span class="bar-label">+{} more tools</span><div class="bar-track"></div><span class="bar-size"></span></div>
"#,
            shown.len() - max_display
        ));
    }

    if rows.is_empty() {
        rows.push_str(
            r#"<div class="bar-row"><span class="bar-label">No tools detected with disk usage</span><div class="bar-track"></div><span class="bar-size"></span></div>
"#,
        );
    }

    rows
}

fn build_recommendations(entries: &[ToolEntry]) -> String {
    let mut recs = Vec::new();

    let safe_bytes: u64 = entries
        .iter()
        .filter(|e| e.risk == "safe" && e.exists)
        .map(|e| e.size_bytes)
        .sum();
    if safe_bytes > 0 {
        recs.push(format!(
            "Run <strong>aidisk clean --safe-only --quarantine-root C:\\Quarantine\\ai-footprint</strong> to safely reclaim <strong>{}</strong> of AI tool caches and temporary files with minimal risk.",
            format_size(safe_bytes)
        ));
    }

    let review_bytes: u64 = entries
        .iter()
        .filter(|e| e.risk == "review" && e.exists)
        .map(|e| e.size_bytes)
        .sum();
    if review_bytes > 0 {
        recs.push(format!(
            "Review <strong>{}</strong> of model weights, checkpoints, and config files before removal. Use <strong>aidisk plan --category ai-model</strong> to inspect.",
            format_size(review_bytes)
        ));
    }

    let dangerous_bytes: u64 = entries
        .iter()
        .filter(|e| e.risk == "dangerous" && e.exists)
        .map(|e| e.size_bytes)
        .sum();
    if dangerous_bytes > 0 {
        recs.push(format!(
            "<strong>{}</strong> classified as dangerous. These include project sources and installed tools — inspect individually with <strong>aidisk doctor</strong>.",
            format_size(dangerous_bytes)
        ));
    }

    let missing = entries.iter().filter(|e| !e.exists).count();
    let total = entries.len();
    if missing > 0 {
        recs.push(format!(
            "<strong>{missing}</strong> out of <strong>{total}</strong> configured paths are absent (tools not installed or previously removed)."
        ));
    }

    recs.push(
        "Run <strong>aidisk scan --json</strong> periodically and visualize to track AI footprint growth over time."
            .to_string(),
    );

    recs.into_iter()
        .map(|r| format!("<li class=\"rec-item\">{r}</li>"))
        .collect::<Vec<_>>()
        .join("\n    ")
}

fn format_size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut value = bytes as f64;
    let mut unit = 0_usize;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{} {}", bytes, UNITS[unit])
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn make_entry(
        category: &str,
        name: &str,
        size_bytes: u64,
        risk: &str,
        exists: bool,
    ) -> ToolEntry {
        ToolEntry {
            category: category.to_string(),
            tool_name: name.to_string(),
            path: format!("C:\\test\\{}", name),
            size_bytes,
            risk: risk.to_string(),
            exists,
        }
    }

    #[test]
    fn generate_dashboard_writes_html_file() {
        let temp = tempdir().expect("tempdir should exist");
        let reports_dir = temp.path().join("reports");
        fs::create_dir_all(&reports_dir).expect("reports dir should exist");

        fs::write(
            reports_dir.join("scan-20260611-103000-000.json"),
            r#"{"findings": [{"id": "test", "name": "Test Tool", "category": "ai-ide", "path": "/test", "exists": true, "size_bytes": 1024, "risk": "safe"}]}"#,
        )
        .expect("scan should be written");

        let output = temp.path().join("dashboard.html");
        generate_dashboard(&reports_dir, &output).expect("dashboard should generate");

        let html = fs::read_to_string(&output).expect("output readable");
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("AI Disk Footprint"));
    }

    #[test]
    fn empty_reports_dir_returns_error() {
        let temp = tempdir().expect("tempdir should exist");
        let reports_dir = temp.path().join("empty-reports");
        fs::create_dir_all(&reports_dir).expect("reports dir should exist");

        let output = temp.path().join("empty.html");
        let err = generate_dashboard(&reports_dir, &output).unwrap_err();
        assert!(err.to_string().contains("no scan snapshots"));
    }

    #[test]
    fn picks_latest_scan_by_filename() {
        let temp = tempdir().expect("tempdir should exist");
        let reports_dir = temp.path().join("reports");
        fs::create_dir_all(&reports_dir).expect("reports dir should exist");

        fs::write(
            reports_dir.join("scan-20260601-000000-000.json"),
            r#"{"findings": [{"id": "old", "name": "Old Tool", "category": "test", "path": "/old", "exists": true, "size_bytes": 100, "risk": "safe"}]}"#,
        )
        .unwrap();
        fs::write(
            reports_dir.join("scan-20260611-103000-000.json"),
            r#"{"findings": [{"id": "new", "name": "New Tool", "category": "test", "path": "/new", "exists": true, "size_bytes": 500, "risk": "review"}]}"#,
        )
        .unwrap();

        let output = temp.path().join("latest.html");
        generate_dashboard(&reports_dir, &output).expect("dashboard should generate");

        let html = fs::read_to_string(&output).unwrap();
        assert!(html.contains("New Tool"));
        assert!(!html.contains("Old Tool"));
    }

    #[test]
    fn dashboard_contains_required_sections() {
        let entries = vec![
            make_entry("ai-ide", "Cursor Cache", 500_000_000, "safe", true),
            make_entry("ai-model", "Ollama Models", 10_000_000_000, "review", true),
            make_entry("ai-runtime", "CUDA Toolkit", 8_000_000_000, "dangerous", true),
        ];
        let html = build_dashboard_html(&entries);

        assert!(html.contains("Total AI Footprint"));
        assert!(html.contains("Safe to Reclaim"));
        assert!(html.contains("Tools Detected"));
        assert!(html.contains("By Category"));
        assert!(html.contains("Tool Breakdown"));
        assert!(html.contains("Recommendations"));
    }

    #[test]
    fn no_external_dependencies_in_html() {
        let entries = vec![make_entry("ai-ide", "Test", 1024, "safe", true)];
        let html = build_dashboard_html(&entries);

        assert!(!html.contains("cdn"));
        assert!(!html.contains("http://"));
        assert!(!html.contains("https://"));
    }

    #[test]
    fn no_css_decorations_in_html() {
        let entries = vec![make_entry("ai-ide", "Test", 1024, "safe", true)];
        let html = build_dashboard_html(&entries);

        assert!(!html.contains("border-radius"));
        assert!(!html.contains("box-shadow"));
        assert!(!html.contains("linear-gradient"));
        assert!(!html.contains("radial-gradient"));
    }

    #[test]
    fn format_size_displays_correct_units() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(500), "500 B");
        assert_eq!(format_size(1024), "1.0 KB");
        assert_eq!(format_size(1_048_576), "1.0 MB");
        assert_eq!(format_size(1_073_741_824), "1.0 GB");
    }

    #[test]
    fn html_escape_handles_special_chars() {
        assert_eq!(html_escape("<script>"), "&lt;script&gt;");
        assert_eq!(html_escape("a & b"), "a &amp; b");
        assert_eq!(html_escape("\"quoted\""), "&quot;quoted&quot;");
    }

    #[test]
    fn bar_chart_limits_to_20_entries() {
        let entries: Vec<ToolEntry> = (1..=25)
            .map(|i| make_entry("test", &format!("Tool {i}"), i * 1000, "safe", true))
            .collect();
        let html = build_bar_chart(&entries);

        let count = html.matches("bar-row").count();
        assert_eq!(count, 21);
        assert!(html.contains("+5 more tools"));
    }

    #[test]
    fn recommendations_include_safe_advice() {
        let entries = vec![make_entry("ai-ide", "Cache", 1_000_000_000, "safe", true)];
        let html = build_dashboard_html(&entries);

        assert!(html.contains("aidisk clean --safe-only"));
        assert!(html.contains("quarantine-root"));
    }

    #[test]
    fn recommendations_note_missing_paths() {
        let entries = vec![
            make_entry("ai-ide", "Installed", 100, "safe", true),
            make_entry("ai-cli", "Not Here", 0, "safe", false),
        ];
        let html = build_dashboard_html(&entries);

        assert!(html.contains("1</strong> out of <strong>2</strong>"));
    }

    #[test]
    fn treemap_picks_highest_risk_per_category() {
        let entries = vec![
            make_entry("ai-model", "Safe Model", 100, "safe", true),
            make_entry("ai-model", "Dangerous Model", 200, "dangerous", true),
            make_entry("ai-model", "Review Model", 300, "review", true),
        ];
        let html = build_dashboard_html(&entries);

        assert!(html.contains("treemap-block dangerous"));
    }

    #[test]
    fn empty_entries_produces_valid_html() {
        let entries: Vec<ToolEntry> = Vec::new();
        let html = build_dashboard_html(&entries);

        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("AI Disk Footprint"));
        assert!(html.contains("No data"));
    }
}
