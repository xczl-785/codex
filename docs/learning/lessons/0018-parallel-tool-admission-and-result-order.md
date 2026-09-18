# 第十八课：工具并行资格与结果顺序

本课研究同一次模型响应包含多个工具调用时，Codex 怎样决定它们能否重叠执行，以及执行完成顺序为什么不必等于结果进入对话历史的顺序。并行声明只描述处理器能否进入共享执行通道，不负责判断具体调用之间的业务依赖。

## 并行资格挂在工具处理器上

`ToolExecutor::supports_parallel_tool_calls()` 默认返回 `false`。工具没有显式覆盖该方法时，会采用保守的串行行为。`ToolCallRuntime` 为同一步中的调用共享一个 `RwLock<()>`：声明支持并行的调用取得共享读锁，没有声明或返回 `false` 的调用取得独占写锁。

因此，并行关系不要求调用来自同一种工具。两个不同工具只要都声明支持并行，就可以同时持有共享锁；任意一方走独占通道时，它就不能和同一执行门内的其他调用重叠。

该门禁不会分析文件路径、Shell 脚本副作用或业务因果关系。模型仍需把依赖前一结果的操作留到下一次采样。例如 B 的参数依赖 A 的输出时，模型必须先发 A，收到结果后再生成 B。

## 当前实现呈现出的工具类型

明确支持并行的内置工具包括 `exec_command`、`write_stdin`、`view_image`、`tool_search`，以及 MCP 资源的列举和读取工具。Web Search 扩展也明确支持并行；History/Notes 扩展除追加和写文件外，查询类操作均支持并行。

普通 MCP 工具按服务端能力动态决定：MCP Server 明确支持并行，或者工具具有 `readOnlyHint=true` 时，处理器返回 `true`；否则返回 `false`。

`apply_patch`、Goal、Memory、Skills、用户交互、权限请求、计划、等待、Code Mode 和 Multi-agent 等大量处理器没有覆盖该方法，因而继承默认的 `false`。`list_available_plugins_to_install` 和 `request_plugin_install` 明确返回 `false`。只读工具也可能保持默认串行，这说明只读是并行的有利证据，但不是自动规则。

`exec_command` 是“有副作用不等于不能并行”的直接反例。它声明的是处理器可以同时承载多个调用，并不承诺任意两条命令没有外部资源冲突。高级领域工具可以选择在处理器层统一维持顺序；低层通用执行工具则把具体调用的依赖判断留给模型或工作流。

## 设计新工具时的判断顺序

先检查处理器和底层依赖能否安全承受两个同时进入的调用，包括共享字段、客户端、流、临时资源、输出归属和取消传播。再判断工具是否天然承担顺序契约，例如补丁基线、目标状态、生命周期或用户交互。最后评估并行带来的等待时间收益。

当前布尔声明不能表达“不同路径可以并行、相同路径必须串行”。需要这种粒度时，应在工具内部按资源键加锁、使用版本或哈希做乐观并发控制，或者由上层工作流明确排序。无法确认并发契约时，保持默认 `false`。

## 下一步：完成、事件与历史顺序

工具 Future 按模型输出的调用顺序加入 `FuturesOrdered`。工具处理器可以并发运行，较后的调用也可能更早完成并发出完成事件；采样结束后，`drain_in_flight` 使用 `FuturesOrdered::next()` 取结果，因此结果按加入顺序记录到对话历史。下一次模型采样要等本轮在途工具被 drain。

这形成三条需要分别观察的时间线：真实副作用发生顺序、工具生命周期事件顺序、结果进入模型历史的顺序。后续用慢调用 A 与快调用 B 的场景核验这三者。

## 并行调用收到取消时

每个直接工具调用从当前采样的取消令牌派生 child token。Turn 取消后，已经到达终态的调用保留成功或失败结果；尚未完成的派发任务尝试停止，并在确认取消后形成 `aborted` 输出。采样循环仍先 drain 在途工具、按调用顺序记录可用结果，然后返回 `TurnAborted`，不会在原 Turn 内继续下一次模型采样。

取消不是外部副作用回滚。OneShot 命令会在取消路径终止对应进程，但此前的写入仍可能存在。Interactive `exec_command` 若已经返回 `session_id`，本次工具调用已经完成；进程管理器持有的后台进程属于另一条生命周期，可以在 Turn 中断后继续运行。不能把“工具调用完成”和“它创建的进程退出”合并成一个状态。

## 源码入口

- [工具执行契约](../../../codex-rs/tools/src/tool_executor.rs)：`supports_parallel_tool_calls` 的默认值。
- [并行执行门](../../../codex-rs/core/src/tools/parallel.rs)：共享/独占锁及取消处理。
- [工具结果汇集](../../../codex-rs/core/src/session/turn.rs)：`FuturesOrdered`、`push_back` 与 `drain_in_flight`。
- [工具生命周期通知](../../../codex-rs/core/src/tools/registry.rs)：工具完成通知发生在处理器结果返回之前。
- [MCP 并行声明](../../../codex-rs/core/src/tools/handlers/mcp.rs)：服务端 opt-in 与只读提示。
- [History/Notes 工具](../../../codex-rs/ext/history-notes/src/tools.rs)：查询并行、写入串行的示例。
