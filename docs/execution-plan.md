# Execution Plan

## Goal

按原始项目规划，把实现拆成可交付、可验证的阶段，优先形成只读扫描 MVP，再逐步进入风险计划和安全清理。

## Phase 0: Foundation

已完成：

- 仓库骨架
- 基础文档
- `aidisk` Rust crate 初始化
- 示例规则文件
- 最小 smoke test
- 卷空间摘要
- glob 路径规则扫描

验收标准：

- `cargo test` 通过
- `cargo run -- scan --json` 可输出结构化结果

## Phase 1: Read-Only Scan MVP

当前范围：

- 规则加载
- 环境变量路径展开
- 存在性检查
- 目录大小统计
- 卷空间摘要
- `text` / `json` / `markdown` 输出
- `--category` 过滤
- glob 路径规则支持
- 风险聚合 summary
- top findings
- `plan` 只读 dry-run 骨架

待补项：

- 更精确的 Windows 系统路径识别
- 更细的 finding 排序和摘要
- 更真实的 Playwright / Docker / WSL fixtures

验收标准：

- 扫描结果可稳定列出规则命中的路径
- 输出包含风险等级、建议动作、原因和警告
- 过滤分类后仍能正确统计 summary
- `plan --safe-only` 只输出可进入安全候选集的项

## Phase 2: Risk Planning

目标：

- 增强 `aidisk plan`
- 引入 `safe-only`
- 加入 secret / credential 阻断
- 加入最近修改时间窗口检查
- 加入按策略分组和 quarantine 目标生成

当前已完成：

- `safe-only` 过滤
- 敏感路径关键字阻断
- 最近修改时间过滤
- action groups
- skipped reason 输出

验收标准：

- 所有 `plan` 结果默认 dry-run
- 敏感路径不会进入候选集
- 最近仍在变化的路径默认跳过

## Phase 3: Quarantine Cleanup

目标：

- 增加 `aidisk clean --dry-run`
- 增加 `aidisk clean --yes --quarantine`
- 生成恢复说明和执行日志

当前已完成：

- `clean --dry-run`
- dry-run action groups
- skipped 继承输出
- quarantine path planning
- `clean --yes --quarantine` 执行骨架
- 恢复索引与执行日志写入
- active-file 保守跳过
- `restore` 命令骨架

验收标准：

- dry-run 与实际执行目标一致
- 失败项不会中断整体任务
- 可验证 quarantine 前后大小和文件数量

## Phase 4: Doctor Commands

目标：

- `doctor --wsl`
- `doctor --docker`
- `doctor --ollama`
- `doctor --playwright`

验收标准：

- 每个子命令优先给出解释和官方建议，而不是直接删除建议

当前已完成：

- `doctor --docker`
- `doctor --wsl`
- `doctor --ollama`
- `doctor --playwright`
- `doctor --huggingface`
- 空结果与未命中路径解释
- 结构化建议输出
- 无参数时运行完整诊断集合

Doctor V2 status: Completed

当前 doctor 已经能安全解释专项 topic，但它仍然主要复用 scan 结果并追加静态建议。真实机器测试显示，Docker/WSL/Ollama/Playwright/HuggingFace 不是主要空间来源，实际大户是 `.gemini`、`.claude`、`opencode`、`.codex` 等 AI Agent 目录。因此 Phase 4 的下一轮目标是让 doctor 从“专项摘要”升级为“会钻进去分析的诊断层”。

Doctor V2 roadmap：

| 优先级 | 方向 | 验收标准 |
|---|---|---|
| P0 | `doctor --agents` | 默认诊断包含 AI Agent topic，并能覆盖 `.gemini`、`.claude`、`.codex`、`opencode` 等实际大户 |
| P0 | AI 工具覆盖扩展 | 规则覆盖新安装的 AI agent / IDE / CLI / installed app / runtime cache / installer / test artifact |
| P0 | 子目录分解 | 对大型 existing finding 输出 top child breakdown，说明空间主要在 cache/session/log/model/browser/runtime 哪类目录里 |
| P1 | 数据驱动建议 | 根据 `exists`、size、risk、action、breakdown 生成建议；空目录或 1 字节占位应提示 no action needed |
| P1 | 工具检测 | Docker/WSL/Ollama/Playwright 未安装或未运行时明确标记 not detected/skip，而不是只输出泛化建议 |
| P1 | 输出降噪 | Markdown/Text 只展开 active findings，missing paths 汇总为 Not detected 计数；JSON 保留完整 findings |
| P2 | 外部命令探测 | `--probe-tools` 可选调用 `docker system df`、`wsl --list --verbose`、`ollama list`，用于补充而不是替代文件系统诊断 | Completed |
| P2 | 增长率诊断 | 结合 `.aidisk/reports` 和 `diff --latest` 回答哪些目录最近增长最快 | Completed |
| P3 | 动态 topic registry | 内置 `DoctorTopicSpec` registry 已集中 topic 名称、默认启用、matcher、建议和 probe metadata；外部化 topic metadata 仍是后续方向 | Completed |

详细执行计划：`docs/plans/2026-06-03-doctor-v2-roadmap.md`。

当前状态说明：`doctor --agents`、bounded breakdown、data-driven recommendations、tool presence detection、optional probes、growth-aware doctor、dynamic topic registry 均已落地。Doctor 的剩余工作不再是 Phase 4 / Doctor V2 收尾，而是后续 roadmap 中可能独立立项的外部 topic metadata 或新的诊断主题扩展。

## Phase 5: Skill Integration

目标：

- 完整 skill workflow
- wrapper scripts
- reference docs 对接

当前已完成：

- SKILL.md 完整工作流与触发词
- references 拆分为 workflow / risk-cheatsheet / category-map
- 6 个可执行 PowerShell wrapper scripts

验收标准：

- skill 可从 scan 结果中稳定提炼 top findings 和风险说明

## Phase 6: Hardening & Optimization

Phase 8 status: Completed

多视角审查（brainstorming / architecture / security）后加入的加固路线。

### P0 — 安全与架构脆弱点

| 条目 | 说明 | 优先级 |
|---|---|---|
| 跨盘 quarantine fallback | `fs::rename` 在 Windows 上跨盘会失败，须 fallback 到 `copy + delete` | P0 |
| restore index 结构校验 | 防篡改 index 导致恢复越界或 OOM | P0 |

### P1 — 防御性加固

| 条目 | 说明 | 优先级 |
|---|---|---|
| doctor 输出当前 policy 可见性 | 让 agent 和用户知道当前生效的敏感关键字与允许动作 | P1 |
| scan depth limit 与 partial 标记 | 防止 WalkDir 无限递归；超大目录超时标 partial | P1 |

### P2 — 历史感知

| 条目 | 说明 | 优先级 |
|---|---|---|
| `aidisk diff` 历史对比 | 基于 `.aidisk/reports/` 对比两次扫描，回答 "谁长大了" | Completed |
| scan snapshot + `diff --latest` | `scan` 自动落盘，`diff --latest` 自动选最近两个 snapshot | Completed |

### P3 — 生态扩展

| 条目 | 说明 | 优先级 |
|---|---|---|
| 规则库远程拉取与社区贡献模型 | rules 单独成 repo，支持 `--rules-repo <url>` | Completed |

## Phase 7: Coverage And Discovery Roadmap

基于 P1 结构化 JSON 错误完成后的下一轮产品优先级。目标是先扩大“立刻有用、低风险、规则驱动”的覆盖面，再补通用发现能力；避免进入复杂、低胜率的完整全盘扫描和分类清理。

| 优先级 | 方向 | 验收标准 |
|---|---|---|
| P1 | 扩大规则覆盖面 | Completed: 内置规则覆盖 `node_modules`、`target/`、`.gradle`、`__pycache__`、`dist/`、`.next/`、`.turbo/` 等常见开发产物；不改核心扫描/清理架构；`scan --json` 和 `plan --safe-only --json` 可稳定呈现新增命中 |
| P2 | 大文件发现模式 | Completed: 增加 `scan --large-files --min-size <SIZE>`，输出按大小排序的大文件/目录列表；不分类、不给清理建议 |
| P3 | 跨平台规则适配 | Completed: 增加 `~` 展开支持，ollama/huggingface/docker 规则已包含 linux/macOS 路径 |

明确不建议作为近期目标：完整全盘扫描 + 自动分类清理。该方向实现复杂度高，且正面对抗 WinDirStat/TreeSize，当前产品胜算不高。

### Immediate Next Steps

1. Phase 7 P1 / P2 / P3 当前已全部完成；下一轮应先重新评估 roadmap，再决定新的产品切片。
2. v1.3.0 release readiness is complete；当前主线已同步 `CHANGELOG.md`、README、release notes、crate version，并已执行 release smoke。
3. 继续保持 `doctor` 默认只读、`--probe-tools` 显式 opt-in、JSON 结构稳定、Markdown/Text 输出降噪。
4. 每项完成后跑测试并本地提交。

## Release Readiness

当前已完成：

- `CHANGELOG.md`
- `docs/release-notes/v1.0.0.md`
- `docs/release-notes/v1.1.0.md`
- `docs/release-notes/v1.2.0.md`
- `docs/release-notes/v1.3.0.md`
- `docs/release-notes/v1.4.0.md`
- `scripts/release-smoke.ps1`
- `aidisk` crate version `1.4.0`

## Phase 9: Local Scheduled Governance

Phase 9 status: Completed

目标：

- 将 `aidisk` 从按需诊断 CLI 升级为本地持续治理哨兵。
- 第一版聚焦增长异常，而不是自动清理。
- 使用混合架构：Rust 提供异常判断核心，脚本负责调度和通知编排。

架构边界：

- Rust 核心新增 `anomaly` 能力，基于 scan snapshots / diff 结果做双阈值增长异常判断。
- 阈值模型采用绝对增长 + 相对增长组合，避免小目录噪声和大目录低敏感度问题。
- 调度层先由 Windows PowerShell + Task Scheduler 落地，但接口应能映射到 cron / launchd / systemd timer。
- 通知层采用可插拔 adapter，先支持 local file / generic webhook payload，后续扩展微信、企业微信、飞书、Slack、Telegram、Discord、email。

当前已完成：

- Rust 核心新增 `aidisk anomaly --latest`，可读取最近两个 scan snapshots 并输出增长异常报告。
- `aidisk anomaly --before <FILE> --after <FILE>` 支持显式快照对比。
- JSON 输出稳定，Markdown 输出适合直接投递到 IM / webhook。
- `run-governance.ps1` 完成本地治理链路：scan → anomaly → report artifact。
- `governance-event.json` 提供稳定事件封装，覆盖 `anomaly_found`、`pending_history`、`no_anomaly`。
- 事件包含消息友好字段：`headline`、`summary_markdown`、`top_anomaly_path`、`top_anomaly_growth_bytes`。
- generic webhook adapter 支持投递治理事件，并在失败时写出 `webhook-failure.json`。
- Windows Task Scheduler 工具链已闭环：`register-governance-task.ps1`、`show-governance-task.ps1`、`unregister-governance-task.ps1`、`test-run-governance-task.ps1`。
- 第一版保持边界：不做后台常驻、不自动清理、不绑定单一 IM 服务。

验收标准：

- `aidisk anomaly --latest` 可读取最近两个 scan snapshots 并输出增长异常报告。
- `aidisk anomaly --before <FILE> --after <FILE>` 支持显式快照对比。
- JSON 输出稳定，Markdown 输出适合直接投递到 IM / webhook。
- Windows 脚本能完成一次本地治理 run：scan → anomaly → report artifact。
- 第一版不做后台常驻、不自动清理、不绑定单一 IM 服务。

### Phase 9 Immediate Next Steps

1. Phase 10 采用 scheduler-first 路线，先做 cross-platform scheduler adapters，按 cron / launchd / systemd timer 分平台落地。
2. notifier adapter expansion 后置；应先定义 adapter boundary，再绑定具体飞书 / Slack / 微信等平台。
3. 保持当前治理边界：继续复用 `run-governance.ps1` 与稳定事件契约，不把具体平台通知逻辑硬塞回 Rust 核心。

## Phase 10: Cross-Platform Scheduler Adapters

Phase 10 status: Completed

目标：

- 把当前 Windows Task Scheduler 治理调度能力扩展到 cron / launchd / systemd timer。
- 继续复用现有 `run-governance.ps1` / 治理事件契约，不改 Rust anomaly core。

边界：

- 先做本地调度适配层，不先做具体 IM notifier adapter。
- 保持第一版只做注册 / 查看 / 卸载 / 测试运行，不引入后台常驻服务。

详细执行计划：`docs/plans/2026-06-09-phase-10-cross-platform-scheduler-adapters.md`。

实施成果：

- cron adapter: `scripts/governance/cron/` (register, show, unregister, test-run)
- launchd adapter: `scripts/governance/launchd/` (register, show, unregister, test-run)
- systemd timer adapter: `scripts/governance/systemd/` (register, show, unregister, test-run)
- 跨平台治理入口: `scripts/governance/run-governance.sh`
- 所有平台遵循统一 scheduler adapter contract
- 测试覆盖: `aidisk/tests/release_artifacts.rs` 包含三个平台的契约测试

## Phase 10 Immediate Next Steps

1. 用户手册：编写各平台的 scheduler 使用文档
2. 示例配置：添加常见调度场景的配置示例
3. 未来增强：考虑 notifier adapter 扩展（飞书 / Slack / 微信等）

## Phase 11: Unix Governance Entrypoint

Phase 11 status: Completed

目标：

- 补齐 cron / launchd / systemd timer 脚本引用的 `scripts/governance/run-governance.sh`。
- 在 Unix-like 平台提供与 Windows `run-governance.ps1` 对齐的 scan -> anomaly -> governance event -> notifier workflow。

实施成果：

- 新增 `scripts/governance/run-governance.sh`，支持 `--reports-dir`、`--output-dir`、`--min-growth`、`--min-growth-percent`、`--notifier-adapter`、`--webhook-url`、`--webhook-timeout-seconds`。
- 使用 `jq` 生成稳定 `governance-event.json`，保留 `anomaly_found`、`pending_history`、`no_anomaly` 事件类型与关键字段。
- 使用 `curl` 处理 generic webhook delivery，并保留 `webhook-failure.json` 与 `delivery_status` 字段。
- 新增 release artifact 测试覆盖 Unix 治理入口脚本的关键模式与非破坏性边界。

## Current Roadmap Snapshot

当前主线已经完成从 v1.0.0 到 v1.4.0 的核心闭环，并完成了下一轮跨平台本地治理能力：

- v1.0.0：本地 scan / plan / clean / restore / doctor / diff 基础闭环。
- v1.1.0：Doctor V2，覆盖 AI Agent / tooling 诊断、breakdown、probe、growth-aware doctor。
- v1.2.0：规则覆盖扩展、大文件发现、跨平台规则路径、JSON 错误契约与可运维元数据。
- v1.3.0：Windows 本地定时治理，包含 anomaly 核心、governance event、generic webhook、Windows Task Scheduler 闭环。
- v1.4.0：cron / launchd / systemd timer adapters 与 Unix `run-governance.sh` 已完成。

当前不建议立刻进入具体 IM notifier adapter（飞书 / Slack / 微信等）。notifier 会引入密钥管理、平台 API、失败重试、幂等、限流和交付语义，应该作为 v1.5.0 之后的独立大切片，而不是混入 v1.4.0 release readiness。

## Phase 12: v1.4.0 Cross-Platform Governance Release Readiness

Phase 12 status: Completed

目标：

- 将 Unreleased 中已完成的跨平台 scheduler + Unix governance entrypoint 固化为 v1.4.0 可发布版本。
- 补齐用户能直接使用的文档、示例、release notes 和 smoke 验证。
- 明确 v1.4.0 的边界：跨平台本地调度治理，不包含具体 IM notifier adapter。

建议任务：

1. 更新 README / README.zh-CN 的 What's New、Key Features 和 governance 使用说明，补充 cron / launchd / systemd timer 示例。
2. 新增 `docs/release-notes/v1.4.0.md`，覆盖 scheduler adapters、`run-governance.sh`、依赖项（`bash` / `jq` / `curl` / `cargo`）和已知限制。
3. 更新 release artifact 测试，要求 CHANGELOG、README、release notes 覆盖 v1.4.0 范围。
4. 运行并记录 v1.4.0 release smoke：Windows 原有测试 + Unix 脚本静态契约 + 可选 Linux/macOS 手工验证。
5. 如决定正式发版，再 bump crate version、Cargo.lock、README badge 和 release references。

v1.4.0 完成状态：

- 功能开发：Completed。核心跨平台调度治理能力已完成。
- 发布准备：Completed。README、release notes、CHANGELOG、版本号、release artifact tests 和完整测试套件均已同步。
- 后续大切片：具体 notifier adapter expansion（飞书 / Slack / 微信等）建议作为 v1.5.0 或更后的独立阶段。

## Phase 13: Notifier Adapter Foundation

Phase 13 status: Completed

目标：

- 在不改变 `governance-event.json` 契约的前提下，增加脚本层 Notifier Adapter Foundation。
- 继续保留 generic webhook，同时新增第一个具体平台 adapter：Feishu。
- 明确 secrets 处理边界：Feishu webhook URL 只通过 `FEISHU_WEBHOOK_URL` 环境变量注入，不写入命令行示例或失败产物。

实施成果：

- 新增 `scripts/governance/send-governance-event.sh`，通过 `--adapter`、`--event-path`、`--output-dir` 分发 `local-file`、`webhook`、`feishu`。
- 新增 `scripts/governance/notifiers/feishu.sh`，从 `governance-event.json` 读取 `headline` 与 `summary_markdown`，向 Feishu 发送 text 消息。
- Feishu 失败时写出 `feishu-failure.json`，但不保存 `FEISHU_WEBHOOK_URL` 或 resolved webhook URL。
- `run-governance.sh` 支持 `--notifier-adapter feishu` 并委托 dispatcher 交付。
- 文档入口：`docs/notifier-adapters.md`。

后续方向：

- Slack / WeChat / DingTalk / email 等 adapter 可复用该 dispatcher 和 `governance-event.json` 契约。
- 重试、幂等、限流和消息模板可以作为后续独立阶段，而不是塞进 Phase 13。

## Phase 14: Governance Reliability, Documentation, and Cross-Platform Verification

Phase 14 status: M1 Completed, M2 Completed

目标：

- 加固 notifier foundation：添加重试和幂等（dedup），不改变 dispatcher/adapter 边界。
- 创建跨平台治理用户手册：覆盖 Windows/cron/launchd/systemd timer 使用说明及 Feishu/webhook 投递。
- 扩展 CI 到 Ubuntu/macOS runner，并编写手工验证 checklist。

### M1: Notifier Reliability Enhancements — Completed

- `scripts/governance/dedup-governance-event.sh`：基于 event hash 的幂等去重，防止重复投递。
- `scripts/governance/retry-governance-notify.sh`：最多 3 次重试，60s 线性延迟，写出 `retry-failure.json`。
- 已将 dedup + retry 集成到 `run-governance.sh` 治理流程中。
- 新增 `scripts/governance/templates/feishu-governance.tmpl` 可自定义 Feishu 消息模板。

### M2: Cross-Platform User Manual — Completed

- 新增 `docs/governance-manual.md`：覆盖 Prerequisites、Quick Start（四平台）、Governance Flow、Notifier Adapters、Reliability（dedup + retry）、Troubleshooting。
- 更新 `docs/notifier-adapters.md` 添加 Reliability 小节。
- README 双语文档新增 governance manual 链接。
- CHANGELOG 新增 M1/M2 条目。

### M3: Cross-Platform Real Environment Verification — Pending

- 扩展 `.github/workflows/ci.yml` 以包含 Ubuntu 和 macOS runner。
- 编写 `docs/cross-platform-verification.md` 手动验证 checklist。

后续方向：

- M3 完成 real environment CI 和手动验证后，准备下一版本发布。
