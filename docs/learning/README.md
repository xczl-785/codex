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
16. [压缩怎样整理历史，以及哪些信息仍然可见](lessons/0016-compaction-mechanism-and-visible-history.md)
    跟踪压缩触发、摘要请求、replacement history、检查点恢复与信息损失边界。
17. [停止 Turn，究竟停止了什么](lessons/0017-cancellation-and-process-lifetimes.md)
    区分取消传播、任务清理、命令执行模式和完成竞态。
18. [工具并行资格与结果顺序](lessons/0018-parallel-tool-admission-and-result-order.md)
    区分工具能力声明、具体调用依赖、共享/独占门禁、完成顺序和历史记录顺序。

## 已完成的 Subagent 专题

按理解顺序先读创建，再读通信、生命周期与跨系统扩展；编号是文档标识，不要求机械按数字学习。

- [设计、创建、历史与配置（0022）](lessons/0022-subagent-history-ownership-and-configuration.md)：独立 Thread 对照、委派归属、创建链路与能力来源。
- [消息交付与执行（0021）](lessons/0021-subagent-message-delivery-and-execution.md)：send_message、followup_task、等待、状态和模型可见性。
- [中断、回收、关闭与归档（0023）](lessons/0023-subagent-interruption-eviction-and-archive.md)：任务、运行体、协作关系和归档状态的区别。
- [A2A 跨系统委派（0024）](lessons/0024-a2a-and-cross-system-delegation.md)：任务生命周期、实际框架接入及协议与自定义委派的关系。

专题已按约定目标收尾，保留未核实边界，不等于所有分支均经运行验证。

Subagent 实践的历史依赖路线如下；当前状态仍以维护账本为准。

实践的依赖顺序是[课程 0002](lessons/0002-start-steer-or-queue.md)的 Steer 机制与[实践 0006 的 M0.5](practice/0006-subagent-lifecycle-lab.md#m05单-agent-steer-前置实践)，之后是 Subagent 实践。理论回顾与实现验收是不同环节；当前完成情况和下一动作统一见 [MAINTENANCE.md](MAINTENANCE.md)，不因这条依赖路线重复开设已完成的理论专题。

## 第二批课程：Skills 与能力接入

- [0025：Skill 怎样从文件变成模型可用的指令](lessons/0025-skills-catalog-loading-and-refresh.md)：目录、显式选择、正文注入与长度限制、Core/扩展去重，以及文件变化如何刷新列表。

- [0026：Skill 包的职责、脚本权限与使用记录](lessons/0026-skills-package-execution-and-usage-tracking.md)：辅助资源的分工、脚本权限，以及隐式使用识别的动机、消费者和证据限制。

- [0027：Skill 发现范围与输入身份](lessons/0027-skills-discovery-and-identity.md)：模块工作目录、扫描根、路径身份，以及界面名称与结构化提交。

- [0028：Skill 清单怎样随上下文更新](lessons/0028-skills-context-updates-and-incremental-tools.md)：world state、提示缓存、增量工具与协议边界。

- [0029：怎样设计并验证一个有用的 Skill](lessons/0029-skill-design-and-validation.md)：选择依据、正文与引用分工、脚本和正反例验证。

- [0030：Skills 接入职责与验收边界](lessons/0030-skills-integration-contracts.md)：完整请求链、状态区分及实践需要验证的行为。

- [0031：从只读理解到编码执行](lessons/0031-coding-agent-tools-and-permission-boundaries.md)：编码闭环、补丁工具定位、通用权限边界与面向 Agent 的工具设计。

- [0032：文件变化、部分失败与历史重建](lessons/0032-file-change-conflicts-and-history-recovery.md)：补丁匹配、并发边界、变更归属、撤销和 rollout 中的恢复依据。

- [0033：验证闭环的三层职责](lessons/0033-verification-tools-and-model-judgment.md)：宿主工具能力、检查程序规则、LLM 判断及强制门禁的区别。

- [0034：子 Agent 写入与集成](lessons/0034-subagent-writes-and-integration.md)：实际执行约束与模型分工、共享工作区冲突、完成报告及组合结果判断。

第一批已由 Subagent 与 A2A 对照收尾。第二批先完成 Skills 加载与设计讨论及接入实践，再在本仓库开展 [CodebaseAgent 转型课程](roadmap.md#codebaseagent-转型课程与实践)；学完后补充实践任务书并按流程实践。随后继续 MCP 接入、内部工具 / Skill / MCP 横向对比、动态工具生命周期、Plugins/Apps 组合及配置与客户端边界。具体安排见[路线图](roadmap.md#第二批课程能力扩展与接入)，逐课难度与历史理解证据见[记录 0015](learning-records/0015-review-and-extension-phase.md)。已讲内容见课程 0025—0034，其余安排不计入已完成课程。

已另行安排[持久化、投影与恢复](lessons/0020-persistence-projections-and-recovery.md)：当前保存答疑与本机容量样本，正式深入放在第二批能力扩展课程之后，不计入已完成课程。

## 按需答疑

- [规划是能力还是组件](lessons/0019-planning-capability-vs-component.md)：`on-demand` 答疑骨架，正文待补；仅在相关疑问出现时展开，不计入已完成课程，也不改变主线顺序。

## 稳定参考

- [Codex 核心概念速查](reference/glossary.md)：快速回忆 Thread、Session、Turn、Task、Op、Event 等概念。
- [权限机制落地参考](reference/permission-enforcement-backends.md)：配合第 12 课，查阅控制面、执行适配与沙箱后端的分工、产品实现和接入边界。
- [Codex 工程介绍](repository-inventory.md)：目录分层、模块职责、核心链路和构建体系。
- [学习路线图](roadmap.md)：完整主题范围、源码阅读顺序和重点专题。
- [资料索引](RESOURCES.md)：课程使用的源码入口与外部技术资料。

## 阶段复盘

- [Skills 整体回顾（0025—0030）](reviews/0003-skills-end-to-end-review.md)：从一次任务串联发现、选择、上下文与执行，定位实践验收边界。

- [课程 0001—0015 整体知识地图](reviews/0001-lessons-0001-0015-stage-review.md)：按生命周期、状态投影、工具闭环、安全执行和真实进程重新组织前十五课，并提供综合自测场景。
- [权限实验之后的上下文、取消与并行复盘](reviews/0002-after-permission-context-cancellation-parallel.md)：归纳课程 0016—0018 的理解边界，区分源码场景校准与可运行实践证据。

## 实践入口

- Skills 实践回看材料：[0007：Skills 接入体系](practice/0007-skills-integration-lab.md)。用户已确认实践完成；任务书保留课程内容与设计参考，具体实现范围及证据由 CodebaseAgent 工程维护。当前教学衔接见 [MAINTENANCE.md](MAINTENANCE.md)。

实践输入与契约、验收材料和暂存的工程改进调研统一从[实践目录](practice/README.md)查阅，不纳入课程阅读顺序。教学衔接见 [MAINTENANCE.md](MAINTENANCE.md)，独立工程的实时实施与 deferred 状态以该工程自己的状态文档为准。

## 接续教学

接续已有教学时，按根目录 [AGENTS.md](../../AGENTS.md) 恢复上下文：任务目标 → 当前维护账本 → [教学笔记](NOTES.md) → [学习记录](learning-records/README.md) → 对应课程。学习记录中的早期认识和历史笔记用于解释理解变化，不决定当前下一步。

## 文档边界

`lessons/` 是面向学习者的详细课程，`reviews/` 用于跨课程阶段复盘，`practice/` 的材料分类与责任见其[目录入口](practice/README.md)，`reference/` 是压缩后的稳定速查。`learning-records/` 记录学习者理解状态，`MAINTENANCE.md` 和 `NOTES.md` 记录教学过程；后三者用于后续教学衔接，不应被当作 Codex 架构事实或首次阅读入口。

本目录不是 Codex 的上游用户文档。产品、安装与使用说明见根目录 [上游 README](../../README.codex-upstream.md)，本地构建要求见 [docs/install.md](../install.md)。
