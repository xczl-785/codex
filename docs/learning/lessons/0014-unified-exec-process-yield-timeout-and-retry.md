# 第十四课：一条命令怎样变成可续接的真实进程

本课沿一次 `exec_command` 跟踪真实进程的启动、有限等待、后台保留、轮询、超时和重试。最重要的边界是：

```text
Function Call 已返回，不等于操作系统进程已结束
yield 等待到期，不等于命令超时
命令超时，不等于命令没有产生效果
技术上可以重试，不等于语义上应该重试
```

## 一、`session_id` 实际是进程标识

`exec_command` 返回的 `session_id` 不是前面课程中的 Codex `Session`。源码内部使用 `process_id`，只因模型已经熟悉 `session_id` 这个工具字段而在协议上沿用该名称。

它表示一个仍由 `UnifiedExecProcessManager` 管理的终端进程：

```text
Codex Session
  └─ UnifiedExecProcessManager
       ├─ process_id 41
       └─ process_id 42  <- 对模型显示为 session_id 42
```

## 二、第一次 `exec_command` 怎样执行

`ExecCommandHandler` 依次完成：

1. 解析并验证模型提供的 JSON 参数；
2. 选择 Turn Environment，解析 cwd 和 shell；
3. 解析 sandbox permissions 与 additional permissions；
4. 分配唯一 `process_id`；
5. 经过 ExecPolicy、Approval 和 Sandbox 编排；
6. 启动本地或远程真实进程；
7. 立即启动 stdout/stderr 后台读取；
8. 在 `yield_time_ms` 窗口内收集一批输出。

输出同时流向两个方向：

```text
真实进程 stdout/stderr
  ├─ Event 流 -> TUI / App Server 实时展示
  └─ HeadTailBuffer -> 本次 Tool Output 的有界输出快照
```

如果进程在等待窗口内完成，响应包含最终 `exit_code`，不含 `session_id`。如果等待窗口结束而进程仍存活，进程会被存入 Process Store，响应包含 `session_id`，不含最终退出码。

```text
Function Call #1: exec_command
  -> 返回 FunctionCallOutput，并在协议层结束

Process #42
  -> 仍然运行并继续产生输出
```

## 三、`write_stdin` 是后续工具调用，不是新 Task

模型在下一个 Step 中可以调用：

```json
{
  "session_id": 42,
  "chars": "",
  "yield_time_ms": 30000
}
```

空 `chars` 表示只轮询新输出；非空内容会先写入原进程 stdin，再等待输出。`write_stdin` 是一个新的 Function Call，但仍可位于原 Turn 的同一个 `RegularTask` 中：

```text
RegularTask / Turn
  ├─ Step 1: exec_command -> session_id 42
  ├─ Step 2: write_stdin(42, "") -> 仍在运行
  └─ Step 3: write_stdin(42, "") -> exit_code 0
```

不同终端进程可以并发轮询；同一终端的读写需要串行化，因为它们共享可被 drain 的输出缓冲区和进程生命周期。进程退出后会从 Process Store 移除，继续使用旧 ID 会得到 unknown process 错误。

## 四、为什么要把 Tool Call 与进程生命周期拆开

- Agent 可以在长命令运行时检查其他文件或调用其他工具；
- 交互式程序可以多次读取问题和写入回答；
- 客户端不必让一次工具请求无限悬挂；
- UI 可以实时消费输出 Event，模型只在 Step 边界接收有界输出；
- Agent 能够明确引用原进程，而不是误把“暂时没等到”当成失败并启动副本。

## 五、必须区分的几种时间边界

| 时间机制 | 到期后的动作 | 进程是否继续 |
| --- | --- | --- |
| 初次 yield | 返回当前输出和 `session_id` | 继续 |
| 空 `write_stdin` poll | 返回这一段新输出和状态 | 继续 |
| One-shot hard timeout | 标记超时并终止确切进程 | 不继续 |
| Cancellation | 请求终止或清理，具体行为取决于执行模式 | 取决于模式 |
| 外部请求 timeout | 只能确认没有按时收到结果 | 结果可能未知 |

普通 Unified Exec 暴露 `yield_time_ms`、`session_id` 和 `write_stdin`，让长进程可续接。受管理要求禁用 Unified Exec 时，`exec_command` 使用 one-shot 模式，暴露 `timeout_ms`；默认 10 秒，超时或取消时终止进程且不能恢复。

普通 Unified Exec 会在初次等待前保存仍存活的进程，避免初始 Tool Call 被中断时因最后一个引用消失而意外终止后台进程。因此不能笼统断言“Turn Interrupt 必然杀掉所有后台终端”；后台进程还有显式的单进程/全部进程终止和 Session 清理路径。

## 六、重试不是一个布尔开关

超时只表示在期限内没有观察到最终结果，不表示操作没有执行。远端写操作可能已经成功，只是响应丢失；盲目重试会产生重复订单、重复消息或重复发布。

| 观察结果 | 合理动作 |
| --- | --- |
| 进程仍存活并有 `session_id` | 继续轮询原进程，不启动副本 |
| 只读或确定幂等的临时失败 | 可做有界重试 |
| 语法、路径、编译等确定性错误 | 修正原因后再执行，原样重试无意义 |
| 权限不足 | 改变权限请求或方案，不原样重试 |
| 非幂等外部写入且结果未知 | 查询/对账，使用幂等键或补偿，不盲目重试 |

可靠的通用重试策略需要说明：重试单位、错误分类、最大次数、总时间预算、指数退避、随机抖动、幂等键、结果对账和取消传播。“固定重试三次”只是实现参数，不是完整设计。

## 七、Codex 当前的三种重试层次

### 1. 模型语义重试

普通非零退出、参数错误和 one-shot timeout 会成为 Tool Output。模型读取原因后，可以修改命令、路径或权限并发起新的 Tool Call。新调用有新的 `call_id`，不是 Harness 隐藏重放旧调用。

### 2. Sandbox denial 的受控第二次 Attempt

`ToolOrchestrator` 可以在确认第一次是 Sandbox Denied、且审批策略允许时，以提升后的沙箱策略执行第二次 attempt。它不是无限循环。在 `OnRequest` 下，普通文件沙箱拒绝通常返回模型，由模型明确使用 `require_escalated` 发起新调用；网络审批存在专用路径。

第一次受限执行可能已经在允许范围内产生部分副作用，因此第二次 attempt 仍不能被理解成安全事务重放。

### 3. 同一进程的 Poll

`write_stdin` 轮询不是重试命令。它不会重新启动 shell，而是继续读取同一个 `process_id` 的输出。把 Poll 误当成 Retry 会产生重复进程，是设计上最需要避免的混淆。

## 八、后续源码实验标记

后续可做一次最小、可回撤的源码观察实验：

1. 为进程分配、Store 插入、yield 返回、`write_stdin` poll、exit watcher 和移除添加临时 tracing；
2. 运行一个分段输出的短脚本，记录同一 `process_id` 的事件顺序；
3. 对比自然退出、空轮询、stdin 交互、one-shot timeout 和显式终止；
4. 验证 Tool Call 已结束但真实进程继续存在；
5. 实验后回撤临时代码，只保留观察结论和必要测试。

该实验不是当前静态学习的前置条件，保留为以后有运行环境时的验证项。

## 当前源码入口

- [`tools/handlers/unified_exec/exec_command.rs`](../../../codex-rs/core/src/tools/handlers/unified_exec/exec_command.rs)：参数解析、环境选择、process ID 分配和 interactive/one-shot 分流。
- [`tools/handlers/unified_exec/write_stdin.rs`](../../../codex-rs/core/src/tools/handlers/unified_exec/write_stdin.rs)：后续 poll 和 stdin 输入。
- [`unified_exec/process_manager.rs`](../../../codex-rs/core/src/unified_exec/process_manager.rs)：进程保存、输出收集、状态刷新和移除。
- [`unified_exec/oneshot.rs`](../../../codex-rs/core/src/unified_exec/oneshot.rs)：hard timeout 与 cancellation 后的进程终止。
- [`tools/orchestrator.rs`](../../../codex-rs/core/src/tools/orchestrator.rs)：审批、沙箱 attempt 和受控提升重试。
- [`tools/handlers/shell_spec.rs`](../../../codex-rs/core/src/tools/handlers/shell_spec.rs)：模型看到的 `exec_command`、`write_stdin` 参数和输出 Schema。
