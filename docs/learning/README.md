# Codex 源码学习指南

这个目录用于记录本 fork 的源码学习材料，不作为 Codex 的用户文档或上游项目文档。

开始阅读前，可以先通过 [Codex 工程介绍](repository-inventory.md) 建立目录、模块、运行链路和构建体系的整体认识。

## 课程入口

- [学习任务](MISSION.md)：这套源码学习最终要获得什么能力。
- [第一课：从一句话到一次 Turn](lessons/0001-from-prompt-to-turn.md)：先建立整体运行骨架。
- [第二课：连续发送时，新 Turn、Steer，还是等待？](lessons/0002-start-steer-or-queue.md)：理解输入怎样启动或调整一轮执行。
- [第三课：Thread、Session、Turn 与 Task](lessons/0003-thread-session-turn-task.md)：区分长期身份、内存运行体、执行边界和执行程序。
- [Codex 核心概念速查](reference/glossary.md)：Thread、Turn、Session、Op、Event 等术语的通俗解释。
- [资料索引](RESOURCES.md)：课程使用的当前源码入口与外部资料。

## 学习范围

第一阶段聚焦 `codex-rs` 中一条完整的本地 Agent 运行链路：命令行解析、会话启动、模型交互、工具调用、审批与沙箱，以及 TUI 如何展示运行状态。SDK、发布流程、CI、跨平台打包和实验性服务先作为扩展主题保留，不作为入门主线。

Subagent 生命周期与通信、Thread 拉起与交互是当前优先深入的两个专题，但不是全部学习范围。学习时仍需把它们放回完整的 Codex 架构中理解，避免只看到局部机制。

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

## 推荐阅读顺序

1. 阅读根目录 `README.md` 和 `docs/install.md`，了解产品定位与本地构建入口。
2. 阅读 `codex-rs/Cargo.toml`，建立 workspace 和 crate 边界的整体认识。
3. 从 `codex-rs/cli/src/main.rs` 开始，跟踪默认 TUI、`exec`、登录和服务类子命令的分发。
4. 进入 `codex-rs/tui/src/lib.rs` 与 `codex-rs/tui/src/app.rs`，理解交互界面和应用事件循环。
5. 阅读 `codex-rs/core/src/codex_thread.rs`、`codex-rs/core/src/session/` 和 `codex-rs/core/src/tasks/`，理解线程、会话、轮次和任务生命周期。
6. 阅读 `codex-rs/core/src/tools/`、`codex-rs/core/src/exec.rs` 和 `codex-rs/core/src/sandboxing/`，理解工具路由、命令执行、审批与隔离。
7. 阅读 `codex-rs/protocol/src/protocol.rs`，理解核心模块与 TUI、App Server 等调用方之间的事件协议。
8. 最后按兴趣扩展到 `codex-rs/exec`、`codex-rs/app-server`、`codex-rs/codex-mcp`、`sdk` 和各平台沙箱实现。

## 优先专题一：Subagent 生命周期与通信

这条主线重点回答：父 Agent 如何创建子 Agent、子 Agent 如何获得独立 Thread、消息如何在 Agent 之间传递、状态如何汇总，以及完成或中断后如何回收资源。

建议按以下顺序阅读：

1. `codex-rs/core/src/tools/handlers/multi_agents_v2/`：从 `spawn_agent`、`send_message`、`followup_task`、`wait_agent`、`interrupt_agent` 和 `list_agents` 等模型可见工具入口开始。
2. `codex-rs/core/src/tools/handlers/multi_agents_common.rs`：了解不同版本多 Agent 工具共享的处理逻辑。
3. `codex-rs/core/src/agent/control.rs` 与 `codex-rs/core/src/agent/control/spawn.rs`：跟踪创建、恢复、提交输入、中断和关闭 Agent Thread 的核心控制流程。
4. `codex-rs/core/src/agent/registry.rs`、`codex-rs/core/src/agent/status.rs` 与 `codex-rs/core/src/agent/role.rs`：理解 Agent 注册、状态和角色模型。
5. `codex-rs/core/src/agent_communication.rs`：理解 Agent 间消息及其上下文。
6. `codex-rs/core/src/context/inter_agent_message.rs`、`inter_agent_completion_message.rs` 和 `subagent_notification.rs`：观察通信事件如何转化为模型可见上下文。
7. `codex-rs/core/src/session/multi_agents.rs`：理解多 Agent 能力如何挂接到会话。
8. `codex-rs/core/src/agent/control_tests.rs` 与多 Agent handler 测试：用创建、完成通知、并发限制、恢复和关闭子树等场景校验理解。

建议绘制并持续修正这条生命周期：

```text
模型调用工具 -> 工具 Handler -> AgentControl -> 创建子 Thread
             -> 投递初始输入 -> 子 Agent 执行 Turn
             -> 状态/消息回传 -> 父 Agent 上下文
             -> 完成、中断或关闭 -> 释放并发槽和运行资源
```

## 优先专题二：Thread 拉起与交互

这条主线重点回答：一个 Thread 如何被创建或恢复、用户输入如何形成 Turn、操作和事件如何双向流动、状态如何持久化，以及 TUI 如何消费这些状态。

建议按以下顺序阅读：

1. `codex-rs/cli/src/main.rs`：找到默认交互模式、恢复和派生 Thread 的命令入口。
2. `codex-rs/core/src/thread_manager.rs`：理解 Thread 的创建、恢复、注册和查找。
3. `codex-rs/core/src/codex_thread.rs`：理解调用方如何通过 `submit(Op)` 向运行中的 Thread 提交操作。
4. `codex-rs/protocol/src/protocol.rs`：重点阅读 `Op` 和事件类型，建立输入与输出协议的完整词汇表。
5. `codex-rs/core/src/session/session.rs`、`turn.rs`、`handlers.rs` 与 `tasks/`：跟踪用户输入、Turn 启动、模型响应、工具执行和 Turn 结束。
6. `codex-rs/tui/src/chatwidget.rs` 与 `codex-rs/tui/src/chatwidget/turn_lifecycle.rs`：观察 TUI 如何维护正在运行、结束、中断和预算受限等状态。
7. `codex-rs/thread-store/src/lib.rs` 与 `types.rs`：理解 Thread、Turn 和状态的持久化边界。
8. `codex-rs/app-server-protocol/src/protocol/v2/`：对照外部客户端看到的 Thread/Turn API 和通知。

建议围绕一次输入跟踪完整闭环：

```text
用户输入 -> CLI/TUI -> CodexThread::submit(Op)
        -> Session/Turn -> 模型与工具循环
        -> Event -> TUI/App Server
        -> Thread Store -> 完成、恢复或下一次 Turn
```

## 使用 CodeGraph

仓库已经在本地建立 CodeGraph 索引。索引数据库位于 `.codegraph/codegraph.db`，它体积较大且可重新生成，因此不会提交到 Git。

常用命令：

```powershell
# 检查索引状态
codegraph status .

# 源码变化后增量同步
codegraph sync .

# 围绕问题查找相关源码和调用路径
codegraph explore "CLI 如何启动 TUI"

# 查看符号及其调用关系
codegraph node CodexThread
codegraph callers CodexThread
codegraph callees CodexThread
```

如果换了一台机器或删除了本地索引，可在仓库根目录重新执行：

```powershell
codegraph init .
```

## 外部技术资料

原仓库中只有一个外部链接的跳转文档已合并到这里，减少 `docs/` 目录噪声：

- [Codex 官方文档](https://developers.openai.com/codex)
- [CLI 功能与交互模式](https://developers.openai.com/codex/cli/features#running-in-interactive-mode)
- [身份认证](https://developers.openai.com/codex/auth)
- [基础配置](https://developers.openai.com/codex/config-basic)
- [高级配置](https://developers.openai.com/codex/config-advanced)
- [完整配置参考](https://developers.openai.com/codex/config-reference)
- [配置示例](https://developers.openai.com/codex/config-sample)
- [非交互模式](https://developers.openai.com/codex/noninteractive)
- [执行策略](https://developers.openai.com/codex/exec-policy)
- [沙箱与安全](https://developers.openai.com/codex/security)
- [AGENTS.md](https://developers.openai.com/codex/guides/agents-md)
- [Skills](https://developers.openai.com/codex/skills)
- [Slash commands](https://developers.openai.com/codex/cli/slash-commands)

## 学习笔记约定

- 每份笔记尽量围绕一个可验证的问题，而不是逐文件复述代码。
- 记录关键入口、核心类型、调用路径、状态变化和对应测试。
- 源码发生变化后先运行 `codegraph sync .`，再复核笔记中的路径和结论。
- 将“当前不学习”和“可以从仓库删除”分开判断。构建、测试、代码生成和平台支持文件可能不在学习主线上，但仍可能是源码正确运行所必需的。

## 后续主题

- Subagent 创建、通信、完成通知与资源回收
- Thread 创建、恢复、派生与一次 Turn 的完整生命周期
- 模型上下文的构建、压缩与持久化
- 工具注册、路由、执行和结果回传
- 命令审批、执行策略与沙箱边界
- MCP、Skills、Plugins 和 Apps 的接入方式
- App Server 协议与 SDK 的职责边界
