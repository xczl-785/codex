# Codex 源码学习分支

这是一个用于静态阅读、理解和记录 Codex 源码架构的学习分支。这里的首要目标不是参与上游贡献，而是沿着真实、仍可构建运行的工程，逐步理解 Agent 的运行机制。

学习内容以当前分支源码和测试为事实来源。讲解会先回答“它解决什么问题、为什么这样设计、不这样拆会怎样”，再逐步进入 crate、模块、类型和函数。

## 从这里开始

1. 阅读 [学习任务](docs/learning/MISSION.md)，了解目标、约束和不在当前范围内的内容。
2. 阅读 [Codex 工程介绍](docs/learning/repository-inventory.md)，建立目录、模块和主运行链路的整体地图。
3. 进入 [源码学习课程](docs/learning/README.md)，按问题顺序阅读详细课程。
4. 需要规划后续方向时查看 [学习路线图](docs/learning/roadmap.md)。
5. 遇到概念混淆时查阅 [核心概念速查](docs/learning/reference/glossary.md)。

## 学习原则

- 以静态阅读为主，本机不要求编译或运行整个仓库。
- 保留源码、测试、构建文件、代码生成和平台支持，使学习分支继续具备可运行基础。
- 将“当前不学习”与“仓库不需要”分开判断，不为减少视觉噪声而破坏工程结构。
- 对话是主要学习载体，`docs/learning/lessons/` 保存可独立回顾的详细解释，而不是聊天内容的压缩摘要。
- Thread、Session、Turn、Task、模型上下文、工具调用和 Subagent 是重要主线，但不是全部范围。

## 文档地图

| 需要 | 入口 |
| --- | --- |
| 按顺序学习 | [课程目录](docs/learning/README.md) |
| 了解整体工程 | [工程介绍](docs/learning/repository-inventory.md) |
| 查看完整主题范围 | [学习路线图](docs/learning/roadmap.md) |
| 快速查概念 | [核心概念速查](docs/learning/reference/glossary.md) |
| 查找源码与外部资料 | [资料索引](docs/learning/RESOURCES.md) |
| 查看上游产品、安装与使用说明 | [上游 README](README.codex-upstream.md) |
| 构建当前源码 | [安装与构建](docs/install.md) |
| 查看配置能力 | [配置说明](docs/config.md) |

`docs/learning/MAINTENANCE.md`、`NOTES.md` 和 `learning-records/` 服务于教学过程维护，不属于首次阅读路线。

## 上游资料与工程约束

原 Codex 项目 README 已保留为 [README.codex-upstream.md](README.codex-upstream.md)。原始源码开发规范已保留为 [AGENTS.codex-development.md](AGENTS.codex-development.md)。

如果任务从学习和讲解转为修改或评审源码，必须先阅读根目录 [AGENTS.md](AGENTS.md)，并按其中的开发模式要求加载完整上游规范及目标目录的局部规则。

本仓库继续遵循 [Apache-2.0 License](LICENSE)，第三方归属信息见 [NOTICE](NOTICE)。
