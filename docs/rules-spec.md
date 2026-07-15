# Rules Spec

## Rule Schema v2

New rules should use schema v2. It separates detection facts, policy decisions, and executable actions:

```yaml
schema_version: 2
id: huggingface-cache
name: Hugging Face cache
category: models
platform: cross-platform

detector:
  paths:
    - "%USERPROFILE%\\.cache\\huggingface"
    - "~/.cache/huggingface"
  evidence:
    - path-pattern
    - cache-layout

data_kind: model-cache
recoverability: redownload
sensitivity: none
default_liveness: unknown

decision:
  risk: review
  confidence: medium

action:
  type: official-command
  adapter: huggingface
  supports_dry_run: true
  supports_rollback: false

content_access: metadata-only
exclusions: []
reason: "Review the cache with the official tool before cleanup."
warnings: []
```

Required v2 dimensions are `data_kind`, `recoverability`, `sensitivity`, `default_liveness`,
`decision.risk`, `decision.confidence`, `detector.evidence`, `action.type`,
`action.adapter`, and `content_access`. `action.type` is explicit and is not inferred from
`decision.risk`. Supported action types are `quarantine`, `official-command`, and
`report-only`.

`aidisk rules lint` parses every YAML rule, rejects duplicate IDs, rejects unsupported schema
versions, and reports each source file's SHA-256 digest. `scan --json` includes the same rule
source list under `summary.rule_sources`.

## Rule Schema v1 Compatibility

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
- v1 rules remain loadable and are normalized to v2-compatible metadata with
  `schema_version: 1`, `data_kind: unknown`, and low confidence. New rules should not add more
  v1 files.
- v2 `detector.paths` accepts the same flat list or platform map shapes as v1 `paths`.
