# Windows AI Space Manager

`windows-ai-space-manager` 是一个面向 Windows 的 AI 时代磁盘空间诊断与治理工具。

当前仓库目标：

- 建立 `aidisk` CLI 的最小可运行版本
- 用规则库识别 AI / 浏览器 / 开发工具相关空间占用目录
- 默认只做只读扫描和报告，不做直接删除

## 仓库结构

```text
.
├── README.md
├── docs/
├── aidisk/
├── skills/
└── examples/
```

## 当前阶段

当前已进入 Phase 0 + Phase 1 的落地：

- 基础文档归档
- Rust CLI 初始化
- 规则加载能力
- 只读 `scan` 命令最小闭环
- `plan` 只读 dry-run 骨架
- `plan` 安全边界首版
- `clean` dry-run / quarantine 骨架

## 快速开始

要求：

- Rust 1.78+

运行：

```powershell
cd aidisk
cargo run -- scan --json
```

或输出 Markdown：

```powershell
cargo run -- scan --markdown
```

按分类扫描：

```powershell
cargo run -- scan --category browser-cache --json
```

生成只读 dry-run 计划：

```powershell
cargo run -- plan --safe-only --json
```

或保留全部风险等级，但跳过最近仍在变化的路径：

```powershell
cargo run -- plan --json --skip-modified-within-minutes 30
```

生成清理预演：

```powershell
cargo run -- clean --dry-run --safe-only --markdown
```

生成隔离移动计划：

```powershell
cargo run -- clean --dry-run --safe-only --json --quarantine-root "F:\archives"
```

Playwright 项目级缓存规则现已支持 glob 路径，例如：`%USERPROFILE%\projects\**\.playwright-browsers`。

当前 `scan` 输出还包含：

- 卷空间摘要
- 按风险聚合的空间统计
- top findings
- 可回收 safe 空间估算

当前 `plan` 输出还包含：

- 候选项按 action 分组
- 最近修改时间过滤
- 敏感路径阻断
- skipped 列表与原因说明

当前 `clean` 输出还包含：

- dry-run 动作清单
- action groups
- skipped 继承说明
- quarantine 目标路径预演
- 执行结果报告
- 恢复索引和执行日志

策略文件位于：`aidisk/config/policy.yaml`

真实执行隔离移动：

```powershell
cargo run -- clean --yes --safe-only --quarantine-root "F:\archives"
```

从 quarantine index 预演恢复：

```powershell
cargo run -- restore --dry-run --json --index "F:\archives\.aidisk\quarantine-index-YYYYMMDD-HHMMSS.json"
```

执行恢复：

```powershell
cargo run -- restore --yes --json --index "F:\archives\.aidisk\quarantine-index-YYYYMMDD-HHMMSS.json"
```

恢复冲突策略：

- 如果恢复目标路径已存在，当前实现会跳过并标记为 `skipped-conflict`
- 不会覆盖现有目标，也不会自动删除冲突路径

专项诊断：

```powershell
cargo run -- doctor --docker --json
cargo run -- doctor --wsl --ollama --markdown
cargo run -- doctor --playwright --huggingface --markdown
cargo run -- doctor --markdown
```

当前 `doctor` 输出包含：

- 按主题汇总的专项发现
- 空结果与未命中路径的解释
- 贴近执行的建议清单
- 不带主题参数时的完整诊断集合

历史对比：

```powershell
cargo run -- scan --json
cargo run -- scan --json
cargo run -- diff --latest --markdown
```

指定两个 snapshot 对比：

```powershell
cargo run -- diff --before ..\examples\diff-before.example.json --after ..\examples\diff-after.example.json --markdown
```

当前 `diff` 输出包含：

- 两次 scan snapshot 之间的 grew / shrunk / appeared / disappeared
- `scan` 会自动保存 snapshot 到 `aidisk/.aidisk/reports/scan-*.json`
- `diff --latest` 会自动对比最近两个 snapshot
- `exists=false` 占位路径不会再被误报为新增
- 可直接用于回答“最近是谁长大了”

真实场景测试样本位于：`aidisk/tests/fixtures/windows-user`

## 设计原则

- 默认保守：未知路径只报告，不处理
- 默认安全：清理能力后续只以 dry-run 和 quarantine 形式开放
- 规则驱动：路径识别、风险等级、策略建议均来自规则库
- Agent 友好：输出同时兼顾人类阅读和机器解析
