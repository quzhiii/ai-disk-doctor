use std::fs;

#[test]
fn readme_english_exists_and_has_required_sections() {
    let readme = fs::read_to_string("../README.md").expect("README.md should exist");
    assert!(readme.contains("# AI Disk Doctor"), "Should have English title");
    assert!(readme.contains("## Key Features") || readme.contains("## Features"), "Should have Features section");
    assert!(readme.contains("## Architecture"), "Should have Architecture section");
    assert!(readme.contains("## Quick Start"), "Should have Quick Start section");
    assert!(readme.contains("## Command Reference"), "Should have Command Reference section");
    assert!(readme.contains("[中文](./README.zh-CN.md)"), "Should link to Chinese readme");
    assert!(readme.contains("![Version]"), "Should have version badge");
    assert!(readme.contains("policy snapshot"), "Should document policy snapshot report metadata");
    assert!(readme.contains("best-effort, not exact"), "Should document partial scan size semantics");
    assert!(
        readme.contains("rule-driven `scan`, `plan`, and `doctor`"),
        "Should scope policy metadata to rule-driven reports"
    );
    assert!(
        readme.contains("rule-driven `scan --policy`"),
        "Should scope scan --policy to rule-driven scan mode"
    );
    assert!(
        readme.contains("mark sizes as `(partial)`") && readme.contains("best-effort, not exact"),
        "Should describe partial size marker plus warning text"
    );
    assert!(
        readme.contains("Growth Anomaly Detection") && readme.contains("`anomaly`"),
        "Should document anomaly command and growth governance capability"
    );
    assert!(
        readme.contains("run-governance.ps1"),
        "Should document local governance script entrypoint"
    );
    assert!(
        readme.contains("-NotifierAdapter webhook") && readme.contains("-WebhookUrl"),
        "Should document generic webhook notifier usage"
    );
    assert!(
        readme.contains("governance-event.json")
            && readme.contains("anomaly_found")
            && readme.contains("pending_history")
            && readme.contains("no_anomaly"),
        "Should document governance event envelope and event types"
    );
    assert!(
        readme.contains("headline")
            && readme.contains("summary_markdown")
            && readme.contains("top_anomaly_path")
            && readme.contains("top_anomaly_growth_bytes"),
        "Should document message-friendly governance event summary fields"
    );
    assert!(
        readme.contains("register-governance-task.ps1") && readme.contains("-DailyAt"),
        "Should document Windows scheduler registration workflow"
    );
    assert!(
        readme.contains("show-governance-task.ps1") && readme.contains("unregister-governance-task.ps1"),
        "Should document Windows scheduler management companion scripts"
    );
}

#[test]
fn readme_chinese_exists_and_has_required_sections() {
    let readme = fs::read_to_string("../README.zh-CN.md").expect("README.zh-CN.md should exist");
    assert!(readme.contains("# AI Disk Doctor"), "Should have Chinese title");
    assert!(readme.contains("## 核心特性") || readme.contains("## 功能特性"), "Should have Features section");
    assert!(readme.contains("## 架构设计"), "Should have Architecture section");
    assert!(readme.contains("## 快速开始"), "Should have Quick Start section");
    assert!(readme.contains("## 命令参考"), "Should have Command Reference section");
    assert!(readme.contains("[English](./README.md)"), "Should link to English readme");
    assert!(readme.contains("策略快照"), "Should document policy snapshot report metadata");
    assert!(readme.contains("best-effort, not exact"), "Should document partial scan size semantics");
    assert!(
        readme.contains("规则驱动的 `scan`、`plan`、`doctor`"),
        "Should scope policy metadata to rule-driven reports"
    );
    assert!(
        readme.contains("规则驱动的 `scan --policy`"),
        "Should scope scan --policy to rule-driven scan mode"
    );
    assert!(
        readme.contains("size 标记为 `(partial)`") && readme.contains("best-effort, not exact"),
        "Should describe partial size marker plus warning text"
    );
    assert!(
        readme.contains("增长异常检测") && readme.contains("`anomaly`"),
        "Should document anomaly command and growth governance capability"
    );
    assert!(
        readme.contains("run-governance.ps1"),
        "Should document local governance script entrypoint"
    );
    assert!(
        readme.contains("-NotifierAdapter webhook") && readme.contains("-WebhookUrl"),
        "Should document generic webhook notifier usage"
    );
    assert!(
        readme.contains("governance-event.json")
            && readme.contains("anomaly_found")
            && readme.contains("pending_history")
            && readme.contains("no_anomaly"),
        "Should document governance event envelope and event types"
    );
    assert!(
        readme.contains("headline")
            && readme.contains("summary_markdown")
            && readme.contains("top_anomaly_path")
            && readme.contains("top_anomaly_growth_bytes"),
        "Should document message-friendly governance event summary fields"
    );
    assert!(
        readme.contains("register-governance-task.ps1") && readme.contains("-DailyAt"),
        "Should document Windows scheduler registration workflow"
    );
    assert!(
        readme.contains("show-governance-task.ps1") && readme.contains("unregister-governance-task.ps1"),
        "Should document Windows scheduler management companion scripts"
    );
}
