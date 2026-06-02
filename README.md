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

Playwright 项目级缓存规则现已支持 glob 路径，例如：`%USERPROFILE%\projects\**\.playwright-browsers`。

当前 `scan` 输出还包含：

- 卷空间摘要
- 按风险聚合的空间统计
- top findings
- 可回收 safe 空间估算

## 设计原则

- 默认保守：未知路径只报告，不处理
- 默认安全：清理能力后续只以 dry-run 和 quarantine 形式开放
- 规则驱动：路径识别、风险等级、策略建议均来自规则库
- Agent 友好：输出同时兼顾人类阅读和机器解析
