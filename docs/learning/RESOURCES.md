# Codex 源码学习资料

## Knowledge

- [工程介绍](repository-inventory.md)
  用于建立根目录、Rust workspace、核心层次和构建体系的整体地图。
- [CLI 入口](../../codex-rs/cli/src/main.rs)
  用于确认命令行解析、子命令分发和默认 TUI 启动入口。
- [TUI 命令路由](../../codex-rs/tui/src/app/thread_routing.rs)
  用于理解界面如何把用户操作转换为 Thread 和 Turn 请求。
- [App Server Turn 处理](../../codex-rs/app-server/src/request_processors/turn_processor.rs)
  用于理解 `turn/start` 如何进入核心运行时，以及 `turn/interrupt` 如何验证活动 Turn 并等待中断事件。
- [ThreadManager](../../codex-rs/core/src/thread_manager.rs)
  用于理解 Thread 的创建、恢复、派生、注册和关闭。
- [CodexThread](../../codex-rs/core/src/codex_thread.rs)
  用于理解调用方怎样向一个 Thread 投递操作并接收事件。
- [Session 操作处理](../../codex-rs/core/src/session/handlers.rs)
  用于理解核心运行时如何处理 Turn 输入、中断、审批和其他操作。
- [核心协议](../../codex-rs/protocol/src/protocol.rs)
  用于查阅核心 `Op` 与事件类型，而不是凭印象猜测消息格式。
- [Turn 输入协议](../../codex-rs/protocol/src/turn_input.rs)
  用于理解一次新 Turn 或追加输入携带的数据和路由意图。
- [核心 Turn 输入决策](../../codex-rs/core/src/session/turn_input.rs)
  用于理解 `StartOrSteer`、`StartIfIdle` 与严格 `Steer` 在核心中的判定。
- [Session 运行实体](../../codex-rs/core/src/session/session.rs)
  用于理解一个 Thread 在内存中的服务、状态与活动 Turn。
- [Session 状态](../../codex-rs/core/src/state/session.rs)
  用于理解跨 Turn 保留的配置、模型历史、世界状态和压缩窗口。
- [Session 初始化与历史安装](../../codex-rs/core/src/session/mod.rs)
  用于理解新建、恢复和派生时如何构造 Session、启动提交循环并安装初始历史。
- [Rollout 历史重建](../../codex-rs/core/src/session/rollout_reconstruction.rs)
  用于理解恢复时如何采用最新压缩检查点，并重放检查点之后的历史。
- [模型历史管理](../../codex-rs/core/src/context_manager/history.rs)
  用于理解模型可见历史的规范化、替换、版本和 Prompt 投影。
- [Turn 上下文](../../codex-rs/core/src/session/turn_context.rs)
  用于理解一轮执行捕获的配置、模型、权限与环境快照。
- [Turn 运行状态](../../codex-rs/core/src/state/turn.rs)
  用于理解 `ActiveTurn`、`RunningTask` 与 `TaskKind` 的组合关系。
- [Task 执行契约](../../codex-rs/core/src/tasks/mod.rs)
  用于理解具体 Task 如何运行、取消并把结果交回 Session。
- [普通 Task](../../codex-rs/core/src/tasks/regular.rs)
  用于理解一个普通 Task 如何发出 TurnStarted，并驱动完整的 `run_turn` 循环。
- [Turn 内部循环](../../codex-rs/core/src/session/turn.rs)
  用于理解一次 Turn 内的 Step 捕获、模型采样、工具调用、Steer 接入和中途压缩。
- [Turn 计时](../../codex-rs/core/src/turn_timing.rs)
  用于区分整轮墙钟时间、首 Token 时间和采样、工具、压缩等阶段耗时。
- [TUI Turn 收尾](../../codex-rs/tui/src/chatwidget/turn_runtime.rs)
  用于理解界面怎样收起流式内容、插入工作分隔符并显示 Turn 总耗时。
- [TUI 输入流](../../codex-rs/tui/src/chatwidget/input_flow.rs)
  用于理解界面怎样暂存输入，以及 Turn 结束后如何继续处理等待中的消息。
- [TUI 输入恢复](../../codex-rs/tui/src/chatwidget/input_restore.rs)
  用于理解 Steer 被拒绝或 Turn 被中断时，输入如何进入队列或回到编辑框。
- [TUI 历史回溯](../../codex-rs/tui/src/app_backtrack.rs)
  用于理解编辑旧消息如何选择 Revert 边界，在当前 Thread 中回退并恢复输入框。
- [App Server Thread 协议](../../codex-rs/app-server-protocol/src/protocol/v2/thread.rs)
  用于核对 `thread/fork`、`thread/revert` 与旧 `thread/rollback` 的准确边界。
- [App Server Thread 处理](../../codex-rs/app-server/src/request_processors/thread_processor.rs)
  用于理解 Fork 的历史截断，以及 Revert 如何关闭并重新加载同一 Thread。
- [App Server Thread 历史重建](../../codex-rs/app-server-protocol/src/protocol/thread_history.rs)
  用于理解 rollout 如何重建为公开的 Turn 与 ThreadItem，以及压缩为何不删除旧界面记录。
- [TUI Thread 转录投影](../../codex-rs/tui/src/thread_transcript.rs)
  用于理解公开 ThreadItem 如何进一步转换为用户看到的 transcript cells。
- [模型工具规划](../../codex-rs/core/src/tools/spec_plan.rs)
  用于核对普通模型请求实际注册了哪些工具，以及 App Server RPC 为什么不自动成为模型工具。
- [工具路由](../../codex-rs/core/src/tools/router.rs)
  用于理解模型返回的 FunctionCall 如何转换成内部 ToolCall 并交给注册表。
- [工具注册与分发](../../codex-rs/core/src/tools/registry.rs)
  用于理解工具身份查找、类型匹配、Hook、执行和模型可见失败结果。
- [工具并行执行包装](../../codex-rs/core/src/tools/parallel.rs)
  用于理解工具成功结果与非致命错误如何统一转换成写回模型历史的 ResponseItem。
- [流式响应项目处理](../../codex-rs/core/src/stream_events_utils.rs)
  用于理解 FunctionCall 为什么会排入执行队列并令本次采样需要 follow-up。
- [工具参数公共解析](../../codex-rs/core/src/tools/handlers/mod.rs)
  用于核对 FunctionCall 参数怎样由 JSON 字符串反序列化成具体 Rust 类型。
- [History/Notes 扩展](../../codex-rs/ext/history-notes/src/extension.rs)
  用于理解可选历史检索能力的启用条件和 Thread 生命周期挂接方式。
- [History/Notes 工具](../../codex-rs/ext/history-notes/src/tools.rs)
  用于核对模型可用的历史窗口列举、项目读取和内容搜索工具。
- [Anthropic Tool Use](https://docs.anthropic.com/en/docs/agents-and-tools/tool-use/implement-tool-use)
  用于对照另一种主流模型 API 怎样使用 `input_schema`、`tool_use` 和 `tool_result` 形成闭环。

## Subagent 与跨系统协作

- [委派创建入口](../../codex-rs/core/src/thread_spawn.rs)、[子配置](../../codex-rs/core/src/agent/child_config.rs)：创建条件、归属、初始任务与配置覆盖。
- [协作控制](../../codex-rs/core/src/agent/control/api.rs)、[回收](../../codex-rs/core/src/agent/control/residency.rs)：交付、加载与驻留状态。
- [A2A 核心概念](https://a2a-protocol.org/v1.0.1/topics/key-concepts/)、[任务生命周期](https://a2a-protocol.org/v1.0.1/topics/life-of-a-task/)：课程 0024 的外部协议依据，固定已查版本，接入前重查。
- [Google ADK A2A](https://google.github.io/adk-docs/a2a/quickstart-consuming/)、[Microsoft Agent Framework A2A](https://learn.microsoft.com/en-us/agent-framework/journey/agent-to-agent)、[LangSmith A2A](https://docs.langchain.com/langsmith/server-a2a)：实际框架接入示例；支持版本和可选能力属于 2026-10-19 查询快照。

## Skills 发现、加载与刷新

- [元信息与模型](../../codex-rs/skills/src/model.rs)、[frontmatter 解析](../../codex-rs/skills/src/parser.rs)：Skill 身份与描述怎样成为运行时数据。
- [显式选择](../../codex-rs/skills/src/selection.rs)、[提及解析](../../codex-rs/skills/src/mentions.rs)、[选择测试](../../codex-rs/skills/src/selection_tests.rs)：文本、路径、结构化输入、歧义与禁用。
- [目录与预算](../../codex-rs/ext/skills/src/render.rs)、[本地正文注入](../../codex-rs/ext/skills/src/host_prompt.rs)、[Skills 扩展](../../codex-rs/ext/skills/src/extension.rs)：目录与正文的不同角色、来源相关截断及重复片段协调。
- [SkillsWatcher](../../codex-rs/app-server/src/skills_watcher.rs)、[FileWatcher](../../codex-rs/file-watcher/src/lib.rs)、[缓存服务](../../codex-rs/ext/skills/src/host_service.rs)：事件驱动失效、路径匹配、节流及缓存重建，不能与模型历史更新混为一谈。

- [Step 状态构造](../../codex-rs/core/src/session/world_state.rs)、[增量工具声明](../../codex-rs/core/src/context/world_state/top_level_tools.rs)、[请求转换](../../codex-rs/core/src/client.rs)：课程 0028 的状态同步与协议载体依据。

## Wisdom (Communities)

- [OpenAI Codex Pull Requests](https://github.com/openai/codex/pulls)
  用于观察维护者如何解释真实设计取舍、测试范围和模块边界；遇到非显然实现时再查阅。
- [OpenAI Codex Issues](https://github.com/openai/codex/issues)
  用于把源码机制与真实用户问题联系起来，不作为第一阶段的顺序阅读材料。

## Gaps

- 当前缺少一份由上游维护、专门解释 Codex 内部 Thread/Turn 架构的稳定文档，因此课程以当前源码和测试为主要事实来源。
