# Learning Workspace Maintenance

Last entropy pass lesson: 0

## Current index

- Lesson 0001: 从一句话到一次 Turn
- Lesson 0002: 连续发送时：新 Turn、Steer，还是等待？
- Lesson 0003: Thread、Session、Turn 与 Task
- Lesson 0004: 从安全审计需求定位架构扩展点
- Lesson 0005: Fork、Interrupt、编辑旧消息与 Revert
- Reference: Codex 核心概念速查
- Learning record 0001: 已有概念基础与学习方式
- Learning record 0002: Thread、Turn 与 App Server 理解基线
- Learning record 0003: 从生命周期定义转向设计动机
- Learning record 0004: 先确定需求边界，再判断扩展点

## Durable decisions

- 课程使用中文，类比和流程优先于符号记忆。
- 事实优先取自当前分支源码与测试。
- 课程与速查统一使用纯 Markdown；不维护 H5 和样式资产，文档沉淀不得拖慢对话反馈。

## Next likely directions

- 细看 Thread 的创建、恢复、派生与事件通道。
- 进入普通 Turn 内部，理解 Step、模型请求和工具调用循环。
- 由用户围绕 Thread、Session、Turn、Task 主动提问，按问题选择下一条源码链路。
- 深入对话历史与工作区状态为何独立，以及需要同步回退时由谁协调。
- 细看一次 Turn 内部的模型与工具循环。
- 在整体链路稳定后进入 Subagent 生命周期。
