# 核心协议阅读入口

原有说明已移至[历史协议设计资料](history/protocol-v1-design.md)，保留早期术语、交互图示和后续局部修订的追溯价值。它不作为当前实现规范，也不等同于 App Server v1 的兼容性契约。

按当前任务选择事实来源：

- [核心协议类型](../protocol/src/protocol.rs)：当前 `Op`、`EventMsg` 与相关类型。
- [Thread、Session、Turn 与 Task](../../docs/learning/lessons/0003-thread-session-turn-task.md)：基于当前学习分支源码的职责与生命周期解释。
- [App Server 接口](../app-server/README.md)：面向客户端的 API、通知和兼容性说明。
- [工程文档目录](README.md)：构建、MCP 和历史资料的阅读边界。
