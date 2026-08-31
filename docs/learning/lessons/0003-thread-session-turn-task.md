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
