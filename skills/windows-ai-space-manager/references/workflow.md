# Workflow

建议执行顺序：

1. `scan`
2. `plan`
3. `clean --dry-run`
4. `clean --yes --quarantine`
5. `restore`
6. `doctor`

原则：

- 没有明确清理意图时，不进入 `clean --yes`
- 没有明确恢复意图时，不进入 `restore --yes`
- `doctor` 用于专题解释，不替代 `scan`
