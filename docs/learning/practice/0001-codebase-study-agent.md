# 实践任务 0001：实现一个通用代码库理解 Agent

这份文档定义课程 0001—0015 复盘后的第一个实践任务：实现一个能够绑定任意本地代码库、通过只读工具收集证据并回答源码问题的最小 Agent Harness。

Codex 是后续的重要对照样本和压力测试目标，但不是这个 Agent 的硬编码服务对象。第一版不得假定仓库使用 Rust、存在 `.codegraph/`、采用 Git，或具有 Codex 的目录和类型命名。

实践采用“关键控制面手写、基础设施复用、AI 受控协作”的方式。完成后，应能独立解释一次代码问题怎样经过 Turn、Step、Function Call、工具执行、证据积累和历史记录形成最终回答。

## 一、实践目标

输入由两部分组成：

```text
RepositoryRoot：允许读取的代码库根目录
Question：需要回答的代码问题
```

输出不是泛化的模型猜测，而是一份能够追溯到仓库文件和行号的解释：

```text
任意代码库 + 源码问题
  ↓
模型判断缺少什么证据
  ↓
调用 list_files / search_text / read_file
  ↓
记录 FunctionCall 与 FunctionCallOutput
  ↓
下一 Step 继续定位、核对或寻找反证
  ↓
输出结论、证据位置和仍不确定的部分
```

第一阶段至少支持三类问题：

1. **定位**：某项能力、配置或入口在哪里；
2. **追踪**：一个输入、事件或调用经过哪些模块；
3. **解释**：某个模块负责什么，为什么这样拆，不这样做会怎样。

## 二、为什么现在开始实践

开始实践不需要先读完整个 Codex 仓库。合适的起点是已经能够回答：

1. 模型为什么不能直接执行工具；
2. 工具结果怎样进入下一次模型请求；
3. Thread、Session、Turn、Task 和 Step 为什么分开；
4. 哪些状态需要持久化，哪些运行资源不能恢复；
5. 一次失败为什么需要分类，而不能统一重试；
6. 第一版应该主动舍弃哪些能力。

课程 0001—0015 已经建立了这些基础。继续纯概念复习的边际收益开始下降；直接修改 Codex 核心又会被大型工程细节淹没。因此下一阶段采用：

```text
定义通用代码库问答契约
  ↓
用小型固定仓库验证最小 Harness
  ↓
接入真实模型并绑定任意仓库
  ↓
加入持久化和恢复
  ↓
用 Codex 等大型仓库进行对照和压力测试
  ↓
再按取消、并行、Subagent 的顺序扩展
```

## 三、为什么不能把 Codex 写死在第一版里

如果工具、Prompt 和目录结构只面向 Codex，容易把下面两类知识混在一起：

- **通用 Agent Harness 机制**：Turn/Step 循环、工具路由、结果反馈、历史和恢复；
- **Codex 仓库知识**：Rust workspace、`codex-rs/core`、App Server、rollout 等具体实现。

这种混合会带来三个问题：

1. 很难判断 Agent 能力来自通用设计，还是 Prompt 中预先写入的 Codex 知识；
2. 工具和测试会依赖某个仓库当前目录，无法验证抽象是否成立；
3. 实践结束后只得到一次性学习脚本，难以迁移到日常项目。

通用化后的边界是：

```text
Agent Runtime：不知道目标仓库是什么项目
Repository Binding：只提供受限根目录和仓库元信息
Generic Tools：按路径、文本和行号读取证据
Optional Adapters：按仓库能力增加 Git、CodeGraph、LSP 等增强
```

Codex 可以在后期作为一个复杂目标，用于回答“通用最小实现为什么还不够”。

## 四、CLI、TUI、Desktop 和服务端是什么关系

### CLI 是程序入口与命令交互方式

CLI 是 **Command-Line Interface**。它既可以指逐次执行的命令交互，也常被用来指安装在终端中的整个命令行应用。

例如，第一版实践可以采用：

```text
codebase-agent --repo D:\Project\example "登录入口在哪里？"
```

程序读取一次问题、输出答案后退出，适合先验证 Agent Runtime。

### TUI 是运行在终端里的交互式界面

TUI 是 **Terminal User Interface**。它仍运行在终端里，但会管理屏幕布局、键盘事件、光标、滚动、状态栏、弹窗和局部重绘。

CLI 程序可以默认启动 TUI，也可以提供非交互子命令：

```text
Codex CLI
  ├─ codex       → TUI
  └─ codex exec  → 非交互 CLI

OpenCode CLI
  ├─ opencode      → TUI
  └─ opencode run  → 非交互 CLI
```

参考：[OpenCode CLI](https://opencode.ai/docs/cli) 与 [OpenCode TUI](https://opencode.ai/docs/tui)。

### Desktop 是图形客户端形态

Desktop 通常指原生窗口或 Web 技术封装的桌面 GUI。TUI、Desktop 和 IDE 可以复用相同的 Agent Runtime，也可以通过服务协议访问它：

```text
CLI / TUI ──┐
Desktop ────┼─→ Agent Runtime 或 Server API
IDE ────────┘
```

第一版实践只做非交互 CLI。TUI、Desktop 和 Server 都不是验证模型—工具闭环的前置条件。

## 五、第一版范围

### 必须实现

- 通过参数绑定一个显式代码库根目录；
- 接收一个源码问题并输出有证据的回答；
- 一个可持久标识的 Thread；
- 一个内存 Session，并持有本次 Repository Binding；
- 一个 Turn 执行循环；
- 每次模型采样使用稳定的 StepContext；
- 模型可见 ToolSpec；
- ToolRegistry 与参数校验；
- `list_files`、`search_text`、`read_file` 三个通用只读工具；
- FunctionCall、FunctionCallOutput 与 `call_id` 配对；
- 明确的继续和结束条件；
- 有界工具输出和生命周期日志；
- 确定性测试使用的 ScriptedModel；
- 使用固定 fixture repository 验证答案，而不是依赖 Codex 当前源码。

### 第一版明确不做

- TUI、Desktop 或 Server；
- 文件修改工具；
- Shell、编译、测试、审批和沙箱；
- Git 历史和 diff 分析；
- CodeGraph、LSP、AST 或语义索引；
- 自动压缩；
- 并行工具调用；
- 长进程与 yield/poll；
- Subagent；
- Fork、Revert 和完整恢复；
- 多模型、多提供商和插件系统。

这些能力后续可以按真实问题增加。第一版只验证通用代码证据怎样进入模型—工具反馈闭环。

## 六、Repository Binding：通用化的关键边界

Session 不应直接拥有“整台机器都能读取”的能力，而应绑定一个经过验证的仓库根目录：

```text
RepositoryBinding
  ├─ root：规范化后的绝对路径
  ├─ display_name：显示名称
  ├─ capabilities：当前可用的可选能力
  └─ limits：文件大小、结果数量和输出长度上限
```

所有文件工具必须遵守：

1. 只接受相对于仓库根目录的路径；
2. 解析 `..`、符号链接和绝对路径后，最终目标仍必须位于根目录内；
3. 默认拒绝二进制文件和超大文件；
4. 搜索结果、单次读取行数和工具输出必须有硬上限；
5. 文件不存在、编码错误和读取拒绝应成为结构化 Tool Output；
6. 工具结果应携带相对路径和行号，便于最终回答引用。

如果缺少这层边界，所谓“只读 Agent”仍可能读取仓库之外的敏感文件，而且大文件可能直接耗尽模型上下文。

## 七、通用工具契约

### `list_files`

用途：了解仓库轮廓，而不是一次返回整个目录树。

建议参数：

```text
path       相对目录
pattern    可选名称过滤
max_items  最大返回数量
```

### `search_text`

用途：按字符串或正则定位候选证据。

建议参数：

```text
query       搜索内容
path        可选相对范围
file_glob   可选文件过滤
max_matches 最大匹配数量
```

结果必须包含相对路径、行号和有界上下文。

### `read_file`

用途：读取候选文件的精确范围。

建议参数：

```text
path        相对文件路径
start_line  起始行
max_lines   最大行数
```

工具不应默认把完整大文件塞进模型上下文。

### 后续可选适配器

通用基础稳定后，可以按仓库能力增加：

- Git status、diff、log 的只读适配器；
- `.codegraph/` 存在时的 CodeGraph 适配器；
- LSP、AST 或编译数据库适配器；
- 项目级说明和规则文件发现；
- 测试、构建配置和依赖图读取。

可选适配器是增强证据获取效率，不应改变 Turn/Step/Call/Output 的主循环。

## 八、框架、库与手写的边界

“是否使用框架”不能简单回答为全用或全不用。应该根据学习目标，把系统分成控制面和基础设施。

### 应当亲手实现的控制面

这些部分不能交给高层 Agent 框架隐藏：

1. Thread、Session、Turn、Step 的状态与生命周期；
2. Repository Binding 的所有权和访问边界；
3. `run_turn` 外层循环；
4. StepContext 在什么时候捕获；
5. ToolSpec、ToolRegistry 与工具分发；
6. FunctionCall 和 Output 的记录顺序；
7. `call_id` 配对；
8. 工具失败怎样反馈给模型；
9. Turn 继续和终止条件；
10. 证据怎样进入最终回答；
11. 哪些事实写入历史，哪些资源只保留在内存；
12. 对这些行为的确定性测试。

如果使用 LangGraph 或另一种高层 Agent 框架直接提供状态循环、工具路由、历史和重试，本次最需要理解的部分会被框架代劳。

### 可以直接复用的基础设施

- Rust 异步运行时；
- HTTP 客户端；
- JSON 序列化与反序列化；
- CLI 参数解析；
- UUID 或 ID 生成；
- 错误类型辅助库；
- glob、regex 和目录遍历库；
- 临时目录和测试工具；
- 日志与 tracing；
- 真实模型的官方 SDK 或低层客户端；
- JSON Schema 校验库。

原则是：

> 会影响 Agent 行为语义和安全边界的控制流亲手写；只负责搬运、编码和平台接入的基础设施直接复用。

## 九、语言选择

本实践默认使用 Rust。原因不是 Rust 天然更适合所有 Agent，而是长期学习目标仍包括理解 Codex：

- 可以直接体验 trait、enum、所有权和异步任务如何表达运行边界；
- 能把通用实现与 `codex-rs` 逐项比较；
- 后续阅读 CancellationToken、Future 和并行调度时有真实经验。

但是实践的工具契约和仓库模型必须保持语言无关。目标代码库可以是 C++、Rust、Python、TypeScript 或混合仓库。

如果只追求最快验证产品想法，Python 会更省时；当前选择 Rust 是为了让实现成本同时转化为源码学习收益。

## 十、AI 可以参与到什么程度

可以使用 AI，没有必要追求所有代码都亲自输入。关键不在于代码由谁敲出，而在于行为边界由谁决定、出现问题时谁能解释和修改。

### AI 可以代劳

- 创建 Cargo 工程和模块骨架；
- 编写常规 serde 数据结构；
- 接入 HTTP、日志、CLI、glob 和 regex 库；
- 根据已经确定的契约补全机械代码；
- 生成 fixture repository 和重复测试数据；
- 解释 Rust 编译错误；
- 查找可对照的开源实现；
- 对实现做静态审查；
- 提出反例、越界路径和大文件场景。

### AI 不能替你决定后再直接接受

- Thread、Session、Turn、Step 和 Repository Binding 的所有权；
- 一次 Turn 在什么条件下继续或结束；
- 工具调用和结果以什么顺序进入历史；
- 根目录逃逸、符号链接和输出上限怎样处理；
- 参数错误、业务错误和执行错误怎样分类；
- 重试是否安全以及谁负责重试；
- 恢复时哪些状态可以重建；
- 最终回答需要哪些证据才能下结论；
- 测试真正要证明的行为契约。

这些地方可以让 AI 提方案或实现候选，但最终必须经过自己的解释和验证。

### 掌控力度的验收标准

1. 不看实现也能画出主循环；
2. 能说明每个核心模块为什么存在，不这样拆会怎样；
3. 能预测一个工具失败后历史和下一 Step 怎样变化；
4. 能证明文件访问没有逃出 RepositoryRoot；
5. 能在日志中区分 Thread、Turn、Step 和 call ID；
6. 能修改一条行为规则，并指出需要更新哪些测试；
7. 测试失败时能判断是契约错误还是实现错误；
8. 不保留自己无法解释、无法验证的关键控制代码。

推荐协作循环：

```text
你先定义行为契约和场景
  ↓
AI 生成一个小模块或测试候选
  ↓
你解释状态、输入、输出、安全边界和失败路径
  ↓
运行确定性测试
  ↓
再进入下一个模块
```

## 十一、为什么先使用 ScriptedModel

第一里程碑不直接接真实模型，而是实现按脚本返回内容的 `ScriptedModel`：

```text
第一次采样 → 返回 search_text FunctionCall
第二次采样 → 根据搜索结果返回 read_file FunctionCall
第三次采样 → 检查是否收到对应 FunctionCallOutput
第四次采样 → 返回带文件和行号的最终回答
```

这样可以确定性验证：

- 一个 Turn 为什么包含多个 Step；
- Step 与 Tool Call 为什么不同；
- FunctionCall 是否先于执行结果进入历史；
- Output 是否使用正确的 `call_id`；
- 工具失败是否仍能进入下一 Step；
- 最终回答是否引用了真实工具证据；
- 模型给出最终回答后 Turn 是否正确结束。

真实模型具有随机性。若第一天就接真实模型，很难判断失败来自 Harness、Prompt、Schema、模型选择、仓库内容还是网络。

## 十二、实践里程碑

### 里程碑 0：通用行为契约与 fixture repository

先建立一个很小的固定测试仓库，包含：

- 两到三个模块；
- 一个可追踪的入口调用链；
- 一个重复符号或容易误判的干扰项；
- 一份说明文件；
- 一个明确的预期答案。

用它验证 Agent 的行为，不把 Codex 当前源码当测试夹具。

### 里程碑 1：确定性 Harness

- Rust 非交互 CLI；
- Repository Binding；
- ScriptedModel；
- `list_files`、`search_text`、`read_file`；
- 串行工具调用；
- 内存历史；
- 生命周期日志；
- 核心场景测试。

完成标志：不接网络也能确定性完成“搜索候选—读取证据—输出带位置结论”的闭环。

### 里程碑 2：接入真实模型

- 保留统一 Model trait；
- 通过低层 SDK 或 HTTP 客户端接入 Tool Calling；
- 运行时反序列化和校验参数；
- 把非致命工具错误反馈给模型；
- 要求最终回答区分事实、推断和未知。

完成标志：同一二进制可以通过 `--repo` 绑定不同仓库，并对至少两个不同语言的小型仓库完成只读问答。

### 里程碑 3：历史与恢复

- 记录 TurnStarted、UserMessage、FunctionCall、FunctionCallOutput、AssistantMessage、TurnCompleted；
- 保存 Repository Binding 的可恢复描述，不保存失效文件句柄；
- 退出后重新加载同一 Thread；
- 重建新的 Session；
- 检查仓库路径已经移动或消失时的失败语义。

完成标志：能够解释“恢复的是事实和仓库绑定描述，不是旧运行现场”。

### 里程碑 4：大型仓库与可选适配器

用真实项目验证结果边界和探索效率：

1. 先选择一个熟悉的小型业务仓库；
2. 再选择 Codex 作为复杂 Rust workspace；
3. 最后按需要加入 Git、CodeGraph 或 LSP 适配器。

适配器不能成为回答所有问题的硬依赖。

### 里程碑 5：取消与并行

先让取消从 Turn 传播到模型请求和文件工具，再允许多个只读工具并行。并行结果必须通过 `call_id` 配对，不能依赖完成顺序。

### 里程碑 6：Subagent

最后加入独立子 Thread、仓库绑定继承、消息传递、等待、完成通知和资源回收。此时再系统回看 Codex 的多 Agent 实现。

## 十三、第一版关键行为契约

1. 一个用户问题启动一个 Turn；
2. 一个 Turn 可以经历多个 Step；
3. 每个 Step 使用一次稳定的工具视图和 Repository Binding；
4. 模型 FunctionCall 必须先记录，再执行工具；
5. 每个 FunctionCallOutput 必须配回原 `call_id`；
6. 非致命工具失败仍作为 Output 进入下一 Step；
7. 模型只返回最终回答且没有待处理工具时，Turn 结束；
8. 所有文件访问必须限制在规范化后的 RepositoryRoot 内；
9. 所有模型可见工具结果必须有硬上限；
10. 最终事实性结论应引用仓库相对路径和行号；
11. 证据不足时应明确说明未知，不能用常见框架经验补齐事实；
12. Session 关闭时，历史事实与运行资源的处理必须分开。

## 十四、第一批测试场景

### 正常闭环

```text
用户问题
→ ScriptedModel 请求 search_text
→ 根据候选请求 read_file
→ 工具成功并返回路径和行号
→ Output 写回
→ ScriptedModel 返回有证据的答案
→ Turn 完成
```

### 路径越界

模型尝试读取 `../secret.txt`、仓库外绝对路径或通过符号链接逃逸。工具必须在读取前拒绝，并返回结构化错误。

### 输出有界

搜索命中过多、文件过大或请求行数过多时，工具返回截断信息和继续读取方式，不能无界写入模型上下文。

### 参数错误

模型请求不存在的路径或非法行号时，Handler 返回结构化错误；错误进入下一 Step，模型可以修正。

### 未知工具

Registry 拒绝未注册工具，不执行外部动作，并把错误与 `call_id` 一起写回。

### 因果配对

验证 FunctionCall 和 FunctionCallOutput 使用相同 `call_id`，而且 Output 之前已经存在对应 Call。

### 证据不足

fixture repository 故意缺少某项信息。最终回答必须标记未知，而不是生成貌似合理的实现细节。

### 仓库无关性

使用两个不同目录结构、不同语言的 fixture repository 运行同一 Harness，验证核心循环不依赖 Codex 或某种语言。

## 十五、工程位置与代码管理

实践工程建议建立为独立 Rust 项目，不直接加入 Codex workspace，也不把实验实现塞进 `codex-rs/core`。

这样做可以：

- 避免改变 Codex 的真实构建和依赖；
- 允许自由重写早期设计；
- 清楚区分“通用实践实现”与“Codex 当前源码事实”；
- 使用任意仓库作为目标，而不形成反向依赖；
- 后续把相同场景分别投射到自己的实现和 Codex。

具体目录、仓库位置和依赖清单在开始实现前再确定。本学习分支目前只保存任务说明、学习结论和对照文档。

## 十六、实践完成后的对照问题

每个里程碑结束后，先评价通用行为，再与 Codex 对照：

1. Repository Binding 属于 Thread、Session 还是 Turn，为什么？
2. 我们的 Session 中有哪些资源不能持久化？
3. Step 工具视图和仓库绑定是否会在执行途中漂移？
4. Call/Output 是否形成可判定的因果历史？
5. 工具失败由谁分类，模型什么时候有机会修正？
6. 最终回答中的每个事实是否有真实仓库证据？
7. 退出后能恢复哪些事实，不能恢复哪些运行资源？
8. Codex 的 CodeGraph、rollout、权限和事件体系分别解决了最小实现中的什么缺口？
9. 当前简化在哪些真实仓库场景下会产生问题？

实践的价值不在于复制 Codex，而在于先建立一个可迁移的最小 Agent 模型，再用 Codex 解释大型生产系统为什么需要更多层次和约束。
