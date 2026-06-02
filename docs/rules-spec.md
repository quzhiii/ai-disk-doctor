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

- 路径目前支持环境变量占位，如 `%LOCALAPPDATA%`
- 第一版只做精确路径展开，不做 glob 递归匹配
- `**\\.playwright-browsers` 这类项目内模式匹配将在后续单独实现
