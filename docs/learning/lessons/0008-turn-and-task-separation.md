# 第八课：Turn 与 Task 为什么通常一对一仍要分开

这一课解决两个容易纠缠的问题：`SessionTask` 和 `RegularTask` 是什么关系；既然一个活动 Turn 通常只有一个 Task，为什么不把 Turn 与 Task 合并。

## 一、先把四个带 Task 的名字摆正

当前源码里至少有四个容易混淆的名字：

```text
SessionTask   共同执行契约（trait）
RegularTask   普通对话/实施工作的具体实现
RunningTask   某个 Task 启动后的运行控制包装器
TaskKind      方便判断工作种类的枚举标签
```

它们不是四层业务实体，而是“接口、实现、运行实例包装、分类标签”。

## 二、SessionTask：所有 Turn 驱动器的共同契约

`SessionTask` 是 Rust trait。可以先把它类比成其他语言里的接口或抽象基类：

```rust
trait SessionTask {
    fn kind(&self) -> TaskKind;
    fn span_name(&self) -> &'static str;
    async fn run(
        self,
        session,
        turn_context,
        input,
        cancellation_token,
    ) -> SessionTaskResult;
    async fn abort(&self, session, turn_context);
}
```

它不描述“普通 Agent 应该怎样调用模型”，只规定任何一种可调度工作都必须回答：

- 我是什么类型；
- 我的追踪名称是什么；
- 我怎样运行；
- 我被中断时怎样清理。

源码注释对它的定位很直接：这是一个驱动 `Session` 中某次 Turn 的异步 Task。

如果用面向对象语言表达，可以近似写成：

```text
abstract class SessionTask {
    abstract kind()
    abstract run(...)
    virtual abort(...)
}
```

所以用户提出的“主 Task 加若干子类”思路，在执行策略这一层，当前代码实际上已经采用了。

## 三、RegularTask：SessionTask 的普通实现

`RegularTask` 是一个实现了 `SessionTask` 的具体类型：

```text
SessionTask
├─ RegularTask
├─ ReviewTask
├─ CompactTask
└─ UserShellCommandTask 等其他内部工作形态
```

Rust 没有传统类继承，这里使用 trait 多态：

```rust
impl SessionTask for RegularTask { ... }
impl SessionTask for ReviewTask { ... }
impl SessionTask for CompactTask { ... }
```

`RegularTask` 的职责是实现普通 Agent Turn：

1. 发出 `TurnStarted`；
2. 处理启动预热；
3. 调用 `run_turn`；
4. 驱动多次 Step、模型采样和工具调用；
5. 在边界处吸收仍属于当前 Turn 的 pending input；
6. 返回最后助手消息或错误。

当前 `RegularTask` 本身是零字段结构体，因为大部分运行材料通过 `run` 参数传入，不需要存在 Task 对象内部。

## 四、ReviewTask 与 CompactTask 不是 RegularTask 的子类

这里要修正“继承树”的一个细节。

更合适的关系不是：

```text
MainTask
└─ RegularTask
   ├─ ReviewTask
   └─ CompactTask
```

而是并列实现：

```text
SessionTask
├─ RegularTask
├─ ReviewTask
└─ CompactTask
```

原因是 Review 和 Compact 不是“更特殊的普通聊天算法”。

- `RegularTask` 运行正常模型与工具循环，并允许 Steer；
- `ReviewTask` 启动受限的评审对话，处理评审事件和退出 Review Mode；
- `CompactTask` 把历史压缩作为独立 Turn，不能接收普通 Steer。

它们共享的是启动、取消和完成契约，不共享具体算法。若让 Review/Compact 继承 Regular，反而容易继承不适用的行为，例如普通 Steer、普通工具循环或普通最终消息处理。

## 五、AnySessionTask：为了运行时多态存在的适配层

源码还定义了 `AnySessionTask`，并把任何 `T: SessionTask` 转成它：

```text
RegularTask
→ 实现 SessionTask
→ 适配为 dyn AnySessionTask
→ 放入 RunningTask.task
```

这是一个较技术性的 Rust 适配层：不同 Task 的异步 `run` 返回具体 Future，而 `RunningTask` 需要用统一的 trait object 保存未知具体类型。`AnySessionTask` 把 Future 装箱，从而允许：

```rust
Arc<dyn AnySessionTask>
```

对学习架构来说，可以暂时把它看成编译语言为动态分派增加的接口适配，不必把它当成新的业务概念。

## 六、RunningTask：执行策略启动后的控制壳

`RegularTask` 或 `CompactTask` 只是执行策略。它被 `start_task` 启动后，会包装成 `RunningTask`：

```text
RunningTask
├─ kind
├─ task: Arc<dyn AnySessionTask>
├─ cancellation_token
├─ Tokio 异步执行 handle
├─ done 完成通知
├─ turn_context
├─ Agent 执行配额守卫
├─ 活动 Turn 诊断守卫
└─ 总耗时计时器
```

因此可以类比：

```text
RegularTask 是“程序/策略”
RunningTask 是“这个程序现在跑起来后的进程控制块”
```

同一个 `RegularTask` 类型可以在不同 Turn 中反复实例化和运行；每一次运行都有不同的 `RunningTask`、`TurnContext`、取消令牌和异步句柄。

## 七、Turn 在这里不是另一种执行器

Turn 表示“这一轮工作本身”，是领域和协议概念。

App Server 对外的 `Turn` 数据包含：

```text
Turn
├─ id
├─ items
├─ items_view
├─ status
├─ error
├─ started_at
├─ completed_at
└─ duration_ms
```

客户端关心的是：

- 这是哪一轮；
- 这一轮产生了哪些消息、推理摘要和工具项；
- 正在运行、完成、失败还是中断；
- 何时开始和结束；
- 能否恢复、展示、回放或作为历史读取。

客户端不应该接触 Tokio 句柄、取消令牌或 Rust trait object。

核心内部没有把所有 Turn 信息塞在一个大结构体里，而是按职责拆开：

```text
TurnContext：本轮采用的配置、模型、环境、权限和计时等上下文
TurnState：审批、pending input、工具计数等可变运行状态
ActiveTurn：Session 当前活动 Turn 的槽位
RunningTask：负责驱动本轮的执行对象
协议/历史 Turn：可观察和可持久化的一轮结果
```

这也是阅读源码时感觉 Turn “散落在很多地方”的原因。

## 八、一个 Turn 是否真的只有一个 Task

对当前 Session 的正常活动 Turn，可以先记：**是，一个活动 Turn 由一个 `RunningTask` 驱动。**

结构上：

```rust
struct ActiveTurn {
    task: Option<RunningTask>,
    turn_state: Arc<Mutex<TurnState>>,
}
```

不是 `Vec<RunningTask>`，所以同一个 Session 的同一个 ActiveTurn 不会并排挂多个 SessionTask。

不过要加三条边界说明。

### 1. “通常一对一”不等于生命周期完全重合

Task 完成时，`on_task_finished` 会先从 `ActiveTurn.task` 取走 `RunningTask`，然后继续统计、发出 `TurnComplete`、刷新 rollout 和清理 ActiveTurn。因此收尾的短暂阶段里，可以存在：

```text
ActiveTurn 仍存在
RunningTask 已经取走
Turn 终态事件仍在生成
```

### 2. 历史 Turn 没有 RunningTask

一个 Thread 可以保存很多已经完成的 Turn，但这些 Turn 只是历史和协议数据，不再拥有 Tokio handle、取消令牌或 Task 对象。

因此从整个生命周期看：

```text
运行中 Turn：通常有一个 RunningTask
已完成 Turn：有 Turn 记录，没有 RunningTask
```

### 3. Task 可以创建子运行体

例如 `ReviewTask` 会启动一个受限的子 Codex 对话。父 Session 的当前 ActiveTurn 仍只有一个 `ReviewTask`；子对话拥有自己的 Session/Turn/Task。不能把子对话的 Task 计入父 ActiveTurn 的 Task 数量。

## 九、为什么一对一仍然不应该合并

一对一只说明数量关系相近，不说明职责相同。

可以类比：

```text
HTTP Request 与处理它的 Future 通常一对一
订单与履约流程实例通常一对一
进程退出记录与运行中的进程通常一对一
```

但不会因为一对一，就把可长期保存的业务记录和只能在内存运行的执行控制对象做成同一个类型。

### Turn 是“发生了什么”

它需要：

- 稳定 ID；
- 协议状态；
- items；
- 开始、完成和错误；
- 持久化、恢复、回放；
- 被 TUI、App Server 和 SDK 读取。

### Task 是“怎样把它跑完”

它需要：

- 多态执行策略；
- Tokio Future 和 handle；
- 取消令牌；
- abort 清理；
- 运行配额和诊断守卫；
- 对 Session 服务的访问。

前者是可观察的工作记录，后者是短命的执行机制。

## 十、如果把 Turn 和 Task 合并会怎样

假设设计一个大类型：

```text
TurnTask
├─ id、items、status、timestamps
├─ model、permissions、environment
├─ Regular/Review/Compact 分支
├─ cancellation token
├─ Tokio handle
├─ done notifier
└─ run/abort
```

很快会遇到问题。

### 1. 持久化对象混入不可持久化运行资源

Turn 需要写入历史和通过 JSON 返回；Tokio handle、trait object、锁和取消令牌不能也不应该序列化。

### 2. 已完成 Turn 携带无意义字段

完成后，Turn 的 items、状态和时间仍有价值，但执行句柄、取消令牌和 Task 策略已经失效。合并后会产生大量“仅运行时存在”的可选字段。

### 3. 协议被具体执行方式污染

客户端只需知道 Turn 状态，不应知道核心是 `RegularTask` 的循环、Review 子会话还是远端压缩实现。否则更换执行策略会变成协议破坏性变更。

### 4. 执行策略容易变成巨型分支

若把所有工作都写进 Turn：

```rust
match turn.kind {
    Regular => ...,
    Review => ...,
    Compact => ...,
}
```

启动、运行、清理、测试和扩展都会不断扩大同一个核心类型。当前 `SessionTask` trait 允许增加独立实现，同时复用 `start_task` 的统一生命周期壳。

### 5. 生命周期事实上并不完全一致

RunningTask 在完成时被释放；Turn 记录可以长期存在。两个对象的销毁时刻不同，说明它们天然不是同一个生命周期所有者。

## 十一、用户提出的设计与当前实现到底差在哪

用户的想法可以拆成两部分。

### “Regular、Review、Compact 做成主 Task 的子类”

这一部分与当前架构基本相同：

```text
用户设想的 MainTask 抽象
≈ 当前 SessionTask trait

用户设想的子类
≈ RegularTask / ReviewTask / CompactTask 实现
```

区别只是 Rust 倾向组合和 trait，而不是传统继承。

### “Turn 和 Task 直接合并”

当前架构没有这样做，因为 Turn 是业务身份和历史记录，Task 是临时执行器。即使运行时通常一对一，完成后也会变成：

```text
Turn 继续存在
Task 已销毁
```

这条反事实最能说明分离的必要性。

## 十二、名称为什么让人绕

`SessionTask` 这个名字容易被理解成“Session 里面的一条业务任务记录”。从实际职责看，如果叫：

```text
TurnDriver
RegularTurnDriver
ReviewTurnDriver
CompactTurnDriver
```

可能更容易理解。

但源码使用 `SessionTask`，是因为它由 Session 拥有和调度，并统一驱动 Session 的某种 Turn 工作流。阅读时可以在脑中把它翻译成“Turn 执行器”。

## 十三、关于压缩时历史替换的下一问题

已经确认：本地自动压缩会调用 `replace_compacted_history`，Session 中供后续模型请求使用的历史确实发生替换。

但现在先不要把它推导成“Thread 的原始历史被彻底抹掉”。下一专题需要至少分开：

```text
模型可见的运行时历史
追加保存的 rollout / Thread Store 记录
App Server 读取和重建出的 Turn/items
工作区文件状态
```

这些层的“历史”不是同一个容器，压缩对它们的影响不同。这个问题已登记为后续大专题。

## 最终心智模型

```text
Turn = 这一轮工作是什么、发生了什么、结果如何
Task = 用什么执行策略把这一轮工作跑完

SessionTask = 所有执行策略的共同接口
RegularTask = 普通 Agent 执行策略
ReviewTask = 评审执行策略
CompactTask = 独立压缩执行策略
RunningTask = 某个策略启动后的运行控制实例
```

数量上：

```text
一个 Session 同时最多一个 ActiveTurn
一个 ActiveTurn 通常一个 RunningTask
一个 RunningTask 包装一个 SessionTask 实现
一个 Task 内部可以有很多 Step、模型请求和工具调用
Task 结束后，Turn 仍作为历史和协议数据存在
```

## 源码入口

- `codex-rs/core/src/tasks/mod.rs`：`SessionTask`、`AnySessionTask`、`start_task` 和统一收尾。
- `codex-rs/core/src/tasks/regular.rs`：普通执行策略。
- `codex-rs/core/src/tasks/review.rs`：Review 策略以及受限子对话。
- `codex-rs/core/src/tasks/compact.rs`：独立压缩策略。
- `codex-rs/core/src/state/turn.rs`：`ActiveTurn`、`RunningTask`、`TurnState` 和 `TaskKind`。
- `codex-rs/core/src/session/turn_context.rs`：一轮执行采用的 `TurnContext`。
- `codex-rs/app-server-protocol/src/protocol/v2/thread_data.rs`：客户端可见的 `Turn` 数据结构。

## 回忆检查

1. `RegularTask` 与 `SessionTask` 是包含关系、继承关系，还是实现关系？
2. `RunningTask` 比 `RegularTask` 多保存了哪些运行控制能力？
3. 为什么历史中的已完成 Turn 是反对合并 Turn 与 Task 的最好证据？
4. 如果把 `SessionTask` 在脑中改名为 `TurnDriver`，整个结构是否更容易理解？
