# Codex 源码学习资料

## Knowledge

- [工程介绍](repository-inventory.md)
  用于建立根目录、Rust workspace、核心层次和构建体系的整体地图。
- [CLI 入口](../../codex-rs/cli/src/main.rs)
  用于确认命令行解析、子命令分发和默认 TUI 启动入口。
- [TUI 命令路由](../../codex-rs/tui/src/app/thread_routing.rs)
  用于理解界面如何把用户操作转换为 Thread 和 Turn 请求。
- [App Server Turn 处理](../../codex-rs/app-server/src/request_processors/turn_processor.rs)
  用于理解 `turn/start` 如何跨过 API 边界进入核心运行时。
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
  用于理解编辑旧消息为什么通过 Fork 建立源历史保留的新分支。
- [App Server Thread 协议](../../codex-rs/app-server-protocol/src/protocol/v2/thread.rs)
  用于核对 `thread/fork`、`thread/revert` 与旧 `thread/rollback` 的准确边界。
- [App Server Thread 处理](../../codex-rs/app-server/src/request_processors/thread_processor.rs)
  用于理解 Fork 的历史截断，以及 Revert 如何关闭并重新加载同一 Thread。
- [App Server Turn 处理](../../codex-rs/app-server/src/request_processors/turn_processor.rs)
  用于理解 `turn/interrupt` 如何验证活动 Turn 并等待中断事件。

## Wisdom (Communities)

- [OpenAI Codex Pull Requests](https://github.com/openai/codex/pulls)
  用于观察维护者如何解释真实设计取舍、测试范围和模块边界；遇到非显然实现时再查阅。
- [OpenAI Codex Issues](https://github.com/openai/codex/issues)
  用于把源码机制与真实用户问题联系起来，不作为第一阶段的顺序阅读材料。

## Gaps

- 当前缺少一份由上游维护、专门解释 Codex 内部 Thread/Turn 架构的稳定文档，因此课程以当前源码和测试为主要事实来源。
