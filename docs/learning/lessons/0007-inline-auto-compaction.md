# 第七课：自动压缩为什么不会切换 Task

超长 Turn 达到上下文限制时会自动压缩，但这里的“压缩”不是结束 `RegularTask`、启动 `CompactTask`，再恢复原来的 `RegularTask`。当前源码有两种外观相似、生命周期完全不同的压缩路径：**内联自动压缩**和**独立手动压缩**。

## 一、直接纠正心智模型

容易产生的误解是：

```text
RegularTask 运行
→ RegularTask 暂停或结束
→ 调度 CompactTask
→ CompactTask 完成
→ 调度器找到原来的 RegularTask
→ RegularTask 恢复
```

超长 Turn 的实际路径是：

```text
同一个 RunningTask / RegularTask / TurnContext
└─ run_turn 循环
   ├─ 正常模型采样
   ├─ 发现上下文需要压缩
   ├─ await run_auto_compact(...)
   │  └─ 生成压缩结果并替换 Session 中的模型历史
   ├─ 压缩函数返回 Ok(())
   ├─ continue
   └─ 下一 Step 从压缩后的历史继续采样
```

这里没有第二个 `SessionTask`，也没有第二个 `RunningTask`。`RegularTask` 的异步调用栈只是在等待压缩函数完成，和等待一次工具调用的性质相似。

## 二、为什么函数名里仍然有 `task`

自动压缩最终可能调用这些函数：

- `run_inline_auto_compact_task`
- `run_inline_remote_auto_compact_task`
- `run_inline_remote_auto_compact_task_v2`

名字里的 `task` 容易让人以为它们都是 `SessionTask`。但从调度层看，它们只是普通异步函数，不实现 `SessionTask`，也不会交给 `Session::start_task` 或 `Session::spawn_task`。

真正决定“是不是一个可独立调度的 Task”的不是函数名，而是它是否：

- 实现 `SessionTask`；
- 被包装成 `RunningTask`；
- 注册到 `ActiveTurn.task`；
- 拥有自己的启动、取消和统一完成生命周期。

内联自动压缩不满足这些条件。真正的独立压缩类型是 `tasks/compact.rs` 中的 `CompactTask`。

## 三、Turn 中途自动压缩的精确衔接

一次正常采样结束后，`run_turn` 会计算：

- 模型是否仍需 follow-up；
- 是否存在待处理的 Steer；
- 当前上下文是否达到限制；
- 是否有人通过能力请求了新上下文窗口。

只有“仍需继续”并且“应该滚动上下文窗口”时，才进入中途压缩：

```text
needs_follow_up = model_needs_follow_up || has_pending_input

should_roll_over =
    needs_follow_up
    && (requested_new_context_window || token_limit_reached)
```

随后控制流大致是：

```text
if should_roll_over {
    await run_auto_compact(..., CompactionPhase::MidTurn)
    运行必要的 session-start hooks
    调整下一轮是否可以立即吸收 pending input
    continue
}
```

最重要的是最后的 `continue`。它说明“继续工作”的依据不是数据库里保存了一条待恢复 Task 记录，而是普通程序控制流：压缩调用返回到原来的 `run_turn`，然后重新进入循环顶部。

换成同步伪代码更直观：

```text
while turn_not_finished:
    result = model_and_tools()

    if context_is_full and result_needs_follow_up:
        compact_history()
        continue

    if not result_needs_follow_up:
        break
```

异步 `await` 只是让线程在等待压缩期间可以执行其他工作，不会销毁承载它的 Future、`RegularTask` 或 `RunningTask`。

## 四、压缩到底修改了什么，下一 Step 怎样看到它

本地压缩会：

1. 从 Session 克隆当前模型历史；
2. 向模型提交合成的摘要提示；
3. 得到压缩摘要；
4. 保留必要的用户消息、摘要和重新注入的初始上下文；
5. 调用 `replace_compacted_history`，用新历史替换 Session 中用于后续模型请求的历史；
6. 重新计算 Token 用量；
7. 发出 `ContextCompaction` 完成事件。

远端压缩和 Token Budget 路径的具体压缩算法不同，但“在当前 Turn 中等待完成，然后继续”的控制流相同。

回到 `run_turn` 循环顶部后，下一 Step 会重新捕获适用的 `StepContext`。构造下一次模型请求时，代码再次调用 `sess.clone_history()`。这时取得的已经是压缩后的历史，所以模型自然在摘要基础上继续原任务。

它不需要额外保存“未完成任务说明”，因为以下连续性仍然存在：

- 原来的 `RegularTask` 没结束；
- 原来的 `TurnContext` 没换；
- 原来的取消令牌仍在；
- `run_turn` 的局部控制状态仍在调用栈里；
- Session 历史已被原子性地换成压缩后的版本；
- 压缩摘要包含继续推理所需的历史信息；
- 下一次采样仍属于相同 Turn ID。

真正可能损失的是摘要没有完整保留的历史细节，而不是“系统忘了还有一个 Task”。这也是多次压缩后准确性可能下降的根本原因。

## 五、Turn 开始前也可能内联自动压缩

`run_turn` 在正常采样之前会调用 `run_pre_sampling_compact`。它可能因为以下原因先做一次压缩：

- 上一轮留下的历史已经达到当前上下文限制；
- 模型切换后，压缩兼容哈希发生变化；
- 切换到了更小上下文窗口的模型。

这类压缩使用 `CompactionPhase::PreTurn`，但仍在已经启动的 `RegularTask` 调用栈中：

```text
RegularTask::run
→ run_turn
   → run_pre_sampling_compact
      → run_auto_compact(PreTurn)
   → 构造本轮正常输入和第一个 Step
   → 正常采样
```

“PreTurn”描述的是相对于正常采样的阶段，不表示此时尚未存在 `RegularTask`，也不表示它是上一个 Task 和下一个 Task 之间的独立调度单元。

## 六、真正的 `CompactTask` 是什么

用户显式请求手动压缩时，Session handler 会：

```text
创建新的 TurnContext
→ sess.spawn_task(..., CompactTask)
```

`CompactTask` 实现了 `SessionTask`：

```text
kind = TaskKind::Compact
span_name = session_task.compact
run = 执行手动本地、远端或 Token Budget 压缩
```

它拥有独立 Task 和 Turn 生命周期：

- 发出自己的 `TurnStarted`；
- 压缩阶段标记为 `CompactionPhase::StandaloneTurn`；
- 注册成 `ActiveTurn.task`；
- 完成后走统一 `on_task_finished`；
- 发出自己的 `TurnComplete`。

这才是前面课程所说的 `TaskKind::Compact`。

## 七、手动 CompactTask 会恢复被替换的 RegularTask 吗

不会。

`spawn_task` 的通用语义是：

```text
abort_all_tasks(TurnAbortReason::Replaced)
→ clear_connector_selection()
→ start_task(new_task)
```

所以如果真的在一个正在运行的 `RegularTask` 上强行启动独立 `CompactTask`，旧 Task 是被 `Replaced`，不是被挂起。`CompactTask` 完成后不会自动恢复那个旧 Task。

这说明独立手动压缩适合被看成一次明确的 standalone 操作，通常应在 Thread 空闲边界使用；它不是超长 Turn 自动续跑机制的一部分。

如果系统采用“结束 RegularTask，再启动 CompactTask，最后恢复 RegularTask”的设计，就必须额外实现：

- 可暂停和恢复的 Task 状态机；
- 原 Future 局部状态的持久化或重建；
- pending input、工具调用和审批等待的迁移；
- 取消令牌和 Turn 计时的继承；
- 压缩失败后恢复旧历史还是终止原任务的策略；
- 两个 Task 是否共享同一 Turn ID 的协议语义。

当前实现通过内联调用避开了整套复杂度。

## 八、压缩失败时会怎样

自动压缩不是无条件成功后续跑。

- 如果压缩返回 `TurnAborted`，错误会向上传播，当前 RegularTask/Turn 进入中断收尾；
- 对部分其他错误，`run_turn` 会发出错误生命周期事件并结束当前执行，而不是假装压缩成功继续；
- 只有压缩成功返回 `Ok(())`，控制流才走到 `continue`，开始下一 Step。

因此“继续原任务”有一个明确前提：压缩成功并且当前 Turn 没有被取消。

## 九、为什么要保留两种压缩形态

两种压缩解决的不是同一个问题。

### 内联自动压缩

需求是：当前工作还没完成，但上下文放不下了，需要换一个更短的历史继续做。

它必须保留：

- 同一个 Turn；
- 同一个 Task；
- 同一个取消和计时边界；
- 当前执行控制流。

### 独立 CompactTask

需求是：用户明确要求把 Thread 历史整理一下，把“压缩历史”本身作为一次独立操作执行和展示。

它需要：

- 独立 Turn ID；
- 独立开始、完成和错误事件；
- 独立的可取消 Task 身份；
- 不接收普通 Steer。

如果只保留独立 `CompactTask`，超长 Turn 续跑就要实现复杂的暂停/恢复。如果只保留内联函数，用户显式压缩又缺少清晰的协议、界面和生命周期边界。

## 十、一句话记忆

```text
自动压缩：RegularTask “调用一个压缩子过程，然后继续”。
手动压缩：Session “启动一个独立 CompactTask，做完即结束”。
```

所以系统不需要在自动压缩后寻找尚未完成的 `RegularTask`：它从未离开原来的 `RegularTask`。

## 源码入口

- `codex-rs/core/src/session/turn.rs`
  - `run_pre_sampling_compact`
  - `run_auto_compact`
  - `run_turn` 中 `CompactionPhase::MidTurn` 分支
- `codex-rs/core/src/compact.rs`
  - `run_inline_auto_compact_task`
  - `run_compact_task`
  - `replace_compacted_history` 调用链
- `codex-rs/core/src/compact_token_budget.rs`
  - Token Budget 的内联与手动压缩入口
- `codex-rs/core/src/tasks/compact.rs`
  - 真正实现 `SessionTask` 的 `CompactTask`
- `codex-rs/core/src/session/handlers.rs`
  - 手动 `compact` 怎样通过 `spawn_task` 启动独立 Task
- `codex-rs/core/src/tasks/mod.rs`
  - `spawn_task` 的替换语义和统一 Task 生命周期

## 回忆检查

1. 函数名包含 `task`，为什么不代表它是 `SessionTask`？
2. 自动压缩成功后，哪一行控制流让原工作继续？
3. 下一次模型采样为什么能看到压缩后的历史？
4. 为什么独立 `CompactTask` 不能承担“暂停后恢复 RegularTask”的职责？
