# Windows AI Space Manager Skill

## Purpose

调用 `aidisk` 进行 Windows AI 时代磁盘空间诊断，并向用户解释风险等级、建议动作和后续步骤。

## Current Workflow

1. 运行 `aidisk scan --json`
2. 解析 `findings`
3. 总结最大占用目录和风险等级
4. 如果用户要求清理，再进入后续 `plan` / `clean` 阶段

当前版本只支持只读扫描。
