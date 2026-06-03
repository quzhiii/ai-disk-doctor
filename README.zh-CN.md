# AI Disk Doctor

![Version](https://img.shields.io/badge/version-1.0.0-blue)
![Rust](https://img.shields.io/badge/rust-1.78%2B-orange)
![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-green)
![Platform](https://img.shields.io/badge/platform-Windows-lightgrey)

[English](./README.md)

> **面向 AI 时代的 Windows 磁盘空间诊断与治理工具。**
>
> 识别、分析并安全回收被 AI 工具、浏览器和开发环境占用的存储空间——无需猜测哪些可以删除。

---

## 目录

- [功能特性](#功能特性)
- [架构设计](#架构设计)
- [技术栈](#技术栈)
- [快速开始](#快速开始)
- [命令参考](#命令参考)
- [安全第一](#安全第一)
- [截图展示](#截图展示)
- [路线图](#路线图)
- [贡献指南](#贡献指南)
- [许可证](#许可证)

---

## 功能特性

- **智能扫描** — 发现并分类 AI 模型（Ollama、Hugging Face）、浏览器、Docker、WSL、Playwright 及通用开发工具的空间占用
- **规则驱动分类** — 每个路径都通过 YAML 规则评估风险等级：`safe`（安全）、`careful`（谨慎）、`dangerous`（危险）。无硬编码路径。
- **默认仅预览** — 所有破坏性操作在执行前都会预览变更。绝不意外删除。
- **隔离模式** — 将文件移动到指定归档文件夹而非直接删除。支持完整恢复，含冲突检测。
- **专项诊断** — `doctor` 命令提供 Docker、WSL、Ollama、Playwright 和 Hugging Face 的针对性分析与可操作建议。
- **历史对比** — 对比不同时间的扫描快照，回答"什么变大了？"并追踪清理效果。
- **社区规则** — 通过 `--rules-repo` 加载自定义规则库（本地路径或 HTTPS Git 地址）。
- **Agent 友好输出** — JSON 和 Markdown 输出兼顾人工阅读与 AI Agent 解析。

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
       +-- 诊断器 (主题特定分析器)
       +-- 对比引擎 (快照对比)
       +-- 报告器 (JSON / Markdown 输出)
```

### 设计原则

1. **默认保守** — 未知路径仅报告，绝不触碰。
2. **安全优先执行** — 所有变更命令默认 dry-run。真实执行需显式 `--yes`。
3. **隔离优于删除** — 文件移动到用户指定的归档目录，非直接删除。可通过索引恢复。
4. **一切皆规则驱动** — 路径识别、风险分类和策略执行均来自外部 YAML 规则，非内置代码。
5. **Agent 就绪接口** — 结构化输出格式支持与 AI Agent 和自动化工作流集成。

---

## 技术栈

| 组件 | 技术 |
|-----------|-----------|
| CLI 框架 | Rust + `clap` v4 |
| 配置 | YAML (`serde_yaml`) |
| 文件系统 | `walkdir` + `sysinfo` |
| 输出格式 | JSON (`serde_json`) + Markdown |
| Agent 集成 | PowerShell 包装脚本 |
| 测试 | Rust 内置测试框架 |

---

## 快速开始

### 环境要求

- Windows 10/11
- Rust 1.78+ ([通过 rustup 安装](https://rustup.rs/))

### 安装

```bash
# 克隆仓库
git clone https://github.com/quzhiii/ai-disk-doctor.git
cd ai-disk-doctor/aidisk

# 构建 release 二进制文件
cargo build --release

# 从 target 目录运行
./target/release/aidisk.exe --help
```

### 首次扫描

```powershell
# 扫描所有并输出 JSON
cargo run -- scan --json

# 扫描特定分类
cargo run -- scan --category browser-cache --json

# 生成 Markdown 报告
cargo run -- scan --markdown
```

### 生成清理计划

```powershell
# 仅安全项，dry-run
cargo run -- plan --safe-only --json

# 包含谨慎项，跳过最近修改的
cargo run -- plan --json --skip-modified-within-minutes 30
```

### 执行安全清理

```powershell
# 预览隔离计划
cargo run -- clean --dry-run --safe-only --quarantine-root "F:\archives"

# 执行隔离（需要 --yes）
cargo run -- clean --yes --safe-only --quarantine-root "F:\archives"
```

### 从隔离恢复

```powershell
# 预览恢复
cargo run -- restore --dry-run --index "F:\archives\.aidisk\quarantine-index-YYYYMMDD-HHMMSS.json"

# 执行恢复
cargo run -- restore --yes --index "F:\archives\.aidisk\quarantine-index-YYYYMMDD-HHMMSS.json"
```

### 运行诊断

```powershell
# 完整系统诊断
cargo run -- doctor --markdown

# 特定主题
cargo run -- doctor --docker --json
cargo run -- doctor --wsl --ollama --markdown
cargo run -- doctor --playwright --huggingface --markdown
```

### 对比快照

```powershell
# 自动对比最近两次扫描
cargo run -- diff --latest --markdown

# 对比特定快照
cargo run -- diff --before scan-20260101-120000.json --after scan-20260102-120000.json --markdown
```

---

## 命令参考

| 命令 | 描述 | 关键参数 |
|---------|-------------|-----------|
| `scan` | 发现并分类空间占用 | `--category`, `--rules-repo`, `--json`, `--markdown` |
| `plan` | 生成清理建议 | `--safe-only`, `--skip-modified-within-minutes` |
| `clean` | 执行隔离或预览 | `--dry-run`, `--yes`, `--quarantine-root`, `--safe-only` |
| `restore` | 恢复隔离的文件 | `--dry-run`, `--yes`, `--index` |
| `doctor` | 运行针对性诊断 | `--docker`, `--wsl`, `--ollama`, `--playwright`, `--huggingface` |
| `diff` | 对比扫描快照 | `--latest`, `--before`, `--after` |

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

## 截图展示

*截图将在发布后添加。运行上方命令查看输出示例。*

---

## 路线图

### v1.0 ✅ 当前版本
- 核心 scan/plan/clean/restore/doctor/diff 命令
- 规则驱动分类
- 隔离模式与恢复
- 社区规则库支持
- PowerShell Agent 包装脚本

### v1.1（计划中）
- [ ] 实时监控（如社区有需求）
- [ ] 定时清理任务
- [ ] GUI 配套应用
- [ ] 额外平台支持（macOS、Linux）

详见 [CHANGELOG.md](./CHANGELOG.md) 了解详细版本历史。

---

## 贡献指南

欢迎贡献！请参阅我们的 [贡献指南](./CONTRIBUTING.md)（即将推出），了解：

- 报告问题
- 建议新规则
- 提交 Pull Request
- 添加新诊断主题

---

## 许可证

本项目采用双许可证：

- **MIT 许可证** — 详见 [LICENSE-MIT](./LICENSE-MIT)
- **Apache 许可证 2.0** — 详见 [LICENSE-APACHE](./LICENSE-APACHE)

您可任选其一。

---

## 致谢

使用 Rust 构建，为 AI 时代设计。特别感谢 Rust 社区的优秀 crate：`clap`、`walkdir`、`serde`、`sysinfo`。

---

**用 ❤️ 打造，让磁盘更清爽，让思路更清晰。**
