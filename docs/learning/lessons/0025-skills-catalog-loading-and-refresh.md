# Skill 怎样从文件变成模型可用的指令

本课回答：磁盘上的 Skill 怎样被发现、被用户选中、进入模型上下文，以及文件变化后如何更新可用列表。依据当前学习分支源码静态核对；没有运行测试，也没有取得当前桌面客户端的调用追踪。实现结论以本仓库为边界，不把客户端传入的 `<skill>` 内容反推为某个函数必然执行的证据。

建议难度：中高。重点记职责、数据形态和生效时刻；函数名用于源码定位，不要求背诵。本课覆盖目录、显式选择、正文注入、去重及本地刷新，不代表整个 Skills 专题完成。包结构与内容组织、完整来源优先级、隐式选择、跨 Turn/压缩/恢复及接入实践仍按[路线图](../roadmap.md#skills-专题推进单元)继续。

## 1. 为什么已有工具还需要 Skill

一个 Agent 已有读文件、搜索和列目录工具，说明它有获取代码的能力。用户要求分析恢复机制时，它仍需决定从哪些入口开始、怎样区分持久事实与运行投影、检查哪些失败场景，以及什么证据足以支撑结论。Skill 可以保存这些可复用的工作方法，并附带参考资料或辅助脚本。

Skill 正文提供方法；模型理解方法后选择行动；工具与执行设施真正读取文件、执行脚本或访问外部能力。读取 Skill 不会自动创建它要求的工具、依赖或权限。

需要分开三个时刻：知道有这个 Skill、读到正文、实际按照它完成工作。排查“Skill 没起作用”时，应分别检查发现、模型可见目录、选择、正文读取和后续行动，不能只看文件是否存在。

## 2. 目录生成不是创建文件夹

这里的“目录”指可用 Skill 清单。发现与解析先得到内存中的元信息，目录生成再把元信息变成模型能读的文本。

以 `agent-docs` 为例，元信息近似如下，省略不相关字段：

```text
SkillMetadata
  name: agent-docs
  description: Write and maintain project documentation ...
  path_to_skills_md: .../agent-docs/SKILL.md
```

目录条目近似如下：

```text
### Available skills
- agent-docs: Write and maintain project documentation ...
  (file: .../agent-docs/SKILL.md)
```

名称用于指认，描述帮助判断适用性，来源位置说明怎样取得正文。采用短路径时还会提供 Skill roots 映射。目录也可以附带使用说明，但不包含所有 Skill 的完整正文。

这种拆分使模型先用较少上下文判断需要哪种方法，再取得正文和相关资料。如果把所有正文和附件都预先放进每次请求，会增加无关内容与上下文开销；如果只把文件放在磁盘上，不提供发现与读取途径，模型也不能稳定使用它们。这是依据实现归纳的设计理由。

源码入口：[元信息类型](../../../codex-rs/skills/src/model.rs)、[目录条目与预算](../../../codex-rs/ext/skills/src/render.rs)、[目录整体文本与使用说明](../../../codex-rs/ext/skills/src/catalog_prompt.rs)。

## 3. 显式选择解析的是本次输入，不是整个历史

Core 本地路径调用 [collect_explicit_skill_mentions](../../../codex-rs/skills/src/selection.rs)，输入是本次的 `UserInput` 列表、已发现 Skill 的查询对象及连接器名称计数，输出是 `Vec<SkillMetadata>`。它通过代码规则确定用户明确选中了谁，不在这里读取正文或执行工作，也不重新扫描全部聊天历史。

### 普通文本中的 `$agent-docs`

对于 `请用 $agent-docs 整理文档`，内部调用 [extract_tool_mentions](../../../codex-rs/skills/src/mentions.rs)，提取结果近似为：

```text
names       = {agent-docs}
plain_names = {agent-docs}
paths       = {}
```

`$` 是标记，不属于名称；连字符是允许的名称字符。提取名称后，函数遍历已发现的 Skill，检查是否启用、是否重复，并要求同名启用 Skill 恰好一个、没有同名连接器冲突，再把对应元信息加入结果。

因此它不是把名称直接拼成路径。路径来自匹配成功的元信息；文件未被发现、被禁用或名称有歧义时，裸名称不保证选中。输入中的普通自然语言“请用 agent-docs”可能被模型理解，但不能认为这个文本提取函数必然识别不带 `$` 的名称。

### 客户端已经构造了结构化选择

输入也可能是 `UserInput::Skill { name, path }`。函数先处理这种选择，将路径转成统一表示，按 Skill 身份路径或发现路径查找，检查启用状态并按路径去重。随后才扫描普通文本。

结构化选择还会把相应名称放进 `blocked_plain_names`：用户明确指定的路径即使不存在或被禁用，也不会因为文本里还有同名提及，就悄悄改选另一份同名 Skill。带路径的文本链接同样可以用于精确选择。

这一区分保护了用户选择的身份：明确指向 A 位置，不等于允许系统改用 B 位置的同名内容。相应[测试](../../../codex-rs/skills/src/selection_tests.rs)覆盖了结构化输入优先、发现路径、禁用、路径去重、名称歧义、连接器冲突和缺失路径不兜底等情况；本课只静态阅读，没有运行它们。

仓库中扩展层还有一个同名函数，处理统一来源目录。以上算法描述明确针对 `codex-rs/skills/src/selection.rs`，不能把它的所有细节直接套到扩展版本。

## 4. 正文如何进入模型上下文

用户未明确指定时，模型可以依据目录判断需要某个 Skill，再调用相应读取能力取得正文。显式选择则存在运行时预先读取并注入的路径，不必都等模型发起读文件调用。

Core 本地路径如下：

```text
build_skills_and_plugins
  → collect_explicit_skill_mentions
  → mentioned_skills: Vec<SkillMetadata>
  → load_skill_prompts(&mentioned_skills)
  → 将正文片段转换为模型消息项
  → 合并其他输入片段
```

[load_skill_prompts](../../../codex-rs/ext/skills/src/host_prompt.rs)逐个调用 `read_skill_text`，读取成功后构造 `SkillInstructions`，同时记录成功生成片段的 Skill；读取失败则产生警告。返回的 `HostSkillPrompts` 包含：

| 字段 | 职责 |
| --- | --- |
| `fragments` | 成功生成的正文上下文片段 |
| `injected` | 与这些片段对应的 Skill 元信息，仍需后续合并 |
| `warnings` | 读取失败或正文截断等警告 |

这里不会递归加载全部 `references/`，也不会执行 `scripts/`。引用资料按正文中的路由和实际任务另行取得。渐进披露主要控制选择哪些文件，不表示选中一个必读指令文件后只看开头就算读完。

`SkillInstructions` 是上下文组装对象，不是摘要或固定大小的数据块。其内容形态近似为：

```xml
<skill>
<name>agent-docs</name>
<path>.../agent-docs/SKILL.md</path>
这里是读取到的文件内容
</skill>
```

当前[片段实现](../../../codex-rs/ext/skills/src/fragments.rs)将可用目录表达为 developer 角色，将选中的 Skill 正文表达为 user 角色。真正调用和转换见 [turn.rs](../../../codex-rs/core/src/session/turn.rs) 的 `build_skills_and_plugins`。

普通 Core 路径走到该位置时会调用 `load_skill_prompts`，不以“有 `$` 才调用”为条件：匹配结果为空时传入空列表，不读取正文。Guardian 等特殊路径可能提前返回。收到 `<skill>` 内容能够证明正文已进入本次模型输入，但没有客户端追踪时，不能反推当前桌面程序恰好调用了本地分支的这个函数。

## 5. 目录预算与正文长度不是同一个限制

依据 [render.rs](../../../codex-rs/ext/skills/src/render.rs)、[host_prompt.rs](../../../codex-rs/ext/skills/src/host_prompt.rs) 与[扩展注入](../../../codex-rs/ext/skills/src/extension.rs)：

| 对象或路径 | 本课核对到的处理 |
| --- | --- |
| 目录元信息显式配置预算 | 采用配置值，上限 10,000 token |
| 未配置预算、有有效模型上下文窗口 | 取窗口的 2%，最少 1 token；此分支不套用上述配置上限 |
| 无配置且窗口未知 | 回退到 8,000 字符 |
| `load_skill_prompts` 的普通本地 Skill | 此函数不截断正文 |
| `load_skill_prompts` 中被标记为 agent plugin 的 Skill | 正文截到最多 8,000 字节，并产生警告 |
| Skills 扩展路径注入正文 | 调用同一正文截断函数，最多 8,000 字节，并产生警告 |

目录 token 成本是近似计算。正文限制是 UTF-8 字节，不是 token 或字符；截断避开字符编码内部边界。8,000 字节约束针对 `contents`，不包括名称、路径和标签等外层包装。普通本地函数不截断也不代表整个模型上下文无限。

因此不能把“Skill 最多 8,000”当作统一结论，必须先说明目录还是正文、字符还是字节、Core 本地路径还是扩展路径。目录的进一步裁剪、来源优先级及上下文生命周期留待对应单元。

## 6. 为什么一次选择仍需要合并去重

这里的“扩展”指 Codex 内部 Skills 扩展模块，通过 `TurnInputContributor` 等接口贡献上下文，不是用户重复调用 Skill，也不是另一个子 Agent 生成正文。

Core 本地路径能够读取本地 Skill；Skills 扩展能够处理本地、执行环境和云端等来源。两条路径的覆盖有交集，一次 `$agent-docs` 可能分别产生本地正文片段。

```text
一次用户选择
  ├─ Skills 扩展读取正文 → 片段 A → 登记已提供正文的本地路径
  └─ Core 本地读取正文 → 片段 B
                                      ↓
                    合并时按登记过滤重复 B，保留 A
```

扩展将本地路径登记到 `InjectedHostSkillPrompts`；Core 在最终合并时过滤自己的重复片段，然后加入扩展产物。这解决多个内部上下文生产者的交集，不是对模型一轮中多次工具调用去重。

当前过滤发生在 Core 调用 `load_skill_prompts` 之后，因此主要保证最终上下文不重复，不保证文件只读取一次。完整来源替代规则需继续查看扩展逻辑，不只由路径去重决定。

## 7. Skill 清单为什么能在 Thread 存续期间更新

需要分开界面的 `$` 候选列表、当前 Turn 的元信息快照，以及已经进入模型请求的正文。三者并不同时更新。

当前本地链路如下：

```text
文件新增、修改或删除
  → SkillsWatcher 收到相关变化
  → 清除 HostSkillsService 缓存
  → 发送 SkillsChanged
  → 客户端请求强制刷新列表
  → 服务重新取得发现结果
  → 更新界面候选
```

这一过程不需要重开 Thread 或调用模型。TUI 的[通知处理](../../../codex-rs/tui/src/chatwidget/protocol.rs)收到 `SkillsChanged` 后调用 `refresh_skills_for_current_cwd(true)`；[服务端列表处理](../../../codex-rs/app-server/src/request_processors/catalog_processor.rs)支持 `force_reload`。

Core [构建 Turn 上下文](../../../codex-rs/core/src/session/turn_context.rs)时也会取得 Skill 快照，服务可复用缓存或在失效后重建，不限于 Thread 创建时获取一次。已有 Turn 持有旧快照，并不因清缓存被强行改写；候选消失也不表示模型以前读到的正文已经被删除，更不能改变已发出的请求。

用户观察到外部文件变化后 `$` 候选快速更新，符合这种动态刷新机制；没有当前客户端调用追踪，不能断言是哪一次文件事件或主动刷新请求导致了该次更新。

## 8. SkillsWatcher 用事件失效，不做内容哈希比较

[SkillsWatcher](../../../codex-rs/app-server/src/skills_watcher.rs)取得要监听的本地 Skill 根路径并注册递归监听。[FileWatcher](../../../codex-rs/file-watcher/src/lib.rs)通过 `notify::recommended_watcher` 接收后端通知，回调将结果送入通道，由后台异步任务处理。

底层仅保留 `Create`、`Modify`、`Remove` 类事件及非空路径，再按订阅路径匹配和分发；普通访问事件不在这个集合中。它没有周期性读取所有文件并计算内容哈希，也没有先比较 description 或正文的新旧值。

因此，判断是“缓存可能过期”，不是“已证明内容不同”。编辑器即使保存了相同内容，只要产生符合条件的事件，也可能触发刷新。监听根目录还能发现新 Skill，而不必提前注册每个未来文件；目标尚不存在时，FileWatcher 可以从最近存在的祖先目录监听，随路径出现调整实际监听位置。

变化路径暂存于 `BTreeSet`，重复路径合并。`ThrottledWatchReceiver` 控制交付频率：第一次事件可立即交付；后续交付不得早于上次交付后约定间隔。本地 SkillsWatcher 生产配置间隔为 10 秒，测试为 50 毫秒。这不是每 10 秒扫描一次，也不是每个事件都先等待 10 秒。

```text
t=0：第一次变化可立即交付 → 清缓存并通知
t=1、t=2：又有变化 → 路径累计、去重
t=10 左右：允许交付下一批 → 再次清缓存并通知
```

SkillsWatcher 会跳过全部路径都位于内置 system skills 缓存目录下的事件批次。其他相关批次触发 `clear_cache()`，清空服务的 `cache_by_cwd` 和 `cache_by_config`，不是只删一个 Skill 的条目。下一次请求再重建所需结果。

从实现看，这选择了较粗的失效粒度，免去在监听层推导文件变化对多个目录快照、同名选择和配置集合的精确影响；代价是可能重复扫描未受影响的内容。它并不表示所有监听环境都可靠或所有来源都遵循此机制：本课只解释已定位的本地路径，不扩张到远程来源。

## 9. 复习时沿哪条线定位

| 问题 | 首先检查 |
| --- | --- |
| 文件存在，但找不到 Skill | 发现范围、解析错误、启用状态与列表刷新 |
| `$name` 没有选中 | 输入形式、名称歧义、连接器冲突、路径匹配 |
| 选中了却没有正文 | 读取结果、警告、所走注入路径 |
| 正文不完整 | 目录与正文区分、来源对应的截断规则、实际读取结果 |
| 同一正文是否被放了两次 | 扩展登记及 Core 合并过滤，不先假定用户重复调用 |
| 文件改了但模型仍按旧内容工作 | 候选列表、Turn 快照、正文读取时刻及已发出的请求分别核对 |

接着阅读[Skill 包结构](0026-skills-package-execution-and-usage-tracking.md)时，沿同一场景判断：description 负责怎样被发现和选用，正文负责工作方法与路由，引用资料及执行资源如何在需要时取得。本文不预先替接入实践选择平台或修改 CodebaseAgent 的行为契约。
