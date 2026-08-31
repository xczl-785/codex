# 学习记录 0005：运行态是持久事实的可重建投影

## 用户已经形成的关键认识

用户已经主动发现，Codex 在多个层级重复使用“持久化内容与运行态内容隔离”的设计：

- Thread 与 Session；
- Turn 与 Task；
- rollout 与模型可见历史；
- Thread 历史与界面显示状态。

用户还把这种结构与 Agent Harness 中的“状态外置”联系起来。这是一个值得保留的架构理解，而不只是对名词包含关系的记忆。

## 后续讲解应当延续的角度

- 把 Session、Task、ContextManager 和 UI 状态解释为针对不同职责的运行投影。
- 区分事实来源、检查点、内存物化状态和展示投影。
- 遇到恢复、压缩、Fork、Revert 等行为时，优先追问“哪一层的状态变了，哪一层仍保留”。
- 可以继续把 Codex 的具体实现与 Harness 的可恢复执行、执行器可替换、状态外置原则放在一起讨论。

## 当前仍需继续确认的问题

- Thread 与 Session 的身份在 Fork、恢复和子 Agent 场景下怎样传播。
- Thread Store、rollout 文件与 App Server 列表/历史投影之间的精确分工。
- workspace 的实际文件状态为什么不天然包含在对话回退中。

