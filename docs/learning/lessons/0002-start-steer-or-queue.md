# 第二课：连续发送时，新 Turn、Steer，还是等待？

结论：空闲时发送会启动新 Turn；执行中发送会优先 Steer 当前 Turn；当前 Turn 已不能接收调整时，输入会被拒绝、暂存或稍后重新处理。

## Turn 是执行过程，不是聊天气泡

可以把 Turn 想成一个“执行信封”。模型思考、工具调用、审批和继续推理都发生在里面，直到完成、中断或失败才封口。

信封尚未封口时补一句“先别改文件，只分析”，系统可以把它交给当前 Turn。这叫 **Steer**：不另开一轮，而是补充或修正当前方向。

## 四种常见情况

| 状态                               | 处理方式                                 |
| ---------------------------------- | ---------------------------------------- |
| Thread 空闲                        | `turn/start` 启动新 Turn                 |
| Turn 正在运行且可调整              | `turn/steer` 送入当前 Turn，Turn ID 不变 |
| 界面认为在运行，但核心发现它刚结束 | 刷新状态，再按新 Turn 处理               |
| 当前 Turn 不可 Steer               | 保留、排队或恢复输入，不让文字悄悄丢失   |

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

## 为 Subagent 实践补齐的运行时边界

考虑一个普通 Agent：它正在读取恢复相关代码，你补充“把范围缩到历史文件缺失”。原任务仍在继续，但后续分析应纳入这个要求。Steer 解决的是把新输入交给当前执行过程；若每次补充都另开并发 Turn，就必须额外解决同一 Thread 上两个执行过程争用历史和结果归属的问题。

这里至少有三个时刻：

1. 调用方提交补充输入。
2. 运行时确认目标 Turn 可以接收，将输入放入待消费队列。
3. 执行循环在允许消费的边界取出输入，记录到历史，再构造后续模型请求。

第二步成功不能证明第三步已经发生，也不能证明模型已经按新要求采取行动。已经发出的模型请求不会因为队列里新增文字而自动改写。Steer 本身也不是工具取消命令，不能撤销已经发生的副作用。

当前源码的职责分工是：

- `SessionIo::submit_turn_input` 等待 Core 的路由决定；其返回不是等待整个业务任务完成。
- `Session::steer_input` 持有活动 Turn 锁，检查是否有活动任务、预期 Turn ID 是否一致，以及任务是否可 Steer。普通任务可以接收，独立 Review/Compact 任务会被拒绝；这里不能把独立 Compact Task 与常规 Turn 内自动压缩混为一谈。检查通过后，输入进入队列。
- `run_turn` 在允许的循环边界取出待消费输入，运行输入 hooks 并记录历史，随后准备模型请求。源码明确保留初始输入优先采样等延后消费条件，因此不应概括为“任何时刻立即生效”。

还要区分严格 Steer 与 `StartOrSteer`：前者针对预期的活动 Turn，调用方不能把失败理解成已经启动下一轮；后者在没有活动 Turn 时有创建新 Turn 的分支。界面可以根据失败再决定保留或重发，这属于上层策略，不是严格 Steer 自身偷偷切换目标。

这为 Subagent 提供的是公共运行机制的基础：接收方正在执行时，如何接收新的输入。子身份、普通消息邮箱、空闲唤醒和结果通知仍有各自职责，不能把全部 Agent 间通信统称为 Steer。

对应的[实践 0006 M0.5](../practice/0006-subagent-lifecycle-lab.md#m05单-agent-steer-前置实践)先在单 Agent 上验证这些边界，再进入子 Agent 创建与通信。接收和结束恰好同时发生时的完整收口路径留待沿源码继续展开；不能仅凭上述入队路径断言所有输入都一定被模型消费。

## 源码阅读入口

1. [`tui/src/app/thread_routing.rs`](../../../codex-rs/tui/src/app/thread_routing.rs)：看界面如何选择 `turn_steer` 或 `turn_start`。
2. [`core/src/session/turn_input.rs`](../../../codex-rs/core/src/session/turn_input.rs)：看核心如何处理 `StartOrSteer`、`StartIfIdle` 和严格 `Steer`。
3. [`tui/src/chatwidget/input_restore.rs`](../../../codex-rs/tui/src/chatwidget/input_restore.rs)：看未成功送入当前 Turn 的文字如何保留。
4. [`core/src/session/mod.rs`](../../../codex-rs/core/src/session/mod.rs)：`SessionIo::submit_turn_input` 的返回承诺。
5. [`core/src/session/turn.rs`](../../../codex-rs/core/src/session/turn.rs)：`run_turn` 如何在后续请求前消费输入。

## 回忆练习

模型正在调用工具时，你补充“先别改文件，只做静态分析”。Thread 和 Turn 一定会怎样变化？

答案：Thread 不变；界面先尝试 Steer 当前活动 Turn，成功时 Turn 也不变。当前 Turn 已结束或不能接收时，才进入暂存、恢复或后续新 Turn 路径。

参考：[核心概念速查](../reference/glossary.md)
