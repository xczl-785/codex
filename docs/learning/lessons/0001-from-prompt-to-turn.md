# 第一课：从一句话到一次 Turn

目标：先建立整体骨架，不追函数细节。

## 把 Codex 想成一间工作室

- TUI 是前台，收集你的文字、当前目录、模型和权限设置。
- App Server 是统一接单口，把客户端请求翻译成核心操作。
- Thread 是长期保留的工作台，可以持续、恢复和派生。
- Turn 是工作台上一段连续的 Agent 执行过程。
- 核心运行时负责模型交流、工具调用、审批和状态保存。
- Event 把输出和状态变化送回界面。

简化链路：

```text
用户输入 -> TUI -> App Server -> Thread / Turn -> 模型与工具
                                             -> Event -> TUI
```

## Thread 和 Turn 为什么分开

两者生命周期不同：Thread 可以包含很多轮并长期存在；Turn 到完成、中断或失败就结束。

Turn 不严格等于“一条用户消息加一条助手回复”。空闲时的新输入会启动新 Turn；执行中追加的输入可能进入当前 Turn，调整它接下来的方向。

不要把 Thread 理解成操作系统线程。这里的 Thread 是产品领域里的工作会话。

## 第一次只认七个代码路标

1. [`cli/src/main.rs`](../../../codex-rs/cli/src/main.rs)：命令行总入口，默认进入 TUI。
2. [`tui/src/app_command.rs`](../../../codex-rs/tui/src/app_command.rs)：界面内部命令。
3. [`tui/src/app/thread_routing.rs`](../../../codex-rs/tui/src/app/thread_routing.rs)：把界面输入路由成 Thread/Turn 请求。
4. [`app-server/.../turn_processor.rs`](../../../codex-rs/app-server/src/request_processors/turn_processor.rs)：处理 Turn API 请求。
5. [`core/src/thread_manager.rs`](../../../codex-rs/core/src/thread_manager.rs)：创建、恢复、派生和关闭 Thread。
6. [`core/src/codex_thread.rs`](../../../codex-rs/core/src/codex_thread.rs)：向 Thread 投递操作并接收事件的句柄。
7. [`core/src/session/handlers.rs`](../../../codex-rs/core/src/session/handlers.rs)：核心消费操作、推进 Turn 的入口之一。

## 回忆练习

如果用户连续输入三次，会有几个 Thread、几个 Turn？

答案：通常仍是一个 Thread。Turn 数量取决于发送时机；等每轮结束再发通常是三个 Turn，当前轮执行中追加且 Steer 成功则可能仍是一个 Turn。

下一课：[连续发送时，新 Turn、Steer，还是等待？](0002-start-steer-or-queue.md)

