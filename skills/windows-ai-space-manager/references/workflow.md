# Workflow

建议执行顺序：

1. `scan`
2. `diff`（当需要对比两次 scan 时）
3. `plan`
4. `clean --dry-run`
5. `clean --yes --quarantine`
6. `restore`
7. `doctor`

原则：

- 没有明确清理意图时，不进入 `clean --yes`
- 没有明确恢复意图时，不进入 `restore --yes`
- `doctor` 用于专题解释，不替代 `scan`
- `diff` 只比较两次 scan snapshot，不替代实时 `scan`
