# 阶段复盘 0001：课程 0001—0015 整体知识地图

这份复盘用于把课程 0001—0015 从按课推进的线性顺序，重组为一张可以反复使用的 Codex 核心运行地图。它不引入下一课，也不替代原课程中的源码论证；需要深入某个局部时，仍应回到对应课程和当前分支源码。

## 一、先用一句话认识 Codex

Codex 不只是“聊天界面加大模型”，而是一个围绕长期状态、受控执行、失败反馈和恢复能力构建的 Agent Harness。

一条普通执行链可以先压缩成：

```text
用户输入
  ↓
TUI / App Server
  ↓
Thread / Session
  ↓
Turn / Task / Step
  ↓
模型生成 FunctionCall
  ↓
ToolRouter / ToolRegistry / Handler
  ↓
ExecPolicy / 审批 / 沙箱
  ↓
真实工具或操作系统进程
  ↓
FunctionCallOutput / Event
  ↓
下一 Step，或者 Turn 结束
  ↓
Rollout / Thread Store 保存可恢复事实
```

这条链不是简单地把请求逐层转发。每一层都在解决一种不同的问题：

- 客户端层负责接收、翻译和展示；
- 运行时层负责确定身份与生命周期；
- 模型循环负责形成下一步行动；
- 工具层负责把行动意图变成受控执行；
- 安全层负责限制能力；
- 持久化和事件层负责恢复、审计和多种展示。

## 二、五个主题簇

### 1. 执行生命周期：0001、0002、0003、0006、0008

这一组课程回答：一项用户委托怎样成为 Thread 中的一段连续执行。

核心关系是：

```text
Thread：长期工作身份
└─ Session：Thread 当前加载后的内存运行体
   └─ Turn：一次连续工作的领域边界
      └─ RunningTask：当前执行程序的运行控制壳
         └─ SessionTask 实现：Regular / Review / Compact
            └─ Step：一次模型采样使用的稳定请求快照
               └─ Tool Call：Step 中发起的一项具体行动
```

这里最容易犯的错误是按照聊天气泡、函数调用或工具数量划分 Turn。真正的判断依据是：当前是不是还在完成同一份连续委托。

因此，下列行为通常仍属于同一个 Turn：

- 模型调用工具后继续采样；
- 一个 Step 执行一个或多个工具；
- 用户成功 Steer 当前执行；
- Turn 中途进行自动压缩；
- 长命令 yield 后，后续再轮询同一个进程。

而 Turn 完成、中断或失败后再次提交输入，通常会产生新 Turn 和新 Task。

### 2. 历史、恢复与投影：0005、0007、0009、0013

这一组课程回答：系统保存的历史、模型当前看到的内容、界面展示和工作区现实状态为什么不是同一份东西。

```text
Rollout
  保存消息、事件、Turn 边界和压缩检查点等持久事实

ContextManager
  维护未来模型采样实际使用的有效历史

App Server ThreadItem
  从 Thread 记录重建客户端可见的协议历史

TUI transcript
  把协议项目转换为面向用户的展示单元

Workspace
  保存文件、进程、数据库和远端系统等外部现实状态
```

自动压缩可以替换 ContextManager 中的模型有效历史，同时向 rollout 追加压缩检查点。压缩前的旧事实通常仍被保留，所以界面还能显示旧内容；但这不意味着模型仍在当前 Prompt 中直接看到它们。

Fork、Revert 和文件 checkpoint 也必须按状态层分别判断。Codex 的 Thread Revert 可以调整对话历史，却不会天然恢复工作区文件，更不能回滚数据库、进程或远端 API 副作用。

### 3. 模型—工具闭环：0010、0011

这一组课程回答：模型已经生成结构化 Function Call 后，Codex 怎样把它变成可靠、可纠错的真实行动。

```text
当前 Step 构造 ToolSpec 和 ToolRouter
  ↓
模型返回 FunctionCall
  ↓
ToolRouter 识别内部 ToolCall
  ↓
ToolRegistry 查找实现
  ↓
Handler 解析参数并检查业务条件
  ↓
工具执行、拒绝或失败
  ↓
FunctionCallOutput 写回历史
  ↓
同一个 Turn 的下一 Step 再次采样
```

ToolRouter 的定稿范围是一个 Step，而不是整个 Turn。Step 之间可以吸收新的设置、MCP Binding、插件和环境状态；Step 内则必须保证模型看到的工具说明与真正执行调用的 Router 属于同一快照。

FunctionCall 和 FunctionCallOutput 通过 `call_id` 形成因果关系。完整记录不会创造并行能力，也不保证自动重跑未完成调用；它提供的是可配对、可审计、可恢复判断的事实。

### 4. 权限与安全执行：0004、0012

这一组课程回答：为什么结构化工具调用仍然不能直接执行。

一项工具调用需要跨越多层判断：

```text
JSON 语法是否合法
  ↓
参数是否符合工具 Schema
  ↓
工具是否存在，参数是否具有业务意义
  ↓
ExecPolicy 是 Allow、Prompt 还是 Forbidden
  ↓
用户和宿主策略是否允许提升
  ↓
沙箱实际开放哪些能力
  ↓
操作系统或外部服务是否执行成功
```

模型输出应当被看成“带有意图但不可信的提案”。Tool Calling 主要解决协议表达问题；Handler、审批、策略、沙箱和真实环境共同决定行动是否成立。

### 5. 真实进程与输出：0014、0015

这一组课程回答：为什么一次短 Tool Call 可以启动一个更长寿命的真实进程，以及不同消费者怎样观察它。

```text
exec_command Tool Call
  ├─ 进程很快结束：直接返回最终结果
  └─ 到达 yield 边界：Tool Call 返回，真实进程继续存活
                              ↓
                       write_stdin Tool Call
                       继续输入或轮询同一进程
```

同一个命令形成三种主要输出视图：

- UI 实时 delta：让用户及时看到进展；
- 模型 Tool Output：让模型在 Step 边界决定下一步；
- CommandExecutionItem：让客户端得到聚合后的命令生命周期。

对应的三个身份不能混淆：

- `call_id` 识别一次模型工具调用；
- `process_id` 识别一个真实进程；
- `chunk_id` 识别一次有界输出返回。

轮询同一个 `process_id` 不是重试命令。若把 poll 当成 retry，就可能重复启动进程和重复产生副作用。

## 三、贯穿全部课程的四条设计原则

### 原则一：生命周期不同的对象不能强行合并

Thread、Session、Turn、Task、Step、Tool Call 和真实进程的开始、结束与恢复条件不同。

例如，Thread 可以跨进程恢复；旧 Session 的异步任务和运行资源却已经失效。Turn 完成后仍是历史事实；RunningTask 的取消令牌和 JoinHandle 则只在运行期间有意义。一次 Tool Call 已经返回时，它启动的真实进程还可能继续存在。

如果把这些对象合并：

- 持久化数据会混入不可序列化或已经失效的运行资源；
- 恢复时无法判断哪些内容可以重建；
- 历史 Turn 会携带大量无意义的运行字段；
- 长进程只能让 Tool Call 永久悬挂，或者被错误地重复启动。

### 原则二：“系统保存”不等于“模型看得见”

Rollout、ContextManager、ThreadItem、TUI transcript 和 Workspace 都在描述“过去发生了什么”，但它们服务于不同目标。

- Rollout 服务于持久化、恢复和审计；
- ContextManager 服务于下一次模型请求和 Token 预算；
- ThreadItem 服务于稳定客户端协议；
- TUI transcript 服务于用户阅读；
- Workspace 保存对话系统之外的现实副作用。

如果只维护一份万能历史：

- 为保证界面完整，模型上下文会不断增长；
- 为控制模型预算，旧审计事实又会被删除；
- 展示格式会反过来污染模型 Prompt；
- 对话回退容易被误解为工作区事务回滚。

因此分析压缩、恢复、Fork 或 Revert 时，应固定追问：哪一层改变了，哪一层仍然保留？

### 原则三：每跨过一层信任边界，都必须重新校验

结构化输出只证明模型按照某种机器协议表达了意图，不能证明意图正确、被授权或能够成功。

```text
合法 JSON
≠ 合法工具参数
≠ 正确的工具选择
≠ 当前环境中有意义的操作
≠ 已获得用户授权
≠ 操作系统允许执行
≠ 现实执行成功
```

如果相信模型第一次输出永远正确：

- 不存在的工具和错误参数会直接穿透；
- 过期路径、环境变化和并发状态无法处理；
- 权限判断会退化为模型自己给自己授权；
- 非致命失败无法反馈给模型修正。

可靠性来自“结构约束、运行时验证、权限强制、失败反馈、模型修正”组成的闭环。

### 原则四：不同消费者需要不同视图

用户、模型、客户端和持久化系统关心的不是同一件事。

- 用户关心进程现在有没有进展；
- 模型关心这次调用得到了什么有界结果，以及下一步做什么；
- 客户端关心一条命令从开始到结束的完整状态；
- 持久化系统关心恢复时真正需要哪些因果事实。

如果所有消费者共享一种输出：

- 只保留实时 delta，模型上下文会被碎片淹没；
- 只保留 Tool Output，界面无法及时更新；
- 只保留最终命令项，模型必须等进程结束才能行动；
- 依赖输出顺序配对，在并行和乱序完成时会失效。

因此 Codex 使用事件、Tool Output、客户端项目以及多种 ID，为每类消费者提供适合的投影，同时保留它们之间的关联。

## 四、重复内容怎样整理

现有重复多数属于逐层加深，不建议因此删除原课程。阶段复盘中可以按下列方式合并：

| 重复主题 | 涉及课程 | 复盘时的统一入口 |
| --- | --- | --- |
| Thread、Session、Turn、Task 的定义与边界 | 0001、0003、0006、0008 | 生命周期关系图与对象矩阵 |
| 压缩、恢复、回退与历史视图 | 0005、0007、0009、0013 | 持久事实—运行投影—展示投影—外部状态 |
| 模型调用工具后继续采样 | 0006、0010、0011 | FunctionCall—Output—下一 Step 主循环 |
| 沙箱不是预演或事务 | 0005、0012、0014 | 受限但真实的执行原则 |
| Call/Output 不创造并行或自动恢复 | 0011、0015 | 显式身份与因果记录原则 |

第 0004 课不宜并入某个类型说明，因为它承担的是架构分析方法训练。第 0013 课是外部系统对照，应保留为扩展视角，而不是 Codex 当前实现的事实来源。

## 五、当前理解强项与薄弱点

### 已经形成的稳定认识

- 能从生命周期和持久化方式解释 Thread、Session、Turn、Task 的拆分；
- 不再按照聊天气泡或工具调用次数判断 Turn；
- 能区分持久事实、模型有效历史和界面投影；
- 能区分系统保存、模型直接可见和模型按需检索；
- 知道 Tool Calling 不能替代 Harness 运行时校验；
- 已理解 Step 间刷新、Step 内冻结的工具快照；
- 已理解沙箱是真实受限执行，不是预演或自动回滚；
- 评价 checkpoint 时会检查捕获范围、所有权和未覆盖副作用。

### 已学过但还需要贯通的概念

1. Thread Store、rollout、ContextManager 与 App Server 历史之间的精确重建链路。
2. Fork、恢复和 Revert 中 Thread、Session 与历史身份怎样变化。
3. `call_id`、`process_id`、`chunk_id` 在一个完整长命令中的对应关系。
4. 普通工具失败、沙箱拒绝、受控提升重试和模型主动重试之间的区别。
5. Call/Output 因果记录与并行调度、未完成调用恢复之间的严格边界。

### 尚未系统学习，不能算旧课薄弱点

- 取消怎样从 Turn、Task 传播到工具和真实进程；
- 并行工具怎样调度、串行化冲突并处理乱序完成；
- Subagent 怎样创建独立 Thread、通信、汇总状态和回收资源。

## 六、仍需用源码确认的问题

下列问题暂时保留为后续局部静态阅读入口：

1. Thread Store、rollout 文件和 App Server 分页历史各自保存或重建哪些字段。
2. Fork、恢复和 Subagent 创建时，Thread ID、Session source、父子身份及配置怎样传播。
3. 取消令牌如何穿过 RunningTask、工具调度器、Unified Exec 和操作系统进程。
4. 并行工具的读写锁、Future 调度、输出配对和取消边界。
5. Unified Exec 不同历史模式下，实时事件、ResponseItem 与 CommandExecutionItem 的持久化差异。

这些问题不需要一次性通读整个目录。应先由综合场景暴露真正不牢固的连接点，再沿对应源码入口核对。

## 七、综合自测场景

先不看源码，尝试分析下面的过程：

> Codex 恢复一个旧 Thread，用户提交新请求。模型启动一个长命令；第一次 `exec_command` 到达 yield 边界后返回，但真实进程继续存活。用户此时追加一条 Steer。随后 Turn 发生自动压缩，模型再通过 `write_stdin` 观察原进程，最后完成回复。

需要回答：

1. 哪些 Thread、Session、Turn、Task 和 Step 是沿用的，哪些是新建的？
2. 整个场景有几个 Tool Call、几个真实进程？
3. 哪些 `call_id`、`process_id`、`chunk_id` 应保持相同，哪些应变化？
4. 自动压缩改变了哪一层历史，没有改变哪一层？
5. 为什么 UI 能持续显示输出，而模型不是一直“盯着终端”？
6. Steer 在什么时间点进入模型历史，为什么不一定立刻影响当前 Step？
7. 如果中途发生沙箱拒绝，什么情况下可以重试，为什么不能无脑重放？

这组问题不是考试。它的用途是定位不同主题之间尚未接上的地方，再决定下一阶段优先进入取消传播、并行工具调度还是 Subagent 生命周期。

## 八、主要源码入口

- `codex-rs/core/src/thread_manager.rs`：Thread 创建、恢复和内存管理。
- `codex-rs/core/src/session/session.rs`：Session 服务、状态和活动 Turn。
- `codex-rs/core/src/state/turn.rs`：ActiveTurn、RunningTask 与 TaskKind。
- `codex-rs/core/src/tasks/mod.rs`：SessionTask、启动、取消与统一收尾。
- `codex-rs/core/src/session/turn.rs`：Step、模型采样、工具结果、Steer 和自动压缩循环。
- `codex-rs/core/src/tools/router.rs`：模型响应到内部 ToolCall。
- `codex-rs/core/src/tools/registry.rs`：工具查找、Handler 执行与结果转换。
- `codex-rs/core/src/tools/orchestrator.rs`：审批、沙箱 attempt 和受控提升重试。
- `codex-rs/core/src/unified_exec/process_manager.rs`：长进程保存、轮询、状态和清理。
- `codex-rs/core/src/unified_exec/async_watcher.rs`：实时输出与退出观察。
- `codex-rs/rollout/src/policy.rs`：不同事件和响应项目的持久化边界。
- `codex-rs/app-server-protocol/src/protocol/event_mapping.rs`：核心事件到客户端通知的映射。

## 九、阶段记忆锚点

遇到任何 Codex 架构问题，先问四组问题：

1. **生命周期**：它何时创建、何时结束、能否恢复？
2. **事实来源**：哪份数据是持久事实，哪份只是运行或展示投影？
3. **信任边界**：谁提出意图，谁校验，谁授权，谁最终强制执行？
4. **消费者视图**：模型、用户、客户端和恢复系统分别需要看到什么？

只要这四组问题回答清楚，大多数模块边界和设计取舍就能自然推导出来，而不必先记住大量函数名。
