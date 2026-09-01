# Learning Workspace Maintenance

> **Process contract**
> 状态：`working`。本文件负责维护课程索引、长期教学决策和下一步方向；事实来源是当前课程、学习记录与仓库状态，下游使用者是后续教学会话。学习任务结束或由新的维护账本接管时退出当前状态，之后保留为过程证据，不作为 Codex 架构事实或普通课程入口。

## Current progress

- Last completed lesson: 0014
- Current lesson: 准备进入 0015 — 一条命令为什么有三种输出视图
- Next checkpoint: 区分 UI 实时 Event、模型分段 Tool Output 与最终持久化记录，跟踪截断、chunk 和完成事件。
- Current handoff: 已区分 Tool Call、后台真实进程、yield、hard timeout 与语义重试；`session_id` 只是模型可见的内部 `process_id`。后续源码实验需观察同一进程从分配到移除的事件序列。
- Last entropy pass: after lesson 0005。该字段只表示最近一次全量内容复核点，不表示课程停在第五课。

## Current index

- Lesson 0001: 从一句话到一次 Turn
- Lesson 0002: 连续发送时：新 Turn、Steer，还是等待？
- Lesson 0003: Thread、Session、Turn 与 Task
- Lesson 0004: 从安全审计需求定位架构扩展点
- Lesson 0005: Fork、Interrupt、编辑旧消息与 Revert
- Lesson 0006: 一个长任务为什么仍是一个 Turn
- Lesson 0007: 自动压缩为什么不会切换 Task
- Lesson 0008: Turn 与 Task 为什么通常一对一仍要分开
- Lesson 0009: Session 的生命周期、上下文与界面历史
- Lesson 0010: 结构化工具调用为什么仍需要 Harness 校验
- Lesson 0011: 一次 Function Call 如何完成模型—工具闭环
- Lesson 0012: ExecPolicy、权限提升与沙箱到底怎样配合
- Lesson 0013: Claude Code、Cursor 与 LangGraph 的 Checkpoint 模型
- Lesson 0014: 一条命令怎样变成可续接的真实进程
- Reference: Codex 核心概念速查
- Learning record 0001: 已有概念基础与学习方式
- Learning record 0002: Thread、Turn 与 App Server 理解基线
- Learning record 0003: 从生命周期定义转向设计动机
- Learning record 0004: 先确定需求边界，再判断扩展点
- Learning record 0005: 运行态是持久事实的可重建投影
- Learning record 0006: 原生 Tool Calling 已成为理解基线
- Learning record 0007: Step 工具快照与调用因果链
- Learning record 0008: Checkpoint 边界与设计偏好

## Durable decisions

- 课程使用中文，类比和流程优先于符号记忆。
- 事实优先取自当前分支源码与测试。
- 课程与速查统一使用纯 Markdown；不维护 H5 和样式资产，文档沉淀不得拖慢对话反馈。

## Next likely directions

- 下一课区分 UI 实时 Event、模型分段 Tool Output 和持久化命令记录。
- 完成输出视图后，进入并行工具调度，再进入 Subagent 生命周期与通信。
- 后续有运行条件时，用临时 tracing 修改观察真实进程的 `process_id`、yield、poll、exit 与 cleanup；实验代码不长期保留。
- 后续继续细看 Thread 的 Fork、恢复与 Subagent 身份传播。
- 深入对话历史与工作区状态为何独立，以及需要同步回退时由谁协调。
- 将 Codex rollout 与 LangGraph checkpoint 的比较保留为已完成扩展，不继续偏离当前主线。
- 把 Codex 的持久记录、运行投影与 Harness 状态外置放在一起比较。
- 在整体链路稳定后进入 Subagent 生命周期。

## Entropy pass after lesson 0005

- 复核了课程 0001—0005、学习记录 0001—0004、资料索引、教学笔记和速查表。
- 现有课程各自回答不同问题，无过期结论或需要合并删除的文档。
- 合并了 `Next likely directions` 中重复的 Turn 内部循环条目。
- 确认详细解释继续放在 `lessons/`，`reference/` 仅保留压缩后的概念速查。
