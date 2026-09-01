# Learning Workspace Maintenance

> **Process contract**
> 状态：`working`。本文件负责维护课程索引、长期教学决策和下一步方向；事实来源是当前课程、学习记录与仓库状态，下游使用者是后续教学会话。学习任务结束或由新的维护账本接管时退出当前状态，之后保留为过程证据，不作为 Codex 架构事实或普通课程入口。

## Current progress

- Last completed lesson: 0015
- Current lesson: 暂停推进新课，进入课程 0001—0015 的整理与回顾阶段。
- Next checkpoint: 按主题复盘既有课程，合并重复理解、标记仍不牢固的概念，并根据疑问选择少量源码链路回看。
- Current handoff: 已讲到 Unified Exec 的 UI 实时增量、模型分段 Tool Output 和客户端命令生命周期；下一位 Agent 不要默认继续扩展新概念，应先帮助用户整理已有知识并按需静态阅读源码。
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
- Lesson 0015: 同一个真实进程为什么有三种输出视图
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

- 当前阶段先整理和回顾课程 0001—0015，不急于继续主线。
- 建议按“生命周期概念、上下文与压缩、工具闭环、安全执行、Checkpoint 与真实进程”重新分组，找出重复内容和未理解点。
- 用户提出具体疑问时，可沿课程末尾的源码入口做局部静态阅读；不要为了覆盖目录而机械通读。
- 回顾完成后，再从取消传播、并行工具调度或 Subagent 生命周期与通信中选择下一条线。
- 后续有运行条件时，用临时 tracing 修改观察真实进程的 `process_id`、yield、poll、exit 与 cleanup；实验代码不长期保留。
- 后续继续细看 Thread 的 Fork、恢复与 Subagent 身份传播。
- 深入对话历史与工作区状态为何独立，以及需要同步回退时由谁协调。
- 将 Codex rollout 与 LangGraph checkpoint 的比较保留为已完成扩展，不继续偏离当前主线。
- 把 Codex 的持久记录、运行投影与 Harness 状态外置放在一起比较。
- Subagent 生命周期与通信仍是重点专题，但不是当前整理阶段必须立即开始的内容。

## Entropy pass after lesson 0005

- 复核了课程 0001—0005、学习记录 0001—0004、资料索引、教学笔记和速查表。
- 现有课程各自回答不同问题，无过期结论或需要合并删除的文档。
- 合并了 `Next likely directions` 中重复的 Turn 内部循环条目。
- 确认详细解释继续放在 `lessons/`，`reference/` 仅保留压缩后的概念速查。
