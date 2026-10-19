# 第五课：Fork、Interrupt、编辑旧消息与 Revert

这几个操作看起来都像“回到过去”，但作用层级不同：

```text
Interrupt：停止当前 Turn/Task
Fork：保留源 Thread，复制历史前缀形成新 Thread
编辑旧消息：Revert 当前 Thread 到旧消息之前，再把消息放回输入框
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

| 层级       | 响应                                              |
| ---------- | ------------------------------------------------- |
| App Server | 读取源 Thread，截取历史前缀，创建并返回新 Thread  |
| Thread     | 新 `ThreadId`，记录来源；源 Thread 不变           |
| Session    | 新 Thread 获得独立的新 Session                    |
| Turn       | Fork 本身不是普通 Agent Turn；复制的是历史前缀    |
| Task       | Fork 本身不启动 `RegularTask`；新消息发送后才启动 |
| Store      | 为新 Thread 保存独立历史和来源关系                |
| 文件系统   | 不复制、不回滚，仍然是当前工作区状态              |

Fork 是非破坏性分支：保留旧路线，再从一个历史边界开始新路线。

## 二、Interrupt 当前回复

TUI 找到当前活动 Turn ID，调用：

```text
turn/interrupt { threadId, turnId }
```

App Server 会核对 Thread、活动 Turn 和 Turn ID，再向 Core 提交 `Op::Interrupt`。通常要等 Core 发出 `TurnAborted`，请求才算完成。

各层变化如下：

| 层级       | 响应                                                 |
| ---------- | ---------------------------------------------------- |
| TUI        | 记录等待中断的 Turn，避免重复请求；处理 Turn ID 竞争 |
| App Server | 验证活动 Turn，转发 `Op::Interrupt`                  |
| Thread     | 不变                                                 |
| Session    | 不变；取出并清理当前 `ActiveTurn`                    |
| Turn       | 从运行状态变成 Aborted/Interrupted，仍保留在历史中   |
| Task       | 取消令牌触发，异步执行终止并调用 `abort()` 清理      |
| Store      | 写入中断标记和终止事件                               |
| 文件系统   | 已完成的文件、命令、提交和网络副作用不撤销           |

Interrupt 表示“从现在开始不要继续”，不是事务回滚。

## 三、中断后编辑旧消息并重新发送

本节在合入 `origin/main` 的 `89461ea6` 后重新核对。当前 TUI 已从旧版 Fork 路径改为同一 Thread 上的 Revert：

```text
选择历史中的用户消息
-> 找到它所属的持久 Turn
-> thread/revert(threadId, beforeTurnId)
-> 在同一 Thread 中刷新保留的历史
-> 把原消息恢复到输入框
-> 用户修改并重新发送
```

假设 Thread A 原来是 T1 → T2 → T3，选择 T2 编辑：

```text
编辑前：Thread A = T1 → T2 → T3
回退后：Thread A = T1，输入框恢复 T2 的原始消息
重发后：Thread A = T1 → T4（修改后的输入触发的新 Turn）
```

因此，“原地编辑”描述的是用户留在同一对话中。底层并非只改旧消息的一段字符串：目标 Turn 与后续 Turn 退出当前有效历史，运行体按保留历史重建；重新发送再产生新 Turn。Thread ID 保持不变，工作区不回到 T2 之前。

当前 TUI 调用 `backtrack_revert_before_turn_id` 选择边界，随后 `app_server.revert_thread` 执行回退。确认服务端回退后才刷新界面；如果提交后刷新失败，会提示重新恢复会话，不能继续使用旧显示历史假装操作没有发生。

### Steer 消息的限制

一个 Turn 可以因 Steer 包含多条用户消息，但这里使用 Turn 边界。TUI 不能把同一 Turn 内的后续 Steer 气泡当作独立回退边界；当前实现仍对运行中或无法定位的边界做检查。

### 对已经派发的子 Agent 应怎样理解

父 Thread 的有效历史回退，与子 Thread 的生命周期、父子关系记录及外部副作用属于不同对象。不能从“父历史中不再显示那次 spawn”推断子 Agent 已删除，不能从“同一 Thread ID”推断旧子任务自动重做。本次核对确认编辑入口改用 Revert，没有把它验证为整棵子 Agent 树的事务回滚。子 Agent 的具体处置需继续沿对应版本的关闭、恢复和关系管理链核对。

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
4. 让 Thread Store 创建引用保留前缀的新 rollout，并切换当前历史指针；
5. 用截断后的历史重新加载同一个 Thread ID；
6. 恢复设置、监听器和订阅；
7. 发送 `thread/reverted` 通知。

各层变化如下：

| 层级       | 响应                                                     |
| ---------- | -------------------------------------------------------- |
| App Server | 关闭旧运行时、重写 Store、重新加载并通知客户端           |
| Thread     | ID 不变，有效历史被替换                                  |
| Session    | 旧 Session 关闭，基于新历史创建替代 Session              |
| Turn       | 目标 Turn 和后续 Turn 被排除；Revert 本身不是 Agent Turn |
| Task       | Revert 本身不启动 Task；重新发送后才启动新 Task          |
| Store      | 当前有效历史切换为更早前缀，旧 rollout 保留              |
| 文件系统   | 不回滚，客户端需另行处理                                 |

Revert 后必须重建 Session，否则会出现：

```text
Store 中只有 T1
内存上下文却仍然记得 T1/T2/T3
```

这也是 Thread 与 Session 必须分离的实际例子：Thread 身份可以保留，运行体则按新的持久历史重建。

`thread/rollback` 是旧接口，按末尾 Turn 数量回退，已经被标记为废弃；它同样只影响历史，不撤销本地文件。

### 有效历史切换不等于擦除旧日志

当前存储实现创建新的不可变 rollout，引用旧日志中保留的历史前缀，写入完成后通过条件更新切换状态数据库中的 rollout 路径。Thread ID 不变，但 rollout ID 可以变化；旧日志仍保留。只有数据库当前路径仍等于预期旧路径时才能完成切换，避免覆盖并发变化。这也是为什么不能把整个 SQLite 状态库都称作可丢弃缓存。

## 五、完整对照

| 操作           | Thread ID | Session | Turn                     | Task           | 原历史                            | 文件变化 |
| -------------- | --------- | ------- | ------------------------ | -------------- | --------------------------------- | -------- |
| Fork           | 新 ID     | 新建    | 发送后新建               | 发送后新建     | 源 Thread 保留                    | 不回滚   |
| Interrupt      | 不变      | 不变    | 当前轮变 Aborted         | 当前 Task 取消 | 中断轮保留                        | 不回滚   |
| 编辑旧消息再发 | 不变      | 重建    | 修改后新建               | 新建           | 当前有效历史切换，旧 rollout 保留 | 不回滚   |
| Revert 后再发  | 不变      | 重建    | 被截断轮排除，发送后新建 | 新建           | 当前有效历史切换                  | 不回滚   |

## 六、为什么必须是不同操作

- Interrupt 回答：“当前这轮不要继续，但历史保留。”
- Fork 回答：“从过去尝试另一条路线，原路线也保留。”
- Edit 回答：“回退当前对话，在同一对话中修改输入并重新执行。”
- Revert 回答：“丢弃当前 Thread 的后续对话历史，让它回到更早状态。”

如果都叫“撤销”，就无法判断用户究竟想停止执行、保留分支还是重写历史，也无法安全决定文件副作用是否处理。

贯穿三者的原则是：

> 对话历史和工作区状态是两套独立状态。对话回到过去，不代表磁盘也回到过去。

## 七、应用所说的 Revert 究竟可能回退什么

不同 Agent Application 都可能把按钮命名为 Revert、Restore 或 Rewind，但它们覆盖的状态并不相同。判断一个回退功能时，应先把系统状态拆成四层：

| 状态层       | 典型内容                                     | 常见恢复手段                          |
| ------------ | -------------------------------------------- | ------------------------------------- |
| 对话状态     | 用户消息、模型回复、工具调用记录、模型上下文 | 截断历史、恢复历史前缀、Fork Thread   |
| 工作区状态   | 已跟踪和未跟踪文件、配置、生成物             | 文件快照、Shadow Git、正式 Git commit |
| 本机运行状态 | 进程、端口、临时目录和缓存                   | 停止进程、重启服务、人工清理          |
| 外部世界状态 | Git push、数据库写入、部署、消息和远端 API   | 补偿操作或人工恢复                    |

通常只有前两层会被产品称为 checkpoint/revert。本机运行状态和外部副作用很难被通用恢复机制自动逆转。因此“恢复工作区”也不等于让整个执行过程发生时间倒流。

### Codex 的边界

Codex App Server 的 `thread/revert` 改变当前 Thread 的有效对话历史。Thread ID 保持不变，旧 Session 被关闭，再依据新历史建立替代 Session；工作区文件不自动回滚。持久化层会创建新的 rollout 文件，并切换当前历史指针。

当前 TUI 编辑旧 Prompt 也通过同一 Thread 的 Revert 回到所选 Turn 之前，再把旧 Prompt 放回输入框供编辑和重新发送。对话历史回退后，工作区仍保持当前状态。由此可能形成：

```text
对话历史：已经回到创建 B.rs 之前
当前磁盘：B.rs 依然存在
```

新 Session 后续会基于旧对话和当前磁盘继续运行。模型必须重新读取工作区，不能假定磁盘也处于历史时刻。

Codex 不自动恢复文件，可以避免覆盖用户在 Agent 执行后追加的手工修改，也避免同一工作区中的另一个 Thread 被连带回滚。代价是客户端若想提供“对话和文件一起恢复”，必须自行维护文件 checkpoint 并协调冲突。

### 代表性应用的不同选择

| 应用                 | 对话状态                   | 工作区文件                   | 核心边界                                         |
| -------------------- | -------------------------- | ---------------------------- | ------------------------------------------------ |
| Codex App Server     | 同 Thread 历史 Revert      | 不恢复                       | 客户端负责文件恢复                               |
| Codex TUI 编辑旧消息 | 同 Thread 历史 Revert      | 不恢复                       | 历史回退不等于工作区回退                         |
| Claude Code          | 可只恢复对话，也可同时恢复 | 恢复文件编辑工具捕获的快照   | Bash、外部修改和多数后台 Subagent 修改不保证覆盖 |
| Cursor Agent         | 从旧请求节点恢复交互位置   | 恢复 Agent 自己的文件修改    | 本地 checkpoint，不等同于 Git，不跟踪手工编辑    |
| Cline                | 可恢复 Task 或同时恢复     | Shadow Git 保存项目文件状态  | 文件快照较完整，但大仓库成本更高                 |
| Windsurf Cascade     | 提供步骤级回退             | 代码 checkpoint              | 产品文档未公开全部对话存储语义                   |
| Aider                | 不以对话回退为核心         | 撤销 Aider 创建的 Git commit | 使用正式 Git 历史，外部副作用仍不受覆盖          |

当多个 Thread 或 Agent 并行工作时，更可靠的文件隔离方式通常是让每条对话分支对应独立 Git branch 和 worktree：

```text
Thread A -> worktree A -> branch A
Thread B -> worktree B -> branch B
```

这只能隔离文件目录；数据库、端口、远端仓库和其他外部资源仍需单独隔离。

### Sandbox 与 Revert 的关系

二者处理的是不同时间方向的问题：

```text
Sandbox：执行之前限制操作可以影响哪些资源
Checkpoint/Revert：执行之后恢复已经记录下来的部分状态
```

沙箱中的命令成功后不会再在另一个“真实环境”执行一次；它已经在受限边界内真实执行。Checkpoint 也不是事务日志，不能自动逆转未捕获的 Bash 修改、进程、数据库、部署或远端 API 调用。

## 外部对照资料

- [Claude Code Checkpointing](https://code.claude.com/docs/en/checkpointing)
- [Cursor Checkpoints](https://docs.cursor.com/en/agent/chat/checkpoints)
- [Cline Checkpoints](https://docs.cline.bot/core-workflows/checkpoints)
- [Windsurf Cascade](https://docs.windsurf.com/es/windsurf/cascade/cascade)
- [Aider Git integration](https://aider.chat/docs/git.html)

## 当前源码入口

- [`tui/src/app_backtrack.rs`](../../../codex-rs/tui/src/app_backtrack.rs)：旧消息选择、Revert 边界和输入框恢复。
- [`tui/src/app/thread_routing.rs`](../../../codex-rs/tui/src/app/thread_routing.rs)：TUI 的 Interrupt 请求和 Turn ID 竞争处理。
- [`app-server-protocol/src/protocol/v2/thread.rs`](../../../codex-rs/app-server-protocol/src/protocol/v2/thread.rs)：Fork、Revert 和旧 Rollback 协议语义。
- [`app-server/src/request_processors/thread_processor.rs`](../../../codex-rs/app-server/src/request_processors/thread_processor.rs)：Fork 截断与 Revert 关闭、重写、重载流程。
- [`app-server/src/request_processors/turn_processor.rs`](../../../codex-rs/app-server/src/request_processors/turn_processor.rs)：Interrupt 验证与 Core 操作提交。
- [`core/src/tasks/mod.rs`](../../../codex-rs/core/src/tasks/mod.rs)：Task 取消、TurnAborted 和中断历史标记。

新版核对入口：[`tui/src/app/event_dispatch.rs`](../../../codex-rs/tui/src/app/event_dispatch.rs) 的编辑消息分支；[`thread-store/src/local/revert_thread.rs`](../../../codex-rs/thread-store/src/local/revert_thread.rs) 的历史指针切换；[`app-server/tests/suite/v2/thread_revert.rs`](../../../codex-rs/app-server/tests/suite/v2/thread_revert.rs) 的同 Thread ID 与运行体重载测试。本轮静态阅读这些测试，未执行 Rust 测试。
