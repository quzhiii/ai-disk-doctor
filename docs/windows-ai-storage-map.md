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

后续会补充：

- Docker build cache / volumes 更精细解释与原生命令联动
- WSL `ext4.vhdx` 检测后的 compact / relocate 指导
- 同步盘高频项目识别
- npm / uv / pip / cargo 开发缓存
