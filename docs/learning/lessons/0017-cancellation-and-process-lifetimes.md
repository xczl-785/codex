# 第十七课：停止 Turn，究竟停止了什么

本课连接上下文恢复与后续并行调度。核心问题是：用户停止当前执行时，Task、工具调用和真实进程为何可能具有不同结局？取消不是事务回滚，也不是把所有相关对象瞬间变成同一状态。

## 取消请求与实际停止

`Session::interrupt_task` 进入 `abort_all_tasks`，取出活动 Turn 的 RunningTask，再由 `handle_task_abort` 处理。后者取消 CancellationToken，给执行流程主动退出的机会，等待完成通知或当前配置为 100 毫秒的宽限时间，再调用任务句柄的 abort 和具体任务的清理入口，处理历史标记、hooks 与 TurnAborted 事件。

100 毫秒只是其中的主动退出等待阶段，不是全部清理或外部进程退出的时间保证。取消信号需要相关代码观察；通用异步任务 abort 也不是任意代码的瞬时强杀，更不天然等于终止操作系统进程。

## 两个 abort 的对象不同

`RunningTask` 同时持有 `task`（任务实现对象）、`handle`（本次异步执行句柄）和取消令牌。`handle.abort()` 请求取消运行时里的执行；`SessionTask::abort()` 是由管理者另行调用的可选业务收尾入口。具体任务可在后者处理额外资源，但默认实现为空，当前 RegularTask 没有覆盖它。不能以为第二个 abort 必然负责杀掉所有外部进程。

如果一条脚本已生成 A.txt、准备生成 B.txt，收到取消信号不证明 A 被删除，也不证明 B 不会生成。前者取决于明确的回滚或清理，后者取决于执行端是否实际停止及停止时机。

## 必须按执行模式区分进程生命周期

早先教学仅追踪可续接路径，曾把“放进 Session 进程管理器”说成不会随取消终止的充分条件，这是不完整的。准确依据是 `ExecCommandLifetime` 的模式及对应取消路径。

| 模式 | 契约 | 取消边界 |
| --- | --- | --- |
| Interactive | 可以返回进程标识，后续读取输出或发送输入 | 已接管的进程不只因当前 Turn 中断而自动终止 |
| OneShot | 等待执行完成，不向调用方提供可续接进程 | 取消或超时时主动终止对应进程 |

活跃进程进入管理器的判断是尚未退出且没有退出码，并非智能识别“服务器”或预测寿命。持有进程不代表保证其长期存活；它仍可自然完成或被其他机制终止。服务器、监听程序与长命令是生命周期分离的用途说明，不是逐条语义分类规则。

Interactive 路径在初次等待之前存储活跃进程引用，明确避免 Turn 中断释放最后一个引用而终止后台进程。因而持续写文件的进程可能在 Turn 停止后继续产生新副作用；这不是单纯响应延迟。停止 Turn、停止工具等待、停止真实进程必须分别表述。

OneShot 同样复用进程管理器，但 `oneshot.rs` 保存该次启动的精确进程句柄，取消分支执行 `terminate_confirmed()`；cancel-on-drop 将外层等待取消与清理信号连接起来。标准 shell 注册路径启用 UnifiedExec 时注册 Interactive 与 write_stdin，否则注册 OneShot；不能据此推断所有客户端、扩展路径或当前对话工具必然采用哪种模式。

指定进程终止、清理后台终端和正常关闭 Session 有明确进程清理入口。这些入口的存在不代表普通 Turn 中断会自动调用它们，也不保证任意远端业务副作用被撤销。

## 完成与取消的竞态

工具等待层同时等待派发结果和取消信号。取消到达后，如果 `terminal_outcome_reached` 已设置或派发已完成，就继续取回结果；否则尝试取消派发，若仍取得结果则保留，确认被取消后才构造中断响应。

该标记在终态通知前设置，避免工具已经完成却因通知慢而被覆盖成中断。测试 `cancellation_after_handler_finishes_preserves_completed_lifecycle` 阻塞完成通知、发送取消，再放行通知，断言成功结果和成功生命周期仍被保留。本轮静态阅读测试，没有执行测试。

标记不是外部副作用发生的精确时刻，也不是持久化或客户端送达确认。文件可能已写入但流程尚未设置标记；或标记已设置但程序崩溃导致结果未保存。不能由标记推导超出其范围的保证。

用户已正确判断：配置修改完成、验证尚未执行、整个 Turn 被中断可以同时成立。整体中断不能抹去已完成的局部事实，也不能将未执行验证表述成验证失败或通过。

## 下一衔接

用户已正确回答：新 Task 调用 write_stdin 是一次新的 Function Call，访问同一个进程 P。术语需区分运行时持有与持久化：进程管理器保留仍存活进程的句柄，通过标识定位；保存标识或历史不等于持久化进程执行现场，更不能据此在重启后复活进程。

取消已完成、Session 空闲且程序未退出时，用户再次输入“继续”会进入新 Turn/新 RegularTask 的启动路径；它使用保留历史与当前资源，不是恢复旧 Task 的暂停调用栈。若输入到达时仍有活动 Turn，需另按 start/steer 规则判断。下一轮先核验这组生命周期区别，再进入并行工具调度。

## 源码入口

- [任务取消与清理](../../../codex-rs/core/src/tasks/mod.rs)：`handle_task_abort`、`SessionTask::abort`。
- [运行对象](../../../codex-rs/core/src/state/turn.rs)：`RunningTask`。
- [工具取消与竞态测试](../../../codex-rs/core/src/tools/parallel.rs)：`handle_tool_call_with_source` 与完成竞态测试。
- [终态通知](../../../codex-rs/core/src/tools/registry.rs)：`notify_tool_finish_if_unclaimed`。
- [一次性执行](../../../codex-rs/core/src/unified_exec/oneshot.rs)：`exec_command_to_completion`。
- [进程管理](../../../codex-rs/core/src/unified_exec/process_manager.rs)：保存活跃进程及显式终止入口。
- [工具注册](../../../codex-rs/core/src/tools/spec_plan.rs)：`add_shell_tools`。
- [新输入处理](../../../codex-rs/core/src/session/turn_input.rs)：`start_or_steer`。
