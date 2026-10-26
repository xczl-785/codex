# Skill 包的职责、脚本权限与使用记录

本课回答：Skill 除了正文还能携带什么？执行其中的脚本是否得到特殊权限？系统为什么还要识别并记录 Skill 使用？前置内容见[课程 0025](0025-skills-catalog-loading-and-refresh.md)。事实依据是当前分支静态源码，不代表已验证当前桌面客户端的全部运行路径。

## 1. 包中的内容分别解决什么问题

以恢复机制分析为例，一个包可以组织为：

```text
recovery-analysis/
  SKILL.md
  references/
    persistence-model.md
    crash-recovery.md
  scripts/
    inspect-history.py
  assets/
    report-template.md
  agents/
    openai.yaml
```

这些辅助目录均不是每个 Skill 的必需部分；只有一个 SKILL.md 也可以构成完整 Skill。拆分的依据是内容的使用方式，而不是为了填满目录。

| 内容 | 主要职责 | 使用时机 |
| --- | --- | --- |
| YAML 中的 name、description | 标识能力并帮助判断适用场景 | 发现与选择阶段 |
| SKILL.md 正文 | 工作方法、关键约束、步骤和资料路由 | 选中后读取或注入 |
| references | 具体领域或分支所需的详细知识 | 当前任务需要时按引用读取 |
| scripts | 可重复的机械处理，如解析、转换和校验 | Agent 决定调用执行工具时 |
| assets | 模板及生成产物需要的资源 | 生成对应产物时 |
| agents/openai.yaml | 面向宿主或 UI 的接口、依赖及策略配置 | 支持该配置的加载链路处理时 |

description 应包含适用条件：若只有进入正文后才知道什么时候该用，发现阶段就缺少选择依据。正文保留所有任务都应遵守的关键约束，再说明遇到哪个分支读取哪个 reference；不能把必须遵守的要求藏进不一定会读的文件。

辅助文件不会因为位于这些目录就全部自动递归注入。对于本地文件 Skill，相对引用应从 Skill 所在目录解析，不能默认从当前工作区解析。assets 中的 Markdown 模板也可能被读取；文件角色由用途决定，不由扩展名决定。

scripts 提供确定性处理能力，但加载 Skill 不等于运行脚本。执行需要工具、解释器、依赖和相应权限；不要求先把整个脚本放进模型上下文，排错或审查时则可能需要读取源码。agents/openai.yaml 也不是创建 Subagent 的开关。

推荐明确填写 name 与 description，但应区分编写约定和解析器实际兼容性：[parse_skill_frontmatter_metadata](../../../codex-rs/skills/src/parser.rs)允许名称缺失或空白时使用调用方提供的回退名称，仍要求非空 description，并检查名称长度。不能将“推荐填写”说成所有缺失情形都必定解析失败。包组织示例可对照内置 [skill-creator 资料](../../../codex-rs/skills/src/assets/samples/skill-creator/SKILL.md)。

## 2. 脚本属于 Skill，是否改变沙盒权限

假设 Agent 执行工作区之外的 `recovery-analysis/scripts/inspect-history.py`。对操作系统而言，仍然是解释器读取脚本，再访问输入、输出和网络资源。脚本放进 scripts 目录不会自动扩大进程权限。

需要逐项区分：

- 脚本位于工作区外，不意味着一定不可读；取决于实际文件系统权限配置。
- 能读取或执行脚本，不意味着能改写脚本所在目录。
- 输出写进允许写入的工作区与写进受限的 Skill 安装目录，权限结果可能不同。
- 脚本访问网络仍受对应网络策略约束；依赖缺失也不会由 Skill 标签自动解决。

[工具执行编排器](../../../codex-rs/core/src/tools/orchestrator.rs)依据选定环境的权限配置组织审批和沙盒执行。这里不能推导出“Skill 脚本默认豁免”。Skill 指令能指导 Agent 如何申请权限，不能自行授予权限。

不过，系统可以识别命令来自哪个 Skill，并向审查扩展提供来源证据。来源参与审查与权限自动放行是两种不同作用；后者不能由前者推出。

## 3. 为什么要识别隐式 Skill 使用

用户不一定每次都发送 `$skill`。Agent 可能根据任务自行选择一个已知 Skill，通过普通命令读取其文档或运行脚本：

```text
Get-Content .../recovery-analysis/SKILL.md
python .../recovery-analysis/scripts/inspect-history.py
```

普通工具调用只暴露命令。如果没有识别层，系统只能看到“读了文件”“启动了 Python”，无法关联这次行为与已发现的 Skill，也无法让需要该信息的扩展统一获知使用情况。

[maybe_emit_implicit_skill_invocation](../../../codex-rs/core/src/skills.rs)解决的就是这种关联：调用检测器，以已知 Skill 身份及命令信息判断是否存在可识别的使用。识别不到就返回；识别到才继续登记。它不是重新做一次正文加载，也不负责授予执行权限。

## 4. 登记后做了什么

主链路为：

```text
工具收到命令请求
  → 识别是否读取已知 Skill 文档或运行相关脚本
  → 构造 Skill 身份键
  → 在当前 Turn 的隐式使用集合中去重
  → 发出遥测、插件归属和分析事件
  → 通知注册的 SkillInvocationContributor
  → 工具继续后续执行流程
```

本地 Skill 的去重键包含 scope、路径和名称；资源型 Skill 使用资源 ID。相同 Skill 在同一 Turn 中重复被隐式识别，不会反复走后续登记逻辑。这是隐式事件自身的去重，不是课程 0025 中 Core 与扩展之间的正文去重，也不能泛化成所有显式与隐式事件共享一个去重集合。

通知调用携带 `SkillInvocationKind::Implicit`，以及 Skill 资源、Turn ID 和扩展状态存储。Core 负责提供共同信号，消费者决定如何使用，避免执行工具直接依赖每一种分析或审查功能。

当前代码中可以看到三类用途：

1. **统计与归属。** 遥测和分析记录使用情况；来自插件的 Skill 还能登记插件在本 Turn 中的使用。没有这一步，自主选用 Skill 的行为可能漏记。
2. **选择实验的反馈。** [Skills 扩展](../../../codex-rs/ext/skills/src/extension.rs)将隐式使用交给启用的 shadow selection 实验；[实验实现](../../../codex-rs/ext/skills/src/shadow_selection_experiment/mod.rs)将观察到的使用与实验排名对照，记录命中及排名等指标。它是旁路评估，不应说成正常运行必须经过的选择器，也不是证明该选择最优的标签。
3. **审查来源证据。** [GuardianV2 扩展](../../../codex-rs/ext/guardian-v2/src/async_scorer/extension.rs)可以接收调用通知；[trusted_skills](../../../codex-rs/ext/guardian-v2/src/async_scorer/trusted_skills.rs)会规范化路径并校验允许的用户 Skill 根等条件，再登记符合条件的文档证据。它不是无条件相信所有 Skill 路径，也不会因此绕过执行权限。

GuardianV2 这条已核对路径的用户根包括配置的 CODEX_HOME/skills 和用户主目录的 .agents/skills。不能仅因某个外部目录被用来安装 Skill，就断言它自动满足该审查条件。具体作用还取决于扩展是否启用以及对应审查证据状态是否存在。

## 5. “使用记录”究竟证明了什么

[exec_command 处理器](../../../codex-rs/core/src/tools/handlers/unified_exec/exec_command.rs)在解析参数、检查部分调用条件后，实际启动命令之前调用隐式识别。因此这条记录首先证明：系统从收到的命令请求中识别到了相关 Skill 行为。

它单独不能证明：

- 后续审批已通过，命令成功启动并正常结束；
- 文档成功读取且完整进入了模型上下文；
- 模型遵守了 Skill 要求；
- 使用 Skill 改善了任务结果。

命令仍可能在后续权限审查或运行阶段失败。检测器基于可识别的命令和资源信息，包装命令等情况也可能漏识别，所以没有事件也不等于完全没有使用。

代码中的计数器名 `codex.skill.injected` 尤其容易造成误读：这条隐式路径发出该计数器，并不意味着它在这里又注入了一份正文。判断行为应看实际调用及执行顺序，不能只看指标名称。

## 6. 如何用这些边界排查问题

| 现象 | 应分别核对 |
| --- | --- |
| Skill 已加载，但脚本没执行 | 正文指令、Agent 决策和工具调用是否发生 |
| 脚本被拒绝 | 实际读写目标、网络行为、执行环境权限及审批 |
| 有使用记录但任务失败 | 记录发生时刻、审批结果、进程结果及任务验收 |
| 无使用记录但似乎使用了 Skill | 显式或隐式路径、检测器覆盖范围、扩展状态 |
| 审查提到了可信 Skill | 来源校验具体范围及审查用途，不能据此推断沙箱豁免 |

接下来沿主链学习发现与身份：哪些位置会扫描、如何确定一个 Skill、同名与同路径为什么要分别处理。尚未进入 CodebaseAgent 转型课程；该课程仍在 Skills 接入实践之后。
