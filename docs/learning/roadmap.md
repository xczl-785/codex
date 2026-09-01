# Codex 源码学习路线图

这份路线图描述课程的长期覆盖范围和源码阅读方向，不表示当前教学进度。具体教学顺序可以根据问题调整，但每个局部机制最终都要放回完整 Agent 链路中理解。已完成课程、当前衔接点和下一课以 [教学维护账本](MAINTENANCE.md) 为准。

## 第一阶段范围

第一阶段聚焦 `codex-rs` 中一条完整的本地 Agent 运行链路：命令行或客户端输入如何进入运行时，模型如何生成响应和工具调用，工具如何执行并返回结果，审批与沙箱如何约束行为，状态又如何展示和持久化。

SDK、发布流程、CI、跨平台打包和实验性服务作为扩展主题保留，不作为入门主线。Subagent 生命周期与通信、Thread 拉起与交互是优先深入的两条线，但不是全部学习范围。

## 学习主题全景

- 工程结构、crate 边界、构建和启动入口
- 配置加载、功能开关、身份认证和模型选择
- Thread、Session、Turn、Task 的职责与生命周期
- 模型请求、流式响应、重试和错误处理
- 模型可见上下文、指令注入、压缩和 Token 预算
- 工具定义、注册、路由、并行执行和结果回传
- 审批、执行策略、Shell、文件系统权限和沙箱
- Subagent 创建、通信、状态汇总和资源回收
- Rollout、Thread Store、历史记录、恢复和派生
- TUI 事件循环、状态映射、渲染和用户交互
- App Server、协议模型、通知和客户端边界
- MCP、Skills、Plugins、Apps 与外部连接器
- Exec、SDK、遥测、测试体系和跨平台实现

## 推荐源码阅读顺序

1. 先读 [工程介绍](repository-inventory.md)，建立目录和运行链路地图。
2. 阅读 `codex-rs/Cargo.toml`，观察 workspace 和主要 crate 的边界。
3. 从 `codex-rs/cli/src/main.rs` 开始，了解默认 TUI、`exec`、登录和服务类子命令如何分发。
4. 进入 `codex-rs/tui/src/lib.rs` 与 `codex-rs/tui/src/app.rs`，理解交互界面和应用事件循环。
5. 阅读 `codex-rs/core/src/codex_thread.rs`、`core/src/session/` 和 `core/src/tasks/`，理解 Thread、Session、Turn 与 Task。
6. 阅读 `codex-rs/core/src/tools/`、`core/src/exec.rs` 和 `core/src/sandboxing/`，理解工具路由、命令执行、审批与隔离。
7. 阅读 `codex-rs/protocol/src/protocol.rs`，理解核心运行时和 TUI、App Server 等调用方之间的操作与事件协议。
8. 最后按兴趣扩展到 `codex-rs/exec`、`app-server`、`codex-mcp`、`sdk` 和平台沙箱实现。

## 重点专题：Thread 拉起与交互

这条线回答：一个 Thread 如何创建或恢复，用户输入如何形成 Turn，操作和事件如何双向流动，状态如何持久化，以及界面如何消费这些状态。

建议顺序：

1. `codex-rs/cli/src/main.rs`：默认交互、恢复和派生入口。
2. `codex-rs/core/src/thread_manager.rs`：Thread 的创建、恢复、注册和查找。
3. `codex-rs/core/src/codex_thread.rs`：调用方如何通过 `submit(Op)` 提交操作。
4. `codex-rs/protocol/src/protocol.rs`：`Op`、`Event` 和相关协议词汇。
5. `codex-rs/core/src/session/session.rs`、`turn.rs`、`handlers.rs` 与 `tasks/`：Turn 内模型和工具循环。
6. `codex-rs/tui/src/chatwidget.rs` 与 `chatwidget/turn_lifecycle.rs`：界面的运行、结束与中断状态。
7. `codex-rs/thread-store/src/lib.rs` 与 `types.rs`：持久化边界。
8. `codex-rs/app-server-protocol/src/protocol/v2/`：外部客户端看到的 Thread/Turn API。

```text
用户输入 -> CLI / TUI / App Server -> CodexThread::submit(Op)
        -> Session / Turn -> 模型与工具循环
        -> Event -> 界面或客户端
        -> Thread Store -> 完成、恢复或下一次 Turn
```

## 重点专题：Subagent 生命周期与通信

这条线回答：父 Agent 如何创建子 Agent，子 Agent 如何获得独立 Thread，消息如何传递，状态如何汇总，以及完成或中断后如何回收运行资源。

建议顺序：

1. `codex-rs/core/src/tools/handlers/multi_agents_v2/`：模型可见的创建、通信、等待和中断工具。
2. `codex-rs/core/src/tools/handlers/multi_agents_common.rs`：不同版本共享的处理逻辑。
3. `codex-rs/core/src/agent/control.rs` 与 `agent/control/spawn.rs`：创建、恢复、输入、中断和关闭。
4. `codex-rs/core/src/agent/registry.rs`、`status.rs` 与 `role.rs`：注册、状态和角色。
5. `codex-rs/core/src/agent_communication.rs`：Agent 间消息。
6. `codex-rs/core/src/context/inter_agent_message.rs`、`inter_agent_completion_message.rs` 和 `subagent_notification.rs`：通信如何成为模型可见上下文。
7. `codex-rs/core/src/session/multi_agents.rs`：多 Agent 能力如何挂入 Session。
8. `codex-rs/core/src/agent/control_tests.rs` 与 handler 测试：用场景校验生命周期理解。

```text
模型调用多 Agent 工具 -> Handler -> AgentControl -> 创建子 Thread
                    -> 投递输入 -> 子 Agent 执行 Turn
                    -> 状态或消息回传 -> 父 Agent 上下文
                    -> 完成、中断或关闭 -> 释放运行资源
```

## 后续方向

- 沿一次真实 Function Call 阅读 ToolRouter、ToolRegistry、Handler 与 FunctionCallOutput。
- 在模型—工具闭环建立后，分别阅读审批、沙箱和并行工具调度。
- 继续研究 Thread 的 Fork、恢复与 Subagent 身份传播。
- 比较对话持久记录、模型可见上下文、工作区状态与 Harness 状态外置。
- 扩展到 MCP、Skills、Plugins、Apps、App Server 和 SDK 边界。

阅读源码时优先使用仓库 CodeGraph 索引；命令和资料入口见 [资料索引](RESOURCES.md)。
