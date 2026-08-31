# Codex 核心概念速查

先记职责，不急着记实现。

| 概念 | 通俗解释 |
| --- | --- |
| Thread | 可以持续、恢复和派生的工作会话，像长期保存的工作台 |
| Turn | Thread 中一段连续的 Agent 执行过程，到完成、中断或失败为止 |
| Step | Turn 内一次模型采样所使用的稳定配置和环境快照；一次 Turn 可以经历多个 Step |
| Steer | 把新输入追加给仍在运行的 Turn，修正或补充当前方向 |
| Session | Thread 在内存中的运行实体，持有服务、状态和操作处理能力 |
| Task | 驱动一次 Turn 的执行程序，例如普通处理、压缩或代码评审；它不是聊天记录本身 |
| Op | 送进核心 Thread 的操作消息，例如输入、中断、审批或配置变化 |
| Event | 核心向外报告的事件，例如内容增量、工具调用、完成或错误 |
| App Server | 面向客户端的接口与协调层，把请求翻译成核心 Thread 操作 |
| ThreadManager | 管理多个 Thread，负责创建、恢复、派生、查找和关闭 |
| CodexThread | 调用方持有的 Thread 句柄，用来投递操作和订阅事件 |
| Tool | 模型可以请求使用的外部能力，例如执行命令或修改文件 |
| Context | 一次模型请求能看到的信息，包括历史、指令、环境和工具结果 |
| Inline auto-compaction | 在当前 RegularTask/Turn 调用栈中压缩历史，完成后直接继续下一 Step |
| Standalone compaction | 由独立 CompactTask 驱动的一次手动压缩 Turn，不负责恢复被替换的普通 Task |
| Subagent | 由另一个 Agent 创建的子工作者，复用独立 Thread 和正常 Turn 机制 |

注意：App Server 是系统访问边界，不是位于 Thread、Turn 之上的业务实体层。
