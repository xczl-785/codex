# Skill 清单怎样随上下文更新：world state 与增量工具

本课解释 Skill 为什么直接涉及上下文管理。难度在于跨越多个层次，而不是函数本身复杂。优先记住：清单说明现在有哪些能力，正文说明本次怎么做，历史决定下次模型还能看见什么。函数和协议字段用于定位，不要求背诵。

## 1. 请求重组、数据重读与内容变化是三件事

一个 Turn 可有多次采样，每次模型调用工具后，都可能再发起请求。[build_prompt](../../../codex-rs/core/src/session/turn.rs)从本次输入、Step 工具视图和基础指令构造请求对象。重新构造对象不表示所有来源都重新读磁盘，更不表示最终内容发生变化。

本地 Skill 通过快照提供发现信息；具体来源、配置和更新边界决定何时刷新。候选清单、正文与已发出的模型输入不能笼统称为同一次“加载”。模型正在处理的请求不会被外部文件编辑追溯修改。

## 2. world state 是模型可见状态的同步机制

宿主知道当前环境和能力，但模型只看输入。world state 将需要告知模型的当前状态组织成多个 section，分别决定怎样比较、怎样渲染变化。它不是全机快照，也不是完整对话存档。

```text
捕获本次 Step
  → 构造当前 WorldState
  → 与已记录状态基线比较
  → 生成消息或结构化更新条目
  → 写入模型历史
  → 更新比较基线及持久记录
  → 发起采样
```

[record_step_world_state_if_changed](../../../codex-rs/core/src/session/mod.rs)组织这个过程；[历史管理器](../../../codex-rs/core/src/context_manager/history.rs)调用 render_history_diff。内部快照供宿主比较，渲染消息供模型理解，两者不是同一数据对象。

Skill 清单属于其中一种状态。[Skills 的 world-state 实现](../../../codex-rs/ext/skills/src/world_state.rs)比较清单内容及相关状态，没有变化就可不发新片段；变化时发送相应更新。清单消失时存在“当前没有可用 Skill”等提示，避免静默省略导致模型继续认为旧能力有效。

这里的增量不一定是逐行差分。清单变化后可以发送更新后的整段清单。不能把“只在变化时发送”说成“只发送新增 Skill 的那一行”。目录更新也不代表替换以前读过的所有 Skill 正文。

## 3. 两类缓存不能混淆

| 缓存 | 保存内容 | 失效或未命中的含义 |
| --- | --- | --- |
| 本地发现缓存 | 扫描、解析得到的 Skill 信息 | 后续查询需要重建结果 |
| 模型提示缓存 | 已处理的输入前缀 | 本次请求不能复用某段已有前缀 |

文件系统通知使本地缓存失效，并不等于删除服务端提示缓存。重扫后若模型输入相同，仅“发生过重扫”不会改变前缀。

设旧输入为 `[A 稳定指令][B 清单][C 历史]`。原地改成 `[A][B′][C]` 时，相对于旧请求的共同前缀在 B 的首个差异处中断，不能越过差异继续复用 C。若追加为 `[A][B][C][U 更新]`，原前缀仍可保持一致。这里是概念示意，不是实际 API 固定序列化顺序。

因此追加更新有助于保留前缀，但不保证命中；模型、工具、请求设置、缓存可用性等仍有关。实际复用量看响应 cached_tokens，而不是从“清单变了”推算全量失效。外部依据：[OpenAI 提示缓存](https://developers.openai.com/api/docs/guides/prompt-caching)、[缓存诊断](https://developers.openai.com/api/docs/guides/prompt-caching/diagnostics)。具体模型边界及计费以使用时文档为准。

## 4. 增量工具模式把声明放进历史

普通路径使用本次请求的顶层 tools 描述当前工具集合。当前 Codex 增量工具模式则通过历史中的结构化 AdditionalTools 条目提供工具定义，并把基础指令也作为上下文条目管理。build_prompt 因此可把常规工具及基础指令字段留空；含义是承载路径改变，不是模型没有这些信息。

[TopLevelToolsState](../../../codex-rs/core/src/context/world_state/top_level_tools.rs)对序列化工具及命名空间定义计算哈希，并与上次比较：

- 未变化：不追加工具声明。
- 新增或定义变化：通过 AdditionalTools 提供相应定义。
- 删除：产生移除通知，不能仅省略，否则旧声明仍在历史中。
- 没有可靠比较基线：提供当前声明，重新建立状态。

工具定义的哈希比较与 SkillsWatcher 的文件事件失效是两种机制，不应因为都涉及“变化”而混为一谈。实际可执行工具仍由 Step 的路由及权限决定，历史中的旧声明不授予执行能力。

[ModelInstructionsState](../../../codex-rs/core/src/context/world_state/model.rs)在增量模式下也记录指令哈希；基线缺失时重新提供完整设置，有变化时追加指令更新。新窗口或压缩后的初始化不能只依赖被丢弃历史中的旧声明。

增量指的是历史中声明的更新方式，不承诺每个网络请求只传新字节。后续请求仍可能携带包含旧声明的历史；传输增量、提示缓存和上下文状态更新应分别判断。

## 5. 为什么当前实现要求 Responses Lite

[StepContext](../../../codex-rs/core/src/session/step_context.rs)要求 use_responses_lite 与 IncrementalTools 开关同时满足，才为新窗口启用。已有窗口还依据历史中工具声明维持处理模式，见[窗口状态构造](../../../codex-rs/core/src/session/world_state.rs)。

[请求构造器](../../../codex-rs/core/src/client.rs)在 Lite 路径使用 AdditionalTools，而普通 Responses 路径使用顶层 tools。Lite 即使未开启历史增量，也可以在请求构造时生成完整声明前缀。因此“支持 Lite 声明载体”与“启用历史增量策略”是两层条件。

标准 Chat Completions 可以在宿主内计算状态差异、追加环境或 Skill 更新消息，也可调整每次请求的顶层 tools。但不能直接把 Codex 的 AdditionalTools 作为标准聊天消息发送，或默认只发新增工具就会继承上一请求的工具集合。[Chat 请求协议](https://developers.openai.com/api/reference/resources/chat/subresources/completions/methods/create)把原生工具定义放在 tools 中。

将工具说明写成普通 developer 文本不等于正式声明原生工具；自定义文本调用协议可以另行设计，但需要自己的解析、校验与执行适配。当前开关限制的是所依赖载体和实现，不是理论上只有 Lite 能支持动态能力。

## 6. 正文生命周期：文件、历史和适用性

显式选择的正文经 SkillInstructions 构造成消息，再由 Turn 输入路径记录进历史。模型自主通过普通工具读取时，正文通常进入工具结果。两条路径都取得读取当时的文本，不是保持一个随文件实时变化的指针。

同一 Turn 的下一次采样可以继续使用仍在历史里的正文，不必每调用一次工具就重读。跨 Turn 时，历史可见性也不等于任务适用性：上一轮审查 Skill 的文本仍可见，不意味着下一轮无关任务必须继续套用审查流程。

外部文件从 V1 改为 V2 后，发现缓存失效、候选刷新、正文重读是不同动作。历史中的 V1 不会被文件编辑自动改写；需要最新规则时应重新取得正文，并明确采用新版本。禁用或删除 Skill 也不是撤回历史文本，更不替代工具权限撤销。

压缩会构造替换历史及必要初始上下文，见[压缩组装](../../../codex-rs/core/src/compact.rs)。不能仅凭 Skill 曾被使用就保证完整正文原样保留，也不能断言所有路径都会删除正文；应检查具体保留策略和最终请求。清单恢复不等于所有正文恢复。只剩摘要或路径而任务依赖精确要求时，需要重新读取相关文件。

## 7. 多轮对话中实际承载什么

下面是普通 Responses 路径的缩略结构，不是实际抓包。read_file 是示例工具；省略可选 ID、推理条目和其他请求参数。这里展示请求构造阶段的逻辑输入，传输层可能进一步续接或增量发送。

```jsonc
{
  "model": "<当前模型>",
  "stream": true,
  "store": false,
  "tool_choice": "auto",
  "tools": [{
    "type": "function", "name": "read_file", "description": "读取文件",
    "parameters": {
      "type": "object", "properties": {"path": {"type": "string"}},
      "required": ["path"], "additionalProperties": false
    },
    "strict": true
  }],
  "input": [
    // 基础指令转换为 developer 消息。
    {"type": "message", "role": "developer", "content": [
      {"type": "input_text", "text": "你是编码助手。……"}
    ]},
    // 此例选择 world-state 清单路径。
    {"type": "message", "role": "developer", "content": [
      {"type": "input_text", "text": "<skills_instructions>\nreview：审查修改，路径……\n</skills_instructions>"}
    ]},
    {"type": "message", "role": "user", "content": [
      {"type": "input_text", "text": "用 $review 检查 a.rs"}
    ]},
    // 宿主加载的正文；user 角色不意味着用户又手动发了一条消息。
    {"type": "message", "role": "user", "content": [
      {"type": "input_text", "text": "<skill>\n<name>review</name>\n<path>D:/Project/.agents/skills/review/SKILL.md</path>\n先检查行为变化，再检查测试。……\n</skill>"}
    ]},
    {"type": "message", "role": "assistant", "content": [
      {"type": "output_text", "text": "我先读取文件。"}
    ]},
    // 调用是独立条目；arguments 是 JSON 字符串。
    {"type": "function_call", "name": "read_file",
     "arguments": "{\"path\":\"D:/Project/a.rs\"}", "call_id": "call_001"},
    {"type": "function_call_output", "call_id": "call_001", "output": "代码内容……"},
    {"type": "message", "role": "assistant", "content": [
      {"type": "output_text", "text": "发现一个边界问题。……"}
    ]},
    {"type": "message", "role": "user", "content": [
      {"type": "input_text", "text": "再看看对应测试。"}
    ]},
    {"type": "function_call", "name": "read_file",
     "arguments": "{\"path\":\"D:/Project/tests/a_test.rs\"}", "call_id": "call_002"},
    {"type": "function_call_output", "call_id": "call_002", "output": "测试内容……"}
    // 第二轮最终回答尚未生成，不属于本次 input。
  ]
}
```

每次请求带入当时已有的历史前缀：首次到 Skill 正文，工具返回后再加入调用和结果，用户下一轮输入也继续追加。Responses 的调用和结果不是 assistant/tool 角色的聊天消息，而是独立的 function_call/function_call_output；call_id 维持因果关联。普通回复才是 assistant message。

UserInput::Skill { name, path } 是客户端到宿主的选择信息，不是原样发送给模型的输入项。宿主读取后生成上述 Skill 正文消息。若模型自行调用工具读 SKILL.md，正文进入 function_call_output.output，不必另外生成一份 user 角色的 SkillInstructions。

Lite 路径省略顶层 tools，改为 input 中的 additional_tools：

```jsonc
{
  "type": "additional_tools", "role": "developer",
  "tools": [{
    "type": "namespace", "name": "functions", "description": "",
    "tools": [{
      "type": "function", "name": "read_file", "description": "读取文件",
      "parameters": {
        "type": "object", "properties": {"path": {"type": "string"}},
        "required": ["path"], "additionalProperties": false
      },
      "strict": true
    }]
  }]
}
```

对应调用可携带 namespace: "functions"。这里是工具声明载体改变，其他消息仍有各自类型。增量模式再决定声明如何随历史追加，而不是每次重写完整声明前缀。字段依据见[ResponseItem](../../../codex-rs/protocol/src/models.rs)、[Skill 正文片段](../../../codex-rs/ext/skills/src/fragments.rs)、[工具序列化](../../../codex-rs/tools/src/tool_spec.rs)。

## 8. 回顾时只先记四条主线

| 主线 | 核心问题 | 本课的位置 |
| --- | --- | --- |
| 发现 | 当前有哪些候选 Skill | 发现缓存与快照 |
| 选择 | 本次用哪个 Skill | 显式选择或模型自主选用 |
| 上下文 | 模型现在知道什么 | 清单更新、正文、历史和压缩 |
| 执行 | 指令如何落实为动作 | 工具声明、路由、权限与结果 |

本课建立上下文侧的机制与正文生命周期，不要求背诵所有协议实现。接着阅读[设计与验证](0029-skill-design-and-validation.md)及[接入职责](0030-skills-integration-contracts.md)，最后用[整体回顾](../reviews/0003-skills-end-to-end-review.md)连接四条主线。
