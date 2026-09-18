# 实践任务 0004：取消传播与部分完成状态

本文件是 CodebaseAgent 取消切片的实验入口：规定为什么做、做到哪里、按什么顺序实施以及怎样退出。具体行为不变量和 C1—C9 场景见[取消传播验收材料](materials/0004-cancellation-acceptance.md)。

## 接手契约

| 字段 | 当前值 |
| --- | --- |
| 状态 | `prepared / not activated in AgentLab` |
| 目标工程 | `D:\Program\AgentLab\CodebaseAgent` |
| 事实来源 | CodebaseAgent 当前行为与测试、[课程 0017](../lessons/0017-cancellation-and-process-lifetimes.md)、当前 Codex 源码与测试 |
| 进入条件 | 实践 0003 已完成；用户明确开始本实践；更新 CodebaseAgent 的 `docs/STATUS.md` |
| 退出条件 | 取消能从 Turn 传播到模型请求和工具调用，完成、未完成及恢复边界有确定性测试 |
| 下一实践 | 实践 0005：有界并行工具调度 |

创建本文件不会自动开启 AgentLab 实现。实时进度始终以 CodebaseAgent 自己的 `docs/STATUS.md` 为准。

## 实验目标

CodebaseAgent 已有模型—工具循环、多调用批次、持久历史、恢复和压缩，但没有明确的取消传播链。本实践先固定以下问题，避免在实践 0005 中同时调试取消与并行：

- 谁拥有 Turn 的取消源；
- 取消请求和 Turn 终止怎样区分；
- 已完成结果与取消竞争时，以什么时刻决定结果；
- 已记录但未完成的 Call 怎样得到唯一、可恢复的收口；
- 为什么取消后不能继续采样模型。

## 复用范围

继续使用 CodebaseAgent 现有的 Turn/Step 循环、`ScriptedModel`、Function Call 批次、`call_id` 配对、只读工具、持久化与恢复。整个实践保持 `max_parallel_tool_calls = 1`。

本实践不加入真正并行、Subagent、Shell 或操作系统进程治理、权限沙箱、事务回滚，也不依赖真实 Provider 作为核心验收条件。

## 开工前必须写入本地行为契约

实现前先在 CodebaseAgent 中确定：

1. Turn 的 `running → cancel_requested → cancelled` 状态及终态责任；
2. 模型请求和工具调用怎样接收同一 Turn 派生的取消信号；
3. “结果已提交”的线性化点；
4. 结构化取消结果及其 Call/Output 持久化形式；
5. 重复取消和迟到结果的幂等规则。

推荐把取消表达为机器可判定的 outcome，而不是依赖自由文本包含 `aborted`。具体类型名由 CodebaseAgent 现有模型决定。

## 实施顺序

### M0：契约与失败测试

- 固定状态、结果竞争点和结构化取消结果；
- 用可控 test double 建立模型等待取消、结果与取消竞争、部分完成三个失败测试；
- 测试同步使用 barrier、channel 或显式 gate，不使用睡眠猜竞态。

### M1：取消模型等待

- 让 Turn 运行时持有取消源并传给模型请求；
- 区分取消已请求与 Turn 已结束；
- 证明取消后不会进入工具分发或构造下一 Step。

### M2：取消工具并收口历史

- 把取消信号传入当前工具调用；
- 保留已经提交的成功或失败结果；
- 为未完成 Call 形成唯一取消 Output；
- 禁止迟到结果触发第二个 Output 或下一次模型采样。

### M3：恢复与观察面

- 持久化 Turn 终态和每个 Call 的唯一 Output；
- 验证新 Session 不会重跑工具或恢复旧 Future；
- 验证重复取消、观察者断开和迟到结果不破坏不变量；
- 更新 CodebaseAgent 的状态、能力文档和测试索引。

## 验收入口

完整场景、预期事实和事件证据见[取消传播验收材料](materials/0004-cancellation-acceptance.md)。退出本实践至少需要证明：

- 模型等待和工具等待都能响应取消；
- 已完成结果不被稍晚取消改写；
- 未完成 Call 不留悬空记录；
- 取消不是外部副作用回滚；
- 取消后没有下一次模型采样；
- 重启后只根据持久事实恢复，不重放运行资源；
- 现有串行批次、输出上限和历史恢复没有退化。

学习者应能解释这些证据为什么排除了“只停 UI”“丢弃历史”和“事后覆盖完成结果”等错误实现。理解基线见[学习记录 0013](../learning-records/0013-cancellation-and-process-lifetime-boundaries.md)。

## 完成后的处置

实践通过后，先在 CodebaseAgent 中收口 `STATUS`、行为契约和测试证据，再更新本学习工作区的课程或学习记录。只有取消语义稳定后，才激活实践 0005。
