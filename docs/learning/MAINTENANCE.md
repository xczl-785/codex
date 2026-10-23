# Learning Workspace Maintenance

## 教学前置要求：必须加载 teach

任何 Agent 开始或接续本仓库教学前，必须先完整加载并遵守 `teach` Skill，不能用记忆、摘要或仅阅读本计划代替。用户指定位置为 `C:/Users/Admin/.cc-switch/skills/teach/SKILL.md`。同一会话已完整加载且内容未变时可复用；新会话必须重新加载。路径失效时先查找已安装的同名 Skill，无法找到则明确说明，不得静默跳过。

加载后按 MISSION → MAINTENANCE → NOTES → 相关学习记录与课程恢复状态；按“具体场景与设计目的 → 主链路 → 实现与必要边界”讲解，依据用户反馈决定下一步。新建课程前按 teach 检查文档熵阈值。学习工作区位于 `docs/learning/`，不在仓库根目录另建一套教学资料。

> **Process contract**
> 状态：`working`。本文件负责维护课程索引、长期教学决策和下一步方向；事实来源是当前课程、学习记录与仓库状态，下游使用者是后续教学会话。学习任务结束或由新的维护账本接管时退出当前状态，之后保留为过程证据，不作为 Codex 架构事实或普通课程入口。

## Current progress

- Last completed lesson: 0024；Subagent 专题（0021—0024）按已约定学习目标收尾。0019 是按需答疑骨架，0020 是待深入课程，不随编号计入已完成。
- Current lesson: 第一批理论与实践已收束。Queue/Steer 理论及场景回顾见记录 0017；2026-10-23 用户确认 Subagent 实践完成，本次核对了独立工程的 M0.5 与 M1—M4 交付记录。第二批尚未授课。
- Next checkpoint: 进入第二批 Skills 的发现与加载：围绕一个 Skill 从磁盘文件到模型上下文的过程，核对扫描、元信息暴露、显式指定与自动选择、正文及引用资料加载的职责。先讲场景和主链，再查当前源码与测试；不重新安排 M0.5 或 Subagent M1—M4。
- Current handoff: 已讲 Subagent 设计、创建、历史过滤、配置、消息与触发、等待状态、中断、回收、关闭和归档，并完成 A2A 概念扩展。用户反馈设计先行后理解清晰；A2A 与 OpenCode Skill 的映照只作扩展，不继续安排委派器改造。
- Evidence limits: durable sleep 的生产宿主触发和持久恢复链路未定位；App 创建独立 Thread 的专用实现未定位；未据此认定 Codex 原生支持 A2A。均不阻塞本节完成，不标记为已验证。
- Persistence follow-up: [课程 0020](lessons/0020-persistence-projections-and-recovery.md)保留 SQLite 答疑与容量样本，正式深入排在第二批之后。新版编辑同 Thread Revert 已修正于课程 0005，不能推出子树自动回滚。
- Practice boundary: [实践 0006](practice/0006-subagent-lifecycle-lab.md)保留学习目标、设计输入与验收场景；独立工程拥有实现、契约及测试证据。本次依据用户确认和交付记录更新学习衔接，未重新运行测试或评审源码。协作专用界面仍延期，偶发 Responses 协议错误仍由目标工程 STATUS 跟踪，不把实践收束扩大为全部平台、Provider 或异常路径均已验收。
- Practice evidence (2026-10-23): 独立工程 `D:/Program/AgentLab/CodebaseAgent/docs/archive/2026-10-21-steer-delivery-and-experience.md` 记录 M0.5 技术交付与用户体验验收；`docs/archive/2026-10-22-subagent-delivery.md` 记录 M1—M4 交付、恢复修复及用户结束试用。路径均属独立工程，未复制其测试批次或工程待办；当前能力见其 `docs/capabilities/subagents.md`，实时状态见其 `docs/STATUS.md`。
- Last entropy pass: 本次在新增 0023/0024 前完成，覆盖截至 0022 的资料与教学状态。
- Last entropy pass lesson: 22

## Current index

- Lesson 0021: Subagent 消息交付与执行
- Lesson 0022: Subagent 设计、创建、历史与配置
- Lesson 0023: Subagent 中断、回收、关闭与归档
- Lesson 0024: A2A 跨系统委派与协议价值

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
- Practice 0006: Subagent 委派、通信与生命周期（含 M0.5 Steer 前置实践；用户确认完成，作为回看材料保留）
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
- Learning record 0016: Subagent 理解变化、扩展与专题收束
- Learning record 0017: Queue、Steer 与 Turn 边界
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
- 实践 0001 收尾后的依赖顺序为：权限与沙盒实践 → 持久恢复与同 Turn 自动压缩 → 取消传播 → 并行工具调度 → 单 Agent Steer（0006 M0.5）→ Subagent 生命周期。恢复与压缩分为 0003-A/0003-B 两个阶段，但仍属于一个实践主题。
- 权限专题深入控制面和失败语义，但不手搓安全级操作系统沙箱；上下文专题深入事实、投影、检查点和恢复不变量，中等深度定位 Codex 源码，暂不深入 tokenizer、摘要算法、分布式存储或向量记忆。
- 答疑课按需开启，不参与主循环。它以 `on-demand` 状态常驻 `lessons/`，使用独立编号但不写入 Current lesson 与 Next checkpoint，不推进主线顺序，也不在前一课外形成新的阻塞条件；产物是随时可查的概念澄清，而不是线性课程。
- 答疑课的沉淀方式：用户提问、Agent 查源码核对、当场解释，然后建立骨架；正文按用户后续需要再补，不因骨架存在而自动排入主线。

## Next likely directions

- 第一批理论与实践已经收束；下一课进入第二批 Skills 发现与加载。0006 的依赖顺序保留作理解路线，不再作为待实施队列。
- A2A 扩展已达到用户要求深度。协议细节、完整接入和委派器改造均不自动激活；进入 MCP 对比时按需引用课程 0024。
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

## Entropy pass through lesson 0022

- 新课前最高编号 22，上次为 15，差值 7，已达到 teach 的强制检查阈值；此前未及时执行，本次补齐后再创建 0023/0024。
- 检查 RESOURCES 的本地入口与用途，合并重复 Turn 处理入口，移除不匹配的结构化输出外链入口；补入 Subagent 核心源码与 A2A 一手资料。
- 压缩 NOTES 中重复、已过期的“继续 Subagent/等待”指令，保留设计先行、以场景建立因果链、加快不等于堆术语等长期偏好。
- 检查记录 0001—0016 的主题、理解证据与后续用途；保留早期认识历史，标记过期阶段安排，收束 0016 的最新理解边界。未删除用户原始材料。
- reference 两份资料分别承担概念速查与权限后端参考，无需合并；为术语表补充协作与 A2A 概念。assets 为空，没有待迁移的实验代码；practice/materials 是已有实践契约的配套材料，保持入口与职责。
- 同步课程索引、路线与维护账本；0019/0020 继续保持按需/待深入状态。未重新验收独立实践、未逐课重审早期全部源码和所有外部链接。
