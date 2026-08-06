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
- `skipped-active`
- `skipped-conflict`
- `skipped-locked`

执行与恢复状态：

| Status | Meaning |
|---|---|
| `quarantined` | 已从源路径隔离移动到 quarantine 目标；当前 index v2 状态 |
| `moved` | 已从源路径隔离移动到 quarantine 目标；旧 index 兼容状态 |
| `planned` | restore dry-run 预演项，尚未执行 |
| `restored` | 已从 quarantine 恢复到原目标 |
| `skipped-active` | 源路径最近仍在变化，保守跳过 |
| `skipped-conflict` | 恢复目标已存在，不覆盖现有路径 |
| `skipped-locked` | 文件被占用或权限不足，未移动/恢复 |
| `partial-copy` | copy 阶段失败，未完成隔离，需要检查 journal/log |
| `verification-failed` | copy 后校验失败，目标副本已尝试清理 |
| `source-remove-failed` | copy 已验证，但源路径删除失败，需要人工检查源和目标 |
| `failed` | 执行失败，需要查看 message / log |

常见 `stage`：

- `planned` / `renaming` / `copying` / `copied` / `verified` / `source-removing` / `quarantined`
- `restore-planned` / `restoring` / `restored`
- `preflight` / `verifying` / `restore-preflight` / `restore-copying` / `restore-failed`

执行结果中的 `stage` 表示最近阶段，旧 index 中默认为 `unknown`。`recovery` 给出下一步恢复建议，`journal_path` 指向逐阶段 journal。
