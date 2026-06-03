# Windows AI Space Manager

## Purpose

这个 skill 用于在 Windows 上围绕 `aidisk` CLI 做完整的磁盘空间诊断、风险解释、清理预演、隔离清理与恢复编排。

适用场景：

- 用户说 C 盘突然变小、磁盘满了、空间不够
- 用户怀疑 Claude / Codex / Gemini / opencode / Playwright / Docker / WSL / Ollama 占空间
- 用户想先看诊断报告，再决定是否清理
- 用户想做安全的 quarantine 清理，而不是直接删除
- 用户想从 quarantine index 预演或执行恢复
- 用户想比较两次 scan 之间是谁变大了

## Core Principles

- 默认保守：先扫描、解释、预演，再进入执行
- 默认安全：优先 `dry-run` 和 `quarantine`
- 默认可恢复：真实清理优先走 quarantine，并生成 index / log
- 默认分级：按 `safe / review / dangerous / system` 解释，不混淆风险
- 默认不越权：如果用户没有明确要求执行，就停在诊断或预演层

## Trigger Phrases

以下表达应优先触发本 skill：

- "看看 C 盘" / "C 盘怎么满了"
- "帮我分析磁盘空间" / "看下存储占用"
- "Claude / Codex / Gemini / opencode 占了多少"
- "Playwright / Docker / WSL / Ollama 占空间"
- "先做清理预演" / "先别删" / "先 dry-run"
- "隔离清理" / "恢复隔离文件"
- "最近谁变大了" / "对比两次扫描" / "diff scan"

## Workflow

### 1. Scan First

优先执行：

```powershell
pwsh -File scripts/run-scan.ps1 -Json
```

如果用户只关注某个方向，可加分类：

```powershell
pwsh -File scripts/run-scan.ps1 -Category docker -Json
pwsh -File scripts/run-scan.ps1 -Category models -Json
```

输出后先解释：

- top findings
- 风险等级
- safe / review / dangerous / system 的差异
- 是否存在明显的 Docker / WSL / model / browser cache 大头

### 2. Plan Before Clean

如果用户想清理，先执行：

```powershell
pwsh -File scripts/run-plan.ps1 -SafeOnly -Json
```

如果用户想看完整候选：

```powershell
pwsh -File scripts/run-plan.ps1 -Json
```

解释时重点说清：

- 哪些项只是 `guide` / `report-only`
- 哪些项可以进入 quarantine
- 哪些项被 `skipped`
- 被跳过是因为最近仍在变化、敏感路径、还是路径不存在

### 3. Clean Dry-Run

进入清理前，优先执行预演：

```powershell
pwsh -File scripts/run-clean-dry-run.ps1 -SafeOnly -Markdown
```

如果用户指定隔离目录：

```powershell
pwsh -File scripts/run-clean-dry-run.ps1 -SafeOnly -QuarantineRoot "F:\archives" -Json
```

### 4. Real Quarantine Clean

只有当用户明确确认后，才执行真实隔离：

```powershell
pwsh -File scripts/run-clean.ps1 -SafeOnly -QuarantineRoot "F:\archives" -Yes -Json
```

执行后要向用户解释：

- 成功项数量
- 失败项数量
- `skipped-active` / `skipped-locked` / `failed` 的区别
- `index_path` 和 `log_path`

### 5. Restore

恢复前先 dry-run：

```powershell
pwsh -File scripts/run-restore.ps1 -Index "F:\archives\.aidisk\quarantine-index-YYYYMMDD-HHMMSS.json" -DryRun -Json
```

用户确认后再执行恢复：

```powershell
pwsh -File scripts/run-restore.ps1 -Index "F:\archives\.aidisk\quarantine-index-YYYYMMDD-HHMMSS.json" -Yes -Json
```

### 6. Doctor

专项诊断优先命令：

```powershell
pwsh -File scripts/run-doctor.ps1 -Docker -Json
pwsh -File scripts/run-doctor.ps1 -Wsl -Ollama -Markdown
pwsh -File scripts/run-doctor.ps1 -Playwright -HuggingFace -Markdown
pwsh -File scripts/run-doctor.ps1 -Markdown
```

### 7. Diff Between Two Scans

当用户已经有两次 scan 输出，或者明确在问“最近谁涨了”，执行：

```powershell
pwsh -File scripts/run-diff.ps1 -Before "..\examples\diff-before.example.json" -After "..\examples\diff-after.example.json" -Markdown
```

解释时重点说清：

- `appeared` 是新出现的真实占用路径，不包含 `exists=false` 的占位规则路径
- `grew` / `shrunk` 只针对两次都真实存在的路径
- `disappeared` 表示上次存在、这次不存在

## Response Style

在对话中优先按这个顺序反馈：

1. 结论先行：最大占用是什么
2. 风险分层：哪些能动，哪些不能动
3. 下一步建议：scan / plan / clean / restore / doctor / diff 里的一个
4. 如果做了执行：明确写出 index / log 路径

## References

- `references/workflow.md`
- `references/risk-cheatsheet.md`
- `references/category-map.md`

## Scripts

- `scripts/run-scan.ps1`
- `scripts/run-plan.ps1`
- `scripts/run-clean-dry-run.ps1`
- `scripts/run-clean.ps1`
- `scripts/run-restore.ps1`
- `scripts/run-doctor.ps1`
- `scripts/run-diff.ps1`
