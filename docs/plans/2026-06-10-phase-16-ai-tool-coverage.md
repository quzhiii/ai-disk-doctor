# Phase 16: AI Tool Coverage, Detection Accuracy, and Cross-Platform Rules

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Extend a disk's core differentiator — AI tool disk usage detection — with new 2025-2026 tool coverage, better model file detection, and complete cross-platform rules.

**Architecture:** Add new YAML rule files for uncovered AI tools and model file detection. Rules remain YAML-only, no Rust core changes needed. Tests verify rule files load correctly, contain expected paths, and preserve risk classification. Linux/macOS path additions go into existing rule files alongside Windows paths with `platform:` tags.

**Tech Stack:** YAML rules, existing Rust rule loader and scanner, existing scan smoke tests.

---

## Milestone M1: New AI Tool Coverage (2025-2026)

### Task M1.1: 添加 GPU Runner 和本地模型推理缓存规则

**Files:**
- Create: `aidisk/rules/gpu-runners.yaml`

**内容：** 覆盖 LM Studio、llama.cpp、local.ai 等本地 GPU 推理工具的模型缓存和运行时数据。

Windows 路径示例：
- `%USERPROFILE%\.cache\lm-studio`
- `%USERPROFILE%\.cache\llama.cpp`
- `%APPDATA%\LM Studio`
- `%LOCALAPPDATA%\lm-studio`

风险：`review`，方法：`report-only`

```yaml
id: gpu-runner-caches
name: GPU inference runner caches and models
category: ai-model
platforms: [windows, linux, macos]
windows_paths:
  - '%USERPROFILE%\.cache\lm-studio'
  - '%USERPROFILE%\.cache\llama.cpp'
  - '%USERPROFILE%\.lm-studio'
  - '%APPDATA%\LM Studio'
linux_paths:
  - '~/.cache/lm-studio'
  - '~/.cache/llama.cpp'
  - '~/.config/lm-studio'
  - '~/lm-studio'
macos_paths:
  - '~/Library/Caches/lm-studio'
  - '~/.cache/lm-studio'
  - '~/.lm-studio'
risk: review
cleanup:
  method: report-only
reason: "GPU inference runner caches and downloaded models may be large but can be re-downloaded."
```

**Step 1: Write failing test**

In `aidisk/tests/scan_smoke.rs` add a test `loads_gpu_runner_rule_yaml` that asserts the file exists and loads correctly with expected fields.

Run: `cargo test loads_gpu_runner_rule_yaml` → EXPECTED FAIL (file not found)

**Step 2: Create rule file → GREEN**

**Step 3: Commit**

---

### Task M1.2: 添加 Claude Code / Codex CLI / Gemini CLI 规则

**Files:**
- Create: `aidisk/rules/ai-coding-agents.yaml`

**内容：** Claude Code (Anthropic)、Codex CLI (OpenAI)、Gemini CLI (Google)

这些是 2025 下半年以来最火的 AI 编程 CLI 工具，已在开发者中广泛使用。

Windows 路径：
- `%USERPROFILE%\.claude-code`
- `%USERPROFILE%\.codex`
- `%USERPROFILE%\.gemini-cli`
- `%APPDATA%\claude-code`

Linux/macOS 路径：
- `~/.claude-code`, `~/.config/claude-code`
- `~/.codex`, `~/.config/codex`
- `~/.gemini`, `~/.config/gemini-cli`

风险：`review`

**Step 1: TDD (RED → GREEN → commit)**

---

### Task M1.3: 添加 MCP Server 和 Skill 运行时缓存规则

**Files:**
- Create: `aidisk/rules/mcp-servers.yaml`

**内容：** MCP (Model Context Protocol) server 安装和运行时缓存。2025-2026 年 MCP 生态爆发，大量本地 MCP server 产生缓存。

路径：
- `%USERPROFILE%\.mcp`
- `%USERPROFILE%\.config\mcp`
- `%USERPROFILE%\.cache\mcp`
- `%APPDATA%\mcp`
- Linux/macOS: `~/.mcp`, `~/.config/mcp`, `~/.cache/mcp`

风险：`review`，部分可重建

**TDD: RED → GREEN → commit**

---

### Task M1.4: 添加 Roo Code 和新一代 AI IDE 缓存规则

**Files:**
- Create: `aidisk/rules/ai-ides-next.yaml`

**内容：** Roo Code (原 Roo Cline 进化版)、以及 Codeium/Windsurf 等的新增缓存位置

**TDD: RED → GREEN → commit**

---

### Task M1.5: M1 完成验证

- 运行 `cargo test --test scan_smoke` — 预期新增 3-4 个测试通过
- 运行 `cargo test --all` — 预期无回归
- 更新 CHANGELOG Unreleased 添加 M1 条目

---

## Milestone M2: Detection Accuracy

### Task M2.1: 添加模型文件格式专项规则

**Files:**
- Create: `aidisk/rules/model-files.yaml`

**内容：** 直接识别 AI 模型文件格式（不仅仅是目录级别）。使用 glob 模式匹配常见模型文件扩展名。

规则使用 `*.gguf`, `*.safetensors`, `*.onnx`, `*.mlx`, `*.bin` (large model weight files) 等模式。

风险：`safe`（可重新下载），方法：`report-only`

这是 a disk 独有的能力：其他磁盘工具不会告诉你"这里有 45GB 的模型文件可以安全重建"。

**TDD: RED → GREEN → commit**

---

### Task M2.2: 改进 scan 输出的 actionable 信息

**Files:**
- Modify: `aidisk/rules/models.yaml` (补充已有模型规则的实用建议)
- Modify: `docs/governance-manual.md` (添加模型清理指南)

**内容：**
- 为现有规则补充更具体的 `warnings` 和 `reason` 字段
- 在 governance manual 中添加"哪些 AI 缓存可以安全重建"的指导

**TDD: RED → GREEN → commit**

---

### Task M2.3: M2 完成验证

- 运行完整测试套件
- 更新 CHANGELOG

---

## Milestone M3: Linux/macOS 规则补全

### Task M3.1: 为现有 Windows-only 规则补充 Linux/macOS 路径

**Files:**
- Modify: `aidisk/rules/ai-agents.yaml`
- Modify: `aidisk/rules/ai-ides.yaml`
- Modify: `aidisk/rules/ai-caches.yaml`
- Modify: `aidisk/rules/ai-installed-apps.yaml`

这些规则当前标记 `platform: windows`，需要补上对应的 Linux/macOS 路径。

已经有一些规则支持跨平台（如 Ollama、HuggingFace、Docker），主要是 Agent/IDE/CLI/Cache 类规则需要补。

**TDD: 更新现有测试期望，RED → GREEN → commit**

---

### Task M3.2: 验证跨平台规则一致性

**Files:**
- 运行 `cargo test --test scan_smoke`（已有 `loads_cross_platform_rule_paths` 测试）
- 更新测试期望覆盖新增的跨平台路径
- 验证所有规则都能在无 Rust 代码变更的情况下正确加载

**TDD: RED → GREEN → commit**

---

### Task M3.3: M3 完成验证 + 文档更新

- 运行 `cargo test --all`
- 更新 `README` / `README.zh-CN` 的能力描述
- 更新 `CHANGELOG` Unreleased
- 更新 `docs/windows-ai-storage-map.md` 标记新覆盖范围

---

## Final Task: 全量验证 + Roadmap 更新

- `cargo test --all` → PASS
- 更新 `docs/execution-plan.md` Phase 16 条目
- Code review
- Push

---

## 非目标

- 不改 Rust 核心扫描器
- 不增加新的 CLI 命令
- 不改 `governance-event.json`
- 不增加 IM notifier adapter
- 不修改现有规则的风险等级

## 总计

- **12 个 Tasks**
- **3 个 Milestones**
- **7-8 个新 YAML 规则文件**
- **预计 8-10 个独立 commits**
