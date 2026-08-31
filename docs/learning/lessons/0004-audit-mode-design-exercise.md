# 第四课：从安全审计需求定位架构扩展点

这一课不讨论真实产品是否需要 Audit，而是用一个假设能力练习：怎样从需求、边界和交互反推 Thread、Session、Turn、Task 以及周边协议的变化。

## 第一步：先补全需求

不能只凭“新增安全审计模式”就决定改什么。假设需求为：

- 用户在已有 Thread 中通过 `/audit` 发起一次专项工作；
- 默认在当前对话中显示结果，不直接修改仓库；
- 可以读取文件、搜索代码和执行受控的只读诊断；
- 使用独立审计提示、工具策略、扫描算法和报告格式；
- 审计可以完成、中断或失败；运行时不接受 Steer，新输入先排队；
- 报告进入 Thread 历史，用户随后可以继续追问或另开普通 Turn 请求修复。

架构设计前先确认需求、边界和交互，是这道题最重要的步骤。没有这些信息，直接回答“新增 `AuditTask`”只是猜测。

## 主要扩展点

核心执行形态可以设计为：

```text
TaskKind::Audit
AuditTask implements SessionTask
```

- **Thread**：仍表示原来的长期会话，不需要新概念。
- **Session**：仍负责当前 Thread 的内存运行，不改变基本职责。
- **Turn**：复用开始、完成、中断、事件和历史边界，但需要能够标识审计执行。
- **Task**：主要扩展点，`AuditTask` 实现专门的扫描、归类和报告算法。

“抽象不变”不代表代码完全不动。Task 枚举、启动路由、状态映射和 UI 展示仍可能需要认识 Audit。

## 报告输出不等于工作区写权限

审计报告可以走正常结果链路：

```text
AuditTask
-> AgentMessage / Turn Event
-> App Server / TUI
-> Thread rollout 或 store
```

这是 Codex 自身保存对话与运行记录。它与模型调用工具修改用户仓库不是同一条通道。

因此可以同时满足：

- Codex 内部允许保存审计报告；
- AuditTask 对用户工作区只有读取权限；
- 如果用户需要导出 `audit-report.md`，再通过明确的导出或确认动作完成。

## 只读必须由能力边界强制保证

不能只在 Prompt 中写“不要修改”。AuditTask 至少需要限制：

- 工作区文件系统只读；
- Patch 和编辑工具不注册或直接拒绝；
- Shell 只允许受控诊断，避免看似读取的命令产生副作用；
- MCP 只暴露只读工具；
- 网络默认禁止，除非审计目标明确需要漏洞数据库；
- 执行中不允许自行申请提升为可写权限。

Prompt 表达审计意图，Permission Profile、工具注册和执行策略提供强制边界。

## 为什么正式能力需要协议变化

快速原型可以把 `/audit` 转换成普通自然语言提示，但服务端无法稳定识别 Audit，也难以保证权限、范围、状态和机器可读结果。

正式能力可以参考现有 `review/start`，增加类似请求：

```text
audit/start

AuditStartParams {
  threadId,
  target,
  profile,
  delivery
}
```

协议变化的目的不是“给报告找一个写入位置”，而是明确表达：

- 这是哪一种操作；
- 审计什么目标；
- 采用什么检查策略；
- 在哪个 Thread 运行；
- 客户端如何观察和处理结果。

如果 UI 只需要通用的开始、流式文本、完成和中断，现有 Turn 事件可能足够。如果还要显示扫描进度、分类数量或结构化发现，则需要 Audit 专用事件与结果类型。

## AuditTarget：范围必须是显式输入

现有 Review 已经通过 `ReviewTarget` 区分：

- 未提交改动；
- 相对基础分支的变更；
- 某个 commit；
- 自定义指令。

安全 Audit 的范围可能更宽，可以设计为：

```text
AuditTarget =
  UncommittedChanges
  | BaseBranch { branch }
  | Commit { sha }
  | Paths { include, exclude }
  | WholeRepository
  | Custom { instructions }
```

还应明确审计类型，例如 Secrets、DependencyRisk、Injection、PermissionBoundary、Full 或 Custom。范围决定“检查哪里”，Profile 决定“检查什么问题”。

## 时间一致性

审计过程中仓库可能发生变化。若前半段读取旧内容、后半段读取新内容，报告会失去一致性。

请求或结果最好记录：

- 启动时的 `HEAD`；
- 工作区状态或 diff 指纹；
- 审计根目录和排除规则；
- 审计完成时目标是否发生变化。

已提交代码可以固定到 commit；工作区审计至少应检测运行期间是否变化，并在报告中给出警告。

## Thread 历史不是审计事实来源

需要区分三类信息：

1. **会话上下文**：用户为什么审计、关注什么；
2. **审计目标**：本次检查的文件、diff 或 commit；
3. **当前事实**：启动时实际存在的代码和配置。

历史消息可能已经过期，只能帮助理解意图，不能替代读取当前源码。稳妥规则是：

> Thread 历史提供背景，`AuditTarget` 决定范围，当前源码或固定快照提供事实。

对于很长或主题混杂的 Thread，可以支持：

- `inline`：在当前 Thread 运行，继承必要上下文；
- `detached`：创建隔离 Audit Thread，只携带审计请求和目标快照，完成后向原 Thread 返回摘要。

现有 Review 的 App Server 协议已经提供 inline/detached 交付选择，可作为设计参考。

## 最终变化清单

| 区域 | 主要变化 |
| --- | --- |
| Thread | 复用长期身份与历史，不新增概念 |
| Session | 复用运行与资源管理，不改变基本职责 |
| Turn | 复用生命周期，补充 Audit 类型识别 |
| Task | 新增 `AuditTask`、`TaskKind::Audit` 和执行算法 |
| 权限 | 强制只读工具、文件、命令、MCP 与网络策略 |
| 协议 | 增加类型化启动请求、Target、Profile 和可选专用事件 |
| UI | 增加入口、范围确认、状态和结构化结果展示 |
| 存储 | 复用 Thread 历史；可选报告导出必须独立授权 |

## 本题得到的架构方法

定位扩展点时，不要只问“新增哪个类”，还要依次确认：

1. 用户真正发起的是什么操作；
2. 操作属于哪个长期身份和执行边界；
3. 数据范围与事实来源是什么；
4. 哪些能力必须被权限强制限制；
5. 客户端需要哪些稳定协议与交互反馈；
6. 运行期间数据是否可能变化。

这套方法比记住 `AuditTask` 这个答案更重要。

## 当前源码参考

- [`app-server-protocol/src/protocol/v2/review.rs`](../../../codex-rs/app-server-protocol/src/protocol/v2/review.rs)：`review/start`、交付方式和 Review Target。
- [`prompts/src/review_request.rs`](../../../codex-rs/prompts/src/review_request.rs)：Review Target 如何转换为具体检查提示。
- [`core/src/tasks/mod.rs`](../../../codex-rs/core/src/tasks/mod.rs)：`SessionTask` 与 Task 生命周期。
