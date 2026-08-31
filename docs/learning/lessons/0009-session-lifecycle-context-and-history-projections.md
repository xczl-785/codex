# 第九课：Session 的生命周期、上下文与界面历史

这一课回答四个彼此关联的问题：

1. `Session` 什么时候开始，什么时候结束？
2. 模型真正看到的上下文是怎样组装出来的？
3. 压缩替换了 Session 中的历史后，Thread 里的旧记录是否还在？
4. 界面上显示的记录究竟来自哪里？

贯穿这些问题的是同一种架构思想：**持久化内容负责保存事实，运行态内容负责高效执行；运行态应当能够由持久化事实重新构造。**

## 一、先建立总图

不要把 Thread、Session、模型上下文和界面历史理解成四份重复数据。它们是同一段工作在不同用途下的表示：

```text
持久化的 Thread / rollout
  ├─ 保存用户消息、Agent 消息、工具事件、Turn 边界等事实
  ├─ 保存压缩检查点及 replacement history
  └─ 可在进程重启后继续读取
                 │
                 ├─ 重建 Session 的 ContextManager
                 │      └─ 组装下一次模型 Prompt
                 │
                 └─ 重建 App Server 的 Turn / ThreadItem
                        └─ 转成 TUI transcript cells
```

可以先记住这四句话：

- Thread 是可长期保存、恢复和派生的工作记录与身份。
- Session 是某个 Thread 当前装入内存后的运行实体。
- 模型上下文是 Session 为某一次模型请求构造的有效视图。
- 界面历史是客户端从 Thread 记录构造的展示视图。

因此，**模型看见什么**与**用户界面显示什么**并不要求完全相同。

## 二、Session 什么时候开始？

Session 不是从用户提交一条消息时才开始，也不是每个 Turn 都重新创建。

当 Codex 创建、恢复或派生一个 Thread，并准备让它能够接收操作时，就会构造一个新的内存 `Session`。主要入口位于：

- `core/src/thread_manager.rs`：创建、恢复、派生和管理 Thread；
- `core/src/session/mod.rs`：建立通道并调用 `Session::new`；
- `core/src/session/session.rs`：构造 Session 本体和配套服务；
- `core/src/codex_thread.rs`：向外提供可投递 `Op`、接收 `Event` 的句柄。

构造过程不只是创建一个历史数组。它还会准备：

- 当前 Thread 和 Session 的身份；
- 配置、模型提供方与认证服务；
- MCP、工具路由、技能和插件相关服务；
- 执行管理器、环境管理、Hook 等运行设施；
- Session 状态、活动 Turn、输入队列和事件通道；
- 用于写入 rollout/Thread Store 的持久化对象；
- 新建、恢复或派生时所需的初始历史。

随后，Codex 会拉起 `submission_loop`。这个循环持续接收：

- 新的用户输入；
- Steer；
- Interrupt；
- Compact；
- Review；
- 审批结果；
- 配置或权限相关操作；
- Shutdown。

所以更准确的说法是：

> Session 从 Thread 被装载为可工作的内存运行体时开始，而不是从某个 Turn 开始。

## 三、Session 在 Turn 之间会不会结束？

不会。

一次普通 Turn 结束以后：

- 驱动它的 `RunningTask` 被收尾；
- `active_turn` 回到空闲状态；
- Session 仍然存在；
- Session 仍可接收下一次输入；
- 已有模型历史、配置和服务仍可复用。

因此 Session 可以经历：

```text
创建
  → 空闲
  → Turn 1 运行
  → 空闲
  → Turn 2 运行
  → 空闲
  → ……
  → 关闭
```

这正是 Session 与 Turn 分开的价值。如果每个 Turn 都重建 Session，Codex 就要重复恢复历史、启动服务、发现工具和准备环境，而且跨 Turn 状态会失去明确归属。

## 四、Session 什么时候结束？

从控制流看，Session 的主要生命周期由 `submission_loop` 维持。它通常在以下情况下结束：

- 收到 `Op::Shutdown`；
- 执行挂起并关闭的流程；
- 操作输入通道关闭，循环无法继续；
- 所在进程退出。

关闭并不只是让循环 `break`。源码中的运行时收尾还包括：

- 中止仍在运行的 Task；
- 关闭实时会话能力；
- 停止后台进程；
- 关闭 MCP 预热、刷新和连接；
- 关闭异步结果、Hook 与代码模式资源；
- 执行 Session 结束 Hook；
- 刷新并关闭持久化的 LiveThread；
- 发出关闭完成事件。

这里还要区分两个“结束”：

### 1. 逻辑运行结束

`submission_loop` 已退出，服务已收尾，Session 不再接收新操作。

### 2. 内存对象真正释放

Rust 中的 `Session` 被 `Arc` 持有。只有所有引用都释放后，对象内存才真正销毁。

这两个时刻可能接近，但概念上不同。

## 五、关闭 Session 不等于删除 Thread

Session 结束以后，Thread 的持久化记录仍然存在。以后可以读取 rollout，再创建一个新的 Session 运行对象。

因此，同一个逻辑 Thread 在时间上可能对应多个 Session 运行实例：

```text
Thread A
  ├─ Session runtime #1：本次进程中创建和执行
  ├─ 进程退出，runtime #1 消失
  └─ Session runtime #2：下次恢复 Thread A 时重新构造
```

恢复时保留的是 Thread 身份和持久化事实，不是把旧内存对象复活。

源码中 `SessionId` 还可能从 Session 元数据中恢复，子 Agent 场景也存在身份分组关系。因此要避免用“ID 是否相同”判断是不是同一个 Rust 内存对象。

另外，从 `ThreadManager` 的映射中移除 Thread，也不必然立即销毁 Session。其他地方如果还持有 `Arc<CodexThread>`，运行对象仍可能暂时存在。

## 六、“Session 的上下文”不是一个单独的大对象

日常说“Session 上下文”很容易让人误以为：Session 内部有一个完整字符串，每次原样交给模型。

实际结构更接近四层：

### 第一层：Session 级状态与服务

`SessionState` 保存跨 Turn 延续的状态，例如：

- 当前 Session 配置；
- `ContextManager` 管理的模型历史；
- 上一次 Turn 的设置；
- 额外上下文与世界状态基线；
- 自动压缩窗口；
- 已授予的权限和连接器选择。

Session 本体还持有模型、认证、MCP、执行管理、环境、Hook 等服务。

这一层回答的是：**这个 Thread 当前装入内存后，具有哪些长期运行条件？**

### 第二层：TurnContext

新 Turn 开始时，Codex 根据 Session 当前配置和这次 `turn/start` 的覆盖参数构造 `TurnContext`。

它会冻结或记录本轮需要一致的内容，例如：

- Turn ID；
- 模型和提供方；
- 初始与当前设置；
- 环境快照；
- 输出 Schema；
- 开发者指令；
- 时间与运行元数据。

这一层回答的是：**这一轮执行依据什么规则运行？**

### 第三层：StepContext

一个 Turn 可能多次调用模型。每一次模型采样和后续工具循环所依赖的稳定设置，由 `StepContext` 捕获。

它包括：

- 指向当前 `TurnContext` 的引用；
- 本次请求的 Token 预算；
- 本次采样使用的环境快照；
- 已最终确定的工具路由；
- MCP 绑定；
- AGENTS.md 与能力相关信息；
- 遥测和执行器发现结果。

这一层回答的是：**这一次具体模型请求使用哪套一致的能力和环境？**

### 第四层：真正发送给模型的 Prompt

在采样前，Codex 才组装模型请求：

```text
ContextManager 当前有效历史
  + 基础指令
  + 当前 Step 可见的工具说明
  + 输出 Schema
  + 当前模型能力与请求设置
  = 本次模型请求
```

`ContextManager::for_prompt` 会对历史进行规范化，并按照模型支持的输入模态处理内容。工具定义通常属于请求元数据，而不是简单追加成一条聊天消息；基础指令也有自己的请求字段。

所以不存在一份永远不变的“Session context 全文”。模型上下文是在每个 Step 开始前，由多个不同责任边界共同组装出来的。

## 七、新建与恢复时，历史怎样进入 Session？

新建 Thread 时，模型历史一开始可以为空。初始上下文可以推迟到第一个真正 Turn 再注入，这样本轮传入的设置覆盖能够参与最终组装。

恢复 Thread 时，Codex 会读取已有 rollout，并执行历史重建。重建过程需要恢复的不仅是可见消息，还包括：

- 最近有效的压缩检查点；
- 检查点之后的新事件；
- 上一次 Turn 的设置；
- reference context；
- world state 基线；
- 压缩窗口状态。

重建结果被安装到新的 `ContextManager` 中。此后 Session 不需要每次模型调用都从磁盘重放全部记录，而是使用这份内存中的有效投影。

这就是典型的“状态外置”思路：

- rollout 是可恢复的外部事实；
- ContextManager 是为当前运行优化的内存投影；
- Session 可以被丢弃，然后由事实重新构造。

## 八、压缩时到底发生了哪几件事？

一次自动压缩至少要从三个视角理解。

### 视角一：Session 的模型历史被替换

`replace_compacted_history` 会把 `ContextManager` 中原有的长历史替换成压缩后的 replacement history。

这意味着下一次模型采样不再携带全部旧细节，而是携带：

- 压缩后的摘要和必要保留项；
- 压缩后继续产生的新消息与工具结果。

这是真正减少模型上下文长度的地方。

### 视角二：rollout 追加一个压缩检查点

Codex 不会回头把旧 rollout 条目删除。它会追加一个 `Compacted` 条目，其中可以包含：

- 摘要消息；
- replacement history；
- 压缩窗口和来源信息；
- 恢复模型历史所需的相关检查点信息。

因此持久化结构更像：

```text
旧事件 1
旧事件 2
旧事件 3
Compacted checkpoint：以后重建模型上下文从这里采用 replacement history
新事件 4
新事件 5
```

这里发生的是**追加新解释规则**，而不是**原地擦除旧事实**。

### 视角三：恢复时跳过旧事件对模型上下文的影响

下次恢复 Session 时，历史重建逻辑会寻找最新仍然有效的压缩检查点：

1. 先用检查点中的 replacement history 作为模型历史基线；
2. 再重放检查点之后的新事件；
3. 检查点之前的旧事件不再重复加入模型可见历史。

所以旧事件仍在 rollout 中，但不再决定未来模型看见的完整上下文。

## 九、Thread 的旧记录为什么要保留？

如果压缩时直接覆盖持久化历史，会带来几个问题：

- 用户界面无法继续显示完整的既有对话；
- 调试时无法知道摘要是由什么事实产生的；
- 审计、兼容和问题定位能力下降；
- 一旦摘要质量有问题，旧信息已经不可恢复；
- 恢复逻辑与实时写入需要进行风险更高的原地修改。

追加检查点则把两种需求同时满足：

- 模型使用短而有效的上下文；
- Thread 保留完整的历史事实。

这种设计可以称为“事件日志加检查点”，也具有事件溯源的味道，但不必把 Codex 强行等同于一套教科书式完整 Event Sourcing 系统。

## 十、界面上的记录究竟是什么？

界面既不是直接显示 `ContextManager`，也不是直接把 rollout 原始 JSON 一条条打印出来。

它经历了至少两次投影：

```text
rollout / Thread Store
  → App Server 重建公开的 Turn 与 ThreadItem
  → TUI 转换成 transcript cell
  → 用户实际看到的聊天记录与活动摘要
```

### App Server 的投影

`ThreadHistoryBuilder` 会重放持久化项目，把它们整理成公开协议中的：

- Turn；
- UserMessage；
- AgentMessage；
- Reasoning；
- CommandExecution；
- FileChange；
- ToolCall；
- ContextCompaction；
- Turn 的完成、中断等状态。

遇到 `Compacted` 时，它不会删除此前已经重建出的 Turn 和消息，而是标记或产生压缩相关项目。

### TUI 的投影

TUI 再把 `ThreadItem` 转成适合展示的 transcript cell，例如：

- 用户消息单元；
- Agent Markdown 单元；
- Reasoning 摘要单元；
- 工具与命令活动单元；
- “context compacted” 标记。

展示层可以做进一步处理：

- 合并或概括工具活动；
- 隐藏未配置为可见的原始 reasoning；
- 清理 Markdown；
- 对旧历史分页加载；
- 对某些嵌套或内部项目做过滤。

所以界面是一个面向人的、有损的展示投影。它不等于模型的精确 Prompt，也不等于底层的完整持久化格式。

## 十一、压缩以后，模型与用户看到的历史为什么不同？

假设原始 Thread 是：

```text
用户：分析问题 A
Agent：进行了很多调查
工具：产生大量输出
用户：补充要求 B
Agent：继续实施
```

压缩后，模型下一次可能看到：

```text
系统与开发者指令
摘要：此前已调查 A，结论是……；用户又要求 B；当前进度是……
压缩后新产生的消息和工具结果
```

而界面仍可以显示：

```text
用户：分析问题 A
Agent：进行了很多调查
工具活动
用户：补充要求 B
Agent：继续实施
—— 上下文已压缩 ——
压缩后的新活动
```

这种差异不是数据不一致，而是两个视图在服务不同目标：

- 模型视图追求相关、受限和可继续推理；
- 用户视图追求可读、可回顾和交互连续性。

## 十二、与 Harness“状态外置”的关系

你的观察是准确的。Codex 在多个层次重复使用同一种原则：

| 持久或稳定侧 | 临时运行侧 | 运行侧的用途 |
| --- | --- | --- |
| Thread / rollout | Session | 接收操作、维护服务、执行多个 Turn |
| Turn 记录和协议事件 | RunningTask | 实际驱动这一轮执行 |
| rollout + compaction checkpoint | ContextManager | 为模型维护短而有效的历史 |
| ThreadItem | transcript cell | 为客户端提供可读展示 |

Harness 状态外置的核心要求是：执行器不应成为唯一事实来源。执行器可以崩溃、退出或迁移，只要外部事实充分，就可以重建新的执行器继续工作。

Codex 中的对应关系是：

- Session 不是 Thread 的唯一事实来源；
- Task 不是 Turn 的唯一事实来源；
- ContextManager 不是完整聊天记录的唯一事实来源；
- TUI 当前显示的 cell 更不是系统事实来源。

这些对象都是为了当前职责形成的投影或执行体。

## 十三、如果把这些层合在一起会怎样？

如果 Thread 与 Session 合并：

- 持久化记录会被迫携带大量不可序列化或无意义的运行资源；
- 进程退出就难以定义哪些状态可恢复；
- idle、loaded、running、closed 等状态容易混成一团。

如果 rollout 与 ContextManager 合并：

- 要么每次模型请求都重放全部日志，性能和复杂度较差；
- 要么压缩时破坏旧记录，使 UI、调试和恢复丢失事实。

如果模型上下文与 UI 历史合并：

- 为了界面完整，模型上下文会无限增长；
- 为了模型预算，界面历史又会被压缩抹掉；
- 两个目标天然冲突。

因此这些拆分并不是为了制造名词，而是在处理“事实保留、运行效率、模型预算、用户可读性”四种不同约束。

## 十四、本课结论

1. Session 在 Thread 被创建、恢复或派生为内存运行体时开始，不以某个 Turn 为起点。
2. Session 可以跨越多个 Turn，并在 Turn 之间保持空闲。
3. Session 在 submission loop 关闭和运行资源收尾时逻辑结束；Thread 持久记录并不因此删除。
4. 同一个 Thread 可以在不同时间由多个全新的 Session 运行实例承载。
5. 模型上下文由 Session 状态、TurnContext、StepContext、历史、指令、工具和 Schema 等在请求前组装，不是一份固定字符串。
6. 压缩替换的是 `ContextManager` 中未来模型要使用的有效历史。
7. 压缩同时向 rollout 追加检查点，旧事件通常仍然保留。
8. App Server 从 Thread 记录重建 Turn/ThreadItem，TUI 再把它们转换成展示单元。
9. 界面历史、模型上下文和底层 rollout 是三个不同目的的视图。
10. 这一整套结构可以统一理解为：运行态是持久事实的可重建投影。

## 回忆练习

不看上文，尝试回答：

1. 为什么关闭 Session 不等于删除 Thread？
2. 为什么压缩以后，界面还能显示压缩前的消息？
3. `ContextManager` 与 rollout 谁才承担完整历史事实来源？
4. 为什么 TUI transcript 不能被当成模型真实 Prompt？

