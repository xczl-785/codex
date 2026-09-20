# 实践任务 0003：持久恢复与同 Turn 上下文压缩

## 接手契约

| 字段 | 当前值 |
| --- | --- |
| 角色 | 复用 CodebaseAgent，把持久恢复和运行中自动压缩拆成两个可验证阶段，并映射到 Codex 当前实现 |
| 状态 | `planned / awaiting CodebaseAgent activation` |
| 事实来源 | CodebaseAgent 当前源码与测试、课程 0007/0009/0016、当前 Codex 源码与测试 |
| 下游使用者 | 下一次负责激活并推进 CodebaseAgent 实验的 Agent 与学习者 |
| 进入条件 | 实践 0002 已完成；开始时先更新 CodebaseAgent 自己的 `docs/STATUS.md`，只激活 0003-A |
| 退出条件 | 学习者能够独立复述两个阶段的事件序列、状态所有权和失败边界；已激活的实现切片有对应证据 |
| 退出后处置 | 记录未激活的体验优化候选，然后进入实践 0004“取消传播” |

当前是否已经进入本实践只看 [`MAINTENANCE.md`](../MAINTENANCE.md) 与 CodebaseAgent 自己的 `docs/STATUS.md`。本文件规定实验边界，不代替工程实时状态。

## 为什么重新划分

旧版 0003 同时承担两种不同任务：复盘 CodebaseAgent 已完成的显式 checkpoint 与恢复能力，以及理解 Codex 已有的同 Turn 自动压缩。前者在 CodebaseAgent 中已有较完整实现，后者尚未实现；把两者都标记为“已完成”会混淆教学完成和工程能力。

本次保留一个实践编号，但按生命周期拆成两个较完整的阶段：

```text
0003-A：已经发生的事实，退出后怎样可信地恢复？
0003-B：正在运行的 Turn 装不下时，怎样压缩并继续？
```

学习者的要求是理解和复述，而不是掌握全部实现细节。实现和测试用于提供确定性证据，不为追求产品完整度扩张主题。

## 通用推进规则

每个阶段同时维护两类内容：

- **核心机制**：决定该阶段是否完成的控制流、不变量和最小验证场景。
- **关联优化**：依附于同一生命周期、能够改善可观察性或使用体验的功能候选。

关联优化使用以下状态：

- `candidate`：已知与本阶段相关，但尚未要求实施；不是隐性待办，不阻塞阶段完成。
- `active`：学习者明确提出后，写入 CodebaseAgent `docs/STATUS.md`，成为当前实现切片。
- `completed` / `deferred`：完成或依据证据延后。

学习者提出候选优化后，推进 Agent 可以在本阶段内直接落地，无需另建实验；但仍要确认目标体验、责任所有者、事实来源和是否改变核心行为。UI、CLI 或应用投影不得成为第二套恢复、容量或压缩事实来源。

## 0003-A：持久事实、检查点与恢复边界

### 要回答的问题

> 进程退出甚至崩溃后，新 Session 能根据哪些事实继续？哪些状态绝对不能恢复？

### 核心机制

- 持久 Thread 日志、Session 运行态 History、当前 Step 的 ModelRequest 和 UI 历史不是同一个对象；
- 保存不等于模型可见，模型可见不等于可按需检索；
- 显式 checkpoint 替换后续模型使用的有效历史，但不删除旧 Thread 事实；
- 恢复使用最近有效 checkpoint 作为基线，并重放 checkpoint 后的事实；
- 恢复必须创建新 Session，不复活旧 Future、文件句柄、进程或网络请求；
- 未完成 Turn、只有 Call 没有 Output、已有 Output 但没有终态必须得到不同处理；
- 外部副作用未知时不得因为缺少 Output 就自动重试。

### 最小验证场景

1. **正常恢复**

   ```text
   Turn A 完成 → 进程退出 → 新 Session 恢复 A → 继续 Turn B
   ```

2. **连续 checkpoint 与后续事实**

   ```text
   A → checkpoint H1 → B → checkpoint H2 → C → 退出
     → 新 Session 的模型投影为 H2 + C
   ```

   全量日志仍保留 A、H1、B、H2、C；模型投影不得包含被 H2 覆盖的旧原文。

3. **悬空 Call**

   ```text
   FunctionCall 已持久化 → Output 尚未持久化 → 崩溃
     → CallOutcomeUnknown → Interrupted → 不自动执行工具
   ```

4. **已有 Output、Turn 未终止**

   ```text
   Call → 真实 Output 已持久化 → 尚未得到下一次模型响应 → 崩溃
   ```

   恢复必须保留真实 Output，不能改写为 unknown，也不能重复执行工具。

### 当前证据与待补强处

CodebaseAgent 已具备追加式 Thread 日志、显式 checkpoint、新 Session 恢复、Interrupted 与 CallOutcomeUnknown 修复等核心机制。开始本阶段时先静态核对当前源码和测试，不重新实现已有能力。

优先补强三项最有区分力的证据：

- 连续 checkpoint 最终选择 H2，并重放 H2 后事实；
- checkpoint 后已有完整 Turn，退出再恢复时仍得到 summary + post-checkpoint Turn；
- 悬空 Call 恢复过程的工具执行次数为零。

### 关联优化候选

以下项目初始均为 `candidate`：

- 恢复 Session 后展示旧对话历史；
- 在界面中区分旧 Session 历史与当前 Session 新对话；
- 标记 checkpoint 边界以及模型当前使用的恢复基线；
- 展示本次恢复实际执行的尾部修剪、OutcomeUnknown 和 Interrupted 修复；
- 在旧历史中展示结果未知的 Call，并解释未自动重试的原因；
- 提供 checkpoint 前历史的只读审计入口；
- 改善 Session 列表中的创建时间、最近活动时间、消息数量和损坏诊断。

这些优化应消费 Thread store 或应用层提供的只读投影。TUI 不自行解析 JSONL、修复日志或重新推导模型 History。

### 学习者完成检查

学习者应能独立解释：

1. 为什么持久 Thread 有旧消息，模型仍可能看不到原文？
2. checkpoint 为什么不是删除历史？
3. 为什么恢复必须创建新 Session？
4. checkpoint 后事实如何接到最新恢复基线上？
5. 悬空 Call 与已有 Output 的中断 Turn 为什么必须区别处理？
6. 哪些内容属于持久事实、模型投影、UI 投影和不可恢复运行资源？

## 0003-B：上下文容量与同 Turn 自动压缩

### 要回答的问题

> 当前 Turn 尚未结束，但下一次完整模型请求已经装不下时，运行时怎样压缩历史并安全继续？

### 核心机制

- 每个 Step 在调用 Provider 前对完整 ModelRequest 做容量准入；
- 容量判断必须区分内部规范计量与 Provider 的真实 token/window；
- 已完成的 Call/Output 不因后续请求超限而消失；
- 自动压缩与 Turn 外显式 checkpoint 是不同生命周期；
- 自动压缩成功后安装 replacement history，并在原 Turn/Task 中进入下一次采样；
- checkpoint 的持久化、运行态 History 替换和后续采样顺序必须明确；
- 压缩失败、压缩后仍超限以及连续压缩必须有有界终止语义；
- 恢复时仍以最近有效 checkpoint 加后续事实重建，而不是恢复旧 Task。

### 实现深度

第一轮只需要确定性压缩替身，例如 `ScriptedCompactor` 或固定 replacement history。它用于验证控制流、持久化顺序和失败边界，不要求先接真实模型摘要，也不要求实现摘要质量判断。

核心成功路径：

```text
用户问题
→ Step 1 产生 Call/Output
→ 下一次 ModelRequest 达到压缩条件
→ 生成确定性 replacement history
→ 持久化 checkpoint
→ 替换 Session History
→ 原 Turn 进入下一次采样
→ 返回最终回答
```

至少验证两条失败路径：

```text
压缩请求失败
→ 原 Turn 按已确认语义结束
→ 已持久化事实保留
```

```text
压缩后仍然超限或没有有效缩减
→ 有界终止
→ 不进入无限压缩循环
```

具体的触发阈值、失败终态和 checkpoint 安装顺序属于控制面。开始实现前由学习者表达预期，Agent 走查正常、失败和边界路径后再落地。

### 关联优化候选

以下项目初始均为 `candidate`：

- TUI 展示当前上下文使用量和接近阈值提示；
- 时间线展示压缩开始、成功、失败及继续采样；
- 展示压缩前后的 History 条目数或规范字节数；
- 提供显式手动压缩入口；
- 展示模型当前使用哪个 checkpoint 作为基线；
- 恢复 Session 时提示当前模型使用压缩历史而非完整原文；
- 为压缩失败、无有效缩减和重复超限提供稳定用户错误；
- 在真正超限前允许用户主动新建会话。

容量策略、是否压缩和失败终态由 Agent/runtime 拥有；UI 只消费状态与事件，不自行决定或执行压缩。

### 学习者完成检查

学习者应能独立解释：

1. 容量检查为什么发生在每个 Step 的 Provider 调用前？
2. 自动压缩与显式 checkpoint 有什么不同？
3. 自动压缩成功后为什么仍是同一个 Turn/Task？
4. 压缩成功和失败分别留下哪些持久事实？
5. 为什么压缩后仍超限必须有界终止？
6. 进程退出后恢复的是何种历史基线，为什么不是旧 Task？

## 共同视图检查

两个阶段都使用同一张视图表检查概念是否混淆：

| 内容 | 持久日志 | 模型直接可见 | TUI 当前对话 | 可恢复运行资源 |
| --- | --- | --- | --- | --- |
| checkpoint 前原文 | 是 | 否 | 取决于已激活的历史展示能力 | 不适用 |
| replacement history | 是 | 是 | 默认只显示恢复/压缩提示 | 不适用 |
| checkpoint 后事实 | 是 | 是 | 取决于 Session 与展示投影 | 不适用 |
| 旧 Future、进程或连接 | 否 | 否 | 否 | 否 |

## Codex 源码接入方式

开始每个阶段时先用 CodeGraph 重新定位当前实现，不把旧路径当作不变事实：

```powershell
codegraph explore "run_turn auto compact replace_compacted_history rollout recovery model visible history"
```

重点责任包括：

- `codex-rs/core/src/session/turn.rs`：容量判断、采样循环与内联自动压缩；
- `codex-rs/core/src/compact.rs`：压缩输入、摘要请求和 replacement history；
- `Session::replace_compacted_history`：运行投影、持久 checkpoint 与替换顺序；
- Thread/rollout 恢复：最近 checkpoint 与后续事实的投影；
- App Server/TUI 历史：面向人的视图，不等同于模型 Prompt。

## 当前不纳入

- Provider tokenizer 的精确计量与输出 token 预留；
- 摘要质量优化、history search、向量记忆；
- 完整历史浏览产品、数据库迁移和分布式一致性；
- Fork/Revert 的完整存储语义；
- 取消传播和并行工具调度，它们继续由实践 0004、0005 承担。

上述项目若被真实场景阻塞或由学习者明确选中，再建立相应切片；不因技术相关就自动扩大 0003。
