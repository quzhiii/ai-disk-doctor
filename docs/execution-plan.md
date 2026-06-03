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

### Immediate Next Steps

1. 按 P0 → P1 → P2 顺序逐项实现。
2. 每项完成后跑测试并本地提交。
3. 下一批聚焦 Phase 5 skill 端到端示例与 Phase 6 后续优化。

## Release Readiness

当前已完成：

- `CHANGELOG.md`
- `docs/release-notes/v1.0.0.md`
- `scripts/release-smoke.ps1`
- `aidisk` crate version `1.0.0`
