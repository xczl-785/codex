# 学习记录 0007：Step 工具快照与调用因果链

## 用户提出的关键问题

用户在理解 Function Call 运行闭环后，主动追问：

- 已定稿的 ToolRouter 在什么时候定稿；
- Step 之间工具能否变化；
- 是否每个 Step 都重新组装工具列表；
- 同时保存 Function Call 与 Output 为什么和恢复、并行、模型推理有关。

## 已建立的认识

- ToolRouter 是对当前 Step 定稿，不是对整个 Turn 或 Session 永久定稿。
- 每个 Step 都重新捕获 StepContext 并规划 ToolRouter，但可以复用底层 MCP Runtime、缓存、插件管理器和执行环境。
- Step 之间允许吸收模型设置、MCP Binding、环境 readiness、capability、插件和 Connector 的变化。
- Step 内必须保持模型看到的工具视图与实际执行 Router 一致，避免请求和执行使用不同版本。
- Function Call 与 Output 同时保存最直观的价值是完整回放调用、参数与结果。

## 本轮纠正

- Call/Output 持久化不负责实现并行；并行由 ToolCallRuntime、Future 和读写锁调度实现。
- Call/Output 通过 `call_id` 为并行结果提供与完成顺序无关的配对关系。
- 完整记录为恢复逻辑提供“只有调用、调用已有结果、孤立结果”等可判定状态，但不代表未完成工具一定会自动重跑。
- 模型不会因一次 Call/Output 永久学会工具。ToolSpec 负责说明工具能力，Call/Output 负责恢复当前任务的因果上下文。

## 下一步教学方向

进入工具实际执行的安全边界，区分：

- 审批回答“是否需要用户同意”；
- 权限回答“这次允许获得哪些能力”；
- 沙箱回答“操作系统实际上如何限制执行”；
- 执行策略回答“哪些命令可以自动允许、询问或阻止”。
