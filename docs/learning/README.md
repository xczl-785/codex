# Codex 源码学习课程

这里是本学习分支的课程导航。课程按照“先建立运行模型，再深入具体机制”的顺序组织，每一课围绕一个可验证的问题展开。

第一次进入仓库时，建议先阅读 [学习任务](MISSION.md) 和 [Codex 工程介绍](repository-inventory.md)。想先了解完整主题和后续阅读方向，可以查看 [学习路线图](roadmap.md)。

## 当前课程

1. [从一句话到一次 Turn](lessons/0001-from-prompt-to-turn.md)
   建立用户输入、模型请求、工具执行与回复返回的整体骨架。
2. [连续发送时：新 Turn、Steer，还是等待？](lessons/0002-start-steer-or-queue.md)
   理解新输入怎样启动、调整或等待一轮执行。
3. [Thread、Session、Turn 与 Task](lessons/0003-thread-session-turn-task.md)
   区分长期身份、内存运行体、对话边界与执行程序。
4. [从安全审计需求定位架构扩展点](lessons/0004-audit-mode-design-exercise.md)
   从需求、范围、权限和协议判断一个新模式真正改变了什么。
5. [Fork、Interrupt、编辑旧消息与 Revert](lessons/0005-fork-interrupt-edit-and-revert.md)
   理解分支、停止执行和历史重写在不同层级的响应。
6. [一个长任务为什么仍是一个 Turn](lessons/0006-one-long-turn-and-task-lifecycle.md)
   拆开 Turn、Step、工具调用和 Task 的运行与计时边界。
7. [自动压缩为什么不会切换 Task](lessons/0007-inline-auto-compaction.md)
   区分长 Turn 内联压缩与独立 `CompactTask`，理解压缩后的续跑。
8. [Turn 与 Task 为什么通常一对一仍要分开](lessons/0008-turn-and-task-separation.md)
   厘清 `SessionTask`、具体 Task、`RunningTask` 与 Turn 的职责。
9. [Session 的生命周期、上下文与界面历史](lessons/0009-session-lifecycle-context-and-history-projections.md)
   理解 Session 创建与关闭、上下文组装，以及系统保存、模型可见和模型可检索的区别。
10. [结构化工具调用为什么仍需要 Harness 校验](lessons/0010-structured-tool-calls-and-runtime-validation.md)
    区分 JSON mode、Structured Outputs 与 Tool Calling，理解模型能力和运行时验证的边界。
11. [一次 Function Call 如何完成模型—工具闭环](lessons/0011-function-call-runtime-loop-and-step-tool-snapshots.md)
    跟踪 ToolRouter、ToolRegistry、Handler 与 FunctionCallOutput，并理解每个 Step 的工具快照和调用因果链。
12. [ExecPolicy、权限提升与沙箱到底怎样配合](lessons/0012-exec-policy-escalation-and-sandbox.md)
    理解命令规则、模型提权信息来源，以及沙箱为何是受限的真实执行而不是预演。
13. [Claude Code、Cursor 与 LangGraph 的 Checkpoint 模型](lessons/0013-checkpoint-models-claude-cursor-langgraph.md)
    区分文件恢复点与图状态快照，理解连续回退、时间旅行和 LangGraph 状态膨胀。
14. [一条命令怎样变成可续接的真实进程](lessons/0014-unified-exec-process-yield-timeout-and-retry.md)
    区分 Tool Call 与进程生命周期、yield 与 hard timeout，并建立安全重试的判断框架。
15. [同一个真实进程为什么有三种输出视图](lessons/0015-one-process-three-output-views.md)
    区分 UI 实时增量、模型分段 Tool Output 与客户端命令生命周期，并理解三类 ID 和持久化边界。

## 稳定参考

- [Codex 核心概念速查](reference/glossary.md)：快速回忆 Thread、Session、Turn、Task、Op、Event 等概念。
- [Codex 工程介绍](repository-inventory.md)：目录分层、模块职责、核心链路和构建体系。
- [学习路线图](roadmap.md)：完整主题范围、源码阅读顺序和重点专题。
- [资料索引](RESOURCES.md)：课程使用的源码入口与外部技术资料。

## 阶段复盘

- [课程 0001—0015 整体知识地图](reviews/0001-lessons-0001-0015-stage-review.md)：按生命周期、状态投影、工具闭环、安全执行和真实进程重新组织前十五课，并提供综合自测场景。

## 实践任务

- [实现一个最小源码学习 Agent](practice/0001-source-study-agent.md)：通过“关键控制面手写、基础设施复用、AI 受控协作”实现可与 Codex 对照的最小 Agent Harness。

## 文档边界

`lessons/` 是面向学习者的详细课程，`reviews/` 用于跨课程阶段复盘，`practice/` 保存已经确定的实践任务与验收边界，`reference/` 是压缩后的稳定速查。`learning-records/` 记录学习者理解状态，`MAINTENANCE.md` 和 `NOTES.md` 记录教学过程；后三者用于后续教学衔接，不应被当作 Codex 架构事实或首次阅读入口。

本目录不是 Codex 的上游用户文档。产品、安装与使用说明见根目录 [上游 README](../../README.codex-upstream.md)，本地构建要求见 [docs/install.md](../install.md)。
