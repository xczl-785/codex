# 阶段复盘 0002：权限实验之后学到了什么

本复盘覆盖实践 0002 完成之后，到课程 0018 结束为止的学习。它用于判断下一步需要继续源码课程、建立新实验工程，还是回到 CodebaseAgent 做增量实践。Subagent 只完成了开场定位，尚未计入已掌握内容。

## 已完成的三个主题

### 上下文、压缩与恢复

实践 0003 复用了 CodebaseAgent 已实现的容量门禁、持久 Thread、显式压缩检查点和新 Session 恢复，再映射到 Codex 当前源码。学习者已经能够区分持久事实、Session 运行态 History、当前模型请求和按需检索能力；理解压缩以 replacement history 改变后续模型投影，而不是删除全部旧事实。

恢复时以最近有效压缩基线接上后续事实，新 Session 重建运行投影，不复活旧 Future、进程或文件句柄。缺失 FunctionCallOutput 可以在请求归一化时补合成 `aborted`，但这不证明外部调用没有执行；未知副作用必须先核验，不能盲目重试。自动压缩成功后仍在同一 Turn/Task 内继续，失败则结束当前执行路径，最终仍受 Provider 硬上限约束。

### 取消与进程生命周期

学习者已经区分 Turn/Task 取消、工具派发取消和真实进程终止。任务句柄 abort 负责异步执行，`SessionTask::abort()` 是可选业务清理入口，不保证终止全部外部资源。

Unified Exec 必须按 Interactive 与 OneShot 区分：OneShot 在取消路径主动终止对应进程；Interactive 调用返回 `session_id` 后工具调用已经完成，受管后台进程仍可继续。取消不是事务回滚，已经发生的文件或远端副作用不会自动撤销。终态标记保护已经完成的工具结果，不让稍晚到达的取消覆盖它。

### 并行工具调度

学习者已经理解 `supports_parallel_tool_calls` 是工具处理器的准入声明，不是资源冲突分析。不同工具只要都进入共享通道也可以并行；具体调用的业务依赖仍由模型或上层工作流排序。Codex 使用共享/独占门禁容纳并行与串行工具，默认未声明时保守串行。

多个调用可以按真实时间并发完成，生命周期事件也可交错；`FuturesOrdered` 仍按模型调用顺序把结果写入历史。下一次直接模型采样等待本批在途调用 drain。Turn 取消时，已完成结果保留，未完成调用形成 aborted 结果，历史仍按调用顺序整理，原 Turn 不再继续采样。

## 当前证据强度

| 主题 | 对话与源码场景 | 可运行实践 |
| --- | --- | --- |
| 权限、审批、沙箱 | 已完成 | PermissionSandboxLab 已关闭并保存 Fake 与受控真实观察证据 |
| 上下文、压缩、恢复 | 已完成 | CodebaseAgent M3 已有容量、checkpoint、崩溃与新 Session 恢复测试 |
| 取消传播 | 已完成关键源码和场景校准 | CodebaseAgent 尚无 Turn 到模型/工具的取消实现 |
| 并行工具 | 已完成关键源码和场景校准 | CodebaseAgent 当前 `max_parallel_tool_calls` 只接受 1，批次仍串行 |
| Subagent | 仅完成 spawn 的整体定位 | 尚未进入实践，也未完成源码课程 |

当前缺口不是继续增加取消与并行名词，而是把两者落实到一个已经理解的 Harness，观察共享取消令牌、部分完成、结果配对和历史顺序怎样共同工作。

## 下一实践建议

推荐重新打开 CodebaseAgent，建立一个有边界的“取消与并行”切片；不新建第三个 Rust 工程，也不把该能力加到 PermissionSandboxLab。

CodebaseAgent 已有单 Agent Turn/Step 循环、ScriptedModel、多个工具调用、Call/Output 历史、恢复和 `max_parallel_tool_calls` 配置壳。增量实现能够直接暴露新机制与旧主链的接缝。新建工程会重复这些基础设施，PermissionSandboxLab 则服务于执行准入和副作用恢复，不适合承担模型批次与历史调度。

建议分两步：

1. 先让 Turn 取消传到模型请求和可控测试工具，固定“已经完成的结果保留、未完成调用 aborted、取消后不再采样”的行为。
2. 再把只读工具批次从串行扩展为有界并发，用确定性闸门制造慢 A、快 B，验证真实完成顺序可为 B/A，而历史依靠 `call_id` 和既定提交策略保持稳定。

测试应使用 barrier/channel 等确定性同步，不依赖睡眠竞速；继续保持目标仓库只读、有界输出和非致命错误进入历史。Subagent 暂缓到这个切片完成并复核后，因为它会同时引入子 Thread、通信、等待、中断和资源回收。

## 进入门禁

CodebaseAgent 的 `STATUS.md` 当前为 `paused / awaiting-user-practice-topic`，要求学习者明确选择目标、场景和范围。因而本复盘只给出推荐，不修改 AgentLab 源码或状态。学习者确认后，再为该工程写入单一活动切片并按 WORKFLOW 完成设计、对抗审查、实现、测试和理解复核。
