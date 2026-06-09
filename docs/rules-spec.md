# Rules Spec

## Rule Schema

```yaml
id: chrome-cache
name: Chrome cache
category: browser-cache
platform: windows
paths:
  - "%LOCALAPPDATA%\\Google\\Chrome\\User Data\\Default\\Cache"
risk: safe
cleanup:
  method: quarantine
exclusions: []
reason: "Rebuildable browser cache."
warnings: []
```

## Required Fields

- `id`
- `name`
- `category`
- `platform`
- `paths`
- `risk`
- `cleanup.method`
- `reason`

## Notes

- 路径目前支持环境变量占位，如 `%LOCALAPPDATA%`，也支持 Unix 风格 `~/` home 展开。
- 当前实现已支持 glob 递归匹配，`%USERPROFILE%\\projects\\**\\.playwright-browsers`、`%USERPROFILE%\\**\\node_modules` 这类模式会在扫描阶段展开。
- 规则仍保持路径声明式驱动：不在规则层直接执行清理，只描述路径、风险、建议动作与提示信息。
