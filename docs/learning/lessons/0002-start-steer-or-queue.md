# 第二课：连续发送时，新 Turn、Steer，还是等待？

本课回答同一个问题：用户再次输入时，怎样决定它参与当前执行还是成为后续工作？普通立即发送在空闲时启动新 Turn，在执行中优先 Steer；显式 Queue 则等待后续提交。严格 Steer 的接收结果与客户端失败后的补救必须分开理解。

## Turn 是执行过程，不是聊天气泡

可以把 Turn 想成一个“执行信封”。模型思考、工具调用、审批和继续推理都发生在里面，直到完成、中断或失败才封口。

信封尚未封口时补一句“先别改文件，只分析”，系统可以把它交给当前 Turn。这叫 **Steer**：不另开一轮，而是补充或修正当前方向。

## 四种常见情况

| 状态                               | 处理方式                                 |
| ---------------------------------- | ---------------------------------------- |
| Thread 空闲                        | `turn/start` 启动新 Turn                 |
| Turn 正在运行且可调整              | `turn/steer` 送入当前 Turn，Turn ID 不变 |
| 界面认为在运行，但核心发现它刚结束 | 刷新状态，再按新 Turn 处理               |
| 当前 Turn 不可 Steer               | 保留、排队或恢复输入，不让文字悄悄丢失   |

Steer 请求携带预期 Turn ID，是为了避免并发状态变化时把补充消息误送进另一轮。

因此，“连续发送三条消息有几个 Turn”没有固定答案：

- 每次等上一轮结束再发：通常一个 Thread、三个 Turn。
- 第一轮运行时连续补充且 Steer 成功：可能一个 Thread、一个 Turn。

## App Server 属于哪一层

App Server 的确是高度封装的统一入口，但不是比 Thread、Turn 更高一级的领域概念。

- Thread、Turn 是系统管理的领域对象。
- App Server 是客户端访问这些对象的接口和协调边界。
- TUI、IDE、SDK 通过它启动、调整和观察执行。

可以把 Thread/Turn 看成工单和一次执行，把 App Server 看成接单台兼翻译员。

## Queue 与 Steer：交付方式先于内容理解

Agent 正在分析恢复机制时，“重点看历史文件缺失”可以通过 Steer 修正眼前分析，也可以通过 Queue 留到当前分析完成后再做一次聚焦分析。同一句话并不能唯一决定交付方式。

| 维度 | Queue | Steer |
| --- | --- | --- |
| 用户安排 | 完成当前 Turn 后再处理 | 纳入当前 Turn 的后续执行 |
| 等待边界 | 空闲且允许后续提交 | 当前执行内允许消费输入的位置 |
| 正常归属 | 后续 Turn | 当前 Turn |
| 对当前模型请求 | 不提前交付排队正文 | 不改写已经发出的请求 |
| 对已执行工具 | 不自动取消或撤销 | 不自动取消或撤销 |

当前 TUI 中，普通输入的默认 Enter 用于立即提交，运行中走 Steer；Tab 用于 Queue。补全弹窗等输入上下文可能先消费 Tab，不应据此判断 Queue 不存在。桌面 GUI 有自己的 Follow-up behavior 默认设置，不能把其默认动作当成 TUI 的全局模式。操作说明见 [OpenAI 官方文档](https://learn.chatgpt.com/docs/prompting#steering-and-queuing)（2026-10-20 核对）。

TUI 的 `queue_user_message_with_options_and_source` 将等待消息标记为 `Unsent`；`maybe_send_next_queued_input` 在仍有活动或待启动 Turn 时不发送，空闲后也需通过暂停、恢复等其他条件。Steer 的 Core 待消费队列则服务于活动执行。两者都可能使用队列结构，但等待的生命周期边界不同。

## 用户消息、模型采样与 Turn 不是一一对应

| 对象 | 责任 |
| --- | --- |
| 用户消息 | 表达一段要求或补充信息 |
| 模型请求／采样 | 根据当时上下文生成下一步响应 |
| 运行时 Turn | 管理一段执行过程，包含多次采样、工具调用和可能的补充输入 |

```text
Turn T1 开始
  用户消息：分析恢复机制
  模型请求 1 → 请求读取文件
  工具执行 → 返回文件内容
  消费 Steer：重点看历史文件缺失
  模型请求 2 → 根据原任务、工具结果和补充继续分析
  ……按需继续模型—工具循环……
  最终收口，T1 结束
```

一个 Turn 可以包含多条用户消息。运行时并不是让模型从 Turn 开始到结束始终进行同一次生成，而是反复组织模型请求和工具执行。正常消费用户 Steer 时，`record_pending_input` 走 `record_user_prompt_and_emit_turn_item` 路径，将其作为用户输入追加到历史，再用于后续请求；不是回头改写已发请求或删除已有工具结果。

模型通过消息角色、内容与此前上下文理解补充。是否创建新 Turn 由运行时控制，不需要模型先判定 Turn ID 才能理解用户要求。即使 Steer 写着“恢复先不研究了，改查权限”，业务目标变化也不必然创建新运行时 Turn；反过来，新建 Turn 也不意味着换了业务目标。

## 目标过期：Core 拒绝与 TUI 补救是两层

严格 Steer 携带 `expectedTurnId = T1`。Core 持有活动 Turn 锁完成目标检查与入队，锁不覆盖后续模型处理过程。

| Core 当时状态 | 本次严格 Steer 结果 |
| --- | --- |
| T1 活动且允许 Steer | 接收并返回 T1 |
| T1 已结束，没有活动任务 | `NoActiveTurn`，App Server 返回 `no active turn to steer` 请求错误 |
| 活动任务已是 T2 | `ExpectedTurnMismatch`，返回预期与实际 ID |
| 活动任务为不可 Steer 的独立 Review/Compact | `ActiveTurnNotSteerable` |

以上拒绝不表示本次输入已交付给别的 Turn。但当前 TUI 的 `try_submit_user_turn` 收到结果后会执行自己的补救策略：

1. **NoActiveTurn**：清除本地过期活动 ID，退出 Steer 尝试，转入 `turn_start` 提交路径。若成功创建新轮，这次输入属于新 Turn。
2. **ExpectedTurnMismatch**：更新本地活动 ID，使用服务端报告的实际 ID重试一次 Steer。代码注释指出 Review 切换可能先于界面处理通知；这是上层重新选择目标，并非原请求通过了严格校验。
3. **重试仍不匹配**：更新本地记录并返回错误，不无限重试。
4. **不可 Steer**：尝试将被拒输入转为排队；无法排队时显示错误。
5. **其他错误**：返回错误，不能假定自动重发已经成功。

```text
用户立即发送 → TUI 尝试 Steer T1
  → Core：没有活动 Turn，拒绝
  → TUI：更新本地状态，改走 turn_start
  → 成功后成为 T2 的输入
```

界面看起来“消息发出去了”，可能经历了拒绝与另一次提交。SDK 调用方和其他客户端不应被假定拥有与 TUI 相同的补救策略；严格 API 的保证与产品的便利行为要分别描述。

## 为 Subagent 实践补齐的运行时边界

考虑一个普通 Agent：它正在读取恢复相关代码，你补充“把范围缩到历史文件缺失”。原任务仍在继续，但后续分析应纳入这个要求。Steer 解决的是把新输入交给当前执行过程；若每次补充都另开并发 Turn，就必须额外解决同一 Thread 上两个执行过程争用历史和结果归属的问题。

这里至少有三个时刻：

1. 调用方提交补充输入。
2. 运行时确认目标 Turn 可以接收，将输入放入待消费队列。
3. 执行循环在允许消费的边界取出输入，记录到历史，再构造后续模型请求。

第二步成功不能证明第三步已经发生，也不能证明模型已经按新要求采取行动。已经发出的模型请求不会因为队列里新增文字而自动改写。Steer 本身也不是工具取消命令，不能撤销已经发生的副作用。

当前源码的职责分工是：

- `SessionIo::submit_turn_input` 等待 Core 的路由决定；其返回不是等待整个业务任务完成。
- `Session::steer_input` 持有活动 Turn 锁，检查是否有活动任务、预期 Turn ID 是否一致，以及任务是否可 Steer。普通任务可以接收，独立 Review/Compact 任务会被拒绝；这里不能把独立 Compact Task 与常规 Turn 内自动压缩混为一谈。检查通过后，输入进入队列。
- `run_turn` 在允许的循环边界取出待消费输入，运行输入 hooks 并记录历史，随后准备模型请求。源码明确保留初始输入优先采样等延后消费条件，因此不应概括为“任何时刻立即生效”。

还要区分严格 Steer 与 `StartOrSteer`：前者针对预期的活动 Turn，调用方不能把失败理解成已经启动下一轮；后者在没有活动 Turn 时有创建新 Turn 的分支。界面可以根据失败再决定保留或重发，这属于上层策略，不是严格 Steer 自身偷偷切换目标。

这为 Subagent 提供的是公共运行机制的基础：接收方正在执行时，如何接收新的输入。子身份、普通消息邮箱、空闲唤醒和结果通知仍有各自职责，不能把全部 Agent 间通信统称为 Steer。

对应的[实践 0006 M0.5](../practice/0006-subagent-lifecycle-lab.md#m05单-agent-steer-前置实践)先在单 Agent 上验证这些边界，再进入子 Agent 创建与通信。仅凭入队路径不能断言所有输入都一定被模型消费。

## 已接收输入与结束竞态：继续执行和保存历史不同

以下是当前源码的静态核对，不代表已运行故障测试，也不覆盖所有取消、hooks 与持久化失败组合。

正常采样及工具处理返回后，`run_turn` 计算：

```rust
needs_follow_up = model_needs_follow_up || has_pending_input;
```

因此，即使模型这一请求已经给出答复，只要观察到待处理输入，运行时仍有继续循环的理由。模型响应完成不等于 Codex Turn 已完成。是否最终继续还受错误、预算、hooks 等控制条件影响。

`run_turn` 返回后，外层 `RegularTask::run` 还会检查待输入；没有终止错误而存在待输入时，再次调用 `run_turn`，复用相同的 `TurnContext`。函数被再次调用不等于创建新 Turn。

最后的收尾仍可能与新输入竞争。`on_task_finished` 在活动 Turn 锁内取走 task、标记 finishing，然后取出该 Turn 的剩余待输入，通过 `run_hooks_and_record_inputs` 进入记录流程。正常获准输入可以保存在历史里，但该收尾记录步骤不执行一次新的模型采样，不能将其称为本轮模型已处理。hooks 阻断与存储失败仍需分别判断。

Steer 准入也使用活动 Turn 锁：若收尾已经取走 task，之后的 Steer 会因没有活动任务而失败；若 Steer 先入队，收尾能够取到剩余输入。这个互斥边界协调接收与收尾，不提供“所有已接收输入都一定在原 Turn 得到模型答复”的保证。

实践中至少分开观察：请求已提交、Core 已接收、历史已记录、模型请求包含输入、业务已完成。持久保存和模型可见性各有证据，不能用一个“发送成功”状态覆盖全部阶段。

## TUI 如何保留输入并协调显示

客户端保留的待确认输入不是 Core 待消费队列的重复实现：它保存尚待交付证据的本地副本、附件和展示信息，以便处理拒绝、中断及回执。Core 则管理活动执行能消费什么。

当前 TUI 的 `enqueue_rejected_steer` 从 pending steers 取出对应输入，将正文、来源和历史记录信息移到 rejected steers 队列，并刷新预览。它不是丢掉输入后仅显示错误，也不等同于后端已经消费。

Esc 的行为必须从输入状态解释。`interaction.rs` 在任务运行且存在 `pending_steers`、没有弹窗或 Vim 编辑模式抢先消费按键时，会设置 `submit_pending_steers_after_interrupt = true` 并提交 Interrupt。随后 `on_interrupted_turn` 进入中断后提交分支，重新提交仍待确认的 Steer；普通文本输入可以合并后提交。用户观察到的“Steer 还挂着，按 Esc 后中断执行并发送”正对应这条显式路径，不是罕见的手工配置例外。

没有走上述专门分支时，`on_interrupted_turn` 才将待输入恢复到输入框。因此不能把“Esc + pending Steer”与“中断后恢复普通 Queue 草稿”混成一个场景。独立 Review、弹窗和 Vim 模式另有按键处理条件。中断针对当前 Turn，实际工具和进程的停止范围仍遵循各自取消契约。

收到记录用户消息的回执时，`on_committed_user_message` 优先以 client ID 匹配 pending steer，移除待确认项并更新历史显示；旧服务端不回传 ID 时有内容比较回退。它也检查最近显示的 client ID 以避免重复展示。同样的文字可能是两次合法提交，消息身份比只比较正文更可靠；这里的展示去重不代表全链路执行去重。

记录回执证明的是记录/展示链路上的进展，不是模型已读回执。下文补充传输断线后的输入协调；普通请求错误及所有回执时序组合并未穷尽核对，不能从这些局部处理推出所有错误都会安全自动重发。

### pending 显示、Esc 后的 Turn 身份与多条草稿

pending Steer 的预览不是 Core 阶段指示器。`submit_user_message_with_prepared_images` 提交应用操作后保存本地 pending 项；`turn/steer` 的成功响应不直接移除该项，用户消息记录回执才按 client ID 完成显示协调。因此 pending 期间可能尚在客户端派发，也可能已由 Core 接收、等待记录或等待界面处理回执，不能仅凭预览判定是否进了 Core。

有 pending Steer 时按 Esc 的正常串行路径是：Interrupt T1 → 收到 T1 中断终态并清除活动 ID → 重新提交剩余输入 → 无活动 Turn 时走 `turn_start`，形成 T2。同一 Thread 和历史可以继续使用，但 T1 已经结束，不是向已中断的 T1 再 Steer。重新提交复用通用 UserTurn 路由；若另一个客户端抢先启动了新活动 Turn，仍需按当时状态和既有竞态策略处理，不能将正常路径扩大成所有并发下都必定 start。

普通 Queue 恢复时，多条消息按队列顺序以换行拼成一个输入框草稿，原有未发送草稿放在后面，已恢复的队列被清空，并不自动发送。完整恢复收集顺序是 rejected Steer → pending Steer → 普通 Queue → 原输入框草稿，各组保持自身顺序。图片、文本范围和粘贴占位符也会合并或重映射。若用户直接提交合并后的草稿，它就是一条输入，不再自动保留原先多条 Queue 的逐轮调度边界。

已有测试 `interrupt_restores_queued_messages_into_composer` 与 `interrupt_prepends_queued_messages_before_existing_composer_text` 明确断言两条消息合并为 `first queued\nsecond queued`，有现存草稿时追加 `\ncurrent draft`；本次只静态阅读，未运行测试。

### Core 已接收时，Esc 如何与记录回执协调

`turn/steer` 成功只说明 Core 接受路由与入队。若中断时输入仍在旧 Turn 的待处理状态，中断路径会移除活动 Turn 并清理其 pending 状态；TUI 在中断事件到达后重新读取本地 `pending_steers`，对仍未确认的普通输入重新提交，通常启动新 Turn。Core 的清理调用在中断处理之后，不能把上述职责顺序误写成“清空 pending 必然先于发出终态事件”的严格事件顺序。

按 Esc 只设置中断后提交标志，不冻结当时的 pending 列表。若 Core 已记录输入，且相应回执在 TUI 处理中断事件前到达，回执会先移除本地 pending 项，该输入便不再重发。只有一条输入且已获回执时，这条恢复路径不会仅因为之前按了 Esc 就自动启动下一轮。

pending 项消失只证明已有记录回执，不能反推模型尚未看到，也不能反推已经完成。若确实处于“已记录但尚未采样”的窗口，它与正常收尾仅保存输入的情况类似：以后新 Turn 将相关历史带入请求，模型才有机会看到。历史中的输入不是一项保证下一轮执行的待办；模型也可能已处理部分工作后才被中断。

### 断线恢复：没发出与结果不确定必须分开

这里讨论 TUI 与可重连 App Server 之间的传输恢复，不是模型供应商连接重试，也不代表整个客户端退出后本地草稿必然持久保存。

先区分部署与职责：界面 → App Server/Core → 模型服务是两段不同连接。当前 TUI 的 `AppServerTarget` 有 Embedded（同进程内嵌）、LocalDaemon（本机独立服务）和 Remote（远程服务）；此处重连路径排除 Embedded。独立服务可以在本机，Server 名称不等于云端。远程连接断开时，执行端可能仍在工作；本机互联网断开则可能只影响到模型服务的连接。桌面 App 的本地子进程形态见 [OpenAI 架构说明](https://openai.com/index/unlocking-the-codex-harness/)，不能将 TUI 恢复实现直接推定为桌面 GUI 的实现。

这种部署形态并非 Codex 独有：[OpenCode 官方 Server 文档](https://docs.opencode.ai/docs/server/)也说明 TUI 作为客户端连接服务端，并支持独立 Server。本次只核对架构形态，未核对其恢复策略。按用户选定的实践范围，本节是扩展知识，不要求在 M0.5 实现独立服务或传输恢复。

设 T1 正在执行，客户端发送补充输入 S 后连接断开。客户端无法仅凭网络错误判断 S 根本没送达、已入队，还是已经记录甚至参与执行。自动重发可能重复输入，直接删除又可能丢失用户意图。因此当前实现保留本地输入，暂停自动发送，恢复连接与线程状态后再核对提交身份。

`MessageDelivery` 将恢复输入区分为 `Unsent`（未发送）与 `Unconfirmed(Option<String>)`（提交结果未确认，可带 client ID）。重连恢复时，原 pending Steer 被转入恢复队列并标记为 `Unconfirmed(Some(client_id))`；不能因为恢复到队列就当成普通未发送输入。

`restore_reconnected_input` 在重放之后恢复本地输入，再以确认 ID 列表协调。`reconcile_recovered_messages` 只移除 ID 精确匹配的未确认项。没有匹配 ID，包括缺失或旧版历史不能提供身份的情况，不是“确定没发送”的证据，因此不会自动重发。界面提示无法确认该输入是否发送，且尚未重发。

普通队列发送入口 `maybe_send_next_queued_input` 检查是否存在任何未确认输入：存在时阻止自动发送。因此，若 S 后面还排着确定未发送的 Q，S 未核实时 Q 也不会自动越过它。S 被精确回执确认后，其他阻止条件均解除且线程允许发送时，普通队列才可继续。后续实时用户消息回执也能协调未确认项，并尝试推进队列。

这与 Esc 不同：正常中断路径有终态事件和旧 Turn 的清理机制作为依据；传输断线没有证明服务端已经停止，更没有证明它丢弃了输入。重连成功解决了通信恢复，不能直接解决提交结果不确定。

证据为静态阅读 `chatwidget/reconnect.rs`、`input_restore.rs`、`input_flow.rs`、`user_messages.rs` 及 `tests/composer_submission.rs` 中恢复场景：空确认列表、无关 ID 不触发提交；匹配 ID 后允许后续输入继续。未运行故障测试，也不由客户端 ID 推导服务端对所有重试提供 exactly-once 执行保证。

## 源码阅读入口

1. [`tui/src/app/user_turn_submission.rs`](../../../codex-rs/tui/src/app/user_turn_submission.rs)：TUI 的 Steer、ID 竞态重试与转 `turn_start` 主路径。
2. [`core/src/session/turn_input.rs`](../../../codex-rs/core/src/session/turn_input.rs)：看核心如何处理 `StartOrSteer`、`StartIfIdle` 和严格 `Steer`。
3. [`tui/src/chatwidget/input_restore.rs`](../../../codex-rs/tui/src/chatwidget/input_restore.rs)：看未成功送入当前 Turn 的文字如何保留。
4. [`core/src/session/mod.rs`](../../../codex-rs/core/src/session/mod.rs)：`SessionIo::submit_turn_input` 的返回承诺。
5. [`core/src/session/turn.rs`](../../../codex-rs/core/src/session/turn.rs)：`run_turn` 如何在后续请求前消费输入。
6. [`tui/src/chatwidget/input_flow.rs`](../../../codex-rs/tui/src/chatwidget/input_flow.rs)：Queue 入队与后续发送条件。
7. [`app-server/src/request_processors/turn_processor.rs`](../../../codex-rs/app-server/src/request_processors/turn_processor.rs)：`turn_steer_inner` 的拒绝映射。
8. [`core/src/hook_runtime.rs`](../../../codex-rs/core/src/hook_runtime.rs)：`record_pending_input` 将用户输入交给历史记录路径。
9. [`core/src/tasks/regular.rs`](../../../codex-rs/core/src/tasks/regular.rs) 与 [`tasks/mod.rs`](../../../codex-rs/core/src/tasks/mod.rs)：外层待输入复查与 `on_task_finished` 收尾。
10. [`tui/src/chatwidget.rs`](../../../codex-rs/tui/src/chatwidget.rs)：`on_committed_user_message` 的客户端 ID 匹配与显示协调。
11. [`tui/src/chatwidget/interaction.rs`](../../../codex-rs/tui/src/chatwidget/interaction.rs)：有 pending Steer 时的 Esc 中断后提交入口。
12. [`tui/src/chatwidget/reconnect.rs`](../../../codex-rs/tui/src/chatwidget/reconnect.rs)：断线暂停、恢复输入与按提交 ID 协调未确认消息。

## 回忆练习

模型正在调用工具时，你补充“先别改文件，只做静态分析”。Thread 和 Turn 一定会怎样变化？

答案：Thread 不变；界面先尝试 Steer 当前活动 Turn，成功时 Turn 也不变。当前 Turn 已结束或不能接收时，才进入暂存、恢复或后续新 Turn 路径。

参考：[核心概念速查](../reference/glossary.md)
