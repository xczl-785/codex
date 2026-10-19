# A2A：独立 Agent 系统怎样委派工作

状态：扩展已讲，当前深度足够。目标是把 Subagent 的内部协作与跨系统协议分开，并理解标准化的价值；不实现 A2A 服务，不推断 Codex 原生支持该协议。资料查询日期：2026-10-19。

## 同一个模块调研问题，换一个工作边界

A 在 Codex 协作运行时中创建 B 时，运行时掌握父子身份、配置、历史和消息通道。若 B 是另一团队已经部署的代码分析服务，A 无权直接创建或操纵它的内部 Thread，也未必知道其框架和工具。

双方需要约定服务在哪里、能做什么、怎样交付材料、任务是否被接受、何时需要补充输入以及怎样取得报告。A2A（Agent2Agent）把这些交互约定为协议。远端内部仍可使用一个 Agent、多个 Subagent 或其他工作流，调用方不必看到其内部工具和记忆。[官方核心概念](https://a2a-protocol.org/v1.0.1/topics/key-concepts/)

## 调研任务的主链路

以下任务内容是教学场景，不是已执行的服务调用。

```text
A 获取 B 的服务说明 → 发送仓库、版本与调研要求
    → B 返回可跟踪任务 → B 调研或要求补充 → A 获取报告并验收
```

| 对象       | 例子                               | 不应误解为                                |
| ---------- | ---------------------------------- | ----------------------------------------- |
| Agent Card | B 的能力、端点、认证和交互支持说明 | 运行中的 B 或完整实现代码                 |
| Message    | 调研要求、补充版本、澄清问题       | 每条都必须新建一个长期任务                |
| Task       | 本次调研的工作 ID、状态及生命周期  | Codex 内部驱动 Turn 的 Task 类型          |
| Artifact   | 分析报告、调用链图、结构化结果     | 只能是最终聊天文字                        |
| contextId  | 关联一组相关交互或任务             | 跨框架统一的内存 Session 或本地 Thread ID |

A 的模型判断是否委派及如何使用结果，客户端代码承担协议请求和接收；B 自己执行工作。拿到 Task ID 说明有可跟踪的工作，不表示 B 成为 A 的子 Agent。

## 接单、补充信息与完成

以 LangSmith Agent Server 为具体接入例。它为 Assistant 提供 /a2a/{assistant_id}。2026-10-19 查阅文档支持选择立即返回已提交/工作中任务，而非等运行结束；也支持查询、取消和流式发送。任务订阅和推送通知配置当时未实现，不能把协议可选能力当作所有实现默认具备。[官方实现说明](https://docs.langchain.com/langsmith/server-a2a)

B 收到“分析数据库恢复链路”，若版本不明确，工作流可以暂停要求输入。LangSmith 将图的 interrupt 映射为 INPUT_REQUIRED，调用方携带任务和上下文标识补充信息，让原任务继续。其 resume 数据形式是具体实现映射，不是任意 A2A 服务都必须接受的私有字段。

```text
提交 → 工作中 → 需要输入 → 补充信息 → 工作中 → 完成
```

需要输入属于可继续状态；完成、失败、取消等终态结束该 Task。后续新工作不能机械复用已结束任务。协议请求成功、任务被接受、产物足够回答用户问题分别由通信、服务状态与调用方验收判断。[官方任务生命周期](https://a2a-protocol.org/v1.0.1/topics/life-of-a-task/)

## 哪些实现支持，以及“流行”的证据边界

- Google ADK 提供远端 A2A Agent 接入与服务暴露能力：[官方指南](https://google.github.io/adk-docs/a2a/quickstart-consuming/)。
- Microsoft Agent Framework 提供远端调用和 A2A 托管：[官方说明](https://learn.microsoft.com/en-us/agent-framework/journey/agent-to-agent)。
- LangSmith Agent Server 提供 Assistant 的 A2A 端点，具体支持范围以上述版本文档为准。

这些是开发和部署能力，不表示所有现成聊天产品默认互联。A2A 于 2026-03 发布 1.0；Linux Foundation 于 2026-04 报告超过 150 家组织支持及企业生产使用。这支持“已有明显生态影响”的判断，但支持组织数不等于部署数或市场份额。[1.0 发布](https://a2a-protocol.org/dev/blog/2026/03/12/a2a-protocol-ships-v10-production-ready-standard-for-agent-to-agent-communication/) · [采用情况](https://www.linuxfoundation.org/press/a2a-protocol-surpasses-150-organizations-lands-in-major-cloud-platforms-and-sees-enterprise-production-use-in-first-year)

## 与已有 OpenCode 委派 Skill 的映照

用户提供的 codex-opencode-delegate 已约定：Codex 界定任务，启动器落实模式和隔离，OpenCode 执行，返回 run_id、状态、摘要、补丁与审查材料，最后由 Codex 独立验证。任务输入、运行状态和交付物与 A2A 的关注点重叠，但不是字段一一对应；sessionID 与 run_id 怎样映射上下文和任务必须另行设计。

该 Skill v1 的统一入口要求 spawn --wait，异步 spawn、followup 和 cancel 仍为保留能力；不能以命令名存在就判断已经实现。上述依据用户提供并读取的 Skill 文本，未审查启动器全部源码，也未执行 OpenCode。

Skill 还承担权限、worktree/临时副本隔离和验收流程。A2A 不自动提供这些保障。固定 Codex → OpenCode 的调用链可以继续使用自己的契约；多个异构调用方和执行端接入时，共同协议可减少逐对定制适配。将来可以在现有委派器外增加协议适配层，但这只是设计可能，不是本轮实施计划。

用户提出“在一个节点把已有概念整理并稳固下来”，准确指出标准化的价值：委派思想先于协议，协议把共同需求变成可互操作、可演进的具体约定，包括消息格式、任务状态和能力声明。它不负责发明协作，也不能替调用方验收结果。

本扩展到此收束。Skills/MCP 留到各自主题，进入 MCP 横向对比时可复用这一远端服务场景。
