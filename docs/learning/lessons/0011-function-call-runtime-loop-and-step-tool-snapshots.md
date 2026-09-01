# 第十一课：一次 Function Call 如何完成模型—工具闭环

上一课区分了模型原生 Tool Calling 与 Harness 运行时校验。这一课继续沿真实源码回答：模型已经返回一个 Function Call 以后，Codex 如何把调用意图变成受控执行，再把结果交给同一个 Turn 的下一次模型请求。

## 一、核心结论

模型只提出工具调用，Codex Harness 负责解释、匹配、调度、执行、记录和回传：

```text
模型返回 FunctionCall
        ↓
转换为内部 ToolCall
        ↓
ToolCallRuntime 安排执行
        ↓
ToolRouter 构造 ToolInvocation
        ↓
ToolRegistry 找到 Handler
        ↓
Handler 返回 ToolOutput
        ↓
转换为 FunctionCallOutput
        ↓
写入 Session conversation
        ↓
同一个 Turn 发起下一次模型请求
```

`FunctionCall` 是模型协议中的调用意图，不代表 Rust 函数、Shell 命令或外部工具已经执行。

## 二、六种容易混淆的对象

| 对象 | 通俗理解 | 职责 |
| --- | --- | --- |
| `ToolSpec` | 提供给模型的菜单 | 描述工具名称、用途和参数结构 |
| `FunctionCall` | 模型下的订单 | 表示模型选择了哪个工具和哪些参数 |
| `ToolCall` | Codex 内部订单 | 统一 Function、Custom 与 Tool Search 等协议形态 |
| `ToolInvocation` | 带运行环境的工单 | 增加 Session、Turn、Step、取消令牌和调用来源 |
| `ToolOutput` | Handler 的内部结果 | 表示文本、结构化内容、成功状态和元数据 |
| `FunctionCallOutput` | 返回模型的回执 | 用相同 `call_id` 把结果与调用配对 |

模型可能返回：

```json
{
  "type": "function_call",
  "name": "exec_command",
  "call_id": "call_123",
  "arguments": {
    "cmd": "git status --short"
  }
}
```

此时操作系统里还没有执行命令。它仍然需要经过 Codex 的工具身份、参数类型、Hooks、权限、调度和 Handler 边界。

## 三、工具在模型请求前怎样准备

每次模型请求都使用一个 `Prompt`。除了 Conversation 输入，Prompt 还带有当前模型可见的工具说明：

```rust
Prompt {
    input,
    tools: step_context.tool_router.model_visible_specs(),
    parallel_tool_calls: true,
    ...
}
```

`StepContext` 是一次模型采样请求的运行快照，定义在：

```text
codex-rs/core/src/session/step_context.rs
```

它包含：

```text
StepContext
├── 当前 TurnContext
├── 本 Step 的模型设置
├── Token budget
├── 环境就绪状态
├── capability roots
├── executor capability discovery
├── 本 Step 的 MCP Binding
├── 已定稿的 ToolRouter
└── 本 Step 观察到的 AGENTS.md
```

源码注释把它定义为 request-scoped state，也就是可能在模型采样请求之间变化、但在一次请求内部保持一致的状态。

## 四、ToolRouter 在什么时候定稿

构造路径是：

```text
Session::capture_step_context
        ↓
capture_step_context_with_required_mcp_servers
        ↓
capture_step_context_inner
        ↓
built_tools
        ↓
build_tool_router
        ↓
finalize_tool_router
        ↓
Arc<ToolRouter> 写入 StepContext
```

`build_tool_router` 位于：

```text
codex-rs/core/src/tools/spec_plan.rs
```

它从多个来源建立新的 `ToolRegistry`：

```text
空 ToolRegistry
    ↓
Codex 核心工具
    ↓
MCP 工具与暴露策略
    ↓
Extension 工具
    ↓
Dynamic Tools
    ↓
Hosted Model Tools
```

`finalize_tool_router` 随后完成最终裁决：

- 应用工具名称与命名空间覆盖；
- 确定普通工具模式、Code Mode 或 Code Mode Only；
- 处理 Code Mode 保留工具；
- 判断是否增加 `tool_search`；
- 检查名称及命名空间冲突；
- 生成模型可见的 `model_visible_specs`；
- 建立 Code Mode 工具名映射；
- 生成工具命名空间元数据；
- 判断是否具备子 Agent 管理工具。

因此“定稿”表示：

> 当前 Step 中模型看到的工具菜单，以及这些调用实际进入的 Handler 映射，已经基于同一份快照完成一致性裁决。

它不是对整个 Turn、Session 或 Thread 永久定稿。

## 五、每个 Step 是否重新组装工具列表

是的。`run_turn` 在每次循环开始时捕获一个 StepContext。

第一个 Step 使用 Turn 开始时提前准备的 `first_step_context`。如果模型调用工具并要求继续，下一次循环会重新调用 `capture_step_context`：

```text
Turn
├── Step 1：捕获 ToolRouter A → 模型请求 → 工具调用
├── Step 2：捕获 ToolRouter B → 模型请求 → 工具调用
└── Step 3：捕获 ToolRouter C → 模型给出最终回复
```

A、B、C 经常内容相同，但仍然是分别捕获的请求快照。

“重新组装”不等于重启所有底层服务。MCP Runtime、Handler Cache、插件管理器、执行环境和 Session 服务可以复用；每个 Step 主要重新规划当前可执行 Registry 和模型可见工具视图。

## 六、为什么采用“Step 间刷新、Step 内冻结”

如果整个 Turn 永远使用第一次工具列表，长任务后面的请求可能看不到已经就绪或刚刚刷新的能力。

如果工具执行时临时查询最新全局状态，又可能出现：

```text
模型请求时看到工具集合 A
        ↓
环境刷新为工具集合 B
        ↓
模型产生的 A 版调用却按 B 版 Handler 解释
```

Codex 采用类似快照隔离的规则：

```text
Step 之间允许变化
Step 内部保持一致
```

已经捕获的 Step 持有自己的 `step_context.tool_router` 和 `step_context.mcp`。即使底层 MCP 在执行过程中刷新，本 Step 仍按原快照完成；下一 Step 才重新捕获最新状态。

当前测试 `step_context_keeps_its_mcp_runtime_for_tools` 专门验证了这一点。

## 七、Step 之间工具为什么可能变化

### 1. Step Settings 更新

`capture_step_context_inner` 会读取 `turn_context.current_settings` 的不可变快照。运行中的 Turn 可以发布新的模型、reasoning effort、reasoning summary 或 service tier 设置。

已经捕获的 Step 不受影响，新 Step 使用已发布设置。模型变化又可能影响工具模式、Tool Search、Hosted Tools 和模型支持的 ToolSpec 形态。

### 2. MCP Binding 刷新

每个 Step 都通过 `mcp_runtime_for_step` 获得 MCP Binding。MCP 配置、连接、认证或工具目录刷新后，新 Step 可以得到新 Binding，旧 Step 继续保留原 Binding。

### 3. 连续输入提出新的 MCP 需求

如果 Turn 运行中收到 pending input，`run_turn` 会重新分析其中要求的 MCP Server，再调用 `capture_step_context_with_required_mcp_servers`。

例如当前 Step 只在本地读代码，用户随后追加“也查询 Notion 设计文档”，下一 Step 可以把 Notion MCP 需求纳入工具规划。

### 4. 执行环境就绪状态变化

Turn 选择的环境通常保持稳定，但环境可能从 `Starting` 变为 `Ready`。每个 Step 调用 `refresh_readiness`，因此新 Step 可能获得刚刚可用的远程执行、文件系统或浏览器能力。

### 5. Capability、Plugin 与 Connector 变化

每个 Step 重新解析 ready capability roots、executor capability discovery 和绑定到这些能力根的插件。Connector 快照、可访问 App、认证和工具推荐也会参与新的工具计划。

### 6. Dynamic Tools 通常保持不变

Dynamic Tools 每个 Step 都会重新加入 Registry，但它们来自当前 `TurnContext`。如果 TurnContext 没有更新，其内容不会仅因为重建 Router 就自动变化。

## 八、Registry 与模型可见列表不是同一概念

ToolRouter 同时保存：

```text
registry
model_visible_specs
```

Registry 是当前 Step 可路由的运行时实现集合；`model_visible_specs` 是直接发给模型的工具说明。

某些工具可能是：

- `Direct`：直接暴露给模型；
- `Deferred`：通过 Tool Search 才发现；
- `Hidden`：不直接显示；
- 只通过 Code Mode 使用，而不表现为普通 Function Tool。

因此每 Step 既会重建运行 Registry，也会根据暴露策略、Tool Mode 和模型能力重新计算可见 ToolSpec。

## 九、模型响应怎样变成内部 ToolCall

模型流式响应进入 `try_run_sampling_request`。完整响应项到达时，`handle_output_item_done` 调用：

```text
ToolRouter::build_tool_call(item)
```

它把 `ResponseItem::FunctionCall` 转成统一内部表示：

```text
ResponseItem::FunctionCall
├── name
├── namespace
├── arguments
└── call_id
        ↓
ToolCall
├── ToolName
├── ToolPayload::Function
└── call_id
```

Custom Tool Call 和由客户端执行的 Tool Search Call 也在这里转换成 ToolCall，使后面的调度系统不需要分别理解每一种模型协议形态。

## 十、为什么先保存 Function Call 再执行

识别出 ToolCall 后，Codex 先调用 `record_completed_response_item` 保存模型返回的原始调用，然后才把工具 Future 加入运行队列。

Conversation 因而形成：

```text
FunctionCall(call_123, exec_command, arguments)
FunctionCallOutput(call_123, result)
```

Call 是模型产生的意图，Output 是 Harness 的执行观察。两者分别记录因果链的两端。

## 十一、ToolCallRuntime、Router、Registry 与 Handler

`ToolCallRuntime` 负责运行调度：

- 判断工具是否支持并行；
- 使用读写锁安排并行或串行执行；
- 管理取消；
- 统计等待与执行时间；
- 确保完成和中断生命周期只结算一次。

ToolRouter 随后把内部订单补充成 `ToolInvocation`：

```text
ToolInvocation
├── Session
├── TurnContext
├── StepContext
├── CancellationToken
├── DiffTracker
├── call_id
├── tool_name
├── source
└── payload
```

Registry 负责按名称找到 `CoreToolRuntime`，验证 payload 类型，运行 PreToolUse/PostToolUse Hooks、遥测和生命周期通知。具体 Handler 最终通过 `tool.handle(invocation).await` 执行业务行为。

因此两层职责是：

```text
ToolRouter：当前 Step 的完整工具计划和协议路由
ToolRegistry：名称到运行实现的映射与统一执行边界
Handler：某个工具的具体业务行为
```

## 十二、工具结果如何进入下一次模型请求

Handler 返回 `ToolOutput` 后，Registry 生成 `AnyToolResult`。`AnyToolResult::into_response` 使用原 `call_id` 和 payload 调用 `to_response_item`，得到 `FunctionCallOutput` 或对应协议结果。

工具 Future 被放入 `in_flight`。本次模型流结束后，`drain_in_flight` 等待工具完成，并通过：

```text
record_annotated_conversation_items
```

把结果写入 Session Conversation。

模型产生工具调用时，`needs_follow_up` 被设置为 `true`。外层 `run_turn` 因而不会结束 Turn，而是继续下一次循环：

```text
clone_history
    ↓
历史包含 FunctionCall 与 FunctionCallOutput
    ↓
重新捕获 StepContext
    ↓
构造下一次 Prompt
    ↓
再次请求模型
```

所以一个 Turn 可以包含多个模型请求：

```text
Turn
├── Step 1：模型返回 FunctionCall
├── 工具执行
├── Step 2：模型读取结果，可能再次调用工具
├── 工具执行
└── Step 3：模型返回最终回答
```

这不会创建新的 RegularTask，也不会创建新的 Turn。

## 十三、Call 与 Output 为什么都要保存

### 回放与审计

两者同时存在，才能看到模型请求了什么、参数是什么、系统返回了什么，以及结果属于哪个调用。

### 恢复

持久记录中的组合状态具有不同含义：

```text
有 Call，没有 Output
    -> 调用已提出，但没有可证明的终态结果

有 Call，也有 Output
    -> 已有可回传模型的终态结果

只有 Output，没有 Call
    -> 结果失去来源，形成 orphan output
```

Call 和 Output 都存在时，恢复后的模型循环可以重建完整上下文，而不必重新执行已经有结果的工具。只有 Call 时也不能简单重跑，因为原工具可能已经产生副作用，只是结果尚未持久化。

完整记录不直接决定恢复策略，但为恢复逻辑提供判断事实。

### 并行

并行能力由 ToolCallRuntime、in-flight futures 和读写锁实现，不是由历史记录实现。

Call/Output 对并行的价值是使用 `call_id` 消除完成顺序的不确定性：

```text
Call A：读取 Cargo.toml
Call B：读取 package.json

实际完成：B -> A

Output(B) 仍与 Call(B) 配对
Output(A) 仍与 Call(A) 配对
```

因此准确说法是：保存完整 Call/Output 因果链不创造并行，但让并行结果能够稳定关联、持久化和重放。

### 模型推理

模型使用工具的正式说明来自 ToolSpec。Call/Output 不会训练模型，也不会更新模型参数。

它们提供的是当前任务中的因果上下文：

```text
我为什么调用这个工具
我传了什么具体参数
系统观察到了什么结果
我下一步应该继续、纠错还是停止
```

Step 之间是不同的模型采样请求。下一次请求必须重新看到前一次 FunctionCall 和对应 Output，才能恢复连贯推理。只给裸输出时，模型无法可靠判断它来自哪个工具、哪个参数或哪个调用目的。

## 十四、本课结论

```text
ToolRouter 的定稿范围是一个 Step，而不是整个 Turn。

每个 Step 重新规划工具，但底层运行服务可以复用。

Step 之间允许吸收设置、MCP、环境和插件状态变化；
Step 内模型可见工具和实际执行 Router 必须保持同一快照。

FunctionCall 是模型意图，FunctionCallOutput 是执行回执；
两者通过 call_id 构成可恢复、可配对、可供下一次模型采样继续推理的因果链。
```

## 源码阅读入口

- `codex-rs/core/src/session/turn.rs`：`run_turn`、`built_tools`、`try_run_sampling_request` 与 Step 循环。
- `codex-rs/core/src/session/mod.rs`：`capture_step_context_inner`。
- `codex-rs/core/src/session/step_context.rs`：StepContext 的请求级快照边界。
- `codex-rs/core/src/tools/spec_plan.rs`：`build_tool_router` 与 `finalize_tool_router`。
- `codex-rs/core/src/tools/router.rs`：模型响应到内部 ToolCall 和 ToolInvocation。
- `codex-rs/core/src/tools/parallel.rs`：并行、串行、取消与 Future 调度。
- `codex-rs/core/src/tools/registry.rs`：工具查找、Hooks、Handler 执行和结果转换。
- `codex-rs/core/src/stream_events_utils.rs`：模型输出项识别和工具 Future 创建。
- `codex-rs/core/src/session/step_activation_tests.rs`：Step Settings 快照更新测试。
- `codex-rs/core/src/session/tests.rs`：MCP Binding 刷新与 Step 内冻结测试。
