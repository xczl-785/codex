# 实践任务 0003：上下文限制、压缩检查点与历史恢复复盘

## 接手契约

| 字段 | 当前值 |
| --- | --- |
| 角色 | 将已经完成的 Harness M3 实践重新映射到 Codex 生产实现的复盘任务 |
| 状态 | `queued / after-practice-0002` |
| 事实来源 | CodebaseAgent 的实际 M3 行为与测试、课程 0007/0009/0013、当前 Codex 源码与测试 |
| 下游使用者 | 接续上下文与恢复专题的教学 Agent 与学习者 |
| 进入条件 | 实践 0002 完成；这是教学聚焦顺序，不是技术依赖 |
| 退出条件 | 学习者能画出持久事实到新 Session 模型投影的重建链，并解释容量失败、压缩成功和不完整历史三类边界 |
| 退出后处置 | 把稳定结论吸收到阶段复盘或速查，记录仍需触发条件的问题，然后进入取消传播 |

当前是否已经进入本实践只看 [`MAINTENANCE.md`](../MAINTENANCE.md)。本任务不要求再实现一套完整持久化系统。

## 为什么现在学

这部分概念在课程 0007、0009 和 0013 中已经出现，CodebaseAgent 又实际做过：

- 模型请求容量准入；
- 持久 Thread 事实；
- 显式 compaction checkpoint；
- 新 Session 恢复；
- 未完成 Turn 和悬空 Call 的恢复分类。

现在最有价值的不是继续增加存储功能，而是把分散理解收敛成一个统一模型：

```text
持久事实
  → 恢复与投影规则
  → Session 运行态 History
  → 当前 Step 的模型请求
  → 容量判断
  → 正常采样或压缩/失败
```

## 深入边界

### 必须深入：语义和不变量

- 完整持久历史、Session 运行态 History、模型当前可见上下文、UI 历史不是同一个对象；
- 保存不等于模型可见，模型可见不等于模型可检索；
- 容量检查发生在什么请求边界，超限前后哪些事实已经提交；
- 压缩替换的是后续模型使用的有效历史，不等于删除旧 Thread 事实；
- checkpoint 怎样成为恢复时的新投影基线；
- 恢复创建新 Session，不复活旧 Future、文件句柄、进程或网络请求；
- 未完成 Turn、悬空 Call 和结果未知为什么不能自动重试；
- 自动压缩为何可以留在同一个 Turn/Task 内继续，手动压缩为何可以是独立 Task。

### 中等深度：源码与测试

- 沿 `run_turn` 的容量判断和自动压缩分支定位关键调用；
- 阅读 `replace_compacted_history` 如何同时更新运行态投影并追加持久检查点；
- 阅读 Thread/rollout 恢复怎样选取最新有效 baseline 并重放之后事实；
- 找到容量边界、连续压缩、恢复和不完整调用的代表性测试；
- 用事件序列解释一次真实恢复，不要求记忆所有协议字段。

### 按触发条件深入

- Provider tokenizer、真实窗口与输出预留：本地门禁和 Provider 行为不一致时；
- 摘要生成、摘要质量和多次压缩损失：真实长任务丢失关键约束时；
- history search、memory 或向量检索：模型确实需要按需找回 checkpoint 前原文时；
- 数据库索引、迁移、分布式一致性：单机追加日志已经成为真实瓶颈时；
- Fork/Revert 的完整存储语义：正式进入历史分支专题时。

这些内容不是“不重要”，而是目前深入不会提高对 Harness 主链路的解释能力。

## 四个复盘场景

### 场景一：容量边界

构造一个刚好容纳的 ModelRequest 和一个只多出少量内容的请求，确认：

- 精确边界是否允许；
- 超限是否发生在调用 Provider 之前；
- 已经完成的 Call/Output 是否仍保留；
- 失败是否被记录成 Turn 事实，而不是伪造最终回答。

### 场景二：同 Turn 自动压缩

固定：

```text
RegularTask 正在运行
  → 上下文达到阈值
  → 内联压缩成功
  → 替换模型有效历史
  → 同一个 Turn 进入下一 Step
```

确认没有创建一个需要事后恢复的第二个 RegularTask。

### 场景三：退出后恢复

固定一段包含压缩检查点的 Thread：

```text
旧事实
  → Compacted checkpoint / replacement history
  → checkpoint 后的新事实
  → 进程退出
  → 创建新 Session
  → 使用 replacement history + 后续事实重建模型投影
```

分别列出仍保存、模型直接可见、模型可检索和不能恢复的内容。

### 场景四：结果未知

在 FunctionCall 已持久化、FunctionCallOutput 尚未持久化时模拟崩溃。恢复后必须说明：

- 调用意图确实发生过；
- 外部副作用是否完成未知；
- 不能因为缺少 Output 就默认重试；
- 新 Session 只能基于持久事实继续判断。

## Codex 源码接入方式

开始时先用 CodeGraph 重新定位：

```powershell
codegraph explore "run_turn auto compact replace_compacted_history rollout recovery model visible history"
```

当前重点入口包括：

- `codex-rs/core/src/session/turn.rs`：采样循环、容量判断与内联自动压缩；
- `codex-rs/core/src/compact.rs`：压缩上下文和 replacement history；
- `Session::replace_compacted_history`：运行投影、CompactedItem 与持久化顺序；
- `codex-rs/thread-store/`、`codex-rs/history/`、`codex-rs/rollout/`：持久事实与恢复投影；
- App Server thread history：面向客户端的历史视图，不等同于模型 Prompt。

## 完成检查

学习者应能独立回答：

1. 为什么 Thread 保存了旧消息，模型仍可能看不到？
2. 为什么压缩不是删除历史？
3. checkpoint 在实时执行和恢复时分别起什么作用？
4. 自动压缩后为什么还是同一个 Turn？
5. 恢复时为什么必须创建新 Session？
6. 悬空 Call 为什么只能标记结果未知，不能自动补执行？
7. 什么时候才值得深入 tokenizer、摘要算法或历史检索？
