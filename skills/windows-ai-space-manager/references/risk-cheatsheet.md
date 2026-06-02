# Risk Cheatsheet

| Risk | Meaning | Default Handling |
|---|---|---|
| `safe` | 可再生缓存、低风险内容 | 可进入 `plan` / `clean` |
| `review` | 需要人工判断 | 先解释，再确认 |
| `dangerous` | 可能导致数据丢失 | 默认不执行 |
| `system` | 系统级对象 | 只做说明和指导 |

常见跳过原因：

- `filtered out by safe-only mode`
- `blocked because path looks sensitive`
- `skipped because path was recently modified`
- `path does not exist`
- `skipped-conflict`
- `skipped-locked`
