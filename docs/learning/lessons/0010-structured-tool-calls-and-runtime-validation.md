# 第十课：结构化工具调用为什么仍需要 Harness 校验

这一课进入新的学习阶段：模型与工具的执行闭环。

先回答一个现代 Agent 系统中经常被简化的问题：

> 既然现代模型已经支持 Function Calling、JSON Schema 和 Structured Outputs，Agent 还需要处理“模型能否返回合法 JSON”吗？

简短答案是：**需要处理，但问题已经从“怎样靠提示词提取 JSON”升级为“怎样建立分层的验证与恢复边界”。**

## 一、早期问题是什么？

早期 Agent 常通过自然语言要求模型输出 JSON：

```text
请只返回 JSON，不要解释，不要使用 Markdown。
```

期望结果是：

```json
{"cmd":"git status"}
```

但模型可能返回：

````text
当然可以：

```json
{"cmd":"git status"}
```
````

也可能出现：

- 多出解释文字；
- 缺少引号或括号；
- 字段名拼错；
- 数字被写成字符串；
- 输出一个不存在的操作名；
- 在 JSON 前后增加 Markdown 围栏。

Agent 程序不得不使用正则表达式、字符串截取、重试提示词或 JSON 修复器。这时“自然语言回答”和“机器控制协议”混在同一个文本通道里。

## 二、现代工具调用改变了什么？

现代模型 API 允许调用方把工具作为专门的请求字段传给模型。工具通常包含：

- 工具名称；
- 使用场景说明；
- 参数 JSON Schema；
- 是否使用严格 Schema；
- 是否允许并行调用等设置。

例如概念上可以定义：

```json
{
  "name": "exec_command",
  "description": "Run a command in an execution environment",
  "parameters": {
    "type": "object",
    "properties": {
      "cmd": {"type": "string"},
      "workdir": {"type": "string"}
    },
    "required": ["cmd"]
  }
}
```

模型服务返回的不再是一段混合了说明文字的 JSON，而是协议中的独立响应项目：

```text
FunctionCall
  name      = exec_command
  call_id   = call_123
  arguments = {"cmd":"git status"}
```

普通助手消息与工具调用因此成为不同类型：

```text
AgentMessage：展示给用户的文字
FunctionCall：交给 Harness 处理的操作意图
```

这消除了大量“从聊天文本中猜测哪里是 JSON”的工作。

## 三、“模型自带”其实可能属于不同层

说模型“自带 JSON 能力”时，至少可能指三件不同的事。

### 1. 模型通过训练学会遵守格式

模型见过大量 JSON 和工具调用样本，因此比早期模型更擅长输出正确结构。

这提高了概率，但本质上仍是生成行为，不能单独构成系统保证。

### 2. API 提供专门的 Tool Calling 协议

模型的调用意图出现在独立的 `FunctionCall` 或 `tool_use` 响应块中，而不是混在普通文本里。

这主要解决：

- 怎样区分“描述命令”和“请求执行命令”；
- 怎样识别工具名称；
- 怎样用 `call_id` 关联调用与结果；
- 怎样支持一个响应中的多个调用。

### 3. 推理服务使用约束生成

Structured Outputs 或严格 Function Calling 可以根据支持的 JSON Schema 约束生成过程，使输出匹配指定结构。

这种保证不仅来自模型权重，也来自模型服务的解码和协议实现。因此更准确的说法是：

> 现代模型平台提供了原生结构化输出能力。

它不一定是“神经网络自己永远不会写错 JSON”。

## 四、JSON mode、Structured Outputs 与 Tool Calling 不相同

这三个概念容易混淆。

### JSON mode

主要保证输出是合法 JSON。

例如下面是合法 JSON：

```json
{"command": 42, "unknown": true}
```

但它不一定符合应用需要的字段和类型。

### Structured Outputs

调用方提供 JSON Schema，严格模式在支持范围内要求最终结构匹配 Schema。

它适合“最终回答必须符合某个数据结构”的场景，例如：

```json
{
  "summary": "...",
  "riskLevel": "medium"
}
```

### Tool Calling

模型返回的是一个工具调用意图，其中包含：

- 工具名；
- 调用 ID；
- 参数；
- 可能的命名空间。

工具参数也可以结合严格 Schema，但 Tool Calling 更重要的价值是建立“模型决策—外部执行—结果回传”的协议闭环。

OpenAI 当前 API 文档明确区分旧的 `json_object` 与推荐的 `json_schema`，并说明 Function Calling 的 `strict` 控制是否严格遵循所支持的参数 Schema 子集：

- <https://platform.openai.com/docs/api-reference/responses-streaming/response/web_search_call?lang=curl>

Anthropic 的工具协议同样通过 `input_schema` 描述参数，并用独立 `tool_use` 块返回工具名、ID 和输入：

- <https://docs.anthropic.com/en/docs/agents-and-tools/tool-use/implement-tool-use>

## 五、四级“正确”必须分开

工具调用是否可执行，至少包含四级判断。

### 第一级：语法合法

参数是否能解析成 JSON：

```json
{"cmd":"git status"}
```

下面则语法错误：

```text
{"cmd":"git status"
```

### 第二级：结构合法

参数是否符合 Schema 或目标类型：

```json
{"cmd": 123}
```

它是合法 JSON，但如果 `cmd` 必须是字符串，就不是合法工具参数。

### 第三级：业务合法

即使类型正确，参数组合也可能没有意义：

```json
{
  "session_id": 999999,
  "chars": "input"
}
```

`session_id` 类型正确，但目标进程可能不存在或已经结束。

### 第四级：执行合法

调用在业务上成立，也不表示当前允许执行：

```json
{
  "cmd": "Remove-Item -Recurse C:\\",
  "workdir": "C:\\"
}
```

它可能同时满足 JSON 和 Schema，却必须被权限、审批、沙箱和安全策略拒绝。

因此：

```text
严格结构化输出
  ≠ 参数具有业务意义
  ≠ 操作获得授权
  ≠ 工具执行一定成功
```

## 六、当前 Codex 怎样接收 FunctionCall？

Responses API 返回的工具调用会转换成 `ResponseItem::FunctionCall`。其中包含：

```text
id
name
namespace
arguments
encrypted_function_args
call_id
```

源码特别说明：`arguments` 是一段包含 JSON 的字符串，而不是已经解析好的 Rust 对象。

`ToolRouter::build_tool_call` 先把响应项目转换成内部 `ToolCall`：

```text
ResponseItem::FunctionCall
  → ToolName(namespace, name)
  → ToolPayload::Function { arguments }
  → ToolCall { call_id, tool_name, payload }
```

这一步主要识别调用类型、工具名和关联 ID，并不意味着参数已经满足具体工具的 Rust 类型。

## 七、参数在哪里真正解析？

具体 Handler 会把 `arguments` 反序列化成自己的参数类型。多个 Handler 使用共同的 `parse_arguments<T>`：

```rust
serde_json::from_str(arguments)
```

概念上类似：

```rust
struct ExecCommandArgs {
    cmd: String,
    workdir: Option<PathBuf>,
}
```

只有成功转换成 `ExecCommandArgs`，Handler 才能继续执行业务判断。

因此即使模型服务提供原生 FunctionCall，Codex 仍然保留自己的解析边界。

## 八、当前 Codex 是否对所有工具使用严格 Schema？

不是。

工具协议中的 `ResponsesApiTool` 有 `strict` 字段，但当前仓库中的许多内置工具规格明确设置为：

```text
strict: false
```

例如 Shell、计划、用户输入、多个 Multi-agent 工具和 History 工具都存在非严格规格。

这说明当前 Codex 的实际策略不是：

> 模型服务已经保证参数完美，因此运行时不需要验证。

而是：

> 利用原生工具协议提高结构稳定性，同时让 Handler 继续承担解析、类型和业务校验。

部分扩展工具可以使用 `strict: true`，最终回答的 output schema 也有独立严格设置。这些能力需要按具体调用面判断，不能用一个全局结论覆盖。

## 九、工具名不存在或参数错误时怎样处理？

`ToolRegistry` 根据 `ToolName` 查找实际工具。如果工具不存在，会生成模型可见错误，而不是随意猜测另一个工具。

具体 Handler 在参数反序列化失败时，也会返回类似：

```text
failed to parse function arguments: ...
```

这类错误属于 `FunctionCallError::RespondToModel`。工具执行包装层会把非致命错误转换成对应的工具失败响应，并记录进当前历史。

因此失败可以进入下一次模型采样：

```text
模型：FunctionCall，缺少 cmd
  → Handler：参数解析失败
  → FunctionCallOutput：missing field cmd
  → 写入 ContextManager
  → 下一次模型请求
  → 模型修正参数并重试
```

这是一种协议内恢复，而不是让整个 Turn 因一次格式错误直接崩溃。

## 十、一次模型—工具闭环

把当前源码主线压缩成一张流程：

```text
Session 历史 + TurnContext + StepContext
  → 构造 Prompt 和 ToolSpec
  → 请求模型
  → 流式接收 ResponseItem
      ├─ AgentMessage：记录并展示
      ├─ Reasoning：记录对应项目
      └─ FunctionCall：转成 ToolCall
             → ToolRouter
             → ToolRegistry 查找工具
             → Handler 解析参数
             → 权限、审批、Hook、沙箱等检查
             → 执行工具
             → 生成 FunctionCallOutput
             → 写回 Session 历史
             → needs_follow_up = true
  → 再次请求模型
  → 直到不再需要 follow-up
  → Turn 收尾
```

一个 Turn 因此可以包含多次模型采样：

```text
Step 1：模型决定读取文件
Step 2：模型根据文件内容决定修改文件
Step 3：模型根据修改结果决定检查 diff
Step 4：模型给出最终回复
```

## 十一、为什么“允许失败并修正”比追求一次完美更重要？

即使严格 Schema 能消灭语法和结构错误，以下情况仍然不可避免：

- 文件在调用前被删除；
- 后台进程已经退出；
- 网络请求超时；
- 用户拒绝审批；
- 当前权限不足；
- 工具返回业务错误；
- 模型选择了不合适的工具；
- 参数虽然合法，但并不能解决当前问题。

Agent 是一个与变化环境交互的系统，不可能只靠生成阶段保证所有后续现实条件。

所以健壮架构追求的是：

```text
尽量减少错误
  + 在边界验证错误
  + 把可恢复错误返回模型
  + 让模型根据新事实继续决策
```

而不是：

```text
假设模型第一次输出永远正确
```

## 十二、信任边界

模型输出应被看作“不可信但有意图的提案”。

各层职责是：

| 层 | 主要责任 |
| --- | --- |
| 模型平台 | 提供结构化响应通道，可能执行 Schema 约束生成 |
| ToolRouter | 把响应项目识别成内部 ToolCall |
| ToolRegistry | 确认工具存在、类型相容并找到实现 |
| Handler | 解析具体参数，检查业务条件 |
| 审批与沙箱 | 判断操作能否以当前权限执行 |
| 操作系统或外部服务 | 真正执行并返回现实结果 |
| Turn 循环 | 把结果写回历史，决定是否再次采样 |

只有这些层共同工作，结构化调用才会成为安全、可靠、可恢复的 Agent 行为。

## 十三、本课结论

1. 早期 Agent 的 JSON 问题主要来自把机器协议混入普通文本。
2. 现代 Tool Calling 用独立响应类型承载工具名、调用 ID 和参数。
3. JSON mode 主要保证 JSON 语法；严格 Structured Outputs 才进一步约束 Schema。
4. Tool Calling 与 Structured Outputs 可以结合，但不是同一个概念。
5. Schema 正确不代表业务正确，也不代表操作获得执行权限。
6. 当前 Codex 的许多内置工具仍使用 `strict: false`，运行时解析不能省略。
7. `FunctionCall.arguments` 在协议中仍是 JSON 字符串，具体 Handler 再反序列化成 Rust 类型。
8. 不存在的工具和非致命参数错误会变成模型可见的失败结果。
9. 工具结果写入历史，并令当前 Turn 继续下一次模型采样。
10. Agent 的可靠性来自“结构约束 + 运行时验证 + 失败反馈 + 模型修正”的闭环。

## 回忆练习

1. 合法 JSON 为什么不一定是合法工具参数？
2. 严格符合工具 Schema 的命令为什么仍然不能直接执行？
3. `FunctionCall` 中 `call_id` 的作用是什么？
4. 参数解析失败以后，为什么当前 Turn 仍可能继续？

