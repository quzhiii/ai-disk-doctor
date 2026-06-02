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

## Phase 5: Skill Integration

目标：

- 完整 skill workflow
- wrapper scripts
- reference docs 对接

验收标准：

- skill 可从 scan 结果中稳定提炼 top findings 和风险说明

## Immediate Next Steps

1. 补充卷信息与更合理的 summary。
2. 引入更真实的集成测试 fixtures。
3. 把敏感路径阻断从关键字提升到规则+策略双层约束。
4. 开始 active-file / locked-file 安全边界实现。
