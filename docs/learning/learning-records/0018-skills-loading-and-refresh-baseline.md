# Skills 专题理解变化与实践交接

本记录覆盖本轮 Skills 学习及整体回顾，起于 2026-10-23 的加载与刷新追问。它保存理解证据和教学用途；架构事实由对应课程承载，当前下一步由 [MAINTENANCE](../MAINTENANCE.md) 维护。

## 理解变化及证据

| 主题 | 用户反馈或行为 | 后续讲解依据 |
| --- | --- | --- |
| 目录、正文与刷新 | 围绕 `$agent-docs` 追问输入解析、正文及长度限制；实际观察到 Thread 内增删 Skill 后候选变化；继续询问是否通过哈希判断 | 区分候选列表、Turn 元信息快照和已读正文；解释事件筛选、路径匹配、节流与缓存失效后反馈理解，见[0025](../lessons/0025-skills-catalog-loading-and-refresh.md) |
| 脚本与使用记录 | 追问 Skill 脚本相对其他工作区外脚本的权限差异，以及为何识别隐式使用；解释后反馈理解并要求保存 | 权限不因 Skill 身份扩大，使用事件不保证实际执行成功，见[0026](../lessons/0026-skills-package-execution-and-usage-tracking.md) |
| 发现范围和身份 | 通过 Project/module 例子追问；区分扫描根内递归与 cwd 祖先链后，复述主要服务于以 module 为工作区的情况；反馈理解结构化输入携带路径 | 不重讲已澄清的递归范围；名称展示与实际绑定分开，见[0027](../lessons/0027-skills-discovery-and-identity.md) |
| 上下文与协议 | 对 world state、增量工具和 Responses Lite 的思路反馈理解，但认为关联多、难记；主动要求带注释的准确多轮请求结构 | 用完整小场景连接清单、正文、工具调用和结果；不把符号记忆当作理解要求，见[0028](../lessons/0028-skills-context-updates-and-incremental-tools.md) |
| 设计与接入 | 设计、接入、子 Agent 和多来源讲解后确认继续，随后启动专题整体回顾并明确反馈“通过” | [0029](../lessons/0029-skill-design-and-validation.md)、[0030](../lessons/0030-skills-integration-contracts.md)与[整体回顾](../reviews/0003-skills-end-to-end-review.md)用于实践映射 |

证据主要是实际观察、机制追问、局部场景复述和理解反馈。整体回顾通过不等于独立实现、测试验收或长期熟练；本轮未以 CodebaseAgent 实践证明这些能力已经接入。

## 教学与实践交接

用户要求全面理解 Skills，并在机制较多时先整理、继续课程，最后整体回顾。后续复习沿发现、选择、上下文、执行四条主线组织，不连续增加记忆测验。避免再用“加载”“片段”“扩展”笼统代替具体状态和来源。

整体回顾通过后，用户要求准备实践任务书，完整记录主循环与 Codex 细节增强等已学内容，但不在任务书中约定实现范围。范围由用户在 CodebaseAgent 工程按实际场景确定。[实践 0007](../practice/0007-skills-integration-lab.md)承接这一要求；不套用旧实践的 MVP、必做项或延期模板。

既定顺序是 Skills 实践完成后才在本仓库开始 CodebaseAgent 转型课程，学完再准备转型任务书并进入独立工程实践。当前记录不能被解释为转型实施授权。
