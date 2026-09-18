# 实践任务 0005：有界并行工具调度与结果顺序

本文件是 CodebaseAgent 并行调度切片的实验入口：规定依赖、范围、实施顺序和退出标准。Shared/Exclusive 契约及 P1—P10 场景见[并行调度验收材料](materials/0005-parallel-scheduling-acceptance.md)。

## 接手契约

| 字段 | 当前值 |
| --- | --- |
| 状态 | `prepared / blocked by practice 0004` |
| 目标工程 | `D:\Program\AgentLab\CodebaseAgent` |
| 事实来源 | 实践 0004 完成后的 CodebaseAgent 行为与测试、[课程 0018](../lessons/0018-parallel-tool-admission-and-result-order.md)、当前 Codex 源码与测试 |
| 进入条件 | 实践 0004 已完成并收口；用户明确开始本实践；更新 CodebaseAgent 的 `docs/STATUS.md` |
| 退出条件 | 并发上限、工具资格、结果配对、历史顺序、取消与失败语义均有确定性测试 |
| 退出后安排 | 阶段复盘，再决定是否恢复 Subagent 生命周期专题 |

本实践不会绕过 0004，也不会仅因文档存在而自动改变 AgentLab 状态。

## 实验目标

CodebaseAgent 已能在一个模型响应中接收多个 Function Call，但仍按顺序执行并拒绝 `max_parallel_tool_calls > 1`。本实践要把它扩展为有界并行，同时保持以下边界：

- 模型或上层工作流表达业务依赖；
- Harness 依据工具声明和并发上限调度，不分析参数猜冲突；
- 工具保证自己声明的并发能力；
- 完成事件可以乱序，模型历史保持稳定；
- 下一次采样等待整批收口。

## 复用范围

保持 Call 批次先整体记录、`call_id` 唯一配对、只读目标仓库、有界输出、普通工具失败不取消兄弟调用，以及实践 0004 已确定的取消和恢复语义。

本实践不构建资源冲突分析、通用 DAG、分布式队列、跨 Turn/Agent 调度或并行 Subagent，也不使用真实延迟和睡眠作为并发正确性的证据。

## 开工前必须写入本地行为契约

实现前先在 CodebaseAgent 中确定：

1. 工具注册项的并行能力，第一版建议 `Shared` 与 `Exclusive`；
2. `max_parallel_tool_calls` 的上限语义及值为 1 时的兼容行为；
3. Exclusive 的公平性规则；
4. 完成事件顺序与历史 Output 顺序的区别；
5. 批次屏障、普通失败、Turn 取消和调度器致命错误的边界。

第一版不需要资源键或路径读写集合。出现真实的细粒度互斥需求后，再单独扩展能力模型。

## 实施顺序

### M0：契约与失败测试

- 固定模型、Harness、工具三层职责；
- 固定 Shared/Exclusive、稳定历史顺序、批次屏障与公平性；
- 用 gate 建立真实重叠、慢 A/快 B、并发上限和取消部分完成的失败测试。

### M1：有界 Shared 调度

- 受控放开 `max_parallel_tool_calls > 1`；
- 限制同时运行的调用数；
- 收集完整批次后按 Call 顺序提交 Output；
- 保持上限为 1 时的串行行为。

### M2：工具能力门禁

- 将 Shared/Exclusive 能力放入工具注册元数据；
- 实现共享与独占准入及已选定的公平性；
- 让同名和异名工具调用遵守同一规则；
- 不在调度器中推断业务依赖。

### M3：取消、恢复和观察面

- 复用实践 0004 的取消源和部分完成契约；
- 唯一收口已完成、运行中和等待许可的调用；
- 分别记录真实完成事件和稳定历史位置；
- 验证恢复不重放批次、并发许可不泄漏；
- 更新 CodebaseAgent 的状态、能力文档和测试索引。

## 验收入口

完整场景和事件证据见[并行调度验收材料](materials/0005-parallel-scheduling-acceptance.md)。退出本实践至少需要证明：

- 两个 Shared 调用可以真实重叠，且不会超过配置上限；
- Exclusive 调用与其他调用不重叠且不会永久饥饿；
- 慢 A、快 B 可以按 B/A 完成，但历史按 A/B 稳定记录；
- 普通失败不取消兄弟调用；
- Turn 取消保留已提交结果并收口其余调用；
- 批次完全收口前不会再次采样模型；
- `max_parallel_tool_calls = 1`、持久化、恢复和输出上限没有退化。

学习者应能说明模型、Harness 和工具各自负责什么，并判断测试证明的是“真实并行”还是仅仅“完成顺序不同”。理解基线见[学习记录 0014](../learning-records/0014-parallel-admission-order-and-responsibility.md)。

## 完成后的处置

实践通过后，在 CodebaseAgent 中收口 `STATUS`、行为契约和测试证据，再回到学习工作区完成阶段复盘。Subagent 是否进入下一实践由复盘决定，不在本实验中顺带实现。
