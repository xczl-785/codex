# Learning Records

本目录记录学习者在不同阶段已经建立的理解、仍存在的疑问，以及后续讲解需要继承的教学上下文。

这些记录是教学过程资产，不是 Codex 架构事实来源。判断实现行为时，应回到当前分支源码、测试和 `lessons/` 中已经核对过的解释；记录中的旧理解可能被后续学习修正。

## 生命周期约定

| 项目 | 说明 |
| --- | --- |
| 角色 | 保存某个阶段的理解基线和教学衔接信息 |
| 状态 | 当前为 `working`，随课程推进持续增加 |
| 事实来源 | 当前分支源码、测试以及对应课程文档 |
| 下游使用者 | 后续教学对话和课程规划 |
| 退出条件 | 学习任务结束，或这些信息被吸收到稳定课程与速查资料中 |
| 退出后处置 | 保留为学习过程证据，不进入普通课程阅读路线 |

## 当前记录

- [0001：已有概念基础与学习方式](0001-prior-knowledge-and-learning-style.md)
- [0002：Thread、Turn 与 App Server 理解基线](0002-thread-turn-and-app-server-baseline.md)
- [0003：从生命周期定义转向设计动机](0003-from-lifecycle-to-design-motivation.md)
- [0004：先确定需求边界，再判断扩展点](0004-requirements-before-extension-points.md)
- [0005：运行态是持久事实的可重建投影](0005-runtime-state-as-projection.md)
- [0006：原生 Tool Calling 已成为理解基线](0006-native-tool-calling-baseline.md)
- [0007：Step 工具快照与调用因果链](0007-step-tool-snapshots-and-causal-history.md)
- [0008：Checkpoint 边界与设计偏好](0008-checkpoint-boundaries-and-design-preference.md)
- [0009：从阶段复盘转入受控实践](0009-stage-review-to-practice.md)
- [0010：从 Harness 实践收尾转入权限与上下文专题](0010-harness-close-to-security-and-context.md)
- [0011：从资源范围与命令规则重新建立权限基础](0011-permission-foundations.md)
- [0012：压缩、恢复与历史可见性](0012-compaction-recovery-and-history-visibility.md)
- [0013：取消与进程生命周期边界](0013-cancellation-and-process-lifetime-boundaries.md)
- [0014：并行准入、结果顺序与职责分离](0014-parallel-admission-order-and-responsibility.md)

新理解推翻旧记录时，不静默重写学习过程；应在新记录中说明它修正或取代了哪项认识，并把稳定结论沉淀到对应课程。

## 更新门槛

学习记录按“理解发生了什么变化”更新，不按对话轮次或课程数量机械增加。出现以下情况时，应在转入新主题前检查是否需要新增记录：

- 用户用场景、复述或实践证明了非平凡理解；
- 一个会影响后续教学的误解已经被纠正；
- 用户已有知识使后续课程可以提高起点；
- 学习目标发生变化。

`NOTES.md` 可以暂存教学过程，`MAINTENANCE.md` 负责当前进度；二者都不能长期代替这里的理解基线。
