# Workflow

建议执行顺序：

1. `scan`
2. `diff`（当需要对比两次 scan 时，使用 `scripts/run-diff.ps1`）
3. `plan`
4. `clean --dry-run`
5. `clean --yes --quarantine`
6. `restore`
7. `doctor`

原则：

- 没有明确清理意图时，不进入 `clean --yes`
- 没有明确恢复意图时，不进入 `restore --yes`
- `doctor` 用于专题解释，不替代 `scan`
- `doctor --agents` 用于钻取 Claude / Codex / Gemini / opencode 等 agent 根目录，并输出 top child breakdown
- `diff` 只比较两次 scan snapshot，不替代实时 `scan`
- 社区规则库用 `--rules-repo` 或 wrapper 的 `-RulesRepo`，优先本地目录；远程只接受 HTTPS git URL

常用 wrapper：

- scan: `scripts/run-scan.ps1`
- diff: `scripts/run-diff.ps1`
- plan: `scripts/run-plan.ps1`
- clean dry-run: `scripts/run-clean-dry-run.ps1`
- clean execute: `scripts/run-clean.ps1`
- restore: `scripts/run-restore.ps1`
- doctor: `scripts/run-doctor.ps1`

社区规则示例：

```powershell
pwsh -File scripts/run-scan.ps1 -RulesRepo "tests/fixtures/community-rules" -Json
pwsh -File scripts/run-plan.ps1 -RulesRepo "tests/fixtures/community-rules" -SafeOnly -Json
```
