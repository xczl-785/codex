# Skill 发现范围与输入身份

本课回答两个问题：模块中的 Skill 什么时候可被发现，以及界面显示的名称如何指向具体 Skill。前置知识见[加载与刷新](0025-skills-catalog-loading-and-refresh.md)和[包结构与执行](0026-skills-package-execution-and-usage-tracking.md)。依据为当前分支静态源码；TUI 的行为不直接代表未核对的桌面客户端实现。

## 1. 先找扫描根，再扫描包

[resolve_skill_roots](../../../codex-rs/ext/skills/src/host_roots.rs)汇总配置层关联的 Skill 根、用户目录、系统缓存、插件根、额外根及项目祖先链中的 .agents/skills。它没有从项目根向下搜索整个仓库中的所有 .agents/skills。

对于项目祖先链，先依据项目标记确定边界，再从发现用的 cwd 沿祖先目录到项目根，检查这些目录各自的 .agents/skills。这里的 cwd 是发现操作的输入；任务涉及哪个文件，不会自行改变这个参数。

```text
Project/
  .agents/skills/common/SKILL.md
  module/.agents/skills/module-special/SKILL.md
```

假设项目根被识别为 Project，没有额外配置改变发现范围：

| 发现用 cwd | 这条项目链路发现的包 |
| --- | --- |
| Project | common |
| Project/module | common、module-special |

从 Project/module 启动 Codex 是第二种情况的直接例子。从 Project 启动后要求修改 module 文件，并不意味着 module-special 自动进入清单；某次 shell 临时 cd 也不能等同于会话发现用 cwd 改变。

[discover_skills](../../../codex-rs/ext/skills/src/loader/discovery.rs)的递归是从已经确定的 Skill 根开始，例如 Project/.agents/skills，而不是从 Project 开始。module/.agents/skills 位于这个扫描根之外，自然不会被顺带找到。

反过来，如果把模块 Skill 放在 Project/.agents/skills/module-special，它就在公共扫描根内部，可以被根目录会话发现。该目录名仅用于组织包，不产生模块发现范围；适用任务可以由 description 说明。

所以模块自己的 .agents/skills 主要服务于以模块为工作目录开展工作的方式。它让模块会话取得上层公共能力和本模块能力，减少无关兄弟模块能力的混入。若使用者始终从项目根工作，单纯将包放入模块内部并不能实现自动按文件选用。明确让 Agent 读取某个 SKILL.md 是另一条指令读取路径，不等于默认目录发现已经注册该包。发现本身也不等于全部正文已注入。

## 2. 路径身份与同名包

[host_merge](../../../codex-rs/ext/skills/src/loader/host_merge.rs)按 path_to_skills_md 去重，不简单按名称覆盖。同一文件被多个扫描来源发现时保留一次；不同路径的同名包可以共存。路径规范化帮助识别符号链接等重复身份。

因此不能套用“项目同名 Skill 必然覆盖用户同名 Skill”的规则。来源 scope、合并顺序和最终排序有各自作用，不等于名称覆盖。名称适合称呼能力，路径用于定位本地资源；选择阶段仍需处理歧义。

## 3. 界面名称与提交数据

[输入协议](../../../codex-rs/protocol/src/user_input.rs)定义 UserInput::Skill，字段为 name 和 path，后者指向 SKILL.md。TUI 可以显示 $review，同时保留名称与目标路径的 mention binding。

[TUI 提交实现](../../../codex-rs/tui/src/chatwidget/input_submission.rs)将绑定路径与候选 Skill 匹配，构造结构化 Skill 输入。一次提交可以既有原始文本，又有 Skill { name, path }。这个结构项不携带整份正文，正文在后续加载。

TUI 还会识别手打文本中的 Skill mention，尝试补成结构化输入。因此关键不是“点选还是手打”，而是后端最终收到什么。

| 后端输入 | Core 所用选择实现的处理 |
| --- | --- |
| Text 中的裸 $review | 提取名称，检查唯一匹配及冲突等条件 |
| Skill { name, path } | 优先按身份路径或发现路径匹配已启用 Skill |

[collect_explicit_skill_mentions](../../../codex-rs/skills/src/selection.rs)还记录结构化输入的名称，阻止失败后按同名文本悄悄选中另一个包。当前 Skills 扩展也有自己的选择实现，不能将 Core 的裸名称歧义规则不加区分地推广到所有入口。

## 4. 谁选择，以及允许怎样选择

用户显式指定时，程序负责解析并定位资源，再加载正文；模型随后理解和执行指令。没有显式指定时，模型可以依据候选描述及宿主指令自主决定读取哪个 Skill。识别工具命令中的隐式使用发生在决策之后，是使用归属记录，不是替模型作出选择的后台关键词规则。

`agents/openai.yaml` 中的 `policy.allow_implicit_invocation` 默认允许。设置为 false 时，[本地提供器](../../../codex-rs/ext/skills/src/provider/host.rs)将条目标记为 hidden_from_prompt，不向模型默认展示，但仍可显式选中。这与 disabled 不同：已禁用条目也会在正常显式选择链路中被过滤。

| 状态 | 默认候选提示 | 显式选择 |
| --- | --- | --- |
| enabled，允许隐式调用 | 可以进入，仍受预算等条件限制 | 可以 |
| enabled，禁止隐式调用 | 隐藏 | 可以 |
| disabled | 不进入 | 正常选择链路过滤 |

这些设置控制 Skill 接入路径，不等于文件访问权限。普通工具是否能读取已知路径由权限决定；改变启用状态也不会自动删除已进入历史的正文。程序注入正文不等于把自然语言流程编译成强制执行的状态机。

项目扫描解决“有哪些候选资源”，选择解决“本次指向哪个资源”，加载解决“取得哪个资源的正文”。上下文如何承载状态变化见[课程 0028](0028-skills-context-updates-and-incremental-tools.md)。
