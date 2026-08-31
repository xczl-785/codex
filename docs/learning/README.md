# Codex 源码学习指南

这个目录用于记录本 fork 的源码学习材料，不作为 Codex 的用户文档或上游项目文档。

## 学习范围

第一阶段聚焦 `codex-rs` 中一条完整的本地 Agent 运行链路：命令行解析、会话启动、模型交互、工具调用、审批与沙箱，以及 TUI 如何展示运行状态。SDK、发布流程、CI、跨平台打包和实验性服务先作为扩展主题保留，不作为入门主线。

## 推荐阅读顺序

1. 阅读根目录 `README.md` 和 `docs/install.md`，了解产品定位与本地构建入口。
2. 阅读 `codex-rs/Cargo.toml`，建立 workspace 和 crate 边界的整体认识。
3. 从 `codex-rs/cli/src/main.rs` 开始，跟踪默认 TUI、`exec`、登录和服务类子命令的分发。
4. 进入 `codex-rs/tui/src/lib.rs` 与 `codex-rs/tui/src/app.rs`，理解交互界面和应用事件循环。
5. 阅读 `codex-rs/core/src/codex_thread.rs`、`codex-rs/core/src/session/` 和 `codex-rs/core/src/tasks/`，理解线程、会话、轮次和任务生命周期。
6. 阅读 `codex-rs/core/src/tools/`、`codex-rs/core/src/exec.rs` 和 `codex-rs/core/src/sandboxing/`，理解工具路由、命令执行、审批与隔离。
7. 阅读 `codex-rs/protocol/src/protocol.rs`，理解核心模块与 TUI、App Server 等调用方之间的事件协议。
8. 最后按兴趣扩展到 `codex-rs/exec`、`codex-rs/app-server`、`codex-rs/codex-mcp`、`sdk` 和各平台沙箱实现。

## 使用 CodeGraph

仓库已经在本地建立 CodeGraph 索引。索引数据库位于 `.codegraph/codegraph.db`，它体积较大且可重新生成，因此不会提交到 Git。

常用命令：

```powershell
# 检查索引状态
codegraph status .

# 源码变化后增量同步
codegraph sync .

# 围绕问题查找相关源码和调用路径
codegraph explore "CLI 如何启动 TUI"

# 查看符号及其调用关系
codegraph node CodexThread
codegraph callers CodexThread
codegraph callees CodexThread
```

如果换了一台机器或删除了本地索引，可在仓库根目录重新执行：

```powershell
codegraph init .
```

## 学习笔记约定

- 每份笔记尽量围绕一个可验证的问题，而不是逐文件复述代码。
- 记录关键入口、核心类型、调用路径、状态变化和对应测试。
- 源码发生变化后先运行 `codegraph sync .`，再复核笔记中的路径和结论。
- 将“当前不学习”和“可以从仓库删除”分开判断。构建、测试、代码生成和平台支持文件可能不在学习主线上，但仍可能是源码正确运行所必需的。

## 后续主题

- CLI 到 TUI 的启动与退出链路
- 一次用户输入的完整生命周期
- 模型上下文的构建、压缩与持久化
- 工具注册、路由、执行和结果回传
- 命令审批、执行策略与沙箱边界
- MCP、Skills、Plugins 和 Apps 的接入方式
- App Server 协议与 SDK 的职责边界
