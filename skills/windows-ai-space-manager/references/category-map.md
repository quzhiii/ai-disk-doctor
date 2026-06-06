# Category Map

主要分类与建议使用方式：

- `ai-agent`: `doctor --agents` 或 `scan --category ai-agent`，先解释 Claude / Codex / Gemini / opencode 相关路径
- `ai-cache`: `doctor --agents` 或 `scan --category ai-cache`，覆盖 promptfoo / evals / LangChain / LlamaIndex / LiteLLM / package cache 等 AI runtime cache
- `ai-cli`: `doctor --agents` 或 `scan --category ai-cli`，覆盖 aider / Continue / opencode 等 CLI 状态与缓存
- `ai-ide`: `doctor --agents` 或 `scan --category ai-ide`，覆盖 Cursor / Windsurf / Trae / Continue / Copilot / Cline / Roo Code 等 IDE 与扩展状态
- `ai-installer`: `plan --safe-only --category ai-installer`，覆盖 Downloads 里的 AI 工具安装包，通常可 quarantine
- `ai-installed-app`: `doctor --agents` 或 `scan --category ai-installed-app`，覆盖 `%LOCALAPPDATA%\Programs` 下 Cursor / Windsurf / Trae / Claude / LM Studio 等安装目录
- `ai-logs`: `scan --category ai-logs`，通常需要 review 后再决定是否 quarantine
- `ai-test-artifact`: `plan --safe-only --category ai-test-artifact`，覆盖 playwright-report / test-results / coverage / promptfoo / evals 等测试与评测产物
- `browser`: `doctor --playwright` 或 `scan --category browser`
- `browser-cache`: `scan --category browser-cache`，通常可进入 safe-only 预演
- `dev-cache`: `scan --category dev-cache` 或 `plan --safe-only`
- `dev-artifact`: 可再生成的开发产物，例如 `node_modules`、Rust `target/`、Gradle cache、Python `__pycache__`、`dist/`、`.next/`、`.turbo/`
- `docker`: `doctor --docker`
- `models`: `doctor --ollama` 或 `doctor --huggingface`
- `sensitive-sample`: 只报告，确认敏感路径阻断是否生效
- `sync-drive`: 先解释风险，不建议直接清理
- `wsl`: `doctor --wsl`
