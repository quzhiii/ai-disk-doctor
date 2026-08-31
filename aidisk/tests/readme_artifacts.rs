use std::fs;

#[test]
fn readme_english_exists_and_has_required_sections() {
    let readme = fs::read_to_string("../README.md").expect("README.md should exist");
    assert!(
        readme.contains("# AI Disk Doctor"),
        "Should have English title"
    );
    assert!(
        readme.contains("## Key Features") || readme.contains("## Features"),
        "Should have Features section"
    );
    assert!(
        readme.contains("## Architecture") || readme.contains("## Safety First"),
        "Should have Architecture or Safety section"
    );
    assert!(
        readme.contains("## Quick Start") || readme.contains("## Command Reference"),
        "Should have Quick Start or Command Reference section"
    );
    assert!(
        readme.contains("[中文](./README.zh-CN.md)"),
        "Should link to Chinese readme"
    );
    assert!(readme.contains("![Version]"), "Should have version badge");
    assert!(readme.contains("version-1.8.0"), "Should show v1.8.0 badge");
    assert!(
        readme.contains("### v1.8.0") && readme.contains("docs/release-notes/v1.8.0.md"),
        "Should document v1.8.0 release notes"
    );
    assert!(
        readme.contains("aidisk capabilities --json")
            && readme.contains("aidisk explain --json --snapshot skip")
            && readme.contains("agent-diagnostic-cli-v1"),
        "Should document v1.8.0 agent diagnostic contract"
    );
    assert!(
        readme.contains("### v1.7.0") && readme.contains("docs/release-notes/v1.7.0.md"),
        "Should document v1.7.0 release notes"
    );
    assert!(
        readme.contains("## One-Click Deploy") && readme.contains("Start-AIDiskDoctor.ps1"),
        "Should document one-click read-only onboarding"
    );
    assert!(
        readme.contains(".aidisk\\quickstart\\")
            && readme.contains("aidisk-dashboard.html")
            && readme.contains("Real cleanup still requires"),
        "Should describe one-click outputs and safety boundary"
    );
    assert!(
        readme.contains("### Prompt for Your Local Agent")
            && readme.contains("Use AI Disk Doctor to help me reclaim disk space safely")
            && readme.contains("Ask for my explicit confirmation")
            && readme.contains("aidisk clean --yes --safe-only --quarantine-root <path>"),
        "Should provide a copyable local-agent cleanup prompt"
    );
    assert!(
        readme.contains("visualize") && readme.contains("ai-footprint"),
        "Should document v1.6.0 visualize and ai-footprint"
    );
    assert!(
        readme.contains("one-click deploy")
            && readme.contains("Packaged resources")
            && readme.contains("Portable local data"),
        "Should document v1.7.0 one-click deployment updates"
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
        readme.contains("-WebhookTimeoutSeconds") && readme.contains("webhook-failure.json"),
        "Should document webhook timeout tuning and failure artifact"
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
        readme.contains("show-governance-task.ps1")
            && readme.contains("unregister-governance-task.ps1"),
        "Should document Windows scheduler management companion scripts"
    );
    assert!(
        readme.contains("test-run-governance-task.ps1") && readme.contains("Start-ScheduledTask"),
        "Should document immediate Windows scheduler test run script"
    );
}

#[test]
fn readme_chinese_exists_and_has_required_sections() {
    let readme = fs::read_to_string("../README.zh-CN.md").expect("README.zh-CN.md should exist");
    assert!(
        readme.contains("# AI Disk Doctor"),
        "Should have Chinese title"
    );
    assert!(
        readme.contains("## 核心特性") || readme.contains("## 功能特性"),
        "Should have Features section"
    );
    assert!(
        readme.contains("## 架构设计"),
        "Should have Architecture section"
    );
    assert!(
        readme.contains("## 快速开始"),
        "Should have Quick Start section"
    );
    assert!(
        readme.contains("## 命令参考"),
        "Should have Command Reference section"
    );
    assert!(
        readme.contains("[English](./README.md)"),
        "Should link to English readme"
    );
    assert!(readme.contains("version-1.8.0"), "Should show v1.8.0 badge");
    assert!(
        readme.contains("**当前版本：** v1.8.0"),
        "Should show v1.8.0 current release"
    );
    assert!(
        readme.contains("### v1.8.0") && readme.contains("docs/release-notes/v1.8.0.md"),
        "Should document v1.8.0 release notes"
    );
    assert!(
        readme.contains("aidisk capabilities --json")
            && readme.contains("aidisk explain --json --snapshot skip")
            && readme.contains("agent-diagnostic-cli-v1"),
        "Should document v1.8.0 agent diagnostic contract"
    );
    assert!(
        readme.contains("### v1.7.0") && readme.contains("docs/release-notes/v1.7.0.md"),
        "Should document v1.7.0 release notes"
    );
    assert!(
        readme.contains("## 一键部署") && readme.contains("Start-AIDiskDoctor.ps1"),
        "Should document Chinese one-click read-only onboarding"
    );
    assert!(
        readme.contains(".aidisk\\quickstart\\")
            && readme.contains("aidisk-dashboard.html")
            && readme.contains("真实清理仍必须"),
        "Should describe Chinese one-click outputs and safety boundary"
    );
    assert!(
        readme.contains("### 给本地 Agent 的提示词")
            && readme.contains("请使用 AI Disk Doctor 帮我安全释放磁盘空间")
            && readme.contains("必须先获得我的明确确认")
            && readme.contains("aidisk clean --yes --safe-only --quarantine-root <path>"),
        "Should provide a Chinese copyable local-agent cleanup prompt"
    );
    assert!(
        readme.contains("可视化仪表盘") && readme.contains("AI 足迹"),
        "Should document v1.6.0 visualize and ai-footprint"
    );
    assert!(
        readme.contains("一键部署")
            && readme.contains("发布包资源完整")
            && readme.contains("本地数据路径可移植"),
        "Should document v1.7.0 one-click deployment updates"
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
        readme.contains("-WebhookTimeoutSeconds") && readme.contains("webhook-failure.json"),
        "Should document webhook timeout tuning and failure artifact"
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
        readme.contains("show-governance-task.ps1")
            && readme.contains("unregister-governance-task.ps1"),
        "Should document Windows scheduler management companion scripts"
    );
    assert!(
        readme.contains("test-run-governance-task.ps1") && readme.contains("Start-ScheduledTask"),
        "Should document immediate Windows scheduler test run script"
    );
}
