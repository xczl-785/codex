# 学习记录 0008：Checkpoint 边界与设计偏好

## 用户提出的判断

在比较 Claude Code、Cursor、LangGraph 与 Codex 后，用户更认可 Claude Code 的 checkpoint 设计，认为它在固定范围内处理得干净、连续且具有明确边界。

## 判断依据

- Claude Code 只追踪受控文件编辑工具触及的文件，不虚构完整工作区事务。
- 对话恢复、文件恢复和二者同时恢复是可独立选择的操作。
- checkpoint 与 Session/Prompt 节点对应，可直接选择历史恢复点，而不是只提供一次性的文件撤销。
- Codex 当前选择不随 Thread 回退工作区，边界保守但缺少标准 checkpoint 体验。
- Cursor 有本地 Agent 文件 checkpoint，但公开契约不足以说明连续 Undo、分支和 Redo 的完整语义。
- LangGraph checkpoint 属于工作流运行状态持久化，不应与编码工作区 Undo 直接排名。

## 后续教学约束

- Checkpoint 作为 Codex 主线之外的横向扩展到此收束。
- 后续提到恢复机制时，继续区分对话、工作区、运行态和外部副作用。
- 评价设计时同时观察能力范围、所有权、连续历史、冲突行为和未覆盖副作用，不以“能否回退”一个布尔值判断。
- 课程主线返回工具执行闭环，下一步阅读 `exec_command` 的实际生命周期，再进入并行工具调度和 Subagent 生命周期。
