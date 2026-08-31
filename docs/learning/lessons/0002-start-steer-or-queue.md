# 第二课：连续发送时，新 Turn、Steer，还是等待？

结论：空闲时发送会启动新 Turn；执行中发送会优先 Steer 当前 Turn；当前 Turn 已不能接收调整时，输入会被拒绝、暂存或稍后重新处理。

## Turn 是执行过程，不是聊天气泡

可以把 Turn 想成一个“执行信封”。模型思考、工具调用、审批和继续推理都发生在里面，直到完成、中断或失败才封口。

信封尚未封口时补一句“先别改文件，只分析”，系统可以把它交给当前 Turn。这叫 **Steer**：不另开一轮，而是补充或修正当前方向。

## 四种常见情况

| 状态 | 处理方式 |
| --- | --- |
| Thread 空闲 | `turn/start` 启动新 Turn |
| Turn 正在运行且可调整 | `turn/steer` 送入当前 Turn，Turn ID 不变 |
| 界面认为在运行，但核心发现它刚结束 | 刷新状态，再按新 Turn 处理 |
| 当前 Turn 不可 Steer | 保留、排队或恢复输入，不让文字悄悄丢失 |

Steer 请求携带预期 Turn ID，是为了避免并发状态变化时把补充消息误送进另一轮。

因此，“连续发送三条消息有几个 Turn”没有固定答案：

- 每次等上一轮结束再发：通常一个 Thread、三个 Turn。
- 第一轮运行时连续补充且 Steer 成功：可能一个 Thread、一个 Turn。

## App Server 属于哪一层

App Server 的确是高度封装的统一入口，但不是比 Thread、Turn 更高一级的领域概念。

- Thread、Turn 是系统管理的领域对象。
- App Server 是客户端访问这些对象的接口和协调边界。
- TUI、IDE、SDK 通过它启动、调整和观察执行。

可以把 Thread/Turn 看成工单和一次执行，把 App Server 看成接单台兼翻译员。

## 源码阅读入口

1. [`tui/src/app/thread_routing.rs`](../../../codex-rs/tui/src/app/thread_routing.rs)：看界面如何选择 `turn_steer` 或 `turn_start`。
2. [`core/src/session/turn_input.rs`](../../../codex-rs/core/src/session/turn_input.rs)：看核心如何处理 `StartOrSteer`、`StartIfIdle` 和严格 `Steer`。
3. [`tui/src/chatwidget/input_restore.rs`](../../../codex-rs/tui/src/chatwidget/input_restore.rs)：看未成功送入当前 Turn 的文字如何保留。

## 回忆练习

模型正在调用工具时，你补充“先别改文件，只做静态分析”。Thread 和 Turn 一定会怎样变化？

答案：Thread 不变；界面先尝试 Steer 当前活动 Turn，成功时 Turn 也不变。当前 Turn 已结束或不能接收时，才进入暂存、恢复或后续新 Turn 路径。

参考：[核心概念速查](../reference/glossary.md)

