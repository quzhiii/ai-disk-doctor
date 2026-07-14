# AI Disk Doctor Strategy And Roadmap

日期：2026-07-11

> Superseded by `docs/AI_DISK_DOCTOR_CORRECTED_ROADMAP_2026-07.md` as the implementation roadmap. This document remains a strategy/market-analysis companion.

本文档整理当前仓库状态、产品判断、竞品对照、商业化机会和下一步迭代计划，供其他 agent 或评审者交叉验证。

## 1. 当前项目概况

### 1.1 一句话定位

AI Disk Doctor 是一个 Rust 实现的、本地优先、安全优先、规则驱动的 AI 时代磁盘空间诊断与治理 CLI。

它面向 Claude、Codex、Gemini、opencode、Cursor、Windsurf、Trae、aider、Continue、Ollama、Hugging Face、LM Studio、MCP servers、Docker、WSL、Playwright、浏览器缓存和开发构建产物等场景，帮助用户识别、解释、预演、安全隔离清理和恢复磁盘占用。

### 1.2 当前版本与成熟度

- 当前 README 标注版本为 `v1.6.0`。
- 核心 crate 位于 `aidisk/`，版本定义在 `aidisk/Cargo.toml`。
- 当前已经具备较完整的开源 CLI 产品闭环：`scan`、`plan`、`clean`、`restore`、`doctor`、`diff`、`anomaly`、`visualize`。
- 当前成熟度接近“可发布的开源开发者工具”，但还不是完整 SaaS 或企业商业化产品。
- 项目已有双语 README、CHANGELOG、release notes、治理手册、规则规范、架构文档、跨平台 CI、release artifact workflow 和 Skill wrapper。

### 1.3 当前核心能力

- **规则驱动扫描**：通过 YAML 规则识别 AI 工具、模型缓存、MCP、AI IDE、AI CLI、Docker、WSL、浏览器和开发产物路径。
- **风险分级**：规则带有 `safe`、`review`、`dangerous`、`system` 等风险语义，避免把所有目录都当作普通可删除缓存。
- **清理预案**：`plan` 基于扫描结果、风险等级、最近修改时间、敏感路径和动作策略生成 dry-run 候选。
- **安全清理**：`clean` 默认 dry-run，真实执行需要显式 `--yes`，并通过 quarantine 隔离而不是直接删除。
- **可恢复性**：`restore` 基于 quarantine index 预演或执行恢复，并处理冲突，不默认覆盖目标路径。
- **专项诊断**：`doctor` 支持 Docker、WSL、Ollama、Playwright、Hugging Face、Agents、AI Footprint 等专题。
- **历史对比**：`diff` 和自动落盘扫描快照支持回答“最近谁变大了”。
- **异常治理**：`anomaly` 基于最近两次快照做增长异常检测，并可被治理脚本调度。
- **可视化**：`visualize --html` 生成本地交互式 HTML dashboard，支持双语、分类过滤、KPI、风险卡和安全回收清单。
- **Agent 集成**：`skills/windows-ai-space-manager/` 提供 SkillHub/agent 包装层和 PowerShell wrappers。

## 2. 仓库结构速览

### 2.1 核心实现

- `aidisk/src/main.rs`：CLI 入口，定义 scan / plan / clean / restore / diff / anomaly / doctor / visualize 等子命令。
- `aidisk/src/rules.rs`：规则加载、平台路径解析、环境变量展开和 `~` 展开。
- `aidisk/src/scanner.rs`：规则扫描、路径统计、finding 和 summary 输出。
- `aidisk/src/planner.rs`：清理计划生成，处理 risk、sensitive marker、recently modified 和 action policy。
- `aidisk/src/cleaner.rs`：quarantine 执行、恢复索引、执行日志和 restore。
- `aidisk/src/doctor.rs`：专题诊断、agent/AI footprint topic、子目录 breakdown、probe metadata。
- `aidisk/src/diff.rs`：扫描快照对比。
- `aidisk/src/anomaly.rs`：增长异常检测。
- `aidisk/src/history.rs`：历史报告和快照读取。
- `aidisk/src/visualize.rs`：HTML dashboard 生成。
- `aidisk/src/reporter.rs`：JSON / Markdown / Text 输出。
- `aidisk/src/rules_repo.rs`：远程或本地规则库加载。

### 2.2 规则库

- `aidisk/rules/ai-agents.yaml`
- `aidisk/rules/ai-coding-agents.yaml`
- `aidisk/rules/ai-ides.yaml`
- `aidisk/rules/ai-ides-next.yaml`
- `aidisk/rules/ai-clis.yaml`
- `aidisk/rules/ai-caches.yaml`
- `aidisk/rules/ai-installed-apps.yaml`
- `aidisk/rules/ai-installers.yaml`
- `aidisk/rules/ai-runtimes.yaml`
- `aidisk/rules/mcp-servers.yaml`
- 以及 Docker、WSL、浏览器、开发缓存、大文件和模型相关规则。

### 2.3 Skill / Agent 集成

- `skills/windows-ai-space-manager/SKILL.md`：面向 agent 的使用说明、触发词、工作流和安全原则。
- `skills/windows-ai-space-manager/scripts/run-scan.ps1`
- `skills/windows-ai-space-manager/scripts/run-plan.ps1`
- `skills/windows-ai-space-manager/scripts/run-clean-dry-run.ps1`
- `skills/windows-ai-space-manager/scripts/run-clean.ps1`
- `skills/windows-ai-space-manager/scripts/run-restore.ps1`
- `skills/windows-ai-space-manager/scripts/run-doctor.ps1`
- `skills/windows-ai-space-manager/scripts/run-diff.ps1`
- `skills/windows-ai-space-manager/references/workflow.md`
- `skills/windows-ai-space-manager/references/risk-cheatsheet.md`
- `skills/windows-ai-space-manager/references/category-map.md`

### 2.4 产品与发布文档

- `README.md`
- `README.zh-CN.md`
- `CHANGELOG.md`
- `docs/architecture.md`
- `docs/execution-plan.md`
- `docs/rules-spec.md`
- `docs/risk-model.md`
- `docs/windows-ai-storage-map.md`
- `docs/governance-manual.md`
- `docs/notifier-adapters.md`
- `docs/release-notes/v1.0.0.md` 到 `docs/release-notes/v1.6.0.md`

## 3. 当前产品判断

### 3.1 已经做得好的地方

- **方向足够差异化**：不是通用磁盘清理，而是聚焦 AI 工具和开发者环境产生的新型磁盘膨胀。
- **安全心智清晰**：scan first、dry-run first、quarantine first、restore available，明显强于简单 `rm -rf` 型工具。
- **规则体系有复利**：YAML 规则让 AI 工具覆盖可以持续扩展，也便于社区和企业自定义。
- **Agent-readable**：稳定 JSON / Markdown 输出和 Skill wrapper 让它天然适合被 Claude Code、Codex、Gemini CLI、Cursor、opencode 等 agent 调用。
- **治理能力已成型**：scheduled governance、anomaly、notifier adapter、Feishu webhook、event dedup 和 retry 让它具备从“工具”升级为“本地哨兵”的基础。
- **可视化补齐了产品感**：本地 HTML dashboard 让非 CLI 用户也能理解结果。

### 3.2 当前短板

- **分发链路还不够商业化**：README 有 release binary，但需要进一步补齐 Homebrew、winget、Linux/macOS artifacts、checksums、SBOM、签名/公证和自动更新说明。
- **GUI / Tray 仍缺失**：当前 dashboard 是生成式 HTML，不是常驻桌面产品，也没有系统托盘、提醒、历史趋势入口。
- **AI agent 数据治理还不够深**：当前能发现 agent 目录和空间占用，但对 transcripts、prompt history、session snapshots、shell snapshots、debug logs、file snapshots、orphaned projects、隐私/凭据邻近风险的解释还可以更前瞻。
- **规则库信任模型还不完整**：`--rules-repo` 已支持社区规则，但商业化场景需要 signed rules、version lock、规则 diff、策略审批和企业 policy pack。
- **模型缓存治理可更深入**：Ollama、Hugging Face、LM Studio、GGUF、SafeTensors、ONNX、MLX 等模型文件可进一步识别 partial downloads、重复模型、孤儿 blobs、detached revisions 和外置盘迁移机会。
- **Team / Fleet 还未形成产品**：当前本地治理和通知已有基础，但还没有集中 dashboard、团队策略、设备健康摘要、MDM/Jamf/Intune 部署和审计。

### 3.3 不建议近期投入的方向

- 不建议短期做完整全盘扫描 + 自动分类清理。该方向实现复杂、风险高，并且会直接对抗 WinDirStat、TreeSize、CleanMyMac 等成熟工具。
- 不建议把定位变成“更好的通用磁盘清理工具”。更有胜算的定位是 AI-aware、developer-aware、agent-ready、local-first governance。
- 不建议默认自动清理。当前产品优势来自保守、安全、可解释和可恢复，自动清理会削弱信任。

## 4. 竞品与市场对照

### 4.1 开源竞品

| 产品 | 定位 | 优势 | 相对 AI Disk Doctor 的差异 |
|---|---|---|---|
| Kondo | 项目目录清理工具，覆盖多语言构建产物 | CLI + GUI，覆盖 20+ 项目类型，分发较成熟 | 更偏项目构建产物清理，不是 AI-aware，不强调治理、历史、quarantine restore 和 agent 数据解释 |
| clean-dev-dirs | Rust 开发目录清理 CLI | 并行扫描、dry-run、交互模式、JSON、配置文件、回收站 | 开发缓存覆盖强，但 AI agent、模型缓存、MCP、治理通知和 AI footprint 维度较弱 |
| dev-cleaner | 一键开发缓存清理 | 跨平台、低门槛、覆盖 Xcode/Flutter/VS/Gradle/npm/NuGet/IDE/浏览器缓存 | 更偏传统开发缓存，安全模型和可恢复治理弱于本项目 |

### 4.2 商业产品信号

- CodeCleaner / DevCleaner 类 macOS 商业工具证明“开发者专用磁盘清理”存在付费市场。
- 商业产品通常主打原生 GUI、隐私本地化、免费扫描、一键清理、Docker/Node/Rust/Python/Go/IDE 缓存覆盖。
- 这说明商业化入口更可能来自 GUI、信任链、安装体验、持续守护和团队管理，而不是单纯 CLI 能力。

### 4.3 AI 工具原生清理碎片化

- Claude Code 的本地目录可能包含 transcripts、prompt history、file snapshots、shell snapshots、caches 和 logs，存在空间占用和隐私解释需求。
- Hugging Face 提供官方 cache CLI，但它只治理 Hugging Face 自己的缓存，不覆盖多工具、多模型、多 agent 的统一视角。
- Ollama 社区有未完成下载、孤儿 blob、`ollama list` 不显示但仍占空间等问题，说明模型缓存治理存在真实痛点。
- 各 AI IDE/CLI 的状态目录、日志、测试产物、安装包和 runtime cache 高度碎片化，用户不可能逐一掌握。

## 5. 推荐战略定位

### 5.1 推荐定位

推荐将 AI Disk Doctor 定位为：

> Local-first governance for AI developer machines.

或者中文表述：

> 面向 AI 开发者机器的本地优先存储与数据治理工具。

### 5.2 核心差异化

- **AI-aware**：理解 AI agent、AI IDE、AI CLI、模型缓存、MCP、runtime 和 test artifacts。
- **Risk-aware**：知道什么是缓存、什么是配置、什么可能包含隐私或凭据、什么不能自动处理。
- **Agent-ready**：稳定 JSON 和 Markdown 输出，可被各种 coding agent 调用并解释。
- **Governance-ready**：有历史、异常、通知、调度和可审计事件。
- **Recoverable**：quarantine + restore 是信任基础。
- **Local-first**：默认不上传路径和文件内容，适合隐私敏感的开发机器。

## 6. 商业化路径

### 6.1 Free / OSS

- CLI core
- 内置规则库
- scan / plan / doctor / diff
- safe-only planning
- quarantine / restore
- HTML dashboard
- 本地治理脚本基础能力

### 6.2 Pro

- 原生 GUI / Tray Sentinel
- 自动更新
- 历史趋势 UI
- 深度 AI agent data hygiene 报告
- 规则自动更新
- 模型缓存 reconcile
- 一键生成安全清理预案
- 更友好的恢复 UI

### 6.3 Team

- 团队 policy packs
- Slack / Feishu / GitHub Issue / email 通知
- 设备健康摘要
- AI cache 增长趋势
- 异常治理事件汇总
- 本地 agent 只上传汇总指标，不上传具体路径和文件内容
- 团队规则审批和发布流程

### 6.4 Enterprise

- 离线规则仓库
- 签名规则和规则审计
- MDM / Intune / Jamf 部署
- on-prem 控制台
- 合规审计日志
- 企业 allow / deny policy
- SLA 和支持服务

## 7. 下一步迭代计划

## 7.1 v1.7.0 建议主题

建议 v1.7.0 聚焦：

> AI Agent Data Hygiene + Trusted Distribution

这会同时强化前瞻差异化和商业化基础。

## 7.2 v1.7.0 P0：AI Agent Data Hygiene

目标：把 `doctor --agents` 从“空间诊断”升级为“AI agent 数据治理”。

建议交付：

- 对 Claude、Codex、Gemini、opencode、Cursor、Windsurf、Trae、aider、Continue 做更细的目录语义分类。
- 识别 transcripts、prompt history、session snapshots、file snapshots、shell snapshots、debug logs、cache、temp、test artifacts、installers、orphaned projects。
- 输出每类数据的风险解释：可安全 quarantine、需要 review、只报告不建议清理、永不触碰。
- 对可能包含隐私、源码片段、命令历史、token 邻近信息的目录给出明确提示。
- 增加 orphaned session / stale project 检测，例如长期未修改的 agent project snapshots。
- Markdown/Text 输出保持简洁，只展示 active findings 和建议；JSON 保留完整结构。

验收标准：

- 用户一条命令可以看清“AI 工具占了多少、哪些是缓存、哪些是会话/隐私数据、哪些可以安全 quarantine、哪些必须保留”。
- 不默认调用外部命令；任何 probe 仍需显式 opt-in。
- 不读取文件内容，只基于路径、元数据、大小、时间和规则做判断，保持隐私边界。

## 7.3 v1.7.0 P0：Trusted Distribution

目标：降低普通用户安装门槛，并建立商业化前的信任链。

建议交付：

- Windows x86_64 release artifact 持续保留。
- 增加 Linux x86_64 / aarch64 release artifact。
- 增加 macOS x86_64 / arm64 release artifact。
- 生成 release checksums。
- 生成 SBOM。
- README 增加 checksum verification 说明。
- 增加 Homebrew tap 方案或文档。
- 增加 winget 发布方案或 manifest 草案。
- 评估 macOS notarization 和 Windows signing。

验收标准：

- 用户无需 Rust toolchain 就能在 Windows / macOS / Linux 安装。
- release 页面可以验证 artifact 完整性。
- 文档清楚说明安装、升级、验证和卸载路径。

## 7.4 v1.8.0 P1：Signed Rules Registry

目标：把规则库从“可加载社区 repo”升级为“可信规则生态”。

建议交付：

- 规则包 manifest。
- 规则版本锁。
- 规则 diff 输出。
- 签名规则验证。
- 企业 allow / deny policy pack。
- 本地缓存更新策略。
- 规则来源和更新时间在 scan report 中可见。

验收标准：

- 用户能知道当前生效规则来自哪里、版本是多少、和上次相比变了什么。
- 企业可以固定规则版本，避免无人审查的规则更新影响清理计划。
- 未签名或来源不可信规则默认不进入高信任执行路径。

## 7.5 v1.8.0 P1：Model Cache Reconcile

目标：专门治理 AI 模型缓存和模型文件膨胀。

建议交付：

- Ollama blob / manifest 关系检查。
- Hugging Face cache revisions / refs / snapshots 大小汇总。
- LM Studio / GGUF / SafeTensors / ONNX / MLX 模型文件发现。
- partial downloads / interrupted downloads 识别。
- detached revisions / orphaned blobs 检测。
- 重复模型和近似重复模型提示。
- 外置盘迁移建议。
- 优先调用官方命令或输出官方建议，不直接删除高风险模型数据。

验收标准：

- 用户能区分“正在被模型管理器使用的模型”和“可能是孤儿或中断下载的空间占用”。
- 对可清理对象保持 conservative policy，默认只 report 或 review。
- 不破坏 Ollama、Hugging Face、LM Studio 等工具自身索引。

## 7.6 v1.9.0 P1：Desktop / Tray Sentinel

目标：把 CLI 和 HTML dashboard 升级为普通用户可感知的持续守护产品。

建议交付：

- 轻量桌面 GUI 或 tray app。
- 定期 scan。
- 异常提醒。
- 历史趋势。
- 点击生成清理预案。
- quarantine index 和 restore UI。
- 本地优先，不上传路径和文件内容。

验收标准：

- 用户不需要记 CLI 命令，也能定期看到 AI footprint 增长和安全清理建议。
- GUI 调用同一个 CLI core，不复制一套清理逻辑。
- Pro 付费能力可以自然承载在 GUI / tray sentinel 上。

## 7.7 v2.0 P2：Team / Fleet Governance

目标：从个人工具升级为团队治理产品。

建议交付：

- 本地 agent 汇总指标。
- 中央 dashboard。
- 团队策略下发。
- Feishu / Slack / GitHub Issue / email 通知。
- 设备健康摘要。
- AI cache 增长趋势。
- 治理事件审计。
- MDM / Jamf / Intune 部署文档。

验收标准：

- 团队能看到哪些机器磁盘即将耗尽、哪些 AI 工具增长异常、哪些策略需要执行。
- 默认不上传具体文件路径和文件内容，只上传聚合指标和脱敏事件。
- 企业可以用 policy pack 控制哪些目录可 report、可 plan、可 quarantine。

## 8. 近期可执行任务清单

建议按以下顺序推进：

1. 梳理 Claude / Codex / Gemini / opencode / Cursor / Windsurf 的本地目录结构和数据语义。
2. 为 AI agent data hygiene 增加规则字段或 topic metadata，区分 cache、session、history、snapshot、log、installer、runtime、model、test-artifact。
3. 扩展 `doctor --agents` 输出数据语义和隐私风险解释。
4. 增加 fixtures 覆盖 agent session、orphaned project、debug logs、prompt history、cache、installer。
5. 增加 v1.7.0 release readiness 文档。
6. 扩展 GitHub Actions release artifacts 到 Linux / macOS。
7. 增加 checksum / SBOM 生成。
8. 更新 README 安装、验证和卸载文档。
9. 设计 signed rules registry 的 manifest 草案。
10. 设计 Pro / Team 功能边界，避免 OSS core 过早混入 billing 或账户体系。

## 9. 交叉验证建议

其他 agent 可以重点验证以下问题：

- 当前竞品对照是否遗漏更强的 AI-specific disk governance 工具。
- Kondo、clean-dev-dirs、dev-cleaner、CodeCleaner 等竞品的功能边界和分发方式是否准确。
- Claude Code、Codex、Gemini CLI、opencode、Cursor、Windsurf 的本地目录语义是否可以安全分类。
- 是否存在读取文件内容才能判断风险的场景；如果有，是否应该明确拒绝或 opt-in。
- `doctor --agents` 当前实现是否适合直接扩展为 data hygiene，还是需要先外部化 topic metadata。
- `--rules-repo` 现有安全边界是否足以支撑 signed registry。
- v1.7.0 是否应优先做 agent data hygiene，还是先补 release distribution。
- GUI / tray sentinel 是否应该独立 repo，还是复用当前 crate。
- Team / Fleet 是否应该走 SaaS，还是先做本地生成 report + webhook 的轻量版本。

## 10. 总结判断

AI Disk Doctor 当前最有价值的资产不是“能清理多少 GB”，而是：

- 对 AI 工具和开发者环境的规则理解。
- 对清理风险的保守分层。
- quarantine / restore 带来的信任感。
- 历史增长和异常治理能力。
- agent-readable 输出和 Skill wrapper。
- 本地优先、隐私友好的产品边界。

下一步应该围绕这些资产继续加深，而不是泛化成普通磁盘清理器。最推荐的近期路线是 v1.7.0 先做 **AI Agent Data Hygiene + Trusted Distribution**，同时为后续 Pro GUI、Signed Rules Registry 和 Team / Fleet Governance 铺路。
