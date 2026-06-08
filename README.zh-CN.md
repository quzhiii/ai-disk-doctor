<div align="center">

# AI Disk Doctor

[![Version](https://img.shields.io/badge/version-1.2.0-blue?style=for-the-badge)](./CHANGELOG.md)
[![Rust](https://img.shields.io/badge/rust-1.78%2B-orange?style=for-the-badge)](https://rustup.rs/)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-green?style=for-the-badge)](./LICENSE-MIT)
[![Platform](https://img.shields.io/badge/platform-Windows-lightgrey?style=for-the-badge)]()

[English](./README.md) · [更新日志](./CHANGELOG.md) · [贡献指南](./CONTRIBUTING.md)

**面向 AI 时代的 Windows 磁盘空间诊断与治理工具。**

识别、分析并安全回收被 AI 工具、浏览器和开发环境占用的存储空间——无需猜测哪些可以删除。

</div>

---

## 目录

[项目动机](#项目动机) · [项目简介](#项目简介) · [最新动态](#最新动态) · [核心特性](#核心特性) · [安装](#安装) · [快速开始](#快速开始) · [命令参考](#命令参考) · [安全第一](#安全第一) · [架构设计](#架构设计) · [常见问题](#常见问题) · [贡献指南](#贡献指南) · [许可证](#许可证)

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

**当前版本：** v1.2.0

详细的架构和设计决策，请参阅 [`docs/architecture.md`](./docs/architecture.md)。

---

## 最新动态

### v1.2.0

Phase 7 在保持保守清理边界的前提下扩展了发现能力和覆盖面：

- **大文件发现** — `scan --large-files --min-size 500MB` 可发现指定根目录下最大的文件和目录，不进行分类或清理建议。
- **开发产物覆盖** — 内置规则可识别 `node_modules`、Rust `target/`、Gradle 缓存、Python `__pycache__`、`dist/`、`.next`、`.turbo` 等常见可再生成产物。
- **跨平台规则路径** — 规则现在同时支持 Unix `~/` home 路径和 Windows `%VAR%` 展开，并为 Ollama、Hugging Face、Docker 增加了 linux/macOS 路径。
- **结构化 JSON 错误** — `--json` 命令失败时输出单一错误对象到 stderr，保持 stdout 为空。
- **可运维元数据** — 规则驱动的 `scan`、`plan`、`doctor` 会展示当前 `policy snapshot`；当遍历不完整时，text/markdown 输出会先将 size 标记为 `(partial)`，再用 warning 解释为 `best-effort, not exact`；并支持通过规则驱动的 `scan --policy` 显式指定策略文件。

完整说明：[CHANGELOG.md](./CHANGELOG.md) · [Release Notes v1.2.0](./docs/release-notes/v1.2.0.md)。

### v1.1.0

Doctor V2 增强了 AI 时代的诊断能力，同时保持默认只读和保守安全边界：

- **AI 工具链诊断** — `doctor --agents` 覆盖 AI Agent 根目录、AI IDE/CLI 状态、runtime caches、安装包、已安装应用根目录和测试产物
- **子目录分解** — active doctor findings 会展示最大的直接子项，便于判断大目录内部组成
- **显式探测** — `--probe-tools` 可按需添加 Docker、WSL、Ollama 外部命令探测，默认不运行外部命令
- **增长感知 doctor** — `doctor --latest` 追加最近扫描快照增长信息，`--reports-dir` 可指定历史目录
- **Registry 驱动 topics** — 内置 doctor topics 使用代码内 `DoctorTopicSpec` registry 统一默认启用、匹配逻辑、建议和 probe metadata

完整说明：[CHANGELOG.md](./CHANGELOG.md) · [Release Notes v1.1.0](./docs/release-notes/v1.1.0.md)。

### v1.0.0

首个稳定版本，带来完整的本地工作流：

- **完整的命令集** — `scan`、`plan`、`clean`、`restore`、`doctor` 和 `diff --latest`
- **社区规则** — 通过 `--rules-repo` 加载自定义规则库（本地路径或 HTTPS git 地址）
- **隔离模式** — 将文件移动到归档文件夹，支持完整恢复
- **历史对比** — 对比扫描快照，追踪空间增长趋势
- **Agent 友好输出** — JSON 和 Markdown 输出，兼顾人工阅读和 AI Agent 解析
- **PowerShell 包装脚本** — `skills/` 目录下提供即用的 Agent Skill 脚本

完整说明：[CHANGELOG.md](./CHANGELOG.md) · [Release Notes v1.0.0](./docs/release-notes/v1.0.0.md)。

---

## 核心特性

| 能力 | 说明 |
|-----------|-------------|
| **智能扫描** | 发现 AI 模型（Ollama、Hugging Face）、AI IDE/CLI、浏览器、Docker、WSL、Playwright、安装包和测试产物的空间占用 |
| **开发产物覆盖** | 识别 `node_modules`、Rust `target/`、Gradle 缓存、Python `__pycache__`、`dist/`、`.next`、`.turbo` 等常见可再生成产物 |
| **跨平台规则路径** | 同时支持 Unix `~/` home 路径与 Windows `%VAR%` 展开，使 Ollama、Hugging Face、Docker 规则可适配 Windows、Linux、macOS 的常见路径布局 |
| **规则驱动分类** | 每个路径通过 YAML 规则评估风险等级：`safe`、`careful`、`dangerous`。无硬编码路径。 |
| **默认仅预览** | 所有破坏性操作执行前预览变更。真实执行需显式 `--yes`。 |
| **隔离模式** | 将文件移动到指定归档文件夹而非直接删除。支持完整恢复，含冲突检测。 |
| **专项诊断** | `doctor` 命令提供 AI Agents、AI IDE/CLI、安装包、测试产物、Docker、WSL、Ollama、Playwright 和 Hugging Face 的针对性分析 |
| **Registry 驱动的 Doctor Topics** | 内置 doctor topics 保持既有公开 flags，同时用一份代码内 registry 统一 topic 名称、默认启用、匹配逻辑、建议和可选 probes |
| **历史对比** | 对比扫描快照，回答"什么变大了？"并追踪清理效果 |
| **增长异常检测** | 基于绝对与相对双阈值识别异常增长路径，用于本地定时治理 |
| **社区规则** | 通过 `--rules-repo` 加载自定义规则库（本地目录或 HTTPS git 地址） |
| **Agent 友好输出** | JSON 和 Markdown 输出，兼顾人工阅读和 AI Agent 解析 |
| **可运维元数据** | 报告包含当前策略快照，并在深度限制或后代路径不可读导致 size 不完整时，将 partial size 标记为 `best-effort, not exact` |
| **跨盘安全** | 隔离操作在跨盘时自动使用 copy+delete 回退（rename 跨盘会失败） |

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
```

治理脚本保持全程只读：它会运行 `scan`、复用扫描快照，并将 anomaly 产物写到本地。若首轮历史快照不足两份，它会写出 pending 提示而不是直接失败。脚本还会统一写出稳定的 `governance-event.json` 事件封装，事件类型包括 `anomaly_found`、`pending_history`、`no_anomaly`。`-NotifierAdapter webhook` 会将该治理事件 payload 投递到通用 HTTP 端点，后续微信 / 企业微信 / 飞书 / Slack / Telegram / Discord 等适配器都可以复用这一契约。

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
