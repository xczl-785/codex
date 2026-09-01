# 学习记录 0009：从阶段复盘转入受控实践

## 阶段状态

用户已经完成课程 0001—0015 的整体复盘和综合场景校准。当前能够正确判断 Thread、Session、Turn、Task、Step、Tool Call 与真实进程的大部分生命周期关系，并已修正以下连接点：

- 自动压缩替换模型有效历史，不替换 Session；
- `write_stdin` 是新的 Tool Call，但继续使用原 `process_id`；
- Step 由模型请求快照划界，不由单个工具调用划界；
- 自动压缩后，模型—工具连续性可能优先于刚进入队列的 Steer。

## 下一阶段决定

不继续进行纯概念复习，也不立即修改 Codex 核心或进入完整 Subagent 实现。下一阶段采用“顶层垂直切片 + 最小 Agent 实践”：

1. 简短映射 CLI、TUI、Thread、Session、Turn、工具和历史边界；
2. 在独立工程实现只读源码学习 Agent；
3. 先使用 ScriptedModel 确定性验证 Harness；
4. 再接入真实模型与持久化；
5. 后续按取消传播、并行工具调度、Subagent 生命周期扩展。

## 实践偏好与教学约束

- 用户希望通过模仿和实践加速理解，但担心完全手写成本过高，也担心高层框架隐藏关键机制。
- 采用“关键控制面手写、基础设施复用、AI 受控协作”的边界。
- 允许 AI 生成骨架、机械代码、测试夹具和外部适配；生命周期、状态所有权、循环终止、失败语义、持久化边界和验收测试必须由用户理解并确认。
- 默认使用 Rust，以便把实践经验直接映射回 Codex；第一版不做 TUI、Shell、沙箱、并行和 Subagent。

详细任务见 [`practice/0001-source-study-agent.md`](../practice/0001-source-study-agent.md)。
