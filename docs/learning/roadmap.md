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

## 第一批课程收尾：Subagent 与 A2A

课程 0001—0018 已建立单 Agent 执行主链，Subagent 是这一批的最后一个专题。当前进度仍由 [MAINTENANCE.md](MAINTENANCE.md) 维护。该专题已按下列问题完成，课程记录为 0021—0024；0019 仍是按需答疑。

| 顺序 | 核心问题                             | 学完应能说明                                                        |
| ---- | ------------------------------------ | ------------------------------------------------------------------- |
| 1    | 创建一个 Subagent 究竟创建了什么？   | 子 Thread、运行体、父子身份与已有单 Agent 主链怎样衔接              |
| 2    | 子 Agent 从哪里获得上下文与权限？    | 历史继承、角色、配置与权限边界分别由谁决定                          |
| 3    | 父子 Agent 怎样通信、等待并汇总？    | 发消息、交付任务、状态查询与完成通知的区别                          |
| 4    | 完成、中断、关闭与恢复分别处理什么？ | 取消、资源回收、并发名额及恢复身份的边界                            |
| 5    | 跨系统 Agent 协作为什么还需要 A2A？  | 本地父子关系与远端 Agent 服务的差异；发现、任务、消息、结果如何对应 |

A2A 按 Agent2Agent 协议理解，是用户明确加入的对照主题。授课时核对 [A2A 官方规范](https://a2a-protocol.org/latest/specification/)和版本，结合 Agent Card、Task、Message、Artifact 等协议对象讲解发现、状态、取消和认证边界；本节已完成概念与生命周期对照，完整取消和认证实现留待实际接入需求，不表示 Codex 已实现或遵循 A2A。先用一个远端分析 Agent 的场景说明通信双方各自拥有什么状态，再回到本地 Subagent 比较。协议字段和完整服务端实现不作为记忆要求。

## 第二批课程：能力扩展与接入

实践衔接先补[课程 0002 的 Steer](lessons/0002-start-steer-or-queue.md)，再做[实践 0006 的 M0.5](practice/0006-subagent-lifecycle-lab.md#m05单-agent-steer-前置实践)，之后继续 Subagent 实践。前置目标是贯通运行中输入的准入、模型可见性与结束竞态；Skills/MCP 的主题顺序保留，当前进度以维护账本为准。

本批次已根据用户兴趣排在 Subagent 后面；以下拆分和顺序是教学建议，尚未授课。主问题是：一种外部能力怎样从“可发现”变成“模型可用”，再进入已有执行闭环？Skills 加载和 MCP 接入是主线，用户的 Skill 想法在相关课内讨论，不自动转为实现任务。

| 顺序 | 主题与问题                                                                                                      | 建议难度 / 深度   | 完成标准                                                                                                    |
| ---- | --------------------------------------------------------------------------------------------------------------- | ----------------- | ----------------------------------------------------------------------------------------------------------- |
| 1    | Skills 的发现与加载：从哪里扫描、先暴露哪些元信息、何时读取正文和引用资料、显式指定与自动选择如何配合？         | 中 / 深入主链     | 能追踪一个 Skill 从文件到模型上下文的过程，分清发现、选择、读取和执行                                       |
| 2    | Skills 的执行边界与设计讨论：指令、脚本、工具、权限各管什么；多个 Skill 如何组合；与 AGENTS.md 的职责如何区分？ | 中高 / 深入边界   | 能用用户自己的想法判断哪些适合 Skill，哪些需要工具或运行时支持，并说明失败后果                              |
| 3    | MCP 接入闭环：配置、连接与初始化、能力协商、工具发现、注册、调用、结果与错误怎样接起来？                        | 中高 / 深入一条链 | 能沿一个 MCP 工具从服务端到 FunctionCallOutput 走完闭环，并与 A2A 比较责任边界                              |
| 4    | 内部工具、Skill、MCP 横向对比：发现与加载、能力声明、执行主体、权限、生命周期、分发和失败反馈分别由谁负责？     | 中高 / 深入选型   | 能沿同一个需求比较三种承载方式，判断该复用、组合还是新增能力；不把 Skill 文本、工具接口与接入协议视作同一层 |
| 5    | 动态工具与连接生命周期：延迟发现、工具列表变化、Step 快照、连接复用、超时、断连和取消怎样协作？                 | 高 / 关键路径     | 能解释“服务已连接、工具已发现、当前 Step 可调用”为何需要分别判断                                            |
| 6    | Plugins、Apps 与 Connectors：Skill、工具和连接配置怎样被组织、安装、启用并暴露？                                | 中 / 中等         | 能画出组合与依赖关系，分清能力内容、分发载体、连接和授权                                                    |
| 7    | 配置与客户端综合串联：配置来源、覆盖与生效时机，TUI/App Server 入口怎样影响能力可见性？                         | 中高 / 中等       | 能定位“已配置却不可用”发生在哪一层；用完整场景贯通前六个主题                                                |

推荐先 Skills 后 MCP：先看已有 Agent 怎样获得新的工作方法，再看怎样接入新的外部执行能力；MCP 接入之后专门对比内部工具、Skill 与 MCP，再用动态生命周期和组合机制把它们放回同一条运行链。A2A 在 Subagent 中先建立概念，进入 MCP 时再用同一远端服务场景比较，避免只记协议名称。

横向对比采用同一个“读取项目资料并生成分析结果”的场景，逐项追踪三种方式的入口、上下文、执行路径和结果反馈；再讨论组合使用。对比的具体结论要由当前源码和协议验证，不预设三者是互斥替代方案。

每个主题采用“一个具体问题 → 当前源码与测试 → 正常路径 → 一个失败场景 → 简短复述”的节奏。重点记职责、不变量和可定位入口，不背全部类型名、协议字段或配置项。Skill 设计想法尚未收集，先保留问题位置，不凭空填写需求。

本批次之后已明确安排[持久化、投影与恢复](lessons/0020-persistence-projections-and-recovery.md)，建议难度高，深入主链和失败语义。已保存本轮答疑，正式授课时重点区分事实、投影、控制元数据、增量追赶、重建与异常恢复，并用容量和故障场景校准理解。该课保持在上述 Skills/MCP 批次之后，不因已保存答疑而提前开课。之后再挑选模型流式响应与重试、TUI 事件映射、SDK 等长期主题。

阅读源码时优先使用仓库 CodeGraph 索引；命令和资料入口见 [资料索引](RESOURCES.md)。
