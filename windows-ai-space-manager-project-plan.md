# Windows AI Space Manager 项目计划书

项目名建议：`windows-ai-space-manager`  
CLI 命令名建议：`aidisk`  
Skill 名建议：`windows-ai-space-manager`

## 一句话定位

做一个面向 Windows 的 AI 时代 C 盘空间诊断与治理工具，专门识别 AI agent、浏览器自动化、模型缓存、WSL、Docker、同步盘、高频日志等新型空间黑洞，并通过安全分级、dry-run、隔离清理、复查报告来防止误删。

## 项目背景

AI agent 出现后，本地电脑的空间占用模式变了。以前主要是系统更新、浏览器缓存、临时文件、软件安装包。现在多了很多传统清理工具不理解的新型大户：

- Claude、Codex、Gemini、opencode 等 agent 产生 session、日志、缓存、工具调用文件
- Playwright、浏览器自动化工具下载多个 Chromium、Edge、WebKit 运行时
- Ollama、Hugging Face、Stable Diffusion、ComfyUI 等本地模型缓存动辄几十 GB
- WSL、Docker Desktop 产生 VHDX、镜像、volume、build cache
- AI 项目常含 `node_modules`、`.next`、`.turbo`、`output`、`dist`、`screenshots`
- AI 项目放进百度云、OneDrive、Nutstore 后，会触发同步软件和安全审计的联动
- 企业安全软件如深信服、LiteSandbox 可能把高频文件访问写成巨型日志
- 普通清理工具只知道“这里有大文件”，但不知道“这个东西能不能删、为什么变大、怎么防止复发”

这个项目要解决的不是“清垃圾”，而是“解释空间为什么消失，并安全治理”。

## 核心目标

- 快速找出 Windows C 盘空间突然减少的原因
- 识别 AI agent 和开发工具相关的大型缓存、日志、模型、临时文件
- 用风险等级区分可安全清理、需要确认、绝对不能碰的内容
- 默认只做 dry-run 和报告，不直接删除
- 支持将低风险内容移动到隔离目录，而不是直接删除
- 提供 opencode/Claude skill，让 AI agent 能安全地调用工具并向用户解释结果
- 形成可扩展规则库，后续不断加入新的 AI 工具路径和清理策略

## 非目标

- 不做传统杀毒软件
- 不做全自动无脑清理器
- 不默认删除模型、凭据、浏览器登录态、聊天记录、配置文件
- 不默认清 Docker volume、WSL VHDX、数据库文件
- 不追求一开始覆盖所有软件
- 不替代 WinDirStat、BleachBit、Czkawka，而是补足它们不懂 AI 工具链的部分

## 目标用户

- Windows 上重度使用 AI 编程工具的人
- 使用 Claude Code、Codex、Gemini CLI、opencode、Cursor、Trae、Qoder、Kiro 等工具的人
- 经常运行 Playwright、browser automation、agent workflow 的人
- 本地使用 Ollama、Hugging Face、ComfyUI、Stable Diffusion 的人
- 使用 WSL、Docker Desktop 做开发的人
- C 盘经常突然爆满，但不知道原因的人
- 想让 AI agent 帮忙管理本机空间，但担心误删的人

## 典型使用场景

- 用户说：“C 盘怎么突然少了 40GB？”
- 用户说：“帮我看看是不是 Codex、Claude、opencode 缓存太大”
- 用户说：“我把 AI 项目放到百度云里后电脑变慢了”
- 用户说：“Playwright 是不是下了很多浏览器？”
- 用户说：“WSL 的 ext4.vhdx 怎么越来越大？”
- 用户说：“Docker Desktop 占了几十 GB，哪些能清？”
- 用户说：“Ollama 模型太大，帮我盘点但不要乱删”
- 用户说：“先把安全缓存搬到 F 盘隔离，不直接删除”

## 产品形态

| 组件 | 名称 | 作用 |
|---|---|---|
| CLI 工具 | `aidisk` | 负责扫描、分类、生成计划、执行 dry-run、隔离清理 |
| 规则库 | `rules/*.yaml` | 描述已知工具路径、风险等级、清理方法、排除项 |
| Skill | `windows-ai-space-manager` | 让 AI agent 会正确使用 `aidisk`，并解释结果 |
| 报告模板 | Markdown/JSON | 输出给用户看的诊断报告和机器可读结果 |
| 测试样本 | `fixtures/` | 模拟 Windows 用户目录、AI 缓存、日志风暴、WSL/Docker 数据 |

## 总体架构

```text
User / AI Agent
    |
    v
windows-ai-space-manager skill
    |
    v
aidisk CLI
    |
    +-- scanner engine
    +-- rule registry
    +-- risk classifier
    +-- cleanup planner
    +-- safety engine
    +-- executor
    +-- reporter
```

## 职责划分

| 层级 | 负责什么 | 不负责什么 |
|---|---|---|
| Skill | 提问、解释、建议、确认风险、总结结果 | 不直接随意删文件 |
| CLI | 扫描文件、计算大小、识别路径、执行安全操作 | 不做模糊判断 |
| 规则库 | 定义路径、类别、风险、清理策略 | 不执行命令 |
| 用户 | 审批 `review` 和 `dangerous` 操作 | 不需要理解所有底层目录 |

## 核心命令设计

```powershell
aidisk scan
aidisk scan --json
aidisk scan --markdown
aidisk scan --category ai
aidisk scan --category browser
aidisk scan --category wsl
aidisk scan --category docker
aidisk plan --safe-only
aidisk plan --older-than 14d
aidisk clean --dry-run --safe-only
aidisk clean --yes --safe-only --quarantine F:\archives
aidisk explain --path "C:\Users\quzhi\AppData\Local\LiteSandbox\logs"
aidisk doctor
aidisk doctor --wsl --docker --ollama --playwright
```

## MVP 功能范围

| 功能 | MVP 是否做 | 说明 |
|---|---:|---|
| 扫描 C 盘顶层目录大小 | 做 | 用于快速判断大头 |
| 扫描用户目录 AI 相关路径 | 做 | `.claude`、`.codex`、`.gemini`、`.opencode` 等 |
| 扫描 Playwright 浏览器缓存 | 做 | `ms-playwright`、`.playwright-browsers` |
| 扫描 LiteSandbox 日志 | 做 | 针对日志风暴场景 |
| 扫描 Chrome/Edge 缓存 | 做 | 只识别 cache，不碰 profile 核心数据 |
| 扫描 WSL VHDX | 做 | 只报告，不自动压缩 |
| 扫描 Docker 占用 | 做 | 优先调用 `docker system df`，不清 volume |
| 扫描 Ollama 模型 | 做 | 只盘点，不默认删除 |
| 扫描 Hugging Face 缓存 | 做 | 识别并建议官方清理方法 |
| dry-run 清理计划 | 做 | 第一版必须做 |
| 隔离移动 | 做 | 默认移动到 quarantine，不直接删 |
| 直接删除 | 暂缓 | 第二阶段再做，必须显式 `--delete` |
| GUI | 不做 | 先 CLI + skill |
| 后台常驻监控 | 不做 | 后续可以加 |
| 自动修复 pagefile | 暂缓 | 先诊断和生成命令，不默认改系统设置 |

## 风险等级模型

| 等级 | 含义 | 默认行为 | 示例 |
|---|---|---|---|
| `safe` | 可重建、低风险缓存或旧日志 | 可 dry-run，可隔离移动 | 浏览器 cache、旧 updater、旧 Temp |
| `review` | 可能可清，但需要用户确认 | 只报告，不自动处理 | `.gemini`、`.codex` 日志、模型缓存、项目 output |
| `dangerous` | 删除可能造成数据丢失或登录失效 | 默认禁止 | token、cookies、credentials、Docker volume、数据库 |
| `system` | 系统级设置或文件 | 只解释和生成步骤 | pagefile、hiberfil、WSL VHDX 压缩 |

## 第一批规则覆盖

| 类别 | 路径示例 | 默认风险 |
|---|---|---|
| Agent 配置 | `%USERPROFILE%\.claude` | `review` |
| Agent 配置 | `%USERPROFILE%\.codex` | `review` |
| Agent 配置 | `%USERPROFILE%\.gemini` | `review` |
| opencode | `%APPDATA%\ai.opencode.desktop` | `review` |
| OpenAI/Codex | `%LOCALAPPDATA%\OpenAI` | `review` |
| Playwright | `%LOCALAPPDATA%\ms-playwright` | `review` |
| Playwright 项目目录 | `**\.playwright-browsers` | `review` |
| LiteSandbox | `%LOCALAPPDATA%\LiteSandbox\logs` | `safe` 或 `review`，按最近写入时间判断 |
| Chrome cache | `%LOCALAPPDATA%\Google\Chrome\User Data\*\Cache` | `safe` |
| Chrome profile | `%LOCALAPPDATA%\Google\Chrome\User Data\*\Login Data` | `dangerous` |
| Edge cache | `%LOCALAPPDATA%\Microsoft\Edge\User Data\*\Cache` | `safe` |
| Docker | Docker build cache | `review` |
| Docker | Docker volumes | `dangerous` |
| WSL | `ext4.vhdx` | `system` |
| Ollama | `%USERPROFILE%\.ollama\models` | `review` |
| Hugging Face | `%USERPROFILE%\.cache\huggingface` | `review` |
| npm | `%APPDATA%\npm-cache` 或 `%LOCALAPPDATA%\npm-cache` | `safe` |
| Rust | `%USERPROFILE%\.cargo\registry` | `review` |
| Rustup | `%USERPROFILE%\.rustup` | `review` |
| Python | pip/uv cache | `safe` 或 `review` |
| 云盘根目录 | `BaiduSyncdisk`、`Nutstore`、`OneDrive` | `dangerous` |
| 云盘中高频项目 | 含 `node_modules`、`.playwright-browsers`、`output` | `review` |

## 规则文件格式草案

```yaml
id: litesandbox-logs
name: LiteSandbox audit logs
category: ai-logs
platform: windows
paths:
  - "%LOCALAPPDATA%\\LiteSandbox\\logs"
risk: safe
cleanup:
  method: quarantine
  min_age_days: 1
  skip_if_modified_within_minutes: 30
exclusions:
  - "*.lock"
reason: "Sandbox audit logs can grow unexpectedly when AI/browser automation performs high-frequency file access."
warnings:
  - "If logs are still being written, skip and report active process instead."
```

## 输出 JSON 草案

```json
{
  "scan_time": "2026-05-30T12:00:00+08:00",
  "volumes": [
    {
      "drive": "C:",
      "free_gb": 71.54,
      "used_gb": 128.47
    }
  ],
  "findings": [
    {
      "id": "litesandbox-logs",
      "path": "C:\\Users\\quzhi\\AppData\\Local\\LiteSandbox\\logs",
      "category": "ai-logs",
      "size_gb": 38.3,
      "risk": "safe",
      "confidence": "high",
      "action": "quarantine",
      "reason": "Known sandbox audit log directory with large JSONL files."
    }
  ]
}
```

## 报告格式草案

```text
Windows AI Space Report

Summary:
C: free 71.54GB
Total reclaimable safe space: 4.2GB
Total review-needed space: 18.7GB
Dangerous/system items: report only

Top Findings:
[SAFE] Browser caches: 1.3GB
[REVIEW] .gemini: 8.79GB
[REVIEW] Ollama models: 32GB
[SYSTEM] pagefile.sys: 10GB
[REVIEW] WSL ext4.vhdx: 18GB

Recommendations:
1. Run safe quarantine for browser/updater/temp caches.
2. Inspect .gemini before deleting.
3. Use native Ollama commands for model removal.
4. Do not run high-frequency AI projects inside sync folders.
```

## Safety Engine 设计

- 所有清理默认先 dry-run
- 所有路径先 canonicalize，确认没有 symlink/junction 越界
- 所有删除前输出路径、大小、规则 ID、理由
- 默认使用 quarantine，不直接删除
- quarantine 目录默认用 `F:\archives\aidisk-quarantine-YYYYMMDD-HHMMSS`
- 最近 30 分钟内还在写入的文件默认跳过
- 正在被进程锁定的文件默认跳过
- 未知目录默认只报告，不处理
- 包含 `token`、`credential`、`secret`、`.env`、`cookies`、`Login Data` 的路径默认禁止处理
- Docker volume、WSL VHDX、模型目录默认只做解释，不自动删
- 浏览器必须关闭后才能清 cache，否则只提示
- 云盘目录默认只给“移出高频项目”的建议，不直接清

## 清理策略

| 策略 | 说明 |
|---|---|
| `report-only` | 只报告，不清理 |
| `native-command` | 调用官方命令，比如 `docker system df`、`ollama list` |
| `quarantine` | 移动到隔离目录，可恢复 |
| `delete` | 直接删除，必须显式确认 |
| `guide` | 生成手动步骤，比如 WSL compact、pagefile 设置 |
| `skip-active` | 如果最近写入或锁定，则跳过 |

## Skill 工作流

用户触发：

```text
帮我看看 C 盘
C 盘又没空间了
AI 工具是不是占太多了
帮我清理 Claude/Codex/Gemini/opencode 缓存
Playwright/Docker/WSL/Ollama 占空间
```

Skill 执行流程：

```text
1. 运行 aidisk scan --json
2. 解析 findings
3. 先给用户解释 top 5 大头
4. 区分 safe/review/dangerous/system
5. 如用户要清理，运行 aidisk plan --safe-only
6. 展示 dry-run
7. 等用户确认
8. 执行 quarantine
9. 复查 C 盘空间
10. 给出防复发建议
```

## 建议仓库结构

```text
windows-ai-space-manager/
├── README.md
├── docs/
│   ├── product-plan.md
│   ├── architecture.md
│   ├── risk-model.md
│   ├── rules-spec.md
│   └── windows-ai-storage-map.md
├── aidisk/
│   ├── src/
│   │   ├── main.rs
│   │   ├── scanner/
│   │   ├── rules/
│   │   ├── planner/
│   │   ├── safety/
│   │   ├── executor/
│   │   └── reporter/
│   ├── rules/
│   │   ├── windows-system.yaml
│   │   ├── ai-agents.yaml
│   │   ├── browsers.yaml
│   │   ├── dev-caches.yaml
│   │   ├── docker.yaml
│   │   ├── wsl.yaml
│   │   └── sync-drives.yaml
│   └── tests/
│       ├── fixtures/
│       └── snapshots/
├── skills/
│   └── windows-ai-space-manager/
│       ├── SKILL.md
│       ├── scripts/
│       └── references/
└── examples/
    ├── scan-report.example.md
    ├── scan-output.example.json
    └── cleanup-plan.example.json
```

## 技术选型

| 部分 | 推荐 | 理由 |
|---|---|---|
| CLI | Rust | 单文件发布、性能好、路径处理安全 |
| 规则格式 | YAML | 易读，方便社区贡献规则 |
| 输出格式 | JSON + Markdown | agent 可读，人也可读 |
| Windows 路径处理 | Rust std + Windows API | 可靠识别 symlink/junction |
| 删除方式 | Quarantine 优先 | 降低误删风险 |
| Skill | opencode/Claude skill | 负责编排和解释 |
| 测试 | fixtures + snapshot tests | 防止规则误删 |

## 开发阶段规划

### Phase 0：项目准备

| 任务 | 产出 |
|---|---|
| 建立仓库结构 | 基础目录 |
| 写 `product-plan.md` | 当前计划书 |
| 写 `risk-model.md` | 风险分级 |
| 写 `rules-spec.md` | 规则格式 |
| 收集真实路径样本 | 初始规则库 |

### Phase 1：只读扫描器

| 任务 | 产出 |
|---|---|
| 实现目录大小扫描 | `aidisk scan` |
| 实现 JSON 输出 | `aidisk scan --json` |
| 实现 Markdown 报告 | `aidisk scan --markdown` |
| 实现规则匹配 | 初始 rules |
| 实现 top findings | 最大占用列表 |

### Phase 2：风险分类和计划生成

| 任务 | 产出 |
|---|---|
| 实现 `safe/review/dangerous/system` | 风险模型 |
| 实现 `aidisk plan` | 清理计划 |
| 实现 `explain` | 路径解释 |
| 实现 secret/config 阻断 | 安全边界 |
| 实现最近修改时间判断 | 防止清正在写的日志 |

### Phase 3：安全隔离清理

| 任务 | 产出 |
|---|---|
| 实现 `clean --dry-run` | 预演 |
| 实现 `clean --yes --quarantine` | 隔离移动 |
| 实现清理后复查 | reclaim report |
| 实现失败/锁文件跳过 | 稳定执行 |
| 实现恢复说明 | 可恢复路径 |

### Phase 4：AI 工具链专项 doctor

| 任务 | 产出 |
|---|---|
| WSL 报告 | `doctor --wsl` |
| Docker 报告 | `doctor --docker` |
| Ollama 报告 | `doctor --ollama` |
| Playwright 报告 | `doctor --playwright` |
| Hugging Face 报告 | `doctor --huggingface` |
| 同步盘高频项目检测 | `doctor --sync` |

### Phase 5：Skill 封装

| 任务 | 产出 |
|---|---|
| 写 `SKILL.md` | skill 主说明 |
| 写 references | 风险模型、路径地图 |
| 写 scripts wrapper | 调用 `aidisk` |
| 测试真实流程 | 从 scan 到 report |
| 打包 skill | 可安装版本 |

### Phase 6：开源准备

| 任务 | 产出 |
|---|---|
| 写 README | 使用说明 |
| 写安全声明 | 不默认删除敏感数据 |
| 写贡献规则 | 如何提交新路径规则 |
| 发布 GitHub release | Windows binary |
| 后续上 Scoop/Winget | 安装渠道 |

## MVP 成功标准

- 能在 60 秒内给出 C 盘主要占用报告
- 能识别至少 30 个 AI/dev/Windows 常见路径
- 能正确把 `LiteSandbox\logs` 标为高风险日志风暴来源
- 能正确把浏览器登录数据标为 `dangerous`
- 能正确把 Docker volume 标为 `dangerous`
- 能正确把 Ollama 模型标为 `review`
- 能正确把 browser cache 标为 `safe`
- 能生成 dry-run 清理计划
- 能将 safe 项移动到 quarantine
- 能清理后复查空间变化
- 在测试 fixtures 中不误删任何 token/config/profile 文件

## 第一版规则优先级

| 优先级 | 类别 | 原因 |
|---|---|---|
| P0 | LiteSandbox logs | 真实遇到，空间爆炸严重 |
| P0 | pagefile/hiberfil/swapfile | Windows C 盘大头 |
| P0 | Chrome/Edge cache | 常见且相对安全 |
| P0 | Playwright | AI agent 高频触发 |
| P0 | Claude/Codex/Gemini/opencode | 目标用户核心 |
| P1 | Docker Desktop | 大户且危险，需要解释 |
| P1 | WSL VHDX | 大户且不能乱动 |
| P1 | Ollama/Hugging Face | 模型大户 |
| P1 | npm/pip/uv/cargo | 开发缓存 |
| P2 | 云盘同步根检测 | 防止百度云、OneDrive、Nutstore 问题 |
| P2 | Czkawka/duplicate integration | 后续做去重 |

## 真实案例映射

| 真实问题 | 工具应该怎么处理 |
|---|---|
| `pagefile.sys` 32GB | 标为 `system`，解释现状，给设置建议 |
| `LiteSandbox` 38GB 日志 | 标为 `safe/review`，建议隔离，提示深信服审计 |
| AI 项目在百度云里 | 标为 `review`，建议移出同步根 |
| Playwright browser cache | 标为 `review`，建议配置独立路径 |
| opencode/codex 不想动 | 支持 protected paths |
| WSL 迁到 F 盘 | doctor 识别 VHDX 位置，不乱动 |
| F 盘 archive 太大 | 报告 quarantine 总量，建议确认后删除 |

## 配置文件草案

```yaml
policy:
  default_mode: dry-run
  quarantine_root: "F:\\archives"
  retention_days: 14
  skip_modified_within_minutes: 30
  follow_symlinks: false
  allow_direct_delete: false

protected_paths:
  - "%USERPROFILE%\\.codex"
  - "%USERPROFILE%\\.claude"
  - "%APPDATA%\\ai.opencode.desktop"
  - "%USERPROFILE%\\Nutstore"

categories:
  ai-logs: true
  browser-cache: true
  dev-cache: true
  docker: report-only
  wsl: report-only
  models: report-only
```

## 安全测试用例

- 不删除 `.env`
- 不删除 `auth.json`
- 不删除 Chrome `Login Data`
- 不删除 browser cookies
- 不删除 Docker volumes
- 不删除 WSL `ext4.vhdx`
- 不删除 `.codex` 根目录
- 不删除 `.claude` 根目录
- 不跟随 junction 到外部路径
- 不处理最近 30 分钟内写入的日志
- dry-run 输出和实际执行目标完全一致
- quarantine 后文件数量和大小可核对
- 清理失败时不会中断整个任务
- 输出 JSON 不包含敏感文件内容

## 未来路线

| 版本 | 重点 |
|---|---|
| v0.1 | 只读扫描和报告 |
| v0.2 | dry-run 计划 |
| v0.3 | quarantine 清理 |
| v0.4 | skill 集成 |
| v0.5 | Docker/WSL/Ollama doctor |
| v0.6 | 云盘高频项目风险检测 |
| v0.7 | 规则社区贡献 |
| v1.0 | 稳定发布，支持 Scoop/Winget |

## 为什么这个项目有价值

普通工具回答：

```text
这里有个 38GB 文件。
```

这个项目应该回答：

```text
这是 LiteSandbox 审计日志。
它在 17:56 到 18:20 之间由 node/Playwright 高频文件访问触发。
源路径位于百度云同步目录下的 AI 项目。
建议把项目移出同步根，并隔离旧日志。
当前日志已停止增长，风险解除。
```

这就是 AI 时代磁盘管理工具和传统清理工具的区别。

## 建议下一步

新开项目文件夹后，先创建这些文件：

```text
docs/product-plan.md
docs/architecture.md
docs/risk-model.md
docs/rules-spec.md
docs/windows-ai-storage-map.md
```

第一轮先不要写清理代码。先把“扫描什么、如何分级、什么绝对不能删”定死。这个项目最怕的不是功能少，而是误删。
