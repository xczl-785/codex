# 第五课：Fork、Interrupt、编辑旧消息与 Revert

这几个操作看起来都像“回到过去”，但作用层级不同：

```text
Interrupt：停止当前 Turn/Task
Fork：保留源 Thread，复制历史前缀形成新 Thread
编辑旧消息：Fork 到旧消息之前，再把消息放回输入框
Revert：保留 Thread ID，用更早的前缀替换其持久历史
```

它们默认都不会回滚已经发生的文件修改和外部副作用。

## 一、Fork 对话

客户端通过 `thread/fork` 指定源 Thread 和历史边界：

- 无边界：复制到最新有效历史；
- `lastTurnId`：包含指定 Turn，排除其后内容；
- `beforeTurnId`：排除指定 Turn 和所有后续 Turn。

边界 Turn 不能仍在运行。

假设源历史是：

```text
Thread A：T1 -> T2 -> T3
```

在 T3 之前 Fork 后：

```text
Thread A：T1 -> T2 -> T3
Thread B：T1 -> T2
```

各层变化如下：

| 层级 | 响应 |
| --- | --- |
| App Server | 读取源 Thread，截取历史前缀，创建并返回新 Thread |
| Thread | 新 `ThreadId`，记录来源；源 Thread 不变 |
| Session | 新 Thread 获得独立的新 Session |
| Turn | Fork 本身不是普通 Agent Turn；复制的是历史前缀 |
| Task | Fork 本身不启动 `RegularTask`；新消息发送后才启动 |
| Store | 为新 Thread 保存独立历史和来源关系 |
| 文件系统 | 不复制、不回滚，仍然是当前工作区状态 |

Fork 是非破坏性分支：保留旧路线，再从一个历史边界开始新路线。

## 二、Interrupt 当前回复

TUI 找到当前活动 Turn ID，调用：

```text
turn/interrupt { threadId, turnId }
```

App Server 会核对 Thread、活动 Turn 和 Turn ID，再向 Core 提交 `Op::Interrupt`。通常要等 Core 发出 `TurnAborted`，请求才算完成。

各层变化如下：

| 层级 | 响应 |
| --- | --- |
| TUI | 记录等待中断的 Turn，避免重复请求；处理 Turn ID 竞争 |
| App Server | 验证活动 Turn，转发 `Op::Interrupt` |
| Thread | 不变 |
| Session | 不变；取出并清理当前 `ActiveTurn` |
| Turn | 从运行状态变成 Aborted/Interrupted，仍保留在历史中 |
| Task | 取消令牌触发，异步执行终止并调用 `abort()` 清理 |
| Store | 写入中断标记和终止事件 |
| 文件系统 | 已完成的文件、命令、提交和网络副作用不撤销 |

Interrupt 表示“从现在开始不要继续”，不是事务回滚。

## 三、中断后编辑旧消息并重新发送

当前 TUI 的 Backtrack 功能不是原地修改消息，而是：

```text
选择历史中的用户消息
-> 找到它所属的持久 Turn
-> thread/fork(beforeTurnId)
-> 切换到新 Thread
-> 把原消息恢复到输入框
-> 用户修改并重新发送
```

假设：

```text
Thread A
T1：用户输入 A
T2：用户输入 B
T3：用户输入 C，执行中被中断
```

选择 T2 编辑后：

```text
原 Thread A：T1 -> T2 -> T3(aborted)
新 Thread B：T1
输入框：恢复 T2 的原始消息
```

修改并发送后：

```text
新 Thread B：T1 -> T4(修改后的 B)
```

因此实际变化是：

- 原 Thread 和被中断 Turn 完整保留；
- 新建 Thread 和 Session；
- 编辑只发生在输入框；
- 重新发送时创建新 Turn 和 `RegularTask`；
- 工作区不回到 T2 发生前。

### Steer 消息的限制

一个 Turn 可以因 Steer 包含多条用户消息，但 App Server 不能在 Turn 中间 Fork。因此 TUI 只能独立重新打开该 Turn 的初始 Prompt，不能把后续 Steer 气泡当成新的 Fork 边界。

## 四、Revert 后重新发送

新的 App Server 接口使用：

```text
thread/revert { threadId, beforeTurnId }
```

它只支持 paginated history，并把目标 Turn 和所有后续 Turn 从有效持久历史中排除。

假设：

```text
Revert 前：Thread A = T1 -> T2 -> T3
Revert before T2
Revert 后：Thread A = T1
```

App Server 的主要步骤是：

1. 加载 Thread 并保存可恢复配置；
2. 关闭当前运行时；
3. 从 `ThreadManager` 移除旧 `CodexThread`；
4. 让 Thread Store 用目标前缀替换持久历史；
5. 用截断后的历史重新加载同一个 Thread ID；
6. 恢复设置、监听器和订阅；
7. 发送 `thread/reverted` 通知。

各层变化如下：

| 层级 | 响应 |
| --- | --- |
| App Server | 关闭旧运行时、重写 Store、重新加载并通知客户端 |
| Thread | ID 不变，有效历史被替换 |
| Session | 旧 Session 关闭，基于新历史创建替代 Session |
| Turn | 目标 Turn 和后续 Turn 被排除；Revert 本身不是 Agent Turn |
| Task | Revert 本身不启动 Task；重新发送后才启动新 Task |
| Store | 持久历史被破坏性替换为更早前缀 |
| 文件系统 | 不回滚，客户端需另行处理 |

Revert 后必须重建 Session，否则会出现：

```text
Store 中只有 T1
内存上下文却仍然记得 T1/T2/T3
```

这也是 Thread 与 Session 必须分离的实际例子：Thread 身份可以保留，运行体则按新的持久历史重建。

`thread/rollback` 是旧接口，按末尾 Turn 数量回退，已经被标记为废弃；它同样只影响历史，不撤销本地文件。

## 五、完整对照

| 操作 | Thread ID | Session | Turn | Task | 原历史 | 文件变化 |
| --- | --- | --- | --- | --- | --- | --- |
| Fork | 新 ID | 新建 | 发送后新建 | 发送后新建 | 源 Thread 保留 | 不回滚 |
| Interrupt | 不变 | 不变 | 当前轮变 Aborted | 当前 Task 取消 | 中断轮保留 | 不回滚 |
| 编辑旧消息再发 | 通过 Fork 得到新 ID | 新建 | 修改后新建 | 新建 | 原 Thread 保留 | 不回滚 |
| Revert 后再发 | 不变 | 重建 | 被截断轮排除，发送后新建 | 新建 | 当前 Thread 历史被替换 | 不回滚 |

## 六、为什么必须是不同操作

- Interrupt 回答：“当前这轮不要继续，但历史保留。”
- Fork/Edit 回答：“从过去尝试另一条路线，原路线也保留。”
- Revert 回答：“丢弃当前 Thread 的后续对话历史，让它回到更早状态。”

如果都叫“撤销”，就无法判断用户究竟想停止执行、保留分支还是重写历史，也无法安全决定文件副作用是否处理。

贯穿三者的原则是：

> 对话历史和工作区状态是两套独立状态。对话回到过去，不代表磁盘也回到过去。

## 当前源码入口

- [`tui/src/app_backtrack.rs`](../../../codex-rs/tui/src/app_backtrack.rs)：旧消息选择、Fork 边界和输入框恢复。
- [`tui/src/app/thread_routing.rs`](../../../codex-rs/tui/src/app/thread_routing.rs)：TUI 的 Interrupt 请求和 Turn ID 竞争处理。
- [`app-server-protocol/src/protocol/v2/thread.rs`](../../../codex-rs/app-server-protocol/src/protocol/v2/thread.rs)：Fork、Revert 和旧 Rollback 协议语义。
- [`app-server/src/request_processors/thread_processor.rs`](../../../codex-rs/app-server/src/request_processors/thread_processor.rs)：Fork 截断与 Revert 关闭、重写、重载流程。
- [`app-server/src/request_processors/turn_processor.rs`](../../../codex-rs/app-server/src/request_processors/turn_processor.rs)：Interrupt 验证与 Core 操作提交。
- [`core/src/tasks/mod.rs`](../../../codex-rs/core/src/tasks/mod.rs)：Task 取消、TurnAborted 和中断历史标记。
