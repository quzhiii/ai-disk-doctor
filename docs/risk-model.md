# Risk Model

| Level | Meaning | Default Behavior |
|---|---|---|
| `safe` | 可重建、低风险缓存或旧日志 | 后续可进入 dry-run / quarantine |
| `review` | 可能可清，但必须人工确认 | 当前只报告 |
| `dangerous` | 可能导致数据丢失或登录失效 | 永不默认处理 |
| `system` | 系统级文件或配置 | 只解释，不自动处理 |

## Current Enforcement

当前代码阶段只负责：

- 识别规则中的风险等级
- 在扫描报告中明确展示风险

后续会补充：

- secret / credential 名称阻断
- 最近修改时间窗口
- 文件锁和活跃进程跳过
- quarantine 恢复机制
