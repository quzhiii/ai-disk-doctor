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

#[derive(serde::Serialize)]
struct ToolEntryJson {
    index: usize,
    name: String,
    category: String,
    path: String,
    size_bytes: u64,
    size_display: String,
    risk: String,
    exists: bool,
    suggestion_zh: String,
    suggestion_en: String,
}

fn tool_suggestion_zh(e: &ToolEntry) -> String {
    match (e.risk.as_str(), e.category.as_str()) {
        ("safe", _) =>
            "可以安全清理的缓存或临时文件。建议使用 aidisk clean --safe-only --quarantine-root C:\\Quarantine\\ai-footprint 清理。"
                .to_string(),
        ("review", "ai-model") =>
            "AI 模型权重文件。清理前请确认不再需要此模型。使用 aidisk plan --category ai-model 检查详情。"
                .to_string(),
        ("review", _) =>
            "需要人工评估的文件。建议检查内容后决定是否保留。"
                .to_string(),
        ("dangerous", "ai-ide") =>
            "IDE 或编辑器相关文件。清理可能影响开发环境配置，请谨慎操作。"
                .to_string(),
        ("dangerous", "ai-runtime") =>
            "运行时环境文件。清理可能导致工具无法正常运行。"
                .to_string(),
        ("dangerous", _) =>
            "高风险文件（项目源码或已安装工具）。建议使用 aidisk doctor 逐个检查。"
                .to_string(),
        _ => "未知风险级别。请手动评估后再决定是否清理。"
            .to_string(),
    }
}

fn tool_suggestion_en(e: &ToolEntry) -> String {
    match (e.risk.as_str(), e.category.as_str()) {
        ("safe", _) =>
            "Cache or temporary files that can be safely cleaned. Use aidisk clean --safe-only --quarantine-root C:\\Quarantine\\ai-footprint to reclaim."
                .to_string(),
        ("review", "ai-model") =>
            "AI model weight files. Confirm you no longer need this model before removal. Use aidisk plan --category ai-model for details."
                .to_string(),
        ("review", _) =>
            "Files requiring manual review. Check contents before deciding to keep or remove."
                .to_string(),
        ("dangerous", "ai-ide") =>
            "IDE or editor related files. Removal may affect your development environment configuration."
                .to_string(),
        ("dangerous", "ai-runtime") =>
            "Runtime environment files. Removal may cause tools to malfunction."
                .to_string(),
        ("dangerous", _) =>
            "High-risk files (project source or installed tools). Use aidisk doctor to inspect individually."
                .to_string(),
        _ => "Unknown risk level. Please evaluate manually before cleaning."
            .to_string(),
    }
}

fn risk_stats(entries: &[ToolEntry], risk: &str) -> (usize, u64) {
    let count = entries
        .iter()
        .filter(|e| e.risk == risk && e.exists)
        .count();
    let total = entries
        .iter()
        .filter(|e| e.risk == risk && e.exists)
        .map(|e| e.size_bytes)
        .sum();
    (count, total)
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

    let json_entries: Vec<ToolEntryJson> = entries
        .iter()
        .enumerate()
        .map(|(i, e)| ToolEntryJson {
            index: i,
            name: e.tool_name.clone(),
            category: e.category.clone(),
            path: e.path.clone(),
            size_bytes: e.size_bytes,
            size_display: format_size(e.size_bytes),
            risk: e.risk.clone(),
            exists: e.exists,
            suggestion_zh: tool_suggestion_zh(e),
            suggestion_en: tool_suggestion_en(e),
        })
        .collect();
    let entries_json = serde_json::to_string(&json_entries).unwrap_or_else(|_| "[]".to_string());

    let i18n = serde_json::json!({
        "zh": {
            "title": "AI 磁盘足迹",
            "generated": "生成时间",
            "total_footprint": "AI 总占用",
            "safe_to_reclaim": "可安全回收",
            "tools_detected": "已检测工具",
            "by_category": "按类别",
            "tool_breakdown": "工具明细",
            "safe_reclaim_title": "可安全回收",
            "select_all": "全部选中",
            "deselect_all": "取消全选",
            "selected_summary": "已选 {n} 项，可回收 {size}",
            "disclaimer": "以上均为只读报告，不会自动删除任何文件。如需清理，请手动操作。",
            "kpi_total_tip": "这是你电脑上所有 AI 工具占用的总空间",
            "kpi_safe_tip": "可以安全清理的缓存和临时文件总大小",
            "kpi_tools_tip": "检测到的 AI 相关工具数量",
            "risk_safe": "安全",
            "risk_review": "需评估",
            "risk_dangerous": "危险",
            "path_label": "路径",
            "category_label": "类别",
            "risk_label": "风险",
            "size_label": "大小",
            "suggestion_label": "建议",
            "no_data": "暂无数据"
        },
        "en": {
            "title": "AI Disk Footprint",
            "generated": "Generated",
            "total_footprint": "Total AI Footprint",
            "safe_to_reclaim": "Safe to Reclaim",
            "tools_detected": "Tools Detected",
            "by_category": "By Category",
            "tool_breakdown": "Tool Breakdown",
            "safe_reclaim_title": "Safe to Reclaim",
            "select_all": "Select All",
            "deselect_all": "Deselect All",
            "selected_summary": "{n} items selected, {size} reclaimable",
            "disclaimer": "This is a read-only report. No files will be automatically deleted. To clean up, please do so manually.",
            "kpi_total_tip": "This is the total space used by all AI tools on your computer",
            "kpi_safe_tip": "Total size of cache and temporary files that can be safely cleaned",
            "kpi_tools_tip": "Number of detected AI-related tools",
            "risk_safe": "Safe",
            "risk_review": "Review",
            "risk_dangerous": "Dangerous",
            "path_label": "Path",
            "category_label": "Category",
            "risk_label": "Risk",
            "size_label": "Size",
            "suggestion_label": "Suggestion",
            "no_data": "No data"
        }
    });
    let i18n_json = serde_json::to_string(&i18n).unwrap_or_else(|_| "{}".to_string());

    let js = JS_TEMPLATE
        .replace("__I18N__", &i18n_json)
        .replace("__TOOL_DATA__", &entries_json);

    let treemap_html = build_treemap_html(entries);
    let bar_chart_html = build_bar_chart_html(entries);
    let risk_cards_html = build_risk_cards_html(entries);
    let reclaim_html = build_reclaim_html(entries);

    format!(
        r#"<!DOCTYPE html>
<html lang="zh">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>AI 磁盘足迹</title>
<style>
{css}
</style>
</head>
<body>
<header>
  <h1 data-i18n="title">AI 磁盘足迹</h1>
  <div class="header-right">
    <button id="lang-toggle">🌐 EN</button>
    <time><span data-i18n="generated">生成时间</span>: {generated}</time>
  </div>
</header>

<section class="kpi-row">
  <div class="kpi-card" id="kpi-total">
    <div class="kpi-value">{kpi_total}</div>
    <div class="kpi-label" data-i18n="total_footprint">AI 总占用</div>
    <div class="kpi-tooltip" data-i18n="kpi_total_tip">这是你电脑上所有 AI 工具占用的总空间</div>
  </div>
  <div class="kpi-card" id="kpi-safe">
    <div class="kpi-value">{kpi_safe}</div>
    <div class="kpi-label" data-i18n="safe_to_reclaim">可安全回收</div>
    <div class="kpi-tooltip" data-i18n="kpi_safe_tip">可以安全清理的缓存和临时文件总大小</div>
  </div>
  <div class="kpi-card" id="kpi-tools">
    <div class="kpi-value">{kpi_tools}</div>
    <div class="kpi-label" data-i18n="tools_detected">已检测工具</div>
    <div class="kpi-tooltip" data-i18n="kpi_tools_tip">检测到的 AI 相关工具数量</div>
  </div>
</section>

<section class="risk-cards">
  {risk_cards_html}
</section>

<section class="treemap-section">
  <h2 class="section-title" data-i18n="by_category">按类别</h2>
  <div class="treemap" id="treemap">
    {treemap_html}
  </div>
</section>

<section class="bar-section">
  <h2 class="section-title" data-i18n="tool_breakdown">工具明细</h2>
  <div class="bar-chart" id="bar-chart">
    {bar_chart_html}
  </div>
</section>

<section class="safe-reclaim" id="safe-reclaim">
  <h2 class="section-title" data-i18n="safe_reclaim_title">可安全回收</h2>
  <div class="reclaim-controls">
    <button id="select-all-btn" data-i18n="select_all">全部选中</button>
    <div class="selected-summary" id="reclaim-summary">已选 0 项，可回收 0 B</div>
  </div>
  <ul class="reclaim-list" id="reclaim-list">
    {reclaim_html}
  </ul>
  <p class="disclaimer" data-i18n="disclaimer">以上均为只读报告，不会自动删除任何文件。如需清理，请手动操作。</p>
</section>

<script>
{js}
</script>
</body>
</html>"#,
        css = CSS,
        generated = generated,
        kpi_total = kpi_total,
        kpi_safe = kpi_safe,
        kpi_tools = kpi_tools,
        risk_cards_html = risk_cards_html,
        treemap_html = treemap_html,
        bar_chart_html = bar_chart_html,
        reclaim_html = reclaim_html,
        js = js,
    )
}

fn build_treemap_html(entries: &[ToolEntry]) -> String {
    use std::collections::BTreeMap;

    let mut cats: BTreeMap<String, (u64, String)> = BTreeMap::new();
    for e in entries {
        let entry = cats.entry(e.category.clone()).or_insert((0, String::new()));
        entry.0 = entry.0.saturating_add(e.size_bytes);
        entry.1 = pick_category_risk(&entry.1, &e.risk);
    }

    if cats.is_empty() {
        return r#"<div class="treemap-block system" style="flex:1"><div class="treemap-block-name" data-i18n="no_data">暂无数据</div><div class="treemap-block-size">0 B</div></div>"#.to_string();
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
                r#"<div class="treemap-block {risk}" data-category="{name}" data-risk="{risk}" style="flex:{flex:.0}"><div class="treemap-block-name">{name}</div><div class="treemap-block-size">{size_display}</div></div>"#,
                risk = risk,
                name = html_escape(&name),
                flex = flex,
                size_display = format_size(size),
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

fn build_bar_chart_html(entries: &[ToolEntry]) -> String {
    let max_size = entries
        .iter()
        .map(|e| e.size_bytes)
        .max()
        .unwrap_or(1)
        .max(1);

    let mut shown: Vec<(usize, &ToolEntry, f64)> = entries
        .iter()
        .enumerate()
        .filter(|(_, e)| e.exists && e.size_bytes > 0)
        .map(|(i, e)| {
            let pct = (e.size_bytes as f64 / max_size as f64 * 100.0)
                .round()
                .max(1.0);
            (i, e, pct)
        })
        .collect();
    shown.sort_by(|a, b| b.1.size_bytes.cmp(&a.1.size_bytes));

    let max_display = 20;
    let mut rows = String::new();

    for (idx, e, pct) in shown.iter().take(max_display) {
        rows.push_str(&format!(
            r#"<div class="bar-group" data-category="{cat}" data-risk="{risk}"><div class="bar-row bar-clickable" data-tool-index="{idx}"><span class="bar-label">{name}</span><div class="bar-track"><div class="bar-fill {risk}" style="width:{pct}%"></div></div><span class="bar-size">{size}</span></div></div>
"#,
            cat = html_escape(&e.category),
            risk = e.risk,
            idx = idx,
            name = html_escape(&e.tool_name),
            pct = pct,
            size = format_size(e.size_bytes),
        ));
    }

    if shown.len() > max_display {
        let remaining = shown.len() - max_display;
        rows.push_str(&format!(
            r#"<div class="bar-group"><div class="bar-row"><span class="bar-label">+{remaining} more tools</span><div class="bar-track"></div><span class="bar-size"></span></div></div>
"#,
            remaining = remaining
        ));
    }

    if rows.is_empty() {
        rows.push_str(
            r#"<div class="bar-group"><div class="bar-row"><span class="bar-label" data-i18n="no_data">暂无数据</span><div class="bar-track"></div><span class="bar-size"></span></div></div>
"#,
        );
    }

    rows
}

fn build_risk_cards_html(entries: &[ToolEntry]) -> String {
    let risks = [
        ("safe", "risk_safe", "安全"),
        ("review", "risk_review", "需评估"),
        ("dangerous", "risk_dangerous", "危险"),
    ];

    risks
        .iter()
        .map(|(risk_key, i18n_key, fallback)| {
            let (count, total) = risk_stats(entries, risk_key);
            format!(
                r#"<div class="risk-card {risk_key}" data-risk="{risk_key}"><div class="risk-card-name" data-i18n="{i18n_key}">{fallback}</div><div class="risk-card-value">{total_size}</div><div class="risk-card-count">{count} items</div></div>"#,
                risk_key = risk_key,
                i18n_key = i18n_key,
                fallback = fallback,
                total_size = format_size(total),
                count = count,
            )
        })
        .collect::<Vec<_>>()
        .join("\n  ")
}

fn build_reclaim_html(entries: &[ToolEntry]) -> String {
    let safe_entries: Vec<&ToolEntry> = entries
        .iter()
        .filter(|e| e.risk == "safe" && e.exists)
        .collect();

    if safe_entries.is_empty() {
        return r#"<li class="reclaim-item"><span class="no-data" data-i18n="no_data">暂无数据</span></li>"#.to_string();
    }

    safe_entries
        .iter()
        .map(|e| {
            format!(
                r#"<li class="reclaim-item" data-size="{size}"><label><input type="checkbox" class="reclaim-checkbox"><span class="reclaim-name">{name}</span><span class="reclaim-path">{path}</span><span class="reclaim-size">{size_display}</span></label></li>"#,
                size = e.size_bytes,
                name = html_escape(&e.tool_name),
                path = html_escape(&e.path),
                size_display = format_size(e.size_bytes),
            )
        })
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

const CSS: &str = include_str!("visualize_css.txt");

const JS_TEMPLATE: &str = include_str!("visualize_js.txt");

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
        assert!(html.contains("AI"));
        assert!(html.contains("Disk Footprint"));
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

        assert!(html.contains("AI"));
        assert!(html.contains("Tool Breakdown"));
        assert!(html.contains("By Category"));
        assert!(html.contains("button id=\"lang-toggle\""));
        assert!(html.contains("data-i18n"));
        assert!(html.contains("window.I18N"));
        assert!(html.contains("addEventListener"));
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
        let html = build_bar_chart_html(&entries);

        let count = html.matches("bar-group").count();
        assert_eq!(count, 21);
        assert!(html.contains("+5 more tools"));
    }

    #[test]
    fn recommendations_note_missing_paths() {
        let entries = vec![
            make_entry("ai-ide", "Installed", 100, "safe", true),
            make_entry("ai-cli", "Not Here", 0, "safe", false),
        ];
        let html = build_dashboard_html(&entries);

        assert!(html.contains("no_data"));
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
        assert!(html.contains("no_data"));
    }

    #[test]
    fn dashboard_contains_bilingual_support() {
        let entries = vec![make_entry("ai-ide", "Test", 1024, "safe", true)];
        let html = build_dashboard_html(&entries);

        assert!(html.contains("data-i18n"));
        assert!(html.contains("window.I18N"));
        assert!(html.contains("button id=\"lang-toggle\""));
        assert!(html.contains("AI"));
        assert!(html.contains("Disk Footprint"));
    }

    #[test]
    fn dashboard_contains_interactive_js() {
        let entries = vec![make_entry("ai-ide", "Test", 1024, "safe", true)];
        let html = build_dashboard_html(&entries);

        assert!(html.contains("addEventListener"));
        assert!(html.contains("treemap-block"));
        assert!(html.contains("bar-clickable"));
    }

    #[test]
    fn dashboard_contains_risk_cards() {
        let entries = vec![
            make_entry("ai-ide", "Safe A", 1000, "safe", true),
            make_entry("ai-model", "Review A", 2000, "review", true),
            make_entry("ai-runtime", "Danger A", 3000, "dangerous", true),
        ];
        let html = build_dashboard_html(&entries);

        assert!(html.contains("risk-card safe"));
        assert!(html.contains("risk-card review"));
        assert!(html.contains("risk-card dangerous"));
    }

    #[test]
    fn dashboard_contains_reclaim_section() {
        let entries = vec![make_entry("ai-ide", "Cache", 1_000_000_000, "safe", true)];
        let html = build_dashboard_html(&entries);

        assert!(html.contains("reclaim-checkbox"));
        assert!(html.contains("select-all-btn"));
        assert!(html.contains("disclaimer"));
    }

    #[test]
    fn risk_stats_calculates_correctly() {
        let entries = vec![
            make_entry("ai-ide", "A", 100, "safe", true),
            make_entry("ai-ide", "B", 200, "safe", true),
            make_entry("ai-ide", "C", 50, "safe", false),
        ];
        let (count, total) = risk_stats(&entries, "safe");
        assert_eq!(count, 2);
        assert_eq!(total, 300);
    }

    #[test]
    fn tool_suggestions_vary_by_risk_and_category() {
        let safe_entry = make_entry("ai-ide", "Cache", 1000, "safe", true);
        let review_model = make_entry("ai-model", "Ollama", 5000, "review", true);
        let dangerous_runtime = make_entry("ai-runtime", "CUDA", 8000, "dangerous", true);

        let zh_safe = tool_suggestion_zh(&safe_entry);
        let zh_model = tool_suggestion_zh(&review_model);
        let zh_runtime = tool_suggestion_zh(&dangerous_runtime);

        assert!(zh_safe.contains("可以安全清理"));
        assert!(zh_model.contains("模型权重"));
        assert!(zh_runtime.contains("运行时环境"));
    }
}
