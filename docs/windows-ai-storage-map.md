# Windows AI Storage Map

第一批重点覆盖路径：

- `%USERPROFILE%\\.claude`
- `%USERPROFILE%\\.codex`
- `%USERPROFILE%\\.gemini`
- `%APPDATA%\\ai.opencode.desktop`
- `%LOCALAPPDATA%\\ms-playwright`
- `%LOCALAPPDATA%\\LiteSandbox\\logs`
- `%LOCALAPPDATA%\\Google\\Chrome\\User Data\\Default\\Cache`
- `%LOCALAPPDATA%\\Microsoft\\Edge\\User Data\\Default\\Cache`
- `%USERPROFILE%\\projects\\**\\.playwright-browsers`
- `%USERPROFILE%\\.ollama\\models`
- `%USERPROFILE%\\.cache\\huggingface`

已完成覆盖：

- Claude / Codex / Gemini / opencode roots
- Playwright project browsers
- Ollama / Hugging Face model caches
- Docker build cache / Docker Desktop data / WSL `ext4.vhdx` 解释型诊断
- 常见开发产物：`node_modules`、`target`、`.gradle`、`__pycache__`、`dist`、`.next`、`.turbo`

后续会补充：

- Docker build cache / volumes 更精细解释与原生命令联动
- WSL `ext4.vhdx` 检测后的 compact / relocate 指导
- 同步盘高频项目识别
- npm / uv / pip / cargo 开发缓存
