# 第十三课：Claude Code、Cursor 与 LangGraph 的 Checkpoint 模型

Checkpoint 不是统一技术名词。三个系统保存的对象不同：

```text
Claude Code：会话节点 + 被文件编辑工具触及的文件快照
Cursor：请求节点 + Agent 文件修改的本地快照
LangGraph：Thread 在每个 super-step 边界的完整图状态
```

前两者主要服务于编码工作区的快速撤销；LangGraph 主要服务于工作流恢复、人工中断、短期记忆和 time travel，并不自动保存文件系统。

> 名称边界：项目可能对外称为“使用 LangChain”，但其 Agent 运行时和 checkpoint 由底层 LangGraph 提供。只有确认代码导入、checkpointer 配置或数据库表后，才能判断大体积记录究竟是 LangGraph checkpoint，还是普通 LangChain 消息历史。本文分析的是前一种情况，不把它当作对某个未核对项目的确定诊断。

## 一、Claude Code：会话记录与文件快照并行

Claude Code 把每条消息、工具调用和结果写入本地 JSONL 会话记录，同时在 Agent 通过 `Write`、`Edit`、`NotebookEdit` 等文件编辑工具修改文件前保存文件内容。每个用户 Prompt 形成可选择的 checkpoint，checkpoint UUID 与会话节点关联。

它并非简单地为每个 checkpoint 复制整个项目。官方文档说明：会话中保留最近 100 个 checkpoint 的文件快照；删除旧 checkpoint 时，只删除已不被任何 checkpoint 引用的快照文件。这表明 checkpoint 与文件备份之间存在引用和复用关系。

回退时可以独立选择：

- 只恢复代码；
- 只恢复对话；
- 同时恢复代码和对话；
- 只压缩指定位置前后的一段对话。

文件恢复会删除会话中后来创建、且被系统跟踪的文件，并把修改过的文件恢复到目标内容。它不会覆盖 Bash 命令直接造成的文件变化、远端副作用、软链接和硬链接，也通常不覆盖后台 Subagent 或其他并发会话造成的修改。

连续回退并不是只能执行“撤销上一步”。用户可以直接选择任意仍被保留的 Prompt checkpoint。只恢复代码时，对话时间线仍在，因而之后还可选择其他 checkpoint；恢复对话时，当前有效会话回到旧节点。若希望明确保留原路线并尝试替代方案，应使用 session branch，而不是把 rewind 当成可靠的双向版本控制。

## 二、Cursor：公开契约明确，内部格式未公开

Cursor 官方只保证：checkpoint 自动保存 Agent 对代码库的修改，存放在本地、独立于 Git，只跟踪 Agent 修改而不跟踪手工编辑，并会自动清理。用户可以从以前的请求或消息节点选择 `Restore Checkpoint`。

Cursor 是闭源产品，公开资料没有说明底层究竟保存完整文件、反向 patch、内容寻址 blob，还是这些方式的组合。因此只能确认产品契约，不能把某种推测写成实现事实。

从行为上可以把它理解成：

```text
请求 C0 -> 请求 C1 -> 请求 C2 -> 请求 C3
                     \-> 每个节点关联 Agent 文件变化的恢复点
```

选择 C1 表示把 Cursor 已跟踪的 Agent 修改恢复到 C1 对应的状态，不代表 Git 历史、手工修改、数据库或外部副作用一起回退。连续回退仍应直接选择目标 checkpoint，而不是依赖重复点击“撤销”恰好逐级返回；官方没有公开恢复后旧 checkpoint 的物理保留、分支或重做细节。

## 三、LangGraph：保存的是整个图运行状态

LangGraph 编译图时可以注入 `checkpointer`。每个 Thread 由 `thread_id` 标识；图在每个 super-step 边界保存一个 checkpoint。一个 super-step 是图的一次调度 tick，其中可以有多个节点并行执行。

典型 checkpoint 包含：

- `checkpoint_id`、时间和父 checkpoint；
- `channel_values`：当时所有状态 channel 的值；
- `channel_versions`：每个 channel 的版本；
- `versions_seen`：各节点已经观察过的版本，用于判断后续调度；
- metadata：来源、step、节点 writes 和 parents；
- 下一步任务、interrupt、error 等可恢复运行信息；
- 当前 super-step 内节点已经完成的 pending writes。

持久化后端可以是内存、SQLite、PostgreSQL、Redis、MongoDB 等，并不必然是数据库；生产环境通常使用持久化数据库。接口核心是 `put`、`put_writes`、`get_tuple`、`list` 和 `delete_thread`。

### 为什么连对话也会保存

LangGraph 不把“对话记录”视为独立于 checkpoint 的特殊日志。如果图状态定义了：

```python
class State(TypedDict):
    messages: Annotated[list, add_messages]
```

那么 `messages` 就是普通的 state channel。它累加用户消息、模型消息以及可能的 tool message；因此每个 checkpoint 的 `channel_values` 都会包含当时累计的消息状态。对话进入数据库，是因为它属于图状态，而不是因为 checkpointer 额外复制了一份聊天表。

### 恢复、Replay 与 Fork

使用 `thread_id` 可以取得最新 checkpoint；同时提供 `checkpoint_id` 可以精确定位历史状态。

LangGraph 的 time travel 通常不删除后来的数据库记录：

- Replay：用旧 checkpoint 的 config 再次 `invoke`。旧 checkpoint 以前的节点不重跑，之后的节点重新执行，LLM、API 和 interrupt 也会再次发生。
- Fork：先在旧 checkpoint 上调用 `update_state`。它不会修改旧 checkpoint，而会创建一个以旧 checkpoint 为父节点的新 checkpoint，再从新状态继续。

因此历史更接近一棵树，而不是只能前后移动的栈：

```text
C0 -> C1 -> C2 -> C3
       |
       +-> C4 -> C5
       |
       +-> C6
```

从 C1 replay 后产生 C4/C5，原来的 C2/C3 仍在。之后还可以再次选择 C0 或 C2 创建其他路线。所谓连续回退，本质是连续选择历史 checkpoint 作为新的执行基点，而不是不断破坏性删除“最新一条”。客户端如果只读取同一 Thread 的最新 checkpoint，会看到最新路线；要展示完整时间旅行，需要读取 state history 和 parent relationship。

## 四、为什么 LangGraph Checkpoint 会非常巨大

默认情况下，LangGraph 在每个 super-step 保存每个 state channel 的完整值。若 `messages` 每轮都增长，而每个 checkpoint 又保存累计消息，就会出现：

```text
CP1 保存 1 份消息
CP2 保存 2 份消息
CP3 保存 3 份消息
...
CPn 保存 n 份消息
```

逻辑数据量是：

```text
1 + 2 + ... + n = n(n + 1) / 2
```

所以长期对话可能接近平方增长，而不是简单线性增长。以下因素会进一步放大数据：

- 一个用户轮次内包含多个 super-step；
- ToolMessage 保存大段终端输出、网页、检索结果或结构化数据；
- state 中直接放入文档、DataFrame、二进制或大对象；
- 并行节点产生 pending writes；
- 子图使用独立 checkpoint namespace；
- replay/fork 保留多条历史分支；
- 没有 retention 或旧 checkpoint 清理策略。

PostgresSaver 会按 channel version 存放非简单值的 blob，并复用未变化的 channel 版本，因此物理数据库不一定机械地复制所有相同字节。但持续追加的 `messages` 每次都会形成新版本，无法从根本上消除累计状态增长。新版 LangGraph 提供 beta `DeltaChannel`，可对追加型 channel 保存增量，不过会引入读取重建成本和版本约束。

常见治理方式：

1. 让图状态保持精简，大型文档和工具结果放对象存储，只在 state 中保存引用。
2. 真正修改 state 中的消息：删除、裁剪或用摘要替换旧消息。只在调用模型前临时 trim，而 checkpoint 仍保存完整 `messages`，不会降低数据库体量。
3. 为 checkpoint 设置 retention，定期清理旧 Thread 或不再需要的分支。
4. 评估 `DeltaChannel` 是否适合追加型数据，并接受恢复时重建增量的代价。
5. 区分 Checkpointer 与 Store：前者保存 Thread 运行状态；后者保存跨 Thread 的长期业务记忆。不要把所有长期数据都塞进 graph state。

## 五、三者与 Codex 的位置

| 系统 | Checkpoint 的核心对象 | 能否恢复 Agent 执行状态 | 能否恢复工作区文件 |
| --- | --- | --- | --- |
| Claude Code | 会话节点和被编辑工具触及的文件 | 主要恢复对话位置 | 可以，范围受工具边界限制 |
| Cursor | 请求节点和 Agent 文件变化 | 公开细节有限 | 可以恢复被跟踪的 Agent 修改 |
| LangGraph | 完整 graph state 和调度进度 | 可以，这是主要目的 | 默认不可以 |
| Codex | rollout 持久事件与可重建 Session | 可从历史恢复 Session，支持 Thread Revert/Fork | 当前不随 Thread 恢复 |

LangGraph checkpoint 是 Codex 当前没有的一层通用能力：它保存任意工作流状态、节点版本和待执行任务，使图能从精确的 super-step 恢复。Codex 则以追加式 rollout 和领域事件重建 Agent 上下文，没有为任意用户定义 state channel 提供通用数据库快照。

## 六、设计评价：为什么 Claude Code 显得更清爽

以下是学习过程中的设计评价，不是产品能力的客观排名。

Claude Code 的优势在于主动限定了 checkpoint 的所有权：只承诺恢复当前 Session 中由受控文件编辑工具捕获的改动，同时把“只恢复对话、只恢复代码、同时恢复二者”作为不同选择暴露出来。它不声称 Bash、后台 Subagent、并发编辑和外部系统也能被完整回滚。这种边界使正常路径容易理解，失败边界也较明确。

Codex 当前采用更保守的选择：Thread Revert/Fork 不碰工作区，彻底避免误删用户或并发 Agent 的文件变化，但也把对话与文件同步恢复的责任全部留给客户端、Git 或用户。安全边界很明确，产品体验上却缺少统一 checkpoint。

Cursor 提供 Agent 文件恢复点，但公开资料没有给出清晰的连续 Undo、分支和 Redo 模型。它可以解决短期误改，却较难仅凭公开契约建立一条可推理的恢复时间线。

因此，“清爽”来自三个因素：

1. checkpoint 的捕获范围固定且可说明；
2. 对话状态与文件状态分离，但允许显式组合恢复；
3. 没有被捕获的副作用明确留在机制之外，不制造完整事务的错觉。

LangGraph 解决的是另一类需求。它的通用图状态 checkpoint 更强，却要求应用开发者自行约束 state 大小、保留周期和分支数量，不能直接用编码工作区 Undo 的简洁性评价。

## 资料来源

- [Claude Code Checkpointing](https://code.claude.com/docs/en/checkpointing)
- [Claude Agent SDK file checkpointing](https://code.claude.com/docs/en/agent-sdk/file-checkpointing)
- [Claude Code 工作方式与本地 JSONL 会话](https://code.claude.com/docs/en/how-claude-code-works)
- [Cursor Checkpoints](https://docs.cursor.com/en/agent/chat/checkpoints)
- [LangGraph Persistence](https://docs.langchain.com/oss/python/langgraph/persistence)
- [LangGraph Time Travel](https://docs.langchain.com/oss/python/langgraph/use-time-travel)
- [LangGraph Memory](https://docs.langchain.com/oss/python/langgraph/add-memory)
- [LangGraph Checkpointer 接口与存储结构](https://github.com/langchain-ai/docs/blob/main/src/oss/langgraph/checkpointers.mdx)
