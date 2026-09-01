# 实践任务 0001：实现一个最小源码学习 Agent

这份文档定义课程 0001—0015 复盘后的第一个实践任务。目标不是复制完整 Codex，也不是尽快做出一个功能繁多的聊天产品，而是亲手实现一个足够小、行为可观察、可以与 Codex 对照的 Agent Harness。

实践采用“关键控制面手写、基础设施复用、AI 受控协作”的方式。完成后，应能独立解释一次输入怎样经过 Turn、Step、Function Call、工具执行和历史记录形成最终回答。

## 一、为什么现在开始实践

开始实践不需要先读完整个 Codex 仓库。合适的起点是已经能够回答：

1. 模型为什么不能直接执行工具；
2. 工具结果怎样进入下一次模型请求；
3. Thread、Session、Turn、Task 和 Step 为什么分开；
4. 哪些状态需要持久化，哪些运行资源不能恢复；
5. 一次失败为什么需要分类，而不能统一重试；
6. 第一版应该主动舍弃哪些能力。

课程 0001—0015 已经建立了这些基础。继续纯概念复习的边际收益开始下降；直接修改 Codex 核心又会被协议兼容、跨平台执行、异步资源、UI 和历史实现细节淹没。因此下一阶段采用：

```text
短暂梳理顶层垂直链路
  ↓
实现最小确定性 Agent Harness
  ↓
接入真实模型
  ↓
加入持久化和恢复
  ↓
再按取消、并行、Subagent 的顺序扩展
```

## 二、CLI、TUI、Desktop 和服务端是什么关系

### CLI 是程序入口与命令交互方式

CLI 是 **Command-Line Interface**，即命令行界面。它首先描述一种通过命令、参数、标准输入和标准输出使用程序的方式。

例如：

```text
codex exec "解释这个仓库"
```

这种调用通常接收一次输入、输出结果后退出，适合脚本、CI 和自动化。

但“CLI 程序”这个称呼也可以指安装到终端里的整个命令行应用。这个应用内部仍然可以启动一个 TUI。因此 CLI 与 TUI 不是非此即彼的产品分类。

### TUI 是运行在终端里的交互式界面

TUI 是 **Terminal User Interface**，即终端用户界面。它仍运行在终端窗口中，但不像普通 CLI 那样只逐行读取和打印文本，而是主动管理：

- 屏幕区域和布局；
- 键盘事件；
- 光标位置；
- 颜色和样式；
- 滚动、弹窗、选择框和状态栏；
- 局部重绘与实时更新。

Codex 的 `codex` 命令默认进入基于 Ratatui 的交互界面，所以准确说法是：

```text
Codex CLI 程序
  ├─ 默认交互模式：TUI
  ├─ 非交互模式：codex exec
  ├─ 服务入口：app-server
  └─ 其他登录、配置、MCP 等子命令
```

### Desktop 是图形客户端形态

Desktop 通常指使用原生窗口或 Web 技术封装的桌面 GUI。它与 TUI 的主要区别在展示和交互层，不一定代表拥有另一套 Agent 核心。

一个 Desktop 客户端可以通过 App Server 或其他协议使用相同的 Thread、Turn、工具和持久化能力：

```text
TUI ───────┐
Desktop ───┼─→ App Server / 核心协议 → Agent Runtime
IDE ───────┘
```

因此“Codex 有 CLI 和 Desktop”是在说产品入口；“CLI 默认启动 TUI”是在描述 CLI 内部的一种界面模式，两句话并不矛盾。

### OpenCode CLI 算不算 TUI

准确说法是：OpenCode 是一个 CLI 应用，其中默认交互模式是 TUI。直接运行 `opencode` 会启动终端用户界面，而 `opencode run` 提供非交互运行方式。

这与 Codex 的结构类似：

```text
opencode       → TUI
opencode run   → 非交互 CLI

codex          → TUI
codex exec     → 非交互 CLI
```

参考：[OpenCode CLI](https://opencode.ai/docs/cli) 与 [OpenCode TUI](https://opencode.ai/docs/tui)。

## 三、实践 Agent 做什么

建议实现一个只读的 Codex 源码学习 Agent，暂称 `source-study-agent`：

```text
用户提出源码问题
  ↓
模型判断缺少哪些证据
  ↓
调用 search_text / read_file
  ↓
记录 FunctionCall 与 FunctionCallOutput
  ↓
下一 Step 根据结果继续调查
  ↓
生成解释
```

选择只读源码 Agent 有三个原因：

1. 与当前学习目标直接相关；
2. 工具结果容易观察和验证；
3. 不需要在第一版解决命令执行、工作区写入和沙箱安全。

## 四、第一版范围

### 必须实现

- 简单命令行输入和输出；
- 一个可持久标识的 Thread；
- 一个内存 Session；
- 一个 Turn 执行循环；
- 每次模型采样对应的 StepContext；
- 模型可见 ToolSpec；
- ToolRegistry 与参数校验；
- `read_file` 和 `search_text` 两个只读工具；
- FunctionCall、FunctionCallOutput 与 `call_id` 配对；
- 明确的继续和结束条件；
- 可观察的生命周期日志；
- 确定性测试使用的 ScriptedModel。

### 第一版明确不做

- TUI 和 Desktop UI；
- 文件修改工具；
- Shell、审批和沙箱；
- 自动压缩；
- 并行工具调用；
- 长进程与 yield/poll；
- Subagent；
- App Server；
- Fork、Revert 和完整恢复；
- 多模型、多提供商和插件系统。

这些功能不是不重要，而是它们会遮住第一阶段真正需要观察的模型—工具反馈闭环。

## 五、框架、库与手写的边界

“是否使用框架”不能简单回答为全用或全不用。应该根据学习目标，把系统分成控制面和基础设施。

### 应当亲手实现的控制面

这些部分正是本次实践需要学习的内容，不能交给高层 Agent 框架隐藏：

1. Thread、Session、Turn、Step 的状态与生命周期；
2. `run_turn` 外层循环；
3. StepContext 在什么时候捕获；
4. ToolSpec、ToolRegistry 与工具分发；
5. FunctionCall 和 Output 的记录顺序；
6. `call_id` 配对；
7. 工具失败怎样反馈给模型；
8. Turn 继续和终止条件；
9. 哪些事实写入 rollout，哪些资源只保留在内存；
10. 对这些行为的确定性测试。

如果使用 LangGraph 或另一种高层 Agent 框架直接提供节点循环、工具路由、状态保存和重试，本次最需要理解的部分会被框架代劳。最终可能做出了 Agent，却仍不能解释 Codex 为什么这样拆。

### 可以直接复用的基础设施

这些内容不是当前学习重点，没有必要重复造轮子：

- Rust 异步运行时；
- HTTP 客户端；
- JSON 序列化与反序列化；
- CLI 参数解析；
- UUID 或 ID 生成；
- 错误类型辅助库；
- 临时目录和测试工具；
- 日志与 tracing；
- 真实模型的官方 SDK 或低层客户端；
- JSON Schema 校验库。

原则是：

> 会影响 Agent 行为语义的控制流亲手写；只负责搬运、编码和平台接入的基础设施直接复用。

### 暂时不要做 TUI

TUI 本身会引入事件循环、按键、终端恢复、增量渲染和布局状态。它很有工程价值，但与第一阶段的 Agent Harness 控制闭环正交。

第一版使用普通 stdin/stdout 即可。等运行时稳定后，再决定是否用 Ratatui 做一个单独的展示层练习。

## 六、是否应该使用 Rust

本实践默认使用 Rust，原因不是 Rust 天然更适合所有 Agent，而是当前长期目标是理解 Codex 源码：

- 可以直接体验 trait、enum、所有权和异步任务如何表达运行边界；
- 能把自己的实现与 `codex-rs` 类型逐项比较；
- 后续阅读 CancellationToken、Future 和并行工具调度时有真实经验。

如果目标只是最快验证 Agent 产品想法，Python 会更省时；但本任务的目标是建立 Codex 源码能力，所以 Rust 带来的额外成本属于学习内容，而不只是负担。

## 七、AI 可以参与到什么程度

可以使用 AI，而且没有必要为了证明“纯手搓”而拒绝正常工具。关键不在于代码是谁敲出来，而在于行为边界由谁决定、出现问题时谁能解释和修改。

### AI 可以代劳

- 创建 Cargo 工程和模块骨架；
- 编写常规 serde 数据结构；
- 接入 HTTP、日志和命令行库；
- 根据已经确定的契约补全机械代码；
- 生成测试夹具和重复测试数据；
- 解释 Rust 编译错误；
- 查找 Codex 中可对照的源码入口；
- 对实现做静态审查；
- 提出反例和边界场景。

### AI 不能替你决定后再直接接受

- Thread、Session、Turn、Step 的所有权；
- 一次 Turn 在什么条件下继续或结束；
- 工具调用和结果以什么顺序落入历史；
- 参数错误、业务错误和执行错误怎样分类；
- 重试是否安全以及谁负责重试；
- 恢复时哪些状态可以重建；
- 安全能力和副作用边界；
- 测试真正要证明的行为契约。

这些地方可以让 AI 提方案或实现候选，但最终必须经过自己的解释和验证。

### 掌控力度的验收标准

不要求能默写每行代码。达到下面这些标准，就仍然是你在掌控项目：

1. 不看实现也能画出主循环；
2. 能说明每个核心模块为什么存在，不这样拆会怎样；
3. 能预测一个工具失败后历史和下一 Step 怎样变化；
4. 能在日志中区分 Thread、Turn、Step 和 call ID；
5. 能修改一条行为规则，并指出需要更新哪些测试；
6. 测试失败时能判断是契约错误还是实现错误；
7. 不保留自己无法解释、无法验证的关键控制代码。

可以采用这样的协作循环：

```text
你先写行为契约和场景
  ↓
AI 生成一个小模块或测试候选
  ↓
你逐项解释状态、输入、输出和失败路径
  ↓
运行确定性测试
  ↓
对照 Codex 源码，记录相同点与差异
```

## 八、为什么先使用 ScriptedModel

第一里程碑不直接接真实模型，而是实现一个按脚本返回内容的 `ScriptedModel`：

```text
第一次采样 → 返回 read_file FunctionCall
第二次采样 → 检查是否收到对应 FunctionCallOutput
第三次采样 → 返回最终回答
```

这样可以确定性验证：

- 一个 Turn 为什么包含多个 Step；
- Step 与 Tool Call 为什么不同；
- FunctionCall 是否先于执行结果写入历史；
- Output 是否使用正确的 `call_id`；
- 工具失败是否仍能进入下一 Step；
- 模型给出最终回答后 Turn 是否正确结束。

真实模型具有随机性。若第一天就接真实模型，很难判断失败来自 Harness、Prompt、Schema、模型选择还是网络。ScriptedModel 先固定模型行为，使 Harness 自身可以被验证。

## 九、实践里程碑

### 里程碑 0：顶层垂直切片

只看与第一版实现相对应的 Codex 边界：

| 实践组件 | Codex 对应区域 |
| --- | --- |
| CLI 输入 | `codex-rs/cli/` 与 `codex-rs/tui/` |
| Thread 入口 | `core/src/thread_manager.rs`、`codex_thread.rs` |
| Session | `core/src/session/session.rs` |
| Turn 驱动器 | `core/src/tasks/` |
| Step 循环 | `core/src/session/turn.rs` |
| 工具路由 | `core/src/tools/router.rs`、`registry.rs` |
| 历史记录 | ContextManager 与 rollout |
| 向外反馈 | Event 与 App Server 映射 |

这一步只建立映射，不通读全部实现。

### 里程碑 1：确定性 Harness

- Rust CLI；
- ScriptedModel；
- `read_file`、`search_text`；
- 串行工具调用；
- 内存历史；
- 生命周期日志；
- 核心场景测试。

完成标志：不接网络也能确定性跑通三次采样、一次工具调用和最终回答。

### 里程碑 2：接入真实模型

- 保留统一 Model trait；
- 通过低层 SDK 或 HTTP 客户端接入 Tool Calling；
- 运行时反序列化和校验参数；
- 把非致命工具错误反馈给模型。

完成标志：模型能够自主选择只读工具，并根据真实结果形成回答。

### 里程碑 3：JSONL rollout 与恢复

- 记录 TurnStarted、UserMessage、FunctionCall、FunctionCallOutput、AssistantMessage、TurnCompleted；
- 退出后重新加载同一 Thread；
- 重建新的 Session；
- 不尝试恢复旧异步任务或其他运行资源。

完成标志：能够解释“恢复的是事实，不是旧运行现场”。

### 里程碑 4：取消传播

把 CancellationToken 从 Turn 传到模型请求和工具执行，观察谁负责结束、清理与记录中断结果。

### 里程碑 5：并行工具调度

允许多个只读工具并行，使用 `call_id` 按身份配对结果，不依赖完成顺序。

### 里程碑 6：Subagent

最后加入独立子 Thread、消息传递、等待、完成通知和资源回收。此时再回看 Codex 的多 Agent 实现。

## 十、第一版关键行为契约

实现前先固定以下契约：

1. 一个用户请求启动一个 Turn；
2. 一个 Turn 可以经历多个 Step；
3. 每个 Step 使用一次稳定的工具视图；
4. 模型 FunctionCall 必须先记录，再执行工具；
5. 每个 FunctionCallOutput 必须配回原 `call_id`；
6. 非致命工具失败仍作为 Output 进入下一 Step；
7. 模型只返回最终回答且没有待处理工具时，Turn 结束；
8. 第一版工具只能读取显式允许的源码根目录；
9. 第一版不自动重试具有副作用的操作；
10. Session 关闭时，历史事实与运行资源的处理必须分开。

## 十一、第一批测试场景

### 正常闭环

```text
用户问题
→ ScriptedModel 请求 read_file
→ 工具成功
→ Output 写回
→ ScriptedModel 返回答案
→ Turn 完成
```

### 参数错误

```text
模型请求不存在的路径
→ Handler 返回结构化错误
→ 错误写回下一 Step
→ 模型修正路径
```

### 未知工具

```text
模型请求未注册工具
→ Registry 拒绝
→ 不执行任何外部动作
→ 错误与 call_id 一起写回
```

### 因果配对

验证 FunctionCall 和 FunctionCallOutput 使用相同 `call_id`，并且 Output 之前已经存在对应 Call。

### 正常结束

模型返回最终消息且没有待处理调用时，只结束当前 Turn，不删除 Thread。

## 十二、工程位置与代码管理

实践工程建议建立为独立 Rust 项目，不直接加入 Codex workspace，也不把实验实现塞进 `codex-rs/core`。

这样做可以：

- 避免改变 Codex 的真实构建和依赖；
- 允许自由重写早期设计；
- 清楚区分“学习实现”与“当前源码事实”；
- 后续可以把相同场景分别投射到自己的实现和 Codex。

具体目录、仓库位置和依赖清单在开始实现前再确定。本学习分支目前只保存任务说明、学习结论和对照文档。

## 十三、实践完成后的对照问题

每个里程碑结束后，不比较函数名是否相同，而比较行为：

1. 我们的 Thread 和 Codex Thread 分别保存什么？
2. 我们的 Session 中有哪些资源不能持久化？
3. Step 工具视图是否会在执行途中漂移？
4. Call/Output 是否形成可判定的因果历史？
5. 工具失败由谁分类，模型什么时候有机会修正？
6. 退出后能恢复哪些事实，不能恢复哪些运行资源？
7. 当前简化在哪些场景下会产生真实问题？

实践的价值不在于实现得像 Codex，而在于能够解释 Codex 为什么需要比最小实现更多的层次和约束。
