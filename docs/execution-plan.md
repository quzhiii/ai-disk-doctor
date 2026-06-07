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

下一步：Doctor V2。

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
2. 若准备发版，统一同步 `CHANGELOG.md`、README、release notes 与 crate version，再执行 release smoke。
3. 继续保持 `doctor` 默认只读、`--probe-tools` 显式 opt-in、JSON 结构稳定、Markdown/Text 输出降噪。
4. 每项完成后跑测试并本地提交。

## Release Readiness

当前已完成：

- `CHANGELOG.md`
- `docs/release-notes/v1.0.0.md`
- `docs/release-notes/v1.1.0.md`
- `docs/release-notes/v1.2.0.md`
- `scripts/release-smoke.ps1`
- `aidisk` crate version `1.2.0`
