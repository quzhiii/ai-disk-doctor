# Category Map

主要分类与建议使用方式：

- `ai-agent`: `scan --category ai-agent`，先解释 Claude / Codex / Gemini / opencode 相关路径
- `ai-logs`: `scan --category ai-logs`，通常需要 review 后再决定是否 quarantine
- `browser`: `doctor --playwright` 或 `scan --category browser`
- `browser-cache`: `scan --category browser-cache`，通常可进入 safe-only 预演
- `dev-cache`: `scan --category dev-cache` 或 `plan --safe-only`
- `docker`: `doctor --docker`
- `models`: `doctor --ollama` 或 `doctor --huggingface`
- `sensitive-sample`: 只报告，确认敏感路径阻断是否生效
- `sync-drive`: 先解释风险，不建议直接清理
- `wsl`: `doctor --wsl`
