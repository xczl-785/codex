# 实践任务 0002：权限、审批、沙箱与提权实验

## 接手契约

| 字段 | 当前值 |
| --- | --- |
| 角色 | 实践 0001 收尾后的下一项受控实验契约 |
| 状态 | `working`（实验契约；实时进度只看 MAINTENANCE.md） |
| 事实来源 | 当前分支源码与测试、课程 0012；开始时必须用 CodeGraph 重新核对入口 |
| 下游使用者 | 接续权限专题的教学 Agent 与学习者 |
| 进入条件 | 独立 CodebaseAgent 完成文档收尾，且学习者准备开始下一主题 |
| 退出条件 | 学习者能独立解释完整准入链，并通过确定性场景区分拒绝、审批、沙箱失败、命令失败与提权重试 |
| 退出后处置 | 将稳定结论写入课程或速查，将理解变化写入 learning record，再进入实践 0003 |

当前是否已经进入本实践只看 [`MAINTENANCE.md`](../MAINTENANCE.md)，不要修改本表来记录日常进度。

## 要解决的问题

用户已经大致知道权限和沙箱，但在完整复述时容易漏掉中间层。实验目标不是记住更多类型名，而是形成一条可用于诊断真实失败的判断链：

```text
模型提出动作
  → 参数与命令策略判断
  → 是否允许发起审批
  → 用户或审查者决定
  → 选择受限执行方式
  → 创建真实执行尝试
  → 成功、策略拒绝、沙箱拒绝、命令失败、超时或取消
  → 记录结果并决定是否允许新尝试
```

必须能分别回答：谁决定、何时还没有进程、何时副作用已经可能发生、失败能否安全重试。

## 为什么放在取消、并行和 Subagent 之前

- 权限决定动作能否开始；取消决定已开始的动作怎样停止。
- 并行会同时产生多个执行尝试，必须先知道每个尝试怎样获准和记录。
- Subagent 还会引入权限继承、显式收窄和责任归属；如果单 Agent 权限链尚不稳定，直接扩展只会放大混淆。

## 深入边界

### 必须深入

- `PermissionProfile`、命令 Policy、`ApprovalPolicy`、具体 Approval 和 Sandbox 的职责差异；
- `Allow` 表示通过命令规则这一关，不能单凭它推断最终沙箱模式或无限权限；
- 沙箱为什么是受限的真实执行，不是副本、预演或事务；
- 审批前、进程创建后、部分副作用发生后的不同失败含义；
- 提权为什么是新的执行尝试，不能把旧进程理解为原地升级；
- 哪些失败可以修参数，哪些需要审批，哪些不能重试；
- 非幂等动作为什么不能在不确认副作用的情况下整条重跑；
- Policy、Approval、Attempt、Process 和 Tool Call 的身份及历史关系。

### 中等深度

- 用 CodeGraph 定位 ExecPolicy、requirements rules、工具 sandboxing、Unified Exec 错误和各平台 sandbox adapter；
- 读一条正常路径、一条审批路径、一条沙箱拒绝路径及对应测试；
- 理解网络权限与文件权限为何可能经过不同执行设施，但共享准入原则。

### 暂不深入

- 自己实现 Windows restricted token、AppContainer、Seatbelt、Landlock 或其他安全边界；
- 企业托管策略的全部配置矩阵；
- 完整 shell 语法解析器和危险命令知识库；
- 生产级审计平台、远程执行或多租户权限系统。

安全级沙箱必须依赖经过验证的操作系统能力。手搓一个看似隔离的进程包装器容易制造虚假安全感，学习价值反而较低。

## 实验一：确定性准入状态机

先不运行真实 Shell，定义一个虚拟动作和 Fake Executor：

```text
RunCommandIntent
  ├─ program
  ├─ arguments
  ├─ working_directory
  └─ requested_permissions

PolicyDecision
  ├─ Allow
  ├─ RequireApproval
  └─ Deny
```

执行历史至少区分：

```text
ToolCallId：模型提出的逻辑工具调用
ApprovalId：一次需要外部决定的审批
AttemptId：一次具体执行尝试
ProcessId：真实阶段在进程创建成功后才存在；Fake 阶段只能生成明确标记的模拟身份
```

这些名称是实验模型，不要求与 Codex 的公开字段逐字一致。

## 落地方式与第一版范围

建议建立独立 Rust 工程 `PermissionSandboxLab`，可选目录为 `D:\Program\PermissionSandboxLab`，尚未创建。CodebaseAgent 提供 Harness 经验，不直接增加 Shell 和沙箱功能；Codex 仓库保存课程与实验契约。实验不依赖特定目标代码库，也不要求真实模型、API Key、网络、TUI 或 Agent 框架。

第一版采用固定请求 + ScriptedReviewer + FakeExecutor。FakeExecutor 在内存文件表中模拟读写，每次操作时检查有效资源权限；不执行任意 Shell，也不宣称形成操作系统安全边界。实验的命令规则匹配结构化 program/arguments，不实现完整 Shell 解析。

建议目录（职责可合并，不以文件数量验收）：

```text
AGENTS.md                 目标、边界、必读入口、AI 协作约束
README.md                 启动方式和一个演示场景
docs/STATUS.md            当前阶段、证据、未决问题、下一动作
docs/DESIGN.md            类型、职责、授权范围和不变量
docs/EXPERIMENTS.md       场景、预期事件、实际证据与解释
src/policy.rs             命令规则与资源 Profile（独立类型）
src/approval.rs           审批制度、请求、决定、授权复用范围
src/executor.rs           FakeExecutor、内存文件表、模拟故障
src/orchestrator.rs       准入、审批、有效权限、执行与结果
src/events.rs             有序事件及关联 ID
tests/scenarios.rs        场景验收
```

独立工程 AGENTS.md 应告知：首先读 README 与 STATUS，按当前任务再读 DESIGN/EXPERIMENTS；Fake 不是安全沙箱；拒绝不能通过换包装绕过；不接真实模型或扩大真实执行范围；实现事实和下一动作写入 STATUS；改变行为契约前先解释理由。不要让每个接手 Agent 默认通读全部课程。

## 分阶段实施

### M0：先写行为与不变量

学习者先定义下述类型的含义和场景预期，允许 AI 生成 Rust 样板：

- PermissionProfile：资源读写范围；ExecPolicy：Allow/Prompt/Forbidden。
- ApprovalPolicy：第一版只实现 OnRequest/Never；Reviewer 先使用预设答复。
- ApprovalRequest：命令、资源请求、原因；ApprovalDecision：批准/拒绝。
- ExecutionMode：默认受限执行、保留沙箱并增加明确权限；无沙箱模式只作为可选模拟对照。
- GrantScope：Once/Session/PersistentRule；先完成 Once，再实现其余两种。
- ExecutionResult：策略拒绝、审批拒绝、创建失败、沙箱拒绝、普通执行失败、成功。错误类别与调用是否已返回分别记录。

审批请求不得自我扩大：只为具体已批准请求计算有效权限。明确禁止在审批前拦截，即使请求携带额外权限也不能跳过命令检查。源 Profile 不因 Once 授权永久改变。

### M1：完成一次性批准与部分副作用

先按下面场景表验证事件、身份、文件状态三个维度。FakeExecutor 顺序运行 Append/Write/Copy 等操作，能够模拟创建失败、磁盘满和部分目标写入。不要用“场景名直接返回成功/失败”代替实际状态变化。

### M2：验证恢复决策与授权时效

第一版拒绝自动整条重跑：将失败返回给确定性的调用方，由调用方检查产物并提出新请求。报告有效后只重试 Copy；A 的日志追加次数必须保持为 1。报告完整性用内容或哈希比较，不仅判断文件存在。

随后实现会话缓存和持久规则：明确匹配键包含什么，范围变化是否命中；测试 Once 不复用、Session 匹配复用且新会话不复用、PersistentRule 保存后可重载且不匹配命令仍受限制。实验可采用严格完整请求键作为会话键，必须说明这不是 Codex 所有工具的统一缓存算法。

复用授权和执行权限分开：命中缓存/规则以后，仍需计算执行方式并落实边界。授权过期不会自动删除副作用；若模拟 yield，工具返回也不终止模拟进程的已有执行权限。

内部第二次 Attempt 作为可选对照，最后再加：同一个 CallId 下的新 Attempt 与“返回调用方后新 CallId”的链路分别测试，不把任一行为宣称为所有 Codex 工具默认行为。

### M3：真实观察与交付

完成 Fake 验收后，复用现成可用的沙箱运行环境，在明确的新建实验目录中观察允许写入与禁止写入。启动前确认实际 Profile、审批模式和执行方式，防止实验被自动提权后失去对照。没有可用沙箱时，标记真实观察未完成，不把 Fake 测试当作真实隔离证据。

交付代码与测试、每类场景的事件记录和前后状态、源码对照差异、学习者复述、STATUS 下一动作。Fake 测试通过与真实观察通过分别记录。

## 核心场景验收表

| 场景 | 关键预期 | 重点观察 |
| --- | --- | --- |
| 默认范围内写入 | 成功，无额外审批 | Profile 与命令准入分别记录 |
| Forbidden + 提权申请 | 策略拒绝，无审批、无进程 | 模型不知道规则也不能越过执行检查 |
| 允许申请但拒绝 | 审批拒绝，无本次进程 | 先前文件不回滚 |
| Never + 超范围明确申请 | 拒绝额外权限，不询问 | Never 不等于无限权限 |
| 保留沙箱，批准 Archive | Archive 写成功，Other 写失败 | 默认权限 + 获批范围；不能扩大到请求外 |
| A 追加日志、B 生成报告、C 越界复制 | A/B 保留，C 失败 | 沙箱拒绝不是事务回滚 |
| 检查报告后只重试 C | 归档成功，日志仍只有一条 | 新 Call/Attempt；不重复 A/B |
| 已获批但磁盘满 | 普通执行失败，可留部分目标 | 不继续提权；空间和残留状态检查 |
| Once/Session/PersistentRule | 按各自范围复用 | 规则、缓存、有效资源权限不能混为一物 |
| 相同原因重复失败 | 达到预设上限后结束 | 明确最大尝试次数；无无穷循环 |

错误已明确时不为展示重试而重试。若测试可恢复故障，使用模拟时钟与有界重试策略，避免真实 sleep。参数错误、策略拒绝、审批拒绝等不应统一进入退避重试。

## 学习者与 AI 的分工

学习者必须先解释场景预期、授权范围、错误分类和重试理由，再审阅关键分支；这些不能仅由 AI 宣布正确。AI 可以搭建目录、生成类型/序列化/测试样板、补事件输出与整理证据。优先把精力花在 orchestrator、授权合成和失败后状态上，复用 Rust 测试设施与现成沙箱，不手写安全级平台实现。

### 必测场景

1. 工作区内只读动作直接在默认沙箱执行。
2. 明显需要额外权限的动作在进程创建前请求审批。
3. 用户拒绝审批，不得创建 Attempt 对应的真实进程。
4. 默认沙箱拒绝后，证据明确指向权限不足，获批后产生新的执行尝试。
5. 命令不存在、参数错误或非零退出不能伪装成沙箱问题。
6. 第一次尝试已在允许范围产生部分副作用时，不得自动重跑非幂等动作。
7. 托管 `Forbidden` 不能被普通用户审批覆盖。
8. 重试有明确上限；相同原因重复失败后结束并报告。

## 实验二：事件与诊断

建议固定下面的可观察事件，而不是只断言最终字符串：

```text
ToolCallReceived
PolicyEvaluated
ApprovalRequested
ApprovalDecided
ExecutionAttemptStarted
ProcessSpawned
SandboxDenied / ProcessExited / TimedOut / Cancelled
ToolOutputRecorded
```

每个场景都要检查：

- 哪些事件必须存在；
- 哪些事件绝不能存在；
- 副作用是否可能已经发生；
- 下一次动作是修改命令、申请权限、重试、取消还是结束。

## 实验三：安全的真实观察

只有确定性状态机通过后，才选择一个临时目录做极小真实实验：

- 读取允许文件；
- 写入允许目录；
- 尝试访问范围外路径；
- 观察拒绝发生在创建进程时还是系统调用时；
- 检查失败前允许范围内的副作用是否仍然存在。

实验不得使用发布、远程写入、删除真实数据或其他非幂等动作。无需把真实沙箱实现长期保留在 CodebaseAgent。

## Codex 源码接入方式

开始时先执行：

```powershell
codegraph explore "ExecPolicy approval PermissionProfile sandbox escalation exec_command"
```

当前可优先核对：

- `codex-rs/config/src/requirements_exec_policy.rs`：托管命令规则怎样进入 Policy；
- `codex-rs/core/src/tools/`：工具准入、审批与 sandboxing 编排；
- `codex-rs/core/src/unified_exec/errors.rs`：创建进程、进程失败和沙箱拒绝怎样区分；
- `codex-rs/shell-escalation/`：平台执行提升设施；
- 相邻测试中的正常、拒绝、批准和提升路径。

源码会变化，以上是定位入口，不是需要背诵的固定调用链。

## 完成检查

学习者应能脱离文档回答：

1. Policy、Approval 和 Sandbox 分别在防什么？
2. `Allow` 为什么不等于 unrestricted？
3. 用户拒绝审批时为什么不应该存在 ProcessId？
4. 沙箱拒绝与命令非零退出怎样区分？
5. 提权重试为什么可能重复第一次的部分副作用？
6. 什么证据出现后才允许申请更高权限？
7. 为什么真正的操作系统沙箱不适合作为本次手搓目标？
