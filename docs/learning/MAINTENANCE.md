# Learning Workspace Maintenance

> **Process contract**
> 状态：`working`。本文件负责维护课程索引、长期教学决策和下一步方向；事实来源是当前课程、学习记录与仓库状态，下游使用者是后续教学会话。学习任务结束或由新的维护账本接管时退出当前状态，之后保留为过程证据，不作为 Codex 架构事实或普通课程入口。

## Current progress

- Last completed lesson: 0018（并行资格、共享/独占门禁、结果顺序及取消后的部分完成）
- Current lesson: Subagent 生命周期专题进行中。已讲创建子 Thread、独立运行体及创建调用完成与子任务完成的区别；当前穿插 SQLite 历史投影答疑。
- Next checkpoint: 先回应新版编辑重发与 SQLite 投影问题，再回到 Subagent 生命周期、恢复与通信；父子工具/Skills/MCP 和权限差异为用户已提出、留待后续的问题。A2A 仍在专题后半段。
- Current handoff: 用户已正确区分 spawn 调用完成与子任务未完成，并追问生命周期、重启、编辑旧消息和历史存储。已合入远端 `origin/main` 的 `89461ea6`，合并提交 `8885f93e`；新版 TUI 编辑旧消息改走同 Thread 的 Revert，旧版 Fork 结论已在课程 0005 和资料索引修正。SQLite 可重建历史投影答疑沉淀于课程 0009；不能据此把所有 SQLite 状态或父子 Agent 关系视作可丢弃缓存。旧实践安排不重新激活，具体实践验收仍由 CodebaseAgent 的 `docs/STATUS.md` 拥有。
- Last entropy pass: after lesson 0015。
- Last entropy pass lesson: 15

## Current index

- Lesson 0016: 压缩怎样整理历史，以及哪些信息仍然可见
- Lesson 0017: 停止 Turn，究竟停止了什么
- Lesson 0018: 工具并行资格与结果顺序

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
- Stage review 0002: 权限实验之后的上下文、取消与并行复盘
- Practice 0001: 实现一个通用代码库理解 Agent
- Practice 0002: 权限、审批、沙箱与提权实验
- Practice 0003: 持久恢复与同 Turn 上下文压缩（0003-A 恢复；0003-B 自动压缩）
- Practice 0004: 取消传播与部分完成状态
- Practice 0005: 有界并行工具调度与结果顺序
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
- Learning record 0012: 压缩、恢复与历史可见性
- Learning record 0013: 取消与进程生命周期边界
- Learning record 0014: 并行准入、结果顺序与职责分离
- Learning record 0015: 逐课回顾、记忆深度与扩展阶段衔接

### 答疑课（不参与主循环）

- [答疑课 0019](lessons/0019-planning-capability-vs-component.md): 规划是能力还是组件 — 状态 `on-demand`，仅在相关疑问触发时开启

## Durable decisions

- 课程使用中文，类比和流程优先于符号记忆。
- 事实优先取自当前分支源码与测试。
- 课程与速查统一使用纯 Markdown；不维护 H5 和样式资产，文档沉淀不得拖慢对话反馈。
- 跨多课的阶段复盘放在 `reviews/`，按主题重组理解，不改变 `lessons/` 的线性课程职责，也不冒充下一课。
- 已确定的实践任务放在 `practice/`，明确目标、非目标、行为契约和验收标准；实验源码不直接加入 Codex workspace。
- 实践默认使用 Rust，采用“关键控制面手写、基础设施复用、AI 受控协作”，目标仓库保持语言无关，先以小型 fixture repository 和 ScriptedModel 验证 Harness，再接真实模型。
- 第一版通过 Repository Binding 限制代码库根目录，所有工具输出有界并携带相对路径和行号；Codex、CodeGraph 和 Git 都不是硬依赖。
- 实践 0001 收尾后，学习顺序固定为：权限与沙盒实践 → 持久恢复与同 Turn 自动压缩 → 取消传播 → 并行工具调度 → Subagent 生命周期。恢复与压缩分为 0003-A/0003-B 两个阶段，但仍属于一个实践主题。
- 权限专题深入控制面和失败语义，但不手搓安全级操作系统沙箱；上下文专题深入事实、投影、检查点和恢复不变量，中等深度定位 Codex 源码，暂不深入 tokenizer、摘要算法、分布式存储或向量记忆。
- 答疑课按需开启，不参与主循环。它以 `on-demand` 状态常驻 `lessons/`，使用独立编号但不写入 Current lesson 与 Next checkpoint，不推进主线顺序，也不在前一课外形成新的阻塞条件；产物是随时可查的概念澄清，而不是线性课程。
- 答疑课的沉淀方式：用户提问、Agent 查源码核对、当场解释，然后建立骨架；正文按用户后续需要再补，不因骨架存在而自动排入主线。

## Next likely directions

- Subagent 已开始，当前 SQLite 答疑结束后继续主线。它是第一批课程最后一个专题：创建与父子 Thread → 上下文与权限 → 通信与等待 → 中断、关闭与回收 → A2A 对照。
- A2A 按 Agent2Agent 协议理解，纳入 Subagent 后半段；以官方规范为外部事实来源，讲清远端 Agent 的发现、任务、消息、结果及生命周期与本地 Subagent 的对应和差异。不预设 Codex 已实现 A2A。
- 第二批课程围绕能力扩展与接入，建议顺序为 Skills 发现与加载 → Skills 执行边界及用户设计想法 → MCP 接入闭环 → 内部工具 / Skill / MCP 横向对比 → 动态工具与连接生命周期 → Plugins/Apps 组合 → 配置与客户端边界综合串联。详细问题与完成标准只在[路线图](roadmap.md#第二批课程能力扩展与接入)维护；主题已排入，具体拆课和顺序可随用户问题调整。
- 实践 0002 已由用户确认完成；0003—0005 保留为实践契约和按需回看材料，不按旧记录重新安排。若要实施或确认某项能力，再到独立工程读取自己的 `docs/STATUS.md`、源码和测试。
- 已有学习记录不是即时记忆测试。重点用权限链路、恢复投影、工具调用与进程的边界做短场景复习，不重讲所有基础名词。
- LangGraph checkpoint 横向比较已收束，仅在具体疑问触发时回看。

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
