<div align="center">

# AI Disk Doctor

[![Version](https://img.shields.io/badge/version-1.6.0-blue?style=for-the-badge)](./CHANGELOG.md)
[![Rust](https://img.shields.io/badge/rust-1.78%2B-orange?style=for-the-badge)](https://rustup.rs/)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-green?style=for-the-badge)](./LICENSE-MIT)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20Linux%20%7C%20macOS-lightgrey?style=for-the-badge)]()

[English](./README.md) · [更新日志](./CHANGELOG.md) · [贡献指南](./CONTRIBUTING.md)

**AI 时代的磁盘空间诊断与治理工具。**

识别、分析并安全回收被 AI 工具、浏览器和开发环境占用的存储空间——无需猜测哪些可以删除。

</div>

---

## 目录

[项目动机](#项目动机) · [项目简介](#项目简介) · [核心特性](#核心特性) · [为什么用 aidisk 而非手动清理](#为什么用-aidisk-而非手动清理) · [最新动态](#最新动态) · [安装](#安装) · [快速开始](#快速开始) · [命令参考](#命令参考) · [安全第一](#安全第一) · [架构设计](#架构设计) · [常见问题](#常见问题) · [贡献指南](#贡献指南) · [许可证](#许可证)

---

## 项目动机

AI 工具已成为现代开发不可或缺的部分，但它们带来了一个隐性成本：**巨大的磁盘空间消耗**。

- **Ollama** 模型每个重达 4–70 GB
- **Hugging Face** 缓存悄无声息地累积在 `%USERPROFILE%\.cache`
- **Docker Desktop** 镜像和 **WSL** 发行版吞噬数十 GB
- **Playwright** 浏览器二进制文件按项目安装
- **AI IDE/CLI 缓存、安装包、测试报告和开发工具产物**数月堆积

现有的磁盘清理工具对所有文件一视同仁。它们要么删除过于激进，要么对 AI 特定的膨胀视而不见。**AI Disk Doctor** 源于一个简单的观察：*AI 时代的存储膨胀有不同的模式、不同的风险，值得一个不同的工具。*

我们相信清理应该是**透明的**（你清楚看到将要发生什么）、**可逆的**（隔离而非删除）、**规则驱动的**（没有硬编码的魔法路径）。每个路径都通过 YAML 规则进行评估，并赋予明确的风险等级——你无需猜测什么是安全的。

---

## 项目简介

AI Disk Doctor 是一款**规则驱动、安全优先**的磁盘空间诊断工具，专为 AI 时代打造。它能够发现 AI 模型缓存、浏览器数据、Docker 镜像、WSL 发行版和开发工具占用的空间，并帮助你安全地清理。

默认姿态是**保守的**：先扫描报告，再 dry-run 预览，最后隔离移动——绝不直接删除。所有破坏性操作在执行前都会预览变更，真实执行需显式 `--yes`。

**当前版本：** v1.6.0

详细的架构和设计决策，请参阅 [`docs/architecture.md`](./docs/architecture.md)。

---

## 核心特性

| 能力 | 说明 |
|-----------|-------------|
| **智能扫描** | 发现 AI 模型、IDE、CLI、浏览器、Docker、WSL、开发产物的空间占用 |
| **AI 感知规则** | 25 条 YAML 规则覆盖 200+ 路径：Claude、Codex、Gemini、Ollama、LM Studio、MCP、CUDA 等 |
| **可视化仪表盘** | `visualize --html` 生成交互式 HTML 仪表盘，支持中英双语、类别筛选、可安全回收清单 |
| **AI 足迹报告** | `doctor --ai-footprint` 聚合 10 个 AI 类别发现，给出可执行建议 |
| **跨平台** | 支持 Windows、Linux、macOS，为所有 AI 工具提供平台原生路径 |
| **规则驱动分类** | 25 条规则，风险等级：`safe`、`review`、`dangerous`。无硬编码路径 |
| **默认仅预览** | 破坏性操作执行前预览变更，真实执行需显式 `--yes` |
| **隔离模式** | 归档而非删除，支持完整恢复 |
| **定时治理** | 通过 Windows Task Scheduler / cron / launchd / systemd timer 调度，含增长异常检测 |
| **历史对比** | 对比扫描快照，追踪空间增长趋势 |
| **增长异常检测** | 基于绝对 + 相对双阈值的异常告警 |

---

## 为什么用 aidisk 而非手动清理

| 维度 | 手动清理（AI Agent / 人工） | aidisk |
|-----------|----------------------------------|--------|
| **覆盖面** | 依赖用户知识，容易遗漏隐藏缓存、模型 blob、AI IDE 状态 | 25 条规则覆盖 200+ 已知 AI 路径 |
| **风险评估** | 凭感觉，可能误删配置或凭据 | 每条路径标注 `safe` / `review` / `dangerous`，有规则依据 |
| **安全性** | 无隔离，删除即永久 | 归档 + 恢复，操作前必须 `--dry-run` |
| **可追溯性** | 临时操作，无历史记录 | 快照对比追踪增长，治理事件记录每次操作 |
| **跨平台** | Agent 行为因 OS 而异 | Windows / Linux / macOS 同一规则、同一输出 |
| **自动化治理** | 需要手动反复执行 | 通过 Task Scheduler / cron / launchd / systemd timer 自动调度，含异常检测 |
| **AI 工具识别** | 依赖已知工具，容易遗漏新工具 | 25 条规则覆盖 Claude、Codex、Gemini、Ollama、LM Studio、MCP、CUDA 等 |
| **仪表盘** | 无；只有原始 CLI 输出 | 可视化 HTML 仪表盘，支持中英双语、类别筛选、可安全回收清单 |
| **时间成本** | 每次清理需 30-60 分钟 | 5 秒扫描，完整报告秒出 |

---

## 最新动态

### v1.6.0

- **可视化仪表盘** — `aidisk visualize --html`：交互式中英双语 HTML 仪表盘，支持类别筛选和可安全回收清单
- **AI 足迹** — `doctor --ai-footprint`：聚合 10 个 AI 类别发现
- **5 条新 AI 规则** — GPU 推理运行器、AI 编程助手、MCP 服务器、新一代 IDE、CUDA/cuDNN 运行时
- **模型文件检测** — GGUF/SafeTensors/ONNX/MLX glob 匹配，标记为安全
- **跨平台规则** — 6 条规则升级为 Windows/Linux/macOS 多平台格式

完整说明：[CHANGELOG.md](./CHANGELOG.md) · [Release Notes v1.6.0](./docs/release-notes/v1.6.0.md)。

### v1.5.0

- **Feishu 通知** — 通过 `FEISHU_WEBHOOK_URL` 环境变量安全投递
- **治理可靠性** — 事件去重 + 可配置重试
- **跨平台 CI** — Windows / Ubuntu / macOS
- **用户手册** — [`docs/governance-manual.md`](./docs/governance-manual.md)

### v1.4.0

- **跨平台定时治理** — cron、launchd、systemd timer + `run-governance.sh`

### v1.3.0

- **本地定时治理** — 异常检测 + 治理事件 + Windows Task Scheduler

### v1.2.0

- **覆盖面扩展** — 大文件发现、跨平台规则、JSON 错误

### v1.1.0

- **Doctor V2** — AI 工具诊断、子目录分解、可选探测

### v1.0.0

- **完整工作流** — scan、plan、clean、restore、doctor、diff

---

## 安装

### 方式 1：预编译二进制文件（推荐 — 无需 Rust）

从 [Releases 页面](https://github.com/quzhiii/ai-disk-doctor/releases) 下载最新版本的 `aidisk.exe`，解压并放到 PATH 中即可使用。

### 方式 2：从源码构建（需要 Rust）

**环境要求：**

| 要求 | 版本 |
|------------|---------|
| Windows | 10/11 |
| Rust | 1.78+ |

如果没有 Rust，通过 [rustup](https://rustup.rs/) 安装。

```bash
git clone https://github.com/quzhiii/ai-disk-doctor.git
cd ai-disk-doctor/aidisk
cargo build --release
# 二进制文件位于 target/release/aidisk.exe
```

### 方式 3：PowerShell Skill 包装脚本（Agent 集成）

无需 Rust 或编译。`skills/windows-ai-space-manager/scripts/` 目录包含独立的 PowerShell 包装脚本，调用 CLI 即可工作。只要预编译二进制文件在 PATH 中，这些脚本立即可用：

```powershell
# 通过 PowerShell 包装脚本扫描
.\skills\windows-ai-space-manager\scripts\scan.ps1

# 运行诊断
.\skills\windows-ai-space-manager\scripts\doctor.ps1
```

### 开发环境

```bash
cd ai-disk-doctor/aidisk

# 构建并测试
cargo build
cargo test
```

验证构建：

```powershell
pwsh -NoProfile -File "scripts/release-smoke.ps1"
```

---

## 快速开始

### 1. 扫描系统

```powershell
# 扫描所有并输出 JSON
aidisk scan --json

# 生成 Markdown 报告
aidisk scan --markdown

# 扫描特定分类
aidisk scan --category browser-cache --json
```

### 2. 生成清理计划

```powershell
# 仅安全项，dry-run
aidisk plan --safe-only --json

# 包含谨慎项，跳过最近修改的
aidisk plan --json --skip-modified-within-minutes 30
```

### 3. 执行安全清理（隔离）

```powershell
# 预览隔离计划
aidisk clean --dry-run --safe-only --quarantine-root "F:\archives"

# 执行隔离（需要 --yes）
aidisk clean --yes --safe-only --quarantine-root "F:\archives"
```

### 4. 如需恢复

```powershell
# 预览恢复
aidisk restore --dry-run --index "F:\archives\.aidisk\quarantine-index-YYYYMMDD-HHMMSS.json"

# 执行恢复
aidisk restore --yes --index "F:\archives\.aidisk\quarantine-index-YYYYMMDD-HHMMSS.json"
```

### 5. 运行诊断

```powershell
# 完整系统诊断
aidisk doctor --markdown

# 特定主题
aidisk doctor --docker --json
aidisk doctor --wsl --ollama --markdown
aidisk doctor --playwright --huggingface --markdown
aidisk doctor --agents --markdown
aidisk doctor --docker --probe-tools --markdown

# 结合最近扫描快照增长信息
aidisk doctor --agents --latest --markdown
aidisk doctor --latest --reports-dir .aidisk/reports --json
```

`doctor --agents` 覆盖 Claude、Codex、Gemini、opencode、Cursor、Windsurf、Trae、aider、Continue、已安装应用、AI runtime caches、安装包和测试产物。
内置 doctor topics 由代码内 registry 统一组织，因此默认运行和显式 topic flags 使用同一份元数据，同时不改变公开 CLI。
Markdown/Text doctor 输出聚焦 active paths，并将缺失匹配汇总为 `Not detected`；JSON 保留完整 finding list 供自动化使用。
使用 `--probe-tools` 可显式启用外部命令探测，例如 `docker system df`、`wsl --list --verbose` 和 `ollama list`。
使用 `--latest` 可追加 `.aidisk/reports` 中最近两次扫描快照的增长摘要；需要自定义快照目录时使用 `--reports-dir`。

### 6. 对比快照

```powershell
# 自动对比最近两次扫描
aidisk diff --latest --markdown

# 对比特定快照
aidisk diff --before scan-20260101-120000.json --after scan-20260102-120000.json --markdown
```

### 7. 运行本地治理

```powershell
# 执行一次本地治理周期
.\scripts\governance\run-governance.ps1

# 将产物写到自定义目录
.\scripts\governance\run-governance.ps1 -OutputDir ".aidisk\governance"

# 调整增长异常阈值
.\scripts\governance\run-governance.ps1 -MinGrowth "2GB" -MinGrowthPercent 50

# 将 anomaly JSON 投递到通用 webhook 端点
.\scripts\governance\run-governance.ps1 -NotifierAdapter webhook -WebhookUrl https://example.test/webhook

# 为较慢的端点调整 webhook 超时
.\scripts\governance\run-governance.ps1 -NotifierAdapter webhook -WebhookUrl https://example.test/webhook -WebhookTimeoutSeconds 30
```

治理脚本保持全程只读：它会运行 `scan`、复用扫描快照，并将 anomaly 产物写到本地。若首轮历史快照不足两份，它会写出 pending 提示而不是直接失败。脚本还会统一写出稳定的 `governance-event.json` 事件封装，事件类型包括 `anomaly_found`、`pending_history`、`no_anomaly`。该事件还包含更适合消息模板直接消费的摘要字段，例如 `headline`、`summary_markdown`、`top_anomaly_path`、`top_anomaly_growth_bytes`。`-NotifierAdapter webhook` 会将该治理事件 payload 投递到通用 HTTP 端点，后续微信 / 企业微信 / 飞书 / Slack / Telegram / Discord 等适配器都可以复用这一契约。若 webhook 投递失败，本地产物仍会保留，脚本还会额外写出 `webhook-failure.json`，记录 `delivery_status`、timeout 和错误上下文，便于后续排查。

### 8. 注册 Windows 每日任务

```powershell
# 每天 09:00 注册一次治理任务
.\scripts\governance\register-governance-task.ps1 -DailyAt "09:00"

# 注册带 webhook 的每日治理任务
.\scripts\governance\register-governance-task.ps1 -DailyAt "09:00" -NotifierAdapter webhook -WebhookUrl https://example.test/webhook

# 查看已注册的治理任务
.\scripts\governance\show-governance-task.ps1

# 立即触发已注册的治理任务进行测试运行
.\scripts\governance\test-run-governance-task.ps1

# 卸载治理任务
.\scripts\governance\unregister-governance-task.ps1
```

调度注册脚本只会向 Windows Task Scheduler 注册一个调用 `run-governance.ps1` 的任务；它不会执行清理，也不会删除任何文件。需要立即验证已注册的本地治理链路时，可使用 `test-run-governance-task.ps1` 对现有任务调用 `Start-ScheduledTask`。

### 9. 注册跨平台治理调度器

```bash
# 直接执行一次 Unix-like 治理周期
./scripts/governance/run-governance.sh --notifier-adapter local-file

# 将 Unix 治理事件投递到通用 webhook
./scripts/governance/run-governance.sh --notifier-adapter webhook --webhook-url https://example.test/webhook

# cron：注册、查看、测试运行、卸载
./scripts/governance/cron/register-governance-cron.sh --schedule "0 9 * * *"
./scripts/governance/cron/show-governance-cron.sh
./scripts/governance/cron/test-run-governance-cron.sh
./scripts/governance/cron/unregister-governance-cron.sh

# launchd：注册、查看、测试运行、卸载
./scripts/governance/launchd/register-governance-launchd.sh --schedule-hour 9 --schedule-minute 0
./scripts/governance/launchd/show-governance-launchd.sh
./scripts/governance/launchd/test-run-governance-launchd.sh
./scripts/governance/launchd/unregister-governance-launchd.sh

# systemd timer：注册、查看、测试运行、卸载
./scripts/governance/systemd/register-governance-systemd.sh --schedule "*-*-* 09:00:00"
./scripts/governance/systemd/show-governance-systemd.sh
./scripts/governance/systemd/test-run-governance-systemd.sh
./scripts/governance/systemd/unregister-governance-systemd.sh
```

Unix-like 治理入口依赖 `bash`、`jq`、用于 webhook 或 Feishu 投递的 `curl`，以及用于本地运行的 `cargo`。cron、launchd 和 systemd timer adapter 只会注册调用 `run-governance.sh` 的平台原生调度任务；它们不会引入后台 daemon，也不会执行清理。在当前分支上，Phase 13 新增了具体的 Feishu notifier adapter，而更广的 adapter 扩展仍留到后续阶段。

### 10. 将治理事件发送到 Feishu

```bash
# 通过环境变量注入 Feishu webhook，不要写在命令行参数里
export FEISHU_WEBHOOK_URL="https://example.test/feishu-webhook"

# 用 notifier dispatcher 发送已有 governance-event.json
./scripts/governance/send-governance-event.sh --adapter feishu --event-path .aidisk/governance/governance-event.json --output-dir .aidisk/governance

# 或在一次治理运行中直接通过 Feishu 投递
./scripts/governance/run-governance.sh --notifier-adapter feishu
```

更多 Notifier Adapter Foundation、Feishu adapter、`FEISHU_WEBHOOK_URL` secrets 处理、generic webhook 兼容性和 `feishu-failure.json` 行为，请查看 [`docs/notifier-adapters.md`](./docs/notifier-adapters.md)。

完整的跨平台治理文档（覆盖四个平台、去重、重试、notifier adapter 及故障排查），请查看 [`docs/governance-manual.md`](./docs/governance-manual.md)。

---

## 命令参考

| 命令 | 描述 | 关键参数 |
|---------|-------------|-----------|
| `scan` | 发现并分类空间占用 | `--category`, `--rules-repo`, `--json`, `--markdown` |
| `scan --large-files` | 发现最大文件和目录 | `--min-size`, `--root`, `--json`, `--markdown` |
| `plan` | 生成清理建议 | `--safe-only`, `--skip-modified-within-minutes` |
| `clean` | 执行隔离或预览 | `--dry-run`, `--yes`, `--quarantine-root`, `--safe-only` |
| `restore` | 恢复隔离的文件 | `--dry-run`, `--yes`, `--index` |
| `doctor` | 运行针对性诊断 | `--agents`, `--docker`, `--wsl`, `--ollama`, `--playwright`, `--huggingface`, `--probe-tools`, `--latest`, `--reports-dir` |
| `diff` | 对比扫描快照 | `--latest`, `--before`, `--after` |
| `anomaly` | 从扫描快照中检测增长异常 | `--latest`, `--before`, `--after`, `--min-growth`, `--min-growth-percent` |

### JSON 错误契约

当选择 `--json` 或 `--format json` 且命令失败时，`aidisk` 会向 stderr 写入一个 JSON 错误对象，并保持 stdout 为空。成功的 JSON 报告仍写入 stdout。

```json
{
  "ok": false,
  "error": {
    "type": "usage",
    "message": "restore execution requires --yes or use --dry-run",
    "command": "restore",
    "details": []
  }
}
```

---

## 安全第一

### 默认行为

- **scan** 始终只读
- **plan** 始终只读
- **clean** 默认 `--dry-run`；真实变更需 `--yes`
- **restore** 默认 `--dry-run`；真实变更需 `--yes`

### 隔离安全

- 跨盘移动使用 copy+delete 回退（Windows `rename` 跨盘失败）
- 恢复前验证索引结构
- 冲突（目标路径已存在）会跳过并报告，绝不覆盖

### 风险等级

| 等级 | 含义 | 默认行为 |
|-------|---------|----------------|
| `safe` | 已知的缓存/临时目录 | `--safe-only` 包含 |
| `careful` | 可能仍需要的用户数据 | 需显式包含 |
| `dangerous` | 系统关键或不可逆 | 被敏感路径过滤器阻断 |

---

## 架构设计

```text
用户 / AI Agent
       |
       v
  aidisk CLI
       |
       +-- 配置加载器 (policy.yaml)
       +-- 规则引擎 (YAML 规则 + glob 展开)
       +-- 扫描器 (WalkDir 带深度限制)
       +-- 规划器 (风险过滤 + 敏感路径阻断)
       +-- 清理器 (隔离/恢复，支持跨盘回退)
        +-- 诊断器 (registry 驱动的主题分析器)
        +-- 对比引擎 (快照对比)
        +-- 异常引擎 (增长阈值检测)
        +-- 报告器 (JSON / Markdown 输出)
```

详细架构文档请参阅 [`docs/architecture.md`](./docs/architecture.md)。

### 设计原则

1. **默认保守** — 未知路径仅报告，绝不触碰
2. **安全优先执行** — 所有变更命令默认 dry-run
3. **隔离优于删除** — 文件移动而非删除；可通过索引恢复
4. **一切皆规则驱动** — 路径识别、风险分类和策略执行均来自外部 YAML 规则
5. **Agent 就绪接口** — 结构化输出格式支持与 AI Agent 集成

---

## 常见问题

### 哪里可以获取预编译二进制文件？

查看 [Releases](https://github.com/quzhiii/ai-disk-doctor/releases) 页面。如果暂时没有二进制文件，从源码构建只需安装 Rust（通过 [rustup](https://rustup.rs/) 几分钟即可完成）。

### 需要使用 Rust 吗？

**不需要** — 从 Releases 下载预编译的 `aidisk.exe` 即可。只有从源码构建或贡献代码时才需要 Rust。

### 可以只用 PowerShell 不用 Rust CLI 吗？

`skills/` 中的 PowerShell 包装脚本底层调用 `aidisk` CLI。你需要二进制文件，但不需要 Rust 工具链。

### 有 Python 版本吗？

目前尚无。核心引擎使用 Rust 以确保性能和安全性。如果社区有需求，未来可能提供 Python 绑定或原生 Python 移植。欢迎贡献！

### `cargo build` 在 Windows 上失败

确保使用最新的稳定版 Rust 工具链：

```bash
rustup update stable
```

如果看到链接器错误，请安装 Visual Studio Build Tools（包含 C++ 工作负载）。

### 扫描未找到任何路径

检查环境变量（如 `%USERPROFILE%`）是否正确设置。AI Disk Doctor 会在规则模式中展开这些变量。

### 隔离失败，提示"拒绝访问"

某些目录被运行中的进程锁定。请在隔离操作前关闭浏览器、Docker Desktop 或 WSL。

### 跨盘隔离速度很慢

当跨驱动器隔离时（如 C: 到 F:），Windows 需要使用 copy+delete 而非快速的 rename。这是为了安全而设计的预期行为。

---

## 贡献指南

欢迎各种形式的贡献——Bug 报告、新规则、文档改进和核心功能开发。请参阅 [`CONTRIBUTING.md`](./CONTRIBUTING.md) 了解：

- 开发环境搭建
- 代码规范
- 添加新规则
- Pull Request 流程

[行为准则](./CODE_OF_CONDUCT.md) · [安全政策](./SECURITY.md) · [许可证](#许可证)

---

## 致谢

- 使用 Rust 构建。特别感谢 Rust 社区的 `clap`、`walkdir`、`serde`、`sysinfo` 等优秀 crate。

## 许可证

本项目采用双许可证：

- **MIT 许可证** — 详见 [LICENSE-MIT](./LICENSE-MIT)
- **Apache 许可证 2.0** — 详见 [LICENSE-APACHE](./LICENSE-APACHE)

您可任选其一。

---

<div align="center">

**用 ❤️ 打造，让磁盘更清爽，让思路更清晰。**

</div>
