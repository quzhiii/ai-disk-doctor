# Architecture

## Runtime Flow

```text
User / AI Agent
    |
    v
aidisk CLI
    |
    +-- config
    +-- rules loader
    +-- scanner
    +-- classifier
    +-- reporter
```

## Current Scope

当前实现只覆盖：

- 规则加载
- 规则路径展开
- 路径存在性检查
- 大小统计
- JSON / Markdown 报告输出

尚未覆盖：

- 清理计划生成
- quarantine 执行
- doctor 专项分析
- 进程活跃写入检查

## Design Constraints

- 未知路径不处理
- 不跟随外部越界软链接的完整安全校验待后续补充
- 所有变更型命令在 Phase 2 之前不开放
