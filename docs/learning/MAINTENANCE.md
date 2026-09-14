# Learning Workspace Maintenance

> **Process contract**
> 状态：`working`。本文件负责维护课程索引、长期教学决策和下一步方向；事实来源是当前课程、学习记录与仓库状态，下游使用者是后续教学会话。学习任务结束或由新的维护账本接管时退出当前状态，之后保留为过程证据，不作为 Codex 架构事实或普通课程入口。

## Current progress

- Last completed lesson: 0015
- Current lesson: 用户于 2026-09-14 确认权限、沙盒实验已完成；实践 0002 结束，下一主题为实践 0003。本轮未重新验收独立实验工程的代码与测试。
- Next checkpoint: 进入实践 0003，先贯通持久事实、运行态 History 与模型可见投影，再用容量边界、同 Turn 自动压缩、退出后恢复和结果未知四个场景对照源码。
- Current handoff: 先读学习记录 0011 的最新进展与实践 0003。不要重新安排权限实验 M0/M1；复用 CodebaseAgent 已有 M3 实践，对照 Codex 的压缩与恢复链路，不另建一套持久化系统。
- Last entropy pass: after lesson 0015。

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
- Stage review 0001: 课程 0001—0015 整体知识地图
- Practice 0001: 实现一个通用代码库理解 Agent
- Practice 0002: 权限、审批、沙箱与提权实验
- Practice 0003: 上下文限制、压缩检查点与历史恢复复盘
- Reference: Codex 核心概念速查
- Learning record 0001: 已有概念基础与学习方式
- Learning record 0002: Thread、Turn 与 App Server 理解基线
- Learning record 0003: 从生命周期定义转向设计动机
- Learning record 0004: 先确定需求边界，再判断扩展点
- Learning record 0005: 运行态是持久事实的可重建投影
- Learning record 0006: 原生 Tool Calling 已成为理解基线
- Learning record 0007: Step 工具快照与调用因果链
- Learning record 0008: Checkpoint 边界与设计偏好
- Learning record 0009: 从阶段复盘转入受控实践
- Learning record 0010: 从 Harness 实践收尾转入权限与上下文专题
- Learning record 0011: 从资源范围与命令规则重新建立权限基础，已完成案例校准

## Durable decisions

- 课程使用中文，类比和流程优先于符号记忆。
- 事实优先取自当前分支源码与测试。
- 课程与速查统一使用纯 Markdown；不维护 H5 和样式资产，文档沉淀不得拖慢对话反馈。
- 跨多课的阶段复盘放在 `reviews/`，按主题重组理解，不改变 `lessons/` 的线性课程职责，也不冒充下一课。
- 已确定的实践任务放在 `practice/`，明确目标、非目标、行为契约和验收标准；实验源码不直接加入 Codex workspace。
- 实践默认使用 Rust，采用“关键控制面手写、基础设施复用、AI 受控协作”，目标仓库保持语言无关，先以小型 fixture repository 和 ScriptedModel 验证 Harness，再接真实模型。
- 第一版通过 Repository Binding 限制代码库根目录，所有工具输出有界并携带相对路径和行号；Codex、CodeGraph 和 Git 都不是硬依赖。
- 实践 0001 收尾后，学习顺序固定为：权限与沙盒实践 → 上下文、压缩与恢复复盘 → 取消传播 → 并行工具调度 → Subagent 生命周期。上下文复盘与权限实践没有技术依赖，这个顺序用于集中解决当前最薄弱概念。
- 权限专题深入控制面和失败语义，但不手搓安全级操作系统沙箱；上下文专题深入事实、投影、检查点和恢复不变量，中等深度定位 Codex 源码，暂不深入 tokenizer、摘要算法、分布式存储或向量记忆。

## Next likely directions

- 如需核对独立 CodebaseAgent 的实现或文档收尾状态，读取该工程自己的 `docs/STATUS.md`；不把此前收尾状态当作下一主题的新增阻塞条件，本仓库不复制其当前文件清单。
- [实践 0002](practice/0002-permission-sandbox-lab.md) 已由用户确认完成；仅在后续具体疑问触发时回看，不重复安排实验。
- 执行 [实践 0003](practice/0003-context-history-recovery-lab.md)：使用 CodebaseAgent 已实现的 M3 行为与 Codex 源码，对照容量准入、模型可见历史、持久事实、压缩检查点和新 Session 恢复。
- 完成两项复盘后进入取消传播；只有取消、提交顺序和竞态语义稳定，才学习真正的并行工具调度。
- Subagent 生命周期最后进入，重点研究父子 Thread、权限继承或收窄、通信、等待、中断和资源回收，不因已有多 Agent 工具就提前模仿接口。
- LangGraph checkpoint 横向比较保留为已完成扩展；只有上下文复盘出现具体疑问时才回看，不另开泛化框架调研。

## 实践 0003 开场准备（2026-09-14）

- 首个问题：Thread 明明保存了旧消息，为什么压缩后模型仍看不到原文？先区分持久记录、Session 运行态 History、当前模型请求和 UI 历史，再跟踪一次压缩前后的变化。
- 本轮轻量核对：工作区原先无未提交变更；CodeGraph 报告索引最新。抽查 `Session::replace_compacted_history`，确认其构造带有 `replacement_history` 的 `CompactedItem`、替换运行态历史并追加 rollout 记录。此结论不代表已经验证全部恢复与容量分支。
- 下一次源码阅读：从 `core/src/session/turn.rs` 的容量与自动压缩分支进入，对照 `core/src/compact.rs` 和 `Session::replace_compacted_history`，随后定位恢复投影和代表性测试。使用 CodeGraph 定位后按需精读，不全仓展开。
- 用一段“旧消息 → 压缩检查点 → 新消息 → 退出 → 新 Session”的事件序列，分别说明保存了什么、模型看得到什么、运行资源能否恢复；再加入一个只有 Call、没有 Output 的崩溃边界。
- 本次只准备衔接，不认定实践 0003 已完成。退出标准仍以实践文档的完成检查为准；通过后依次学习取消传播、并行工具调度、Subagent 生命周期。

## Entropy pass after lesson 0015

- 复核了课程 0001—0015、学习记录 0001—0008、速查、路线图、维护账本和教学笔记。
- 通过当前 CodeGraph 索引抽查了 Turn/Task/Step、自动压缩、模型—工具闭环和 Unified Exec 关键链路，课程结论与当前源码保持一致。
- 将前十五课重组为执行生命周期、历史与投影、模型—工具闭环、权限与安全执行、真实进程与输出五个主题簇。
- 提炼了四条跨模块设计原则：生命周期分离、保存与可见性分离、逐层信任校验、按消费者建立视图。
- 标记了五类已学但仍需贯通的概念，以及取消传播、并行工具调度、Subagent 生命周期三类尚未系统学习的主题。
- 原课程中的重复多数承担逐层加深作用，暂不合并或删除；阶段复盘负责提供跨课程统一入口。

## Entropy pass after lesson 0005

- 复核了课程 0001—0005、学习记录 0001—0004、资料索引、教学笔记和速查表。
- 现有课程各自回答不同问题，无过期结论或需要合并删除的文档。
- 合并了 `Next likely directions` 中重复的 Turn 内部循环条目。
- 确认详细解释继续放在 `lessons/`，`reference/` 仅保留压缩后的概念速查。
