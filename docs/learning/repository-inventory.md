# Codex 工程介绍

本文从源码工程角度介绍 Codex 仓库，帮助建立目录、模块、运行链路和构建体系的整体认识。更具体的阅读顺序和专题入口见 [源码学习指南](README.md)。

## 工程定位

Codex CLI 是一个以 Rust 为核心实现的本地编码 Agent。它同时提供交互式终端、非交互执行、App Server、MCP、SDK 和多 Agent 协作等入口，但这些入口最终共享 Thread、Turn、模型交互、工具执行、审批、沙箱和持久化等基础能力。

本学习分支保留以下基础：

- 能够沿用 Rust、Cargo 和 `just` 的构建与验证方式。
- 保留功能源码、依赖锁文件、代码生成、测试和跨平台实现。
- 保留许可证与第三方归属信息。
- 移除不影响功能的上游贡献、宣传、变更记录和单行跳转文档。

## 根目录结构

| 目录             | 职责                                                                    |
| ---------------- | ----------------------------------------------------------------------- |
| `codex-rs/`      | Rust workspace，包含 CLI、核心 Agent、TUI、协议、服务端、沙箱和工具实现 |
| `codex-cli/`     | npm 分发包装和安装脚本                                                  |
| `sdk/`           | Python、Python Runtime 和 TypeScript SDK                                |
| `docs/`          | 本地构建说明、配置补充和源码学习资料                                    |
| `scripts/`       | 格式化、安装、打包、代码生成和仓库级检查脚本                            |
| `tools/`         | 仓库自有 lint 与开发工具                                                |
| `bazel/`         | Bazel 自定义规则                                                        |
| `patches/`       | Rust、V8、Bazel 和跨平台构建补丁                                        |
| `third_party/`   | V8、PowerShell、Wine、WezTerm 等第三方构建材料                          |
| `.github/`       | CI、发布和 GitHub 仓库自动化                                            |
| `.codex/`        | 本仓库使用的 Codex 环境、评审和测试 Skills                              |
| `.devcontainer/` | Dev Container 开发环境                                                  |
| `.vscode/`       | VS Code 工作区配置                                                      |
| `.codegraph/`    | 本地 CodeGraph 索引；数据库不提交到 Git                                 |

## 根目录配置

### 工程入口

- `README.md`：当前学习分支的人类入口，说明学习目标并路由课程、工程介绍和上游资料。
- `README.codex-upstream.md`：保留的上游产品、安装与使用说明。
- `AGENTS.md`：学习工作区的 Agent 入口，定义静态阅读、讲解和文档沉淀方式。
- `AGENTS.codex-development.md`：完整保留的上游源码开发、编码、评审和验证约束；源码任务必须主动加载。
- `justfile`：格式化、lint、测试和代码生成的统一命令入口。
- `LICENSE`、`NOTICE`：源码许可和第三方归属。

### Rust 与 Bazel

- Rust workspace 位于 `codex-rs/Cargo.toml`，依赖锁定在 `codex-rs/Cargo.lock`。
- `BUILD.bazel`、`MODULE.bazel`、`MODULE.bazel.lock`、`defs.bzl` 和 `rbe.bzl` 构成仓库级 Bazel 入口。
- `.bazelrc`、`.bazelversion` 和 `.bazelignore` 控制 Bazel 环境。
- `workspace_root_test_launcher.*.tpl` 为跨平台 Bazel 测试提供启动模板。

### Node 与文档工具

- `package.json` 和 `pnpm-lock.yaml` 固定仓库维护工具。
- `pnpm-workspace.yaml` 连接 npm wrapper 和 TypeScript SDK。
- `.prettierrc.toml`、`.prettierignore` 和 `.markdownlint-cli2.yaml` 负责文档及脚本格式。
- `.codespellrc` 和 `.codespellignore` 负责拼写检查。

### 可选开发环境

- `flake.nix` 和 `flake.lock` 提供 Nix 环境。
- `.devcontainer/` 和 `.vscode/` 提供编辑器与容器开发体验。
- `.worktreeinclude` 指定创建工作树时需要额外带入的本地配置。

这些配置不都属于运行时，但它们分别支撑构建、验证、代码生成或某个平台。未经验证，不应因为暂时不阅读就直接删除。

## Rust workspace 分层

`codex-rs/` 内 crate 数量较多，可以先按职责分层，而不是逐个阅读。

### 产品入口

- `cli/`：统一命令行入口和子命令分发。
- `tui/`：交互式终端 UI、输入处理、事件循环和状态展示。
- `exec/`：非交互执行模式。
- `app-server/`：面向 IDE、桌面端和 SDK 的服务端入口。
- `mcp-server/`：将 Codex 能力作为 MCP Server 暴露。

### Agent 核心

- `core/`：Thread、Session、Turn、上下文、模型循环、工具和多 Agent 编排。
- `protocol/`：核心操作与事件类型，是模块间通信的基础协议。
- `thread-store/`：Thread 和 Turn 的持久化模型与存储。
- `state/`、`rollout/`、`history/`：运行状态、会话记录和历史数据。
- `context-fragments/`：可注入模型上下文的结构化片段。

### 模型与外部连接

- `model-provider/`、`model-provider-info/`、`models-manager/`：模型提供方、模型元数据和模型选择。
- `backend-client/`、`codex-client/`、`codex-api/`：后端和 API 客户端边界。
- `codex-mcp/`、`rmcp-client/`：MCP 连接、工具和资源访问。
- `connectors/`、`plugin/`、`core-plugins/`、`skills/`：连接器、插件和 Skills 能力。

### 工具、执行与安全

- `tools/`：模型可调用工具的公共类型和配置。
- `exec-server/`、`exec-server-protocol/`：独立命令执行服务及协议。
- `execpolicy/`：命令执行策略匹配。
- `shell-command/`、`shell-escalation/`：Shell 命令表示和权限提升流程。
- `sandboxing/`、`linux-sandbox/`、`windows-sandbox-rs/`、`bwrap/`：通用及平台沙箱实现。
- `apply-patch/`、`file-system/`、`file-search/`：文件修改和检索能力。

### 协议与客户端边界

- `app-server-protocol/`：App Server JSON-RPC 类型、v2 API 和生成的 TypeScript schema。
- `app-server-client/`、`app-server-test-client/`：客户端实现和测试客户端。
- `app-server-transport/`、`app-server-daemon/`：传输层和常驻进程管理。
- `code-mode-*`：Code Mode 协议、运行时和宿主进程。

### 通用基础设施

- `config/`、`features/`、`login/`、`codex-home/`：配置、功能开关、认证和本地目录。
- `http-client/`、`network-proxy/`、`websocket-client/`：网络基础设施。
- `otel/`、`analytics/`、`diagnostics/`、`feedback/`：遥测、分析和诊断。
- `utils/`、`async-utils/`、`git-utils/`、`file-watcher/`：跨模块通用能力。

## 核心运行链路

一次典型交互可以抽象为：

```text
CLI / TUI / App Server / SDK
             |
             v
        ThreadManager
             |
             v
         CodexThread -- submit(Op) --> Session / Turn / Task
                                          |
                         +----------------+----------------+
                         |                                 |
                         v                                 v
                   模型请求与流式响应                 工具路由与执行
                         |                                 |
                         +----------------+----------------+
                                          |
                                          v
                                        Event
                         +----------------+----------------+
                         |                                 |
                         v                                 v
                     TUI / 客户端                     Thread Store
```

建议从这些文件建立第一张调用图：

- `codex-rs/cli/src/main.rs`
- `codex-rs/core/src/thread_manager.rs`
- `codex-rs/core/src/codex_thread.rs`
- `codex-rs/core/src/session/session.rs`
- `codex-rs/core/src/session/turn.rs`
- `codex-rs/core/src/tasks/`
- `codex-rs/core/src/tools/`
- `codex-rs/protocol/src/protocol.rs`
- `codex-rs/tui/src/chatwidget.rs`
- `codex-rs/thread-store/src/lib.rs`

## Subagent 在工程中的位置

Subagent 并不是独立于核心运行时的另一套系统。它复用 Thread、Turn 和工具机制，并在上层增加 Agent 注册、父子关系、通信、并发限制和完成通知。

```text
模型的多 Agent 工具调用
        |
        v
multi_agents_v2 handlers
        |
        v
AgentControl / AgentRegistry
        |
        v
创建或恢复子 CodexThread
        |
        v
消息、状态和完成通知回到父 Agent 上下文
```

主要入口位于：

- `codex-rs/core/src/tools/handlers/multi_agents_v2/`
- `codex-rs/core/src/agent/control.rs`
- `codex-rs/core/src/agent/control/spawn.rs`
- `codex-rs/core/src/agent/registry.rs`
- `codex-rs/core/src/agent_communication.rs`
- `codex-rs/core/src/context/inter_agent_message.rs`
- `codex-rs/core/src/context/inter_agent_completion_message.rs`

## 构建与验证

学习分支优先保留官方的 Rust 开发路径：

```bash
cd codex-rs
cargo build
cargo run --bin codex -- "explain this codebase to me"
```

仓库维护命令从根目录执行：

```bash
just fmt
just fix -p <crate>
just test -p <crate>
```

具体依赖和环境要求见 `docs/install.md`。本仓库约定使用 `just test`，不直接运行 `cargo test`。

## 测试体系

- crate 内单元测试用于局部数据结构和状态机。
- `codex-rs/core/tests/` 与 `core/suite` 覆盖 Agent 行为和模型交互集成场景。
- `codex-rs/app-server/tests/` 通过公开 JSON-RPC API 验证服务行为。
- `codex-rs/tui/` 大量使用 `insta` snapshot 验证可见 UI。
- Bazel 和 CI 补充跨平台、依赖一致性和仓库级规则验证。

## 学习与改动原则

- 先理解边界和事件流，再深入单个实现文件。
- 使用 CodeGraph 跟踪符号、调用者、被调用者和影响范围。
- 将“当前不学习”与“工程不需要”分开判断。
- 清理文件前检查构建引用、运行时资源引用、文档链接和测试夹具。
- 功能改动后按受影响 crate 运行格式化、lint 和测试。
