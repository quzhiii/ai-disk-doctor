# AI Disk Doctor 修正版战略与迭代路线图

> 仓库：`quzhiii/ai-disk-doctor`
> 基线版本：v1.6.0
> 日期：2026-07-14
> 用途：作为后续本地 Agent、Codex 或人工开发的主路线图

---

## 1. 本次调整解决的问题

原 roadmap 的战略定位基本成立，但版本顺序需要调整：

1. “已识别空间、可能释放空间、实际可执行空间”存在语义混用。
2. 已具备跨平台扫描和 CI，但 quarantine / restore 仍有 Windows 路径和跨文件系统假设。
3. README、Cargo、GitHub Release 和跨平台 artifact 分发未完全一致。
4. 现有规则主要依赖 `path + risk + cleanup.method`，不足以支撑模型引用关系、Agent 数据生命周期和企业策略。
5. GUI、Team / Fleet 和商业化分层推进偏早。
6. 后续创新应从“增加路径规则”转向“理解 AI 资产关系、恢复成本和生命周期”。

因此，推荐将后续版本调整为：

```text
v1.7  Trust Foundation + Trusted Distribution
v1.8  Model Asset Intelligence
v1.9  Agent Data Lifecycle
v1.10 Sentinel Beta
v2.0  Team Governance
```

---

## 2. 战略定位

### 2.1 一句话定位

**AI Disk Doctor 是面向 AI 开发者机器的、本地优先、可解释、可恢复的存储与数据治理工具。**

英文：

> Local-first storage and data governance for AI developer machines.

### 2.2 核心差异化

- **AI-aware**：理解 AI agent、AI IDE、AI CLI、模型缓存、MCP、runtime、Docker、WSL 和开发产物。
- **Relationship-aware**：逐步理解模型、revision、manifest、blob、项目和 session 的引用关系。
- **Risk-aware**：区分可重建缓存、公共模型、私有模型、会话数据、配置和凭据邻近数据。
- **Explainable**：每个建议说明证据、风险、恢复方式、置信度和预期释放空间。
- **Recoverable**：默认 dry-run，真实动作优先 quarantine，并保留完整 journal。
- **Local-first / Agent-ready**：默认不上传路径和内容，提供稳定 JSON / Markdown 输出。

### 2.3 近期不做

- 完整全盘智能分类清理；
- 默认自动删除；
- 默认读取 Prompt、源码、会话正文；
- 直接修改 Hugging Face、Ollama、Docker 等工具内部索引；
- 在核心执行层未稳定前开发完整 GUI；
- 在真实团队需求未验证前建设完整 SaaS；
- 把未知模型或自训练模型统一视为安全缓存。

---

# 3. v1.7 — Trust Foundation + Trusted Distribution

## 3.1 版本目标

将 v1.6.0 从“功能完整的开源 CLI”升级为：

> 指标可信、动作可信、恢复可信、发布可信的跨平台基础版本。

v1.7 不以增加支持工具数量为主要目标。

---

## 3.2 P0：修正空间指标语义

必须拆分以下指标：

| 指标 | 定义 |
|---|---|
| `observed_bytes` | 已识别并完成统计的空间 |
| `potential_bytes` | 可能释放，但仍需人工或官方工具确认 |
| `actionable_bytes` | 当前 plan 中存在明确可执行动作的空间 |
| `quarantine_bytes` | 可由 aidisk 自身移入 quarantine 的空间 |
| `official_cleanup_bytes` | 应通过官方 CLI 处理的空间 |
| `report_only_bytes` | 仅展示，不应计入可执行空间 |
| `partial_bytes` | 扫描不完整，只能作为下限估计 |

### 开发任务

- [x] 修改 scanner summary，不再把所有 `risk: safe` 直接计入 reclaimable。
- [x] 修改 planner summary，只统计真实可执行候选。
- [x] `report-only` 不计入 `actionable_bytes`。
- [x] `guide` 不计入 `quarantine_bytes`。
- [x] partial finding 默认不能进入真实 clean。
- [x] 通用 GGUF、SafeTensors、ONNX、MLX 规则改为 `review + report-only`，或使用新版数据语义。
- [x] Dashboard、Markdown、Text、JSON 使用统一指标定义。
- [x] 增加 JSON schema version。
- [x] 增加 regression fixtures：
  - report-only safe item；
  - guide item；
  - partial finding；
  - custom model；
  - official cache；
  - quarantine candidate。

### 验收标准

- `actionable_bytes` 与实际 plan 可执行总量一致。
- 自训练模型和来源未知模型不显示为“安全可清”。
- partial finding 无法进入真实 clean。
- Dashboard 不再使用一个“可安全回收空间”覆盖所有状态。

---

## 3.3 P0：Transactional Quarantine

### 目标

将 quarantine 从简单文件移动升级为可审计、可恢复、可处理中断的事务流程。

### 建议状态机

```text
planned
→ copying
→ copied
→ verified
→ source-removing
→ quarantined
→ restored
```

异常状态：

```text
failed
partial-copy
verification-failed
source-remove-failed
restore-conflict
restore-failed
```

### 开发任务

- [x] 使用 `PathBuf::join` 生成跨平台 destination。
- [x] 增加 quarantine root containment 检查。
- [x] 阻断 source / destination 循环嵌套。
- [x] 根据文件系统错误或设备信息判断跨文件系统。
- [x] 同文件系统优先 rename。
- [x] rename 失败时执行 copy → verify → remove fallback。
- [x] verification 至少检查文件数、目录数、总字节数；关键场景可选 hash。
- [x] 动作开始前写入 journal。
- [x] 每个阶段更新 journal 状态。
- [x] 支持中断后的检查、resume 或明确失败恢复。
- [x] restore 支持跨文件系统 copy-back。
- [x] restore 冲突默认不覆盖。
- [x] quarantine index 增加 schema version。
- [x] 增加失败注入测试：
  - 磁盘空间不足；
  - 文件被占用；
  - 权限变化；
  - 复制中断；
  - 校验失败；
  - 删除源失败；
  - restore 目标已存在。

### 验收标准

- Windows、Linux、macOS 均能完成基础 quarantine / restore。
- 中断不会造成“源和目标均不完整且没有状态记录”。
- 所有真实动作都能追溯到 plan、rule、policy 和 journal。
- 同盘、跨盘和 restore conflict 均有测试。

---

## 3.4 P0：Trusted Distribution

### 开发任务

- [x] 修正 Cargo.toml description，改为跨平台定位。
- [x] 建立 release matrix：
  - Windows x86_64；
  - Windows ARM64；
  - macOS x86_64；
  - macOS ARM64；
  - Linux x86_64；
  - Linux ARM64。
- [x] 统一 artifact 命名。
- [x] 生成 SHA-256 checksums。
- [x] 生成 SBOM。
- [x] 增加 release provenance。
- [x] 同步 README、Cargo.toml、CHANGELOG 和 Release Notes；GitHub Latest Release 在打 tag 后由 workflow 产物补齐。
- [x] 对 release artifact 执行 smoke test。
- [x] 增加 Homebrew tap 草案。
- [x] 增加 winget manifest 草案。
- [x] 评估 crates.io 发布。
- [x] README 增加安装、升级、验证、卸载和数据目录说明。
- [ ] 后续评估 Windows signing 和 macOS notarization。

### 验收标准

- 用户无需 Rust toolchain 即可安装。
- GitHub Latest Release 与 Cargo.toml 版本一致。
- 每个 artifact 均可验证 checksum。
- 文档不再将“跨平台支持”和“Windows-only 安装”混在一起。

---

## 3.5 P0：Rule Schema v2

### 推荐结构

```yaml
schema_version: 2

id: huggingface-cache
category: ai-model-cache

detector:
  paths: []
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
```

### 必须表达的维度

- `data_kind`
- `recoverability`
- `sensitivity`
- `liveness`
- `risk`
- `confidence`
- `evidence`
- `action.type`
- `action.adapter`
- `content_access`

### 开发任务

- [x] 定义 Rule Schema v2。
- [x] 保持 v1 规则兼容，并在加载时提供规范化 migration metadata。
- [x] 将 detector、decision、action 分离。
- [x] scan report 展示规则来源、版本和 digest。
- [x] 为官方 adapter 预留接口。
- [x] 内置规则增加 schema validation。
- [x] PR CI 增加 rule lint。

### 验收标准

- 能区分公共模型缓存、未知模型和自训练模型。
- action 不再由 risk 自动推导。
- 每条规则可以说明判断依据。
- v1 规则仍可加载或可靠迁移。

---

## 3.6 v1.7 明确不做

- 完整 GUI；
- Team dashboard；
- 复杂规则签名注册中心；
- 自动清理未知模型；
- 读取 Agent 会话正文；
- 默认全盘深度 glob；
- 大规模新增未经验证的路径规则。

## 3.7 v1.7 Release Gate

以下条件全部满足后才能发布：

- [ ] 空间指标语义测试通过；
- [ ] Transactional Quarantine 跨平台测试通过；
- [ ] failure-injection 和 restore 测试通过；
- [ ] partial finding 不能进入真实 clean；
- [ ] 主流平台 artifact 可稳定发布；
- [ ] checksum 和 SBOM 可生成；
- [ ] README、Cargo、CHANGELOG、Release 一致；
- [x] Rule Schema v2 最小版本落地；
- [ ] 没有已知规则会把私有模型标记为安全可清。

---

# 4. v1.8 — Model Asset Intelligence

## 4.1 版本目标

从“识别模型文件和缓存目录”升级为：

> 理解模型资产、缓存布局、引用关系、实际物理占用和恢复成本。

## 4.2 Model Asset Inventory

### 当前最小实现

`aidisk models inventory` 提供只读的模型资产 inventory 基础：支持显式 root、Ollama、Hugging Face 和通用模型文件识别，输出逻辑大小、独占/共享物理大小、格式、管理工具、revision、状态、可恢复性、疑似自定义模型、reclaim confidence 和最小 provenance graph。当前版本不读取模型内容，不调用外部工具，不修改官方索引，未知模型始终 `report-only`。

当前实现还会在元数据文件较小且格式有效时解析 Hugging Face `refs`、snapshot/blob 的本地关系，以及 Ollama manifest/blob 的本地关系。解析失败、索引缺失或关系不完整时保持保守状态，不将资产升级为可回收对象。

当前还提供 `aidisk models adapters` 能力报告：默认仅检查本地 refs/manifest 元数据。显式指定 `--probe-official-cli` 后，只调用 `hf` / `ollama` 的 version/help 命令，并受超时、输出上限约束。显式指定 `--run-official-dry-run` 后，只运行 allowlist 中的非修改命令：Hugging Face 为帮助确认后的 `hf cache prune --dry-run --cache-dir <root>`，Ollama 为只读 `ollama ls`；不执行真实清理，不修改官方索引。Hugging Face dry-run 输出会归一化为 report-only `official_cleanup_plan` 并附带 manual-redownload rollback 元数据，Ollama list 输出仅作为证据，不生成 cleanup candidate。

统一展示：

- 模型逻辑名称；
- 模型格式；
- 管理工具；
- revision；
- manifest；
- blob；
- snapshot；
- 来源；
- 最近使用时间；
- 逻辑大小；
- 独占物理大小；
- 共享大小；
- 是否可重新下载；
- 是否疑似自定义模型。

## 4.3 Storage Provenance Graph

最小节点：

- tool
- model
- revision
- manifest
- blob
- snapshot
- local file
- source

最小关系：

- managed-by
- references
- shares
- downloaded-from
- belongs-to
- supersedes

第一阶段无需引入图数据库，可使用内存图和 JSON 输出。

## 4.4 官方 Adapter

优先支持：

1. Hugging Face；
2. Ollama；
3. LM Studio；
4. 后续扩展 llama.cpp、MLX。

Adapter 职责：

- 检查官方索引；
- 列出可 prune 对象；
- 调用官方 dry-run；
- 解析结果；
- 生成 aidisk 统一 plan；
- 明确是否支持 rollback。

原则：

- 官方工具能安全判断时优先使用官方工具；
- aidisk 负责跨工具汇总、解释和 policy；
- aidisk 不直接修改官方内部索引。

## 4.5 状态识别

- [x] referenced（基于本地 refs / manifest 的基础识别）；
- [x] stale（基于访问/修改时间元数据的基础标记）；
- [x] detached revision（Hugging Face ref 缺失时的基础识别）；
- [x] orphan blob（仅在本地索引成功解析后识别）；
- [x] incomplete download（基于文件名标记的基础识别）；
- [x] duplicate logical model（基于逻辑名称、revision 和物理路径的基础识别）；
- [x] shared physical blob 基础去重；
- [x] unknown custom model 基础识别；
- [ ] external-drive candidate。

Adapter 基础能力：

- [x] Hugging Face / Ollama 本地 metadata-only dry-run capability report；
- [x] 官方 CLI version/help capability probe（显式 opt-in、超时和输出上限）；
- [x] 官方 dry-run invocation（显式 opt-in、allowlist、Hugging Face prune dry-run；Ollama read-only list）；
- [x] 统一 official cleanup plan（report-only foundation；Hugging Face dry-run normalized，Ollama evidence-only）；
- [x] rollback capability metadata（report-only；Hugging Face manual redownload，Ollama not-applicable）；

## 4.6 Reclaim Confidence

每个候选项输出可解释证据，例如：

```text
Reclaim confidence: 91/100

Positive evidence:
- 位于官方缓存目录
- 未被任何 manifest 引用
- 120 天未访问
- 官方 CLI 确认可 prune
- 原始来源可重新下载

Risk evidence:
- 预计重新下载 18 GB
- 当前网络不可用
```

评分只用于解释，最终动作仍由 policy 决定。

## 4.7 Cost-aware Eviction

建议同时展示：

- 可释放空间；
- 重新下载大小；
- 预计恢复时间；
- 是否需要网络；
- 是否可离线恢复；
- 是否占用共享 blob；
- 是否适合外置盘迁移。

概念模型：

```text
reclaim utility
= expected freed space
- recovery cost
- data loss risk
- interruption cost
```

初期可使用规则化分级，不必马上实现复杂算法。

## 4.8 Model Cold Storage

建议命令：

```bash
aidisk migrate ollama --to <external-path>
aidisk migrate huggingface --to <external-path>
```

流程：

1. 检查目标磁盘；
2. 检查相关进程；
3. 生成迁移 plan；
4. 复制；
5. 验证；
6. 更新环境变量或配置；
7. health check；
8. 保留 rollback；
9. 用户确认后处理原目录。

## 4.9 验收标准

- shared blob 不重复统计。
- 能区分逻辑大小与物理大小。
- 至少支持 Hugging Face 和 Ollama 的主要引用关系。
- 未知模型默认只报告。
- 官方 dry-run 可转换为统一 plan。
- 至少完成一个工具的 Cold Storage 端到端验证。

---

# 5. v1.9 — Agent Data Lifecycle

## 5.1 版本目标

从“按工具扫描 Agent 目录”升级为：

> 按项目理解 Agent 产生的 session、snapshot、history、log、cache 和恢复数据。

## 5.2 隐私边界

默认允许读取：

- 路径；
- 文件名；
- 大小；
- 时间元数据；
- 目录层级；
- 官方 index / manifest；
- 项目路径引用；
- 文件类型。

默认禁止读取：

- Prompt 正文；
- 对话正文；
- 源码内容；
- shell command 正文；
- token；
- credential；
- 用户文档正文。

未来若增加内容级检查，必须单独 opt-in，并形成独立隐私设计。

## 5.3 Agent Project Aggregation

推荐输出：

```text
Project: easy-paper
Repository: D:\Projects\easy-paper
Project status: active

Claude sessions: 1.2 GB
Codex snapshots: 480 MB
Cursor workspace state: 320 MB
Test artifacts: 2.8 GB

Potential reclaim: 1.6 GB
Protected recovery data: 3.2 GB
```

## 5.4 数据分类

- session；
- transcript；
- prompt history；
- file snapshot；
- shell snapshot；
- debug log；
- test artifact；
- runtime cache；
- installer；
- workspace state；
- orphan project record；
- recovery data；
- credential-adjacent data。

## 5.5 生命周期动作

优先支持：

- report；
- ignore；
- archive；
- compress；
- export index；
- quarantine cache；
- move to project archive。

默认不支持：

- 自动删除 transcript；
- 自动删除 recovery snapshot；
- 自动处理 credential-adjacent 目录。

## 5.6 Orphan Detection

孤立状态不能只由修改时间决定，应结合：

- 项目路径是否存在；
- Git 仓库是否仍可定位；
- Agent index 是否仍引用；
- session 是否承担恢复用途；
- workspace 是否迁移；
- 是否存在未归档 snapshot；
- 用户是否明确标记保留。

## 5.7 验收标准

- 至少支持 Claude、Codex、Cursor 三类工具的项目级聚合。
- 默认不读取会话和源码正文。
- 项目删除后的遗留数据可以识别，但默认只报告。
- recovery data 不进入自动清理。
- 每项建议包含隐私风险和恢复说明。

---

# 6. v1.10 — Sentinel Beta

## 6.1 版本目标

让非 CLI 用户持续感知 AI 存储增长，但不复制一套清理逻辑。

## 6.2 产品顺序

1. background daemon；
2. tray entry；
3. 打开本地 dashboard；
4. 通知；
5. plan / restore UI；
6. 后续再评估完整桌面应用。

## 6.3 核心能力

- 定期 targeted scan；
- 磁盘空间阈值提醒；
- AI footprint 增长提醒；
- 历史趋势；
- 查看最近 plan；
- 查看 quarantine 状态；
- 打开 restore；
- 暂停监控；
- 配置扫描预算；
- 隐私设置。

## 6.4 Causal Growth Timeline

在 diff / anomaly 基础上逐步回答“为什么变大”。

第一阶段：

- 基于定期 snapshot；
- 根据新增路径、manifest、blob 和工具状态归因。

后续实验：

- Windows USN Journal；
- macOS FSEvents；
- Linux inotify / fanotify。

默认不读取文件正文。

## 6.5 验收标准

- Sentinel 只调用同一 Rust core。
- GUI / Tray 不自行实现删除逻辑。
- 提醒必须展示触发证据。
- 默认不上传路径和文件信息。
- 扫描范围、频率和预算可配置。

---

# 7. v2.0 — Team Governance

## 7.1 进入条件

Team / Fleet 不按时间自动启动，应满足以下大部分条件：

- [ ] 至少 20 个持续使用的个人用户；
- [ ] 至少 3 个团队明确提出集中策略需求；
- [ ] Windows、macOS、Linux 真实清理稳定；
- [ ] restore 和 failure-injection 达到目标可靠性；
- [ ] 已形成规则误判反馈闭环；
- [ ] 已明确哪些指标可以上传；
- [ ] 已明确哪些数据不得离开设备。

## 7.2 第一阶段产品

- 本地生成脱敏 health report；
- signed report bundle；
- policy bundle；
- webhook；
- Feishu / Slack / GitHub Issue / email adapter；
- 设备健康摘要；
- AI cache 增长趋势；
- 治理事件审计；
- 规则版本锁；
- MDM / Jamf / Intune 部署文档。

默认不上传：

- 具体路径；
- 文件名；
- Prompt；
- 源码；
- session 内容；
- credential 信息。

## 7.3 后续再评估

- 中央 dashboard；
- SaaS；
- on-prem console；
- 策略下发；
- 企业审批；
- signed rules registry；
- SLA 和计费。

---

# 8. Rule Pack 演进路线

原计划中的 Signed Rules Registry 保留，但拆成三个阶段。

## 8.1 阶段 A：Rule Pack Manifest

增加：

- pack ID；
- version；
- publisher；
- schema version；
- supported aidisk versions；
- source commit；
- content digest；
- capabilities；
- action permissions；
- update channel。

## 8.2 阶段 B：可审阅更新

建议命令：

```bash
aidisk rules status
aidisk rules update --check
aidisk rules diff
aidisk rules pin <commit-or-version>
aidisk rules rollback
aidisk rules verify
```

要求：

- 不静默复用无法识别版本的缓存；
- 更新必须显式发生；
- 生效规则来源可见；
- 规则变更可以 diff；
- 可以回退；
- scan report 记录规则版本和 digest。

## 8.3 阶段 C：签名与企业审批

后续增加：

- signed rule pack；
- trusted publisher；
- enterprise allowlist；
- offline registry；
- approval workflow；
- policy pack；
- 执行动作权限边界。

在 manifest、lock 和 diff 未稳定前，不优先建设复杂注册中心。

---

# 9. 跨版本架构原则

## 9.1 Detector 与 Policy 分离

Detector 输出事实：

- 发现了什么；
- 为什么匹配；
- 大小和时间；
- 引用关系；
- 当前状态；
- 置信度。

Policy 决定：

- 是否展示；
- 是否阻断；
- 是否进入 plan；
- 是否允许 quarantine；
- 是否调用官方工具；
- 是否需要人工确认。

## 9.2 所有动作必须可解释

每个 candidate 至少包含：

```json
{
  "decision": "quarantine",
  "evidence": [
    "matched official cache layout",
    "not modified for 83 days",
    "no active process detected"
  ],
  "rule_pack": "official@2.0.0",
  "rule_digest": "sha256:...",
  "confidence": 0.96,
  "expected_freed_bytes": 123456,
  "rollback": "available"
}
```

## 9.3 官方工具优先

当第三方工具拥有官方索引和清理命令时：

- aidisk 负责统一扫描、解释、计划和 policy；
- 优先调用官方 dry-run；
- 由官方工具执行其内部清理；
- aidisk 记录结果；
- 不直接修改官方内部索引。

## 9.4 默认保守

- 未知数据只报告；
- partial 数据不执行；
- active 数据不执行；
- credential-adjacent 数据阻断；
- private/custom model 不自动处理；
- 默认不读取内容；
- 默认不上传数据。

## 9.5 CLI Core 是唯一执行源

HTML、Tray、GUI、Agent Skill 和 Team Agent 均调用同一 core。

禁止：

- GUI 自行删除；
- wrapper 复制核心判断；
- 不同平台使用不一致风险模型；
- Team Agent 绕过本地 policy。

---

# 10. 测试与质量体系

## 10.1 测试层级

### Unit Tests

- rule parsing；
- metric calculation；
- policy decision；
- path containment；
- state transition；
- adapter parsing。

### Fixture Tests

- Hugging Face cache；
- Ollama blobs；
- Claude sessions；
- Codex snapshots；
- Cursor workspace；
- partial directories；
- custom model；
- sensitive markers。

### Integration Tests

- scan → plan → clean → restore；
- cross-filesystem quarantine；
- official adapter dry-run；
- rule update / diff / rollback。

### Failure Injection

- 磁盘满；
- permission denied；
- locked file；
- process interruption；
- incomplete copy；
- hash mismatch；
- restore conflict；
- damaged journal。

## 10.2 核心质量指标

### 安全

- 自定义模型、配置、session、credential 进入自动清理：0；
- partial finding 进入真实执行：0；
- 所有真实动作有 journal：100%；
- restore 可追溯率：100%。

### 准确性

- 预测释放空间与实际释放空间误差目标：<5%；
- shared blob 重复统计：0；
- report-only 计入 actionable：0；
- 官方索引解析与官方结果一致。

### 性能

- 默认 targeted scan 有明确预算；
- 深度 home glob 必须 opt-in；
- 扫描支持取消；
- 每条规则支持 max depth、timeout 或预算；
- 大目录不阻塞整次报告。

---

# 11. 创新能力优先级

| 能力 | 用户价值 | 差异化 | 实现成本 | 优先级 |
|---|---:|---:|---:|---:|
| 空间指标语义修正 | 极高 | 中 | 低 | P0 |
| Transactional Quarantine | 极高 | 高 | 中高 | P0 |
| 跨平台可信分发 | 高 | 中 | 中 | P0 |
| Rule Schema v2 | 高 | 高 | 中 | P0 |
| Model Asset Inventory | 极高 | 高 | 中 | P1 |
| Storage Provenance Graph | 高 | 极高 | 中高 | P1 |
| 官方 Adapter | 极高 | 高 | 中 | P1 |
| Model Cold Storage | 高 | 高 | 中高 | P1 |
| Agent Project Aggregation | 高 | 高 | 中高 | P1 |
| Reclaim Confidence | 中高 | 高 | 中 | P1 |
| Causal Growth Timeline | 中高 | 极高 | 高 | P2 |
| Tray Sentinel | 中高 | 中 | 中高 | P2 |
| Team / Fleet | 未验证 | 中 | 极高 | 暂缓 |

---

# 12. 近期执行顺序

## Sprint 1：语义修正

1. 重构 summary 字段；
2. 修正 model-files 风险；
3. planner 只统计真实可执行空间；
4. 修改 Dashboard；
5. 增加 regression tests；
6. 更新 JSON schema 文档。

完成定义：

> 报告中的“可执行空间”与实际 plan 一致。

## Sprint 2：Quarantine 跨平台基础

1. PathBuf destination；
2. containment；
3. 同盘 rename；
4. 跨盘 copy / verify / remove；
5. journal state；
6. restore copy-back；
7. Linux / macOS fixtures。

完成定义：

> 三个平台均可以完成基础 quarantine / restore。

## Sprint 3：Release Foundation

1. artifact matrix；
2. checksum；
3. SBOM；
4. smoke test；
5. GitHub Release 同步；
6. README 安装文档；
7. Homebrew / winget 草案。

完成定义：

> 用户无需 Rust 即可安装并验证 artifact。

## Sprint 4：Rule Schema v2

1. schema proposal；
2. compatibility loader；
3. migration；
4. detector / decision / action；
5. validation；
6. rule lint。

完成定义：

> 可以安全进入 Model Asset Intelligence 开发。

---

# 13. 建议的 GitHub Epic

## Epic 1：v1.7 Metric Semantics

- Scanner summary v2
- Planner actionable calculation
- Dashboard metric migration
- Model file risk correction
- JSON schema versioning
- Regression fixtures

## Epic 2：Transactional Quarantine

- Cross-platform path handling
- Destination containment
- Transaction journal
- Cross-filesystem verification
- Resume and rollback
- Restore conflict handling
- Failure-injection tests

## Epic 3：Trusted Distribution

- Release build matrix
- Checksums
- SBOM
- Provenance
- Artifact smoke tests
- Homebrew
- winget
- Documentation consistency

## Epic 4：Rule Schema v2

- Schema specification
- Compatibility loader
- Rule migration
- Evidence model
- Adapter interface
- Rule lint CI

## Epic 5：Model Asset Intelligence

- Hugging Face adapter
- Ollama adapter
- Model inventory
- Shared blob accounting
- Orphan / incomplete detection
- Cold storage migration

## Epic 6：Agent Data Lifecycle

- Agent project resolver
- Metadata-only classifier
- Orphan detection
- Archive workflow
- Privacy report
- Project-level dashboard

---

# 14. 对本地 Agent 的执行约束

1. 不跨版本提前建设 GUI、SaaS 或 Team console。
2. 不为了覆盖率批量添加未经验证的路径。
3. 所有真实文件操作必须先有 plan。
4. 所有真实动作必须有 journal。
5. 所有新规则必须有 fixture。
6. 所有平台相关代码必须有对应测试。
7. 不读取 Prompt、源码和 session 正文。
8. 不把未知模型标为 safe。
9. 不直接修改第三方工具内部索引。
10. 不改变现有 CLI public contract，除非提供 schema version 和迁移说明。
11. 每完成一个 Epic，先更新测试、文档和 CHANGELOG。
12. 任一 Release Gate 未满足时，不发布正式版本。

---

# 15. 第一阶段完成定义

以下结果全部实现后，可以认为 AI Disk Doctor 完成“可信基础阶段”：

- 用户能区分已识别空间、可能释放空间和实际可执行空间；
- 所有自动动作都能解释原因；
- 所有 quarantine 都有可恢复 journal；
- Windows、Linux、macOS 核心行为一致；
- 用户无需 Rust toolchain 即可安装；
- 规则能够表达可恢复性、敏感性、活跃性和动作类型；
- 未知模型、会话和恢复数据不会被自动清理；
- 后续模型资产治理可以建立在稳定 schema 和执行层之上。

在此之前，不建议将项目扩展为完整 GUI 或团队 SaaS。
