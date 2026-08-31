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

## Wisdom (Communities)

- [OpenAI Codex Pull Requests](https://github.com/openai/codex/pulls)
  用于观察维护者如何解释真实设计取舍、测试范围和模块边界；遇到非显然实现时再查阅。
- [OpenAI Codex Issues](https://github.com/openai/codex/issues)
  用于把源码机制与真实用户问题联系起来，不作为第一阶段的顺序阅读材料。

## Gaps

- 当前缺少一份由上游维护、专门解释 Codex 内部 Thread/Turn 架构的稳定文档，因此课程以当前源码和测试为主要事实来源。
