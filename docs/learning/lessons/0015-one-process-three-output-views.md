# 第十五课：同一个真实进程为什么有三种输出视图

本课继续沿 Unified Exec 的真实进程向外观察。一个命令启动后，Codex 不会把同一份原始输出直接交给所有消费者，而是根据用途形成三种视图：

```text
真实进程 stdout/stderr
  ├─ UI 实时增量：让用户立即看到进度
  ├─ 模型 Tool Output：让模型在 Step 边界决定下一步
  └─ 命令生命周期项：让客户端表示一条命令从开始到结束
```

这不是三份彼此独立的事实，而是同一运行过程面向不同消费者的三种投影。三者在时效、体积、标识方式和持久化策略上都不同。

## 一、先区分三个标识

假设模型启动一个长命令，第一次等待后进程仍在运行，随后又两次调用 `write_stdin` 读取输出：

```text
Function Call C1: exec_command -> process 42 -> chunk A
Function Call C2: write_stdin  -> process 42 -> chunk B
Function Call C3: write_stdin  -> process 42 -> chunk C + exit_code
```

| 标识 | 标识的对象 | 生命周期 |
| --- | --- | --- |
| `call_id` | 一次模型 Function Call | 一次工具调用 |
| `process_id` / `session_id` | 一个受 Unified Exec 管理的真实进程 | 从进程创建到退出或清理 |
| `chunk_id` | 一次返回给模型的输出批次 | 一次 `exec_command` 或 `write_stdin` 返回 |

协议中的 `session_id` 仍然只是内部 `process_id` 的模型可见名称，不是 Codex 的 Session。一个真实进程可以被多次 Function Call 操作，因此它可以对应多个 `call_id` 和多个 `chunk_id`。

## 二、视图一：UI 实时增量

后台 watcher 持续读取 PTY 输出，并发送 `ExecCommandOutputDelta`。App Server 将它映射成客户端通知 `item/commandExecution/outputDelta`，所以界面不必等待下一次模型推理，就能逐渐显示程序输出。

这条通道优先考虑低延迟：

- 单个增量有大小限制；
- 发送增量的数量有配额；
- 不完整的 UTF-8 字节会留到下一批，避免切坏中文字符；
- 进程退出后还会短暂等待并排空尾部输出，降低丢失最后几行的概率；
- Unified Exec 的 PTY 输出在这条路径上表现为一个合并的终端流。

实时增量不适合直接进入模型上下文。操作系统每次读到多少字节具有偶然性，一行文字可能被拆成多个事件；编译、下载或日志程序还可能产生大量重复输出。若把每个增量都注入模型，不仅会消耗上下文，还会频繁改变请求前缀和缓存条件。

Rollout 持久化策略也明确把 `ExecCommandOutputDelta` 归为临时事件。它用于直播当前状态，不是恢复历史时必须逐包重放的事实来源。

## 三、视图二：模型分段 Tool Output

初次 `exec_command` 到达 yield 边界，或者一次 `write_stdin` 等待结束时，`UnifiedExecProcessManager` 会返回 `ExecCommandToolOutput`。它最终转换为与当前工具调用配对的 `FunctionCallOutput`，典型文本为：

```text
Chunk ID: A
Wall time: 10.0000 seconds
Process running with session ID 42
Original token count: 2500
Output:
正在处理……
```

进程完成时则包含：

```text
Chunk ID: C
Wall time: 2.1345 seconds
Process exited with code 0
Output:
任务完成
```

其主要路径是：

```text
工具 Handler 返回 ExecCommandToolOutput
  -> ToolOutput::to_response_item(call_id)
  -> FunctionCallOutput
  -> drain_in_flight
  -> record_annotated_conversation_items
  -> 当前模型历史和 rollout
```

模型并不持续观看终端直播，而是在一个 Function Call 完成后获得一份有界结果，再在下一个 Step 中决定是否继续轮询、输入字符、终止进程或进行其他工作。

每次 `write_stdin` 都是新的 Function Call，所以模型历史中会出现多组 Call/Output；它们通过各自的 `call_id` 配对，又通过共同的 `session_id/process_id` 指向同一个真实进程。

### 模型输出为什么还要独立截断

`ExecCommandToolOutput` 保存本次调用收集到的原始字节和省略信息，但转换成模型可见文本时仍受 token/byte 预算限制。输出头部会预留空间给 `chunk_id`、运行时间、进程状态、退出码和原始 token 数，避免正文耗尽预算后连状态都无法告诉模型。

这一视图的目标不是保留全部终端字节，而是提供足够、稳定且有界的判断依据。

## 四、视图三：客户端命令生命周期

用户通常把下面几次内部调用理解成同一条命令：

```text
exec_command(C1)
write_stdin(C2)
write_stdin(C3)
```

因此 Codex 还维护一个以原始命令为中心的 `CommandExecutionItem`：

```text
CommandExecution started
  -> 多个 outputDelta
  -> CommandExecution completed
```

进程退出后，后台 exit watcher 会等待退出信号和输出排空，生成完成项，其中包括：

- 原始命令和工作目录；
- `process_id`；
- 最终状态和退出码；
- 执行时长；
- stdout、stderr 和聚合输出；
- 格式化后的最终输出。

App Server 或 TUI 可以据此把同一条命令卡片从“运行中”更新为“已完成”，而不必把每一次 `write_stdin` 展示成新的用户命令。

这也意味着界面和模型获知进程结束的时间可能不同。进程可以在模型上一次收到“仍在运行”之后自然退出：后台 watcher 会立即完成客户端命令项，但当前模型请求不会被异步修改；模型要在后续工具调用或推理边界重新观察最终状态。

## 五、聚合输出怎样控制体积

命令生命周期使用 `HeadTailBuffer` 聚合输出，当前上限为 1 MiB。超过上限时保留开头和结尾，并在中间插入省略标记：

```text
启动参数、环境与初始日志
……中间省略若干字节……
最终错误、摘要或成功结果
```

开头常包含执行方式、配置和初始错误，结尾常包含退出原因和结果摘要；重复进度通常集中在中间，因此 head-tail 比只保留开头或只保留结尾更适合命令输出。

模型分段 Tool Output 和命令聚合输出有各自的限制。前者围绕模型上下文预算，后者围绕一条命令生命周期的展示和记录预算，不能把二者理解成同一个缓冲区的简单别名。

## 六、为什么不能只保留一种视图

### 只保留 UI 实时增量

- 高频碎片会撑大上下文和历史；
- 块边界不稳定，难以作为可靠的模型输入；
- 很难判断哪一批构成一次完整 Tool Output；
- 回放需要重新拼接大量临时事件。

### 只保留模型 Tool Output

- 只有模型主动轮询时界面才会更新；
- 长命令会显得卡住；
- 多次 Function Call 会把用户眼中的同一条命令拆散；
- 进程自然退出后，界面可能不能及时结束命令卡片。

### 只保留最终命令项

- 长时间没有进度反馈；
- 无法处理中途交互提示；
- 模型必须等到命令最终结束才能采取下一步行动；
- 用户难以及时发现卡死或早期错误。

所以三种视图分别回答：

```text
UI 增量：现在正在发生什么？
Tool Output：模型下一步应该做什么？
命令完成项：这条命令最终发生了什么？
```

## 七、持久化边界

三种视图不能简单等同于“三份持久化记录”：

- `FunctionCallOutput` 属于模型对话历史，会被记录到 rollout；
- `ExecCommandOutputDelta`、`ExecCommandBegin`、`TerminalInteraction` 等实时事件会被持久化策略过滤；
- 新的 paginated history 会通过 `ItemCompleted` 保存 `CommandExecutionItem`；
- legacy history 不依赖完整命令项恢复，旧式 `ExecCommandEnd` 本身也被视为临时事件。

因此恢复 Thread 能恢复“模型过去看到了什么”，在 paginated 模式下还能恢复客户端命令项；它不会逐包恢复终端直播，也不会复活已经消失的操作系统进程。

## 八、与并行和恢复的关系

Call/Output 持久化不创造并行能力，也不会自动恢复未完成进程。它提供的是可判定的因果事实。

当多个进程并发输出时：

- `call_id` 把结果配回正确的模型调用；
- `process_id` 把多次操作关联到正确的真实进程；
- `chunk_id` 区分一次次有界返回；
- 命令完成项把内部多次轮询重新聚合为用户看到的一条命令。

并行使执行顺序、返回顺序和展示顺序不再天然一致，所以必须显式保存身份和关联，而不能依靠“上一条输出属于上一条命令”这种位置假设。

## 九、与 wait 模式的关系

长命令不是让一次 Function Call 一直悬挂，而是把短工具调用与长进程生命周期拆开：

```text
短 Function Call：启动或观察
长真实进程：继续运行
短 Function Call：再次观察或输入
```

这样可以在进程运行期间继续更新 UI、响应用户中断、轮询多个进程，并让每批模型可见结果进入明确的 Call/Output 因果链。

## 当前源码入口

- [`unified_exec/async_watcher.rs`](../../../codex-rs/core/src/unified_exec/async_watcher.rs)：PTY 输出读取、实时 delta、退出观察和尾部排空。
- [`unified_exec/head_tail_buffer.rs`](../../../codex-rs/core/src/unified_exec/head_tail_buffer.rs)：1 MiB head-tail 聚合策略。
- [`unified_exec/process_manager.rs`](../../../codex-rs/core/src/unified_exec/process_manager.rs)：每次 yield/poll 的输出收集和进程状态。
- [`tools/context.rs`](../../../codex-rs/core/src/tools/context.rs)：`ExecCommandToolOutput` 如何转换为模型可见结果。
- [`tools/events.rs`](../../../codex-rs/core/src/tools/events.rs)：命令开始、完成和 `CommandExecutionItem` 的生成。
- [`session/turn.rs`](../../../codex-rs/core/src/session/turn.rs)：工具结果如何进入 conversation history。
- [`rollout/src/policy.rs`](../../../codex-rs/rollout/src/policy.rs)：实时事件、ResponseItem 和不同历史模式的持久化边界。
- [`app-server-protocol/src/protocol/event_mapping.rs`](../../../codex-rs/app-server-protocol/src/protocol/event_mapping.rs)：核心事件如何映射为客户端通知。

## 后续衔接

原计划可继续研究取消信号如何从 Turn/Task 传到工具和真实进程，但当前先暂停推进新课。下一阶段以课程 0001—0015 的整理、回顾和按疑问回看源码为主，再根据复盘结果决定进入取消传播、并行工具调度或 Subagent 生命周期。
