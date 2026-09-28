# 答疑课 0019：规划是能力还是组件

> 状态：`on-demand`（随时开启的答疑课，骨架已建立，正文待补）。
> 本文件**不参与主课程线性序列**：不占用 `MAINTENANCE.md` 的 Current lesson 与 Next checkpoint 衔接点，不推进主线顺序，可在任意时间由用户提问开启。
> 正文尚未按课程标准完成，不作为已完成的课程结论引用。

本课要回答一个概念性的问题："规划"到底是 Agent 的一个核心组件，还是只是一种能力要素？以及为什么在 Codex 源码里找不到一个独立的 planner 模块。

## 待回答的问题（本课的驱动问题）

1. LLM/Tools/Memory/Planning 这个"四分核心"是从哪里来的，它描述的是什么？
2. 为什么在 Codex 源码里看不到独立"规划组件"？是缺失还是有意设计？
3. 具体应用落地时，这四种能力分别被映射到了哪里？
4. 哪些 agent application 真的采用或更适合 Plan-and-Execute，判据是什么？
5. ReAct 与 PAE 是否已经被内化进模型内部？内化的边界在哪里？

## 章节结构

### 第一节：四分法的来源与它描述的对象
- 待补：说明该分类出自 LLM agent 综述视角，属于**能力清单**而非**模块清单**。
- 待补：区分"隐式规划"（模型权重能力）与"显式规划"（外部可干预工件）。
- 待补：指出这一混淆是"看不到规划组件"这一困惑的根源。

### 第二节：Codex 里规划呈现的三种形态
- 形态 1：模型内部隐式规划（主体）。
  - 已核实锚点：`codex-rs/core/src/session/turn.rs:142-156`（`run_turn` 的行为契约注释）、`session/turn.rs:304`（主循环）。
  - 已核实锚点：`codex-rs/protocol/src/prompts/base_instructions/default.md:52`（`## Planning` 引导段）。
- 形态 2：`update_plan` 工具（薄外壳，非控制器）。
  - 已核实锚点：`codex-rs/protocol/src/plan_tool.rs:9-29`（`StepStatus` / `UpdatePlanArgs`，注释自认 `not plan mode`）。
  - 已核实锚点：`codex-rs/core/src/tools/handlers/plan.rs:87-98`（Plan mode 下拒绝；仅发送 `EventMsg::PlanUpdate`，不改变控制流）。
  - 待补：说明该工具的真实职责是可协作性与自我对齐，不是执行顺序来源。
- 形态 3：Plan mode（`ModeKind::Plan`，协作模式）。
  - 已核实锚点：`codex-rs/collaboration-mode-templates/templates/plan.md:3`（plan 必须 `decision complete`，可交接给他人或另一 agent）。
  - 已核实锚点：`plan.md:11-15`（Plan Mode 与 `update_plan` 互相排斥）。
  - **待补（重点）**：纠正"Codex 没有 PAE"的直觉——Plan mode 就是 PAE 形态，Codex 是"默认 ReAct、按需切换 PAE"，而不是二选一。

### 第三节：为什么 Codex 不把规划做成独立组件
- 待补：单决策者原则（多决策者带来的同步、仲裁、错误归属成本）。
- 待补：对话连续性——每步观测都会改变后续决策，外部 plan 偏离后只能强行执行或频繁重规划。
- 待补：prompt cache 与上下文前缀稳定性。
- 待补：PEA 真正的适用区间（可预先分解、子步骤独立、需并行或复用）。

### 第四节：PAE 的适用判据与实例
- 待补：整理判断信号表（步骤可预先枚举、成本/能力不对称、审批闸门、可并行分发、上下文有界、跨 session 交接）。
- 待补：显式 PAE 实例——Workflow 类平台、LangChain `Plan-and-Execute`、BabyAGI、AutoGPT、Devin、Manus（混合型，plan 主要作可见性投影）。
- 待补：研究侧实例——Voyager（计划作为可复用资产）、MetaGPT/ChatDev（SOP 分工）、orchestrator-worker（分解 + 并行 + 汇总）。
- 待补：反推 Codex 默认不使用 PAE 的原因，并指出"把 Codex 当无人值守 worker 批量跑"时 PAE 收益重新出现——这解释 Plan mode 与 subagent 的存在。

### 第五节：内化的边界
- 待补：ReAct 作为**提示词格式**已过时（reasoning models 在 latent 中完成思考，interleaved thinking 把工具调用插入推理流）。
  - 已核实锚点：`codex-rs/core/src/session/turn_context.rs:262`、`:429`（`ReasoningEffortConfig`，思考预算已成为可调参数）。
- 待补：ReAct 作为**执行循环**不会过时——"采样→执行→回填"必须留在 harness，因为环境在模型之外。
- 待补：PAE 中 plan 作为**工件**时承载的四件模型内化不了的事：可见性与授权、跨上下文存活、跨 agent 交接、跨时间复用。
- 待补：给出统一结论——被内化的是**能力**（规划、推理、决定何时行动），留在 harness 的是**接口与约束**（存储、可见、审批、执行、观测）。
- 待补：由此推出四分法的当代用法：不是"模型需要哪些模块"，而是"harness 需要替模型补齐哪些它做不到的事"。
- 待补：框架"瘦身"现象的对照解释（提示词模板退场，执行循环/记忆/权限加重）。

### 第六节：Memory 的同构性
- 待补：说明 Memory 同样不是独立模块——Codex 的记忆分布在对话历史（thread/rollout）、compaction（`codex-rs/core/src/compact.rs`）与可选的 `codex-rs/memories/` 之间。
- 待补：用这一点证明"四分法都不应被读成模块清单"。

## 源码入口

- [主循环与行为契约](../../../codex-rs/core/src/session/turn.rs)：`run_turn` 的注释与 `loop`。
- [Plan mode 模板](../../../codex-rs/collaboration-mode-templates/templates/plan.md)：PAE 形态的 plan 工件与交接契约。
- [`update_plan` 参数类型](../../../codex-rs/protocol/src/plan_tool.rs)：checklist 语义，注释自认非 plan mode。
- [`update_plan` 处理器](../../../codex-rs/core/src/tools/handlers/plan.rs)：Plan mode 下拒绝、仅发送事件。
- [基础指令中的 Planning 段](../../../codex-rs/protocol/src/prompts/base_instructions/default.md)：自然语言层面的规划引导。
- [思考预算配置](../../../codex-rs/core/src/session/turn_context.rs)：`ReasoningEffortConfig`。

## 待核验与待补充

- 待核验：`ModeKind` 的完整取值集合与各模式对工具白名单的实际影响。
- 待核验：`<proposed_plan>` 块在 harness 侧是否被解析为结构化数据，还是仅作为文本交给下游。
- 待补充：各 PAE 实例的公开资料引用（不在本仓库内，需标注为外部资料）。
