# 第三课：Thread、Session、Turn 与 Task

这一课解决一个核心问题：源码里的 Thread、Session、Turn 和 Task 都在描述“Agent 正在工作”，它们究竟有什么区别？

## 先看完整关系

```text
Thread：这段长期会话是谁
└─ CodexThread：调用方操作这段会话的运行句柄
   └─ Session：这段会话当前在内存中如何运行
      └─ Active Turn：现在正在处理哪一轮
         └─ Running Task：用哪种执行程序完成这一轮
```

它们不是四层同类容器，而是在不同维度描述同一套运行过程：

- Thread 关注长期身份与历史边界。
- Session 关注当前进程中的运行状态和服务。
- Turn 关注一次执行从开始到结束的边界。
- Task 关注用什么具体程序推进这次执行。

## Thread：长期会话身份

Thread 是最稳定的领域概念，代表一段可以持续、恢复、派生和保存的工作会话，并由 `ThreadId` 标识。

它可以包含很多 Turn。程序关闭后，Thread 的持久化记录仍然可以存在；以后恢复时，逻辑上仍是同一个 Thread。父 Agent 创建 Subagent 时，子 Agent 也会获得独立 Thread。

`ThreadManager` 负责创建、恢复、查找和维护 Thread。当前已经加载到内存的 Thread，会以 `Arc<CodexThread>` 的形式保存在管理器中。

### Thread 与 CodexThread 的区别

`CodexThread` 不是全部历史数据本身，而是调用方操作当前 Thread 的运行句柄。它内部主要持有：

- `Arc<Session>`：核心运行实体；
- `SessionIo`：提交操作、接收事件的通信通道；
- Thread 来源、初始化配置和 rollout 路径等信息。

因此可以这样区分：

> Thread 是产品与持久化意义上的会话身份；`CodexThread` 是程序当前用来操作它的句柄。

## Session：Thread 的内存运行实体

`Session` 的源码注释称它为“初始化后的模型 Agent 上下文”，并明确规定一个 Session 同一时刻最多只有一个正在运行的 Task。

它保存 Thread 当前运行所需的内容，例如：

- `thread_id`；
- 当前 `SessionState`；
- 对外事件通道；
- 模型、MCP、工具和沙箱等服务；
- 输入队列；
- 当前活动 Turn；
- 中断、取消和 Agent 状态。

Session 不是另一段历史，也不是新的对话层级。它是 Thread 被加载以后，在当前进程中工作的那台“机器”。

- Thread 回答：“这是谁的会话？”
- Session 回答：“这段会话现在靠什么运行？”

程序退出后，内存中的 Session 可以消失。以后根据持久化记录恢复出新的 Session，逻辑上仍然可以属于原来的 Thread。

### 为什么不能把 Thread 和 Session 合并

两者分离的是**持久身份**与**临时运行资源**。

Thread 的身份和历史需要跨进程保留；Session 中的异步通道、锁、任务句柄、输入队列、MCP 连接和活动 Turn 只对当前进程有意义，程序重启后必须重新建立。

如果把它们合成一个大对象，会产生几个问题：

1. **难以恢复**：持久历史与不可恢复的连接、锁和任务状态混在一起。
2. **浪费运行资源**：大量历史 Thread 可能被迫携带只有当前活动会话才需要的服务。
3. **污染持久化格式**：更换通道、连接或锁的实现也可能影响历史数据格式。
4. **异常恢复危险**：进程崩溃后，很难清楚区分仍然有效的会话历史和已经失效的运行资源。

拆开后，恢复过程变得清晰：读取旧 Thread 的身份与历史，再根据当前环境创建一个新 Session。

> Thread/Session 分离的意义，是让会话身份能够跨进程延续，同时允许运行资源安全地创建、销毁和重建。

## Turn：一次有明确边界的执行

Turn 从本轮工作被接受和启动开始，到完成、中断或失败结束。它不严格等于“一条用户消息加一条助手回复”，因为运行中的 Turn 可以通过 Steer 接收后续输入。

Turn 在源码中不是一个单独的大型对象，而由几个结构共同实现：

- `TurnContext`：本轮的配置、模型、权限、工作目录、环境、动态工具和追踪信息。
- `ActiveTurn`：表示 Session 当前存在一轮活动执行。
- `TurnState`：保存本轮持续变化的状态。
- `RunningTask`：保存实际 Task、取消令牌、异步句柄和对应的 `TurnContext`。

Session 中的活动 Turn 使用下面这种状态表达：

```text
Mutex<Option<ActiveTurn>>
```

- `None`：当前空闲，可以启动新 Turn。
- `Some(...)`：已有 Turn 正在执行，可以尝试 Steer、等待或中断。

这也是为什么只搜索 `struct Turn` 容易迷路：Turn 首先是一个生命周期概念，实现分布在上下文、状态和运行任务等结构中。

## Task：驱动本轮的具体执行程序

这里的 Task 不只是用户语言中的“待办事项”，更接近运行策略或执行器。

`SessionTask` trait 要求实现者提供：

- `kind()`：说明任务类型；
- `run(...)`：执行具体工作；
- `abort(...)`：被中断后进行清理。

主要 `TaskKind` 包括：

- `Regular`：普通用户输入触发的 Agent 工作；
- `Review`：代码评审；
- `Compact`：上下文压缩。

Task 运行时会拿到当前 `Session`、`TurnContext`、本轮输入和取消令牌，并在 `run()` 中推进模型请求、工具调用和事件输出。

因此不要简单理解成“一个 Turn 里面放了许多业务 Task”。更准确的是：

> Turn 是本轮执行的业务边界，Task 是驱动这一轮的具体程序形态。

Task 完成、取消或失败后，Session 负责完成本轮收尾并清理活动 Turn；Session 自身通常继续存在，等待同一个 Thread 的下一轮输入。

### 为什么不能把 Turn 和 Task 合并

两者分离的是**业务执行边界**与**具体执行方法**。

Turn 负责表达本轮属于哪个 Thread、何时开始、接收了什么输入、产生了什么事件，以及最终完成、中断还是失败。Task 负责选择具体算法来推进它，例如普通 Agent 循环、代码评审或上下文压缩。

如果把执行算法全部写进 Turn，会产生几个问题：

1. **生命周期与算法混杂**：普通、Review、Compact 等分支不断挤进同一个核心状态机。
2. **取消和收尾重复**：每种执行方式各自处理取消令牌、异步句柄、完成事件和资源清理。
3. **外部 API 被内部实现污染**：更换内部执行策略可能迫使 Thread/Turn API 一起变化。
4. **扩展风险增大**：新增任务类型必须修改 Turn 核心逻辑，而不是实现统一执行契约。

`SessionTask` 让 Session 可以统一管理启动、中断和完成，具体 Task 只负责自己的执行算法。

> Turn/Task 分离的意义，是让一轮工作的身份与生命周期保持稳定，同时允许内部采用不同执行策略。

## 为什么在普通路径上看起来多此一举

最常见的情况近似一一对应：一个活动 Thread 对应一个 Session，一次普通请求产生一个 Turn，再由一个 `RegularTask` 执行。只看这条路径，合并概念似乎更简单。

但它们的变化原因和寿命并不相同：

| 概念 | 回答的问题 | 主要变化时机 |
| --- | --- | --- |
| Thread | 这是哪段长期会话？ | 新建、派生或切换会话 |
| Session | 这段会话当前怎样运行？ | 加载、重启、迁移或关闭 |
| Turn | 当前是哪一轮执行？ | 新输入、完成、中断或失败 |
| Task | 这一轮采用什么执行程序？ | 根据普通、评审、压缩等形态选择 |

当两个概念在简单场景下一一对应，但生命周期、持久化方式和变化原因不同，就值得拆开。

## 完整生命周期

```text
1. ThreadManager 创建或恢复 Thread
2. 调用方得到 CodexThread 运行句柄
3. CodexThread 内部持有 Session
4. 用户在空闲状态提交输入
5. Session 为本轮构造 TurnContext
6. Session 建立 ActiveTurn
7. 启动对应的 RunningTask，普通输入通常使用 RegularTask
8. Task 推进模型、工具、审批和事件循环
9. Task 完成、中断或失败
10. ActiveTurn 被清理
11. Session 等待这个 Thread 的下一轮输入
12. Thread 历史与状态可以持久化，供以后恢复
```

## Step 暂时放在哪里

在 `TurnContext` 附近还能看到 `StepContext`。Step 比 Turn 更细，通常对应 Turn 内某次模型交互所采用的设置快照。

普通 Turn 可能经历多次“请求模型 → 调用工具 → 把结果交回模型”，所以一轮 Turn 内可以有多个步骤。下一阶段理解模型与工具循环时再展开 Step，现在只需知道它不是 Thread、Session、Turn、Task 的同义词。

## 源码入口

1. [`core/src/thread_manager.rs`](../../../codex-rs/core/src/thread_manager.rs)：Thread 创建、恢复和内存管理。
2. [`core/src/codex_thread.rs`](../../../codex-rs/core/src/codex_thread.rs)：Thread 的运行句柄与操作入口。
3. [`core/src/session/session.rs`](../../../codex-rs/core/src/session/session.rs)：Session 的服务、状态和活动 Turn。
4. [`core/src/session/turn_context.rs`](../../../codex-rs/core/src/session/turn_context.rs)：本轮上下文快照。
5. [`core/src/state/turn.rs`](../../../codex-rs/core/src/state/turn.rs)：`ActiveTurn`、`RunningTask` 和 `TaskKind`。
6. [`core/src/tasks/mod.rs`](../../../codex-rs/core/src/tasks/mod.rs)：`SessionTask` 的运行与中断契约。

## 记忆锚点

- Thread 是长期身份。
- Session 是内存运行体。
- Turn 是一次执行边界。
- Task 是驱动这一轮的程序。

## 回忆练习

关闭 Codex 后，第二天恢复原来的对话并发送一条普通请求：Thread、Session、Turn 和 Task 分别是沿用旧的，还是重新产生的？为什么？

答案：沿用旧 Thread 的身份和历史；旧进程的运行资源已经失效，所以重建 Session；新请求需要独立执行边界，所以新建 Turn；普通 Turn 再启动新的 `RegularTask` 负责实际执行。
