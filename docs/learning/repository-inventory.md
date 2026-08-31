# 仓库根目录盘点

这份清单服务于学习分支的降噪工作。最低边界是保留基于 Rust、Cargo 和 `just` 的构建、运行及基础验证能力。当前只做分类，不执行删除。

## 目录

| 目录             | 作用                                               | 当前建议                                         |
| ---------------- | -------------------------------------------------- | ------------------------------------------------ |
| `.codegraph/`    | 本地源码索引；数据库已被目录内的 `.gitignore` 忽略 | 保留 `.gitignore`，数据库只留本地                |
| `.codex/`        | 仓库开发环境及代码评审、PR、测试相关 Skills        | 暂时保留；不影响运行，后续可单独精简             |
| `.devcontainer/` | 容器化开发环境                                     | 可选；不使用 Dev Container 时可清理              |
| `.github/`       | CI、发布、依赖更新、Issue、CLA 和仓库治理          | 需要逐项拆分；保留基础 CI，治理和发布内容可清理  |
| `.vscode/`       | VS Code 开发配置                                   | 可选；不使用对应配置时可清理                     |
| `bazel/`         | Bazel 自定义规则                                   | 保留到确认只采用 Cargo 构建之后                  |
| `codex-cli/`     | npm 分发包装与安装脚本                             | 功能相关；运行源码不一定需要，但不是杂项         |
| `codex-rs/`      | Rust workspace 和 Codex 主实现                     | 必须保留                                         |
| `docs/`          | 本地文档、上游治理资料和学习资料                   | 重组；技术文档迁入学习体系，治理和宣传资料可清理 |
| `patches/`       | Bazel、V8、Rust 和跨平台构建补丁                   | 功能相关；保留到构建链审计完成                   |
| `scripts/`       | 格式化、打包、安装、测试和代码生成脚本             | 功能相关；后续逐文件审计，不整目录删除           |
| `sdk/`           | Python、Python Runtime 和 TypeScript SDK           | 独立产品接口；当前保留，后续按学习范围决定       |
| `third_party/`   | V8、PowerShell、Wine、WezTerm 等第三方构建材料     | 功能相关；与 Bazel 和跨平台能力绑定              |
| `tools/`         | 仓库自有开发和 lint 工具                           | 基础验证相关，建议保留                           |

## 根目录文件

### 源码学习与仓库基础

| 文件               | 作用                                 | 当前建议                     |
| ------------------ | ------------------------------------ | ---------------------------- |
| `README.md`        | 项目入口、安装方式和重要链接         | 保留，后续可改成学习分支入口 |
| `AGENTS.md`        | 本仓库开发和验证约束                 | 保留                         |
| `justfile`         | 格式化、lint、测试和代码生成统一入口 | 必须保留                     |
| `.gitignore`       | 忽略构建和本地状态                   | 必须保留                     |
| `.gitattributes`   | Git 属性与跨平台文本行为             | 保留                         |
| `.worktreeinclude` | 创建工作树时额外包含 `user.bazelrc`  | 使用相关工具链时保留         |

### 法律文件

| 文件      | 作用                  | 当前建议                               |
| --------- | --------------------- | -------------------------------------- |
| `LICENSE` | Apache-2.0 许可证正文 | 保留；学习 fork 仍在复制和修改上游源码 |
| `NOTICE`  | 上游版权和归属声明    | 保留；与许可证合规一起看待             |

版权文件对理解代码帮助不大，但它们承担的是复制、修改和再分发时的许可与归属，不应作为普通噪声删除。

### Bazel 与远程构建

| 文件                                   | 作用                            | 当前建议                        |
| -------------------------------------- | ------------------------------- | ------------------------------- |
| `.bazelignore`                         | Bazel 扫描排除项                | 随 Bazel 构建链保留             |
| `.bazelrc`                             | Bazel 构建选项                  | 随 Bazel 构建链保留             |
| `.bazelversion`                        | 固定 Bazel 版本                 | 随 Bazel 构建链保留             |
| `BUILD.bazel`                          | 根构建目标、平台和工具链定义    | 随 Bazel 构建链保留             |
| `MODULE.bazel`                         | Bazel 模块和依赖声明            | 随 Bazel 构建链保留             |
| `MODULE.bazel.lock`                    | Bazel 依赖锁文件                | 随 Bazel 构建链保留             |
| `defs.bzl`                             | 仓库级 Bazel 定义               | 随 Bazel 构建链保留             |
| `rbe.bzl`                              | Remote Build Execution 平台定义 | 本地 Cargo 路径不需要；暂缓清理 |
| `workspace_root_test_launcher.bat.tpl` | Windows Bazel 测试启动模板      | 随 Bazel 测试链保留             |
| `workspace_root_test_launcher.sh.tpl`  | Unix Bazel 测试启动模板         | 随 Bazel 测试链保留             |

这些文件不是阅读主线，但与官方构建和 CI 绑定。第一批清理不碰它们，后续可以在验证 Cargo/`just` 路径后，将 Bazel 支持作为一个整体决策。

### Nix 构建

| 文件         | 作用               | 当前建议              |
| ------------ | ------------------ | --------------------- |
| `flake.nix`  | Nix 开发和构建定义 | 不使用 Nix 时可清理   |
| `flake.lock` | Nix 依赖锁文件     | 与 `flake.nix` 同进退 |

### Node、格式化与文档工具

| 文件                      | 作用                                    | 当前建议                     |
| ------------------------- | --------------------------------------- | ---------------------------- |
| `package.json`            | 根目录维护脚本和 Prettier 依赖          | 保留基础验证能力时保留       |
| `pnpm-lock.yaml`          | Node 依赖锁文件                         | 与 `package.json` 同进退     |
| `pnpm-workspace.yaml`     | npm wrapper 和 TypeScript SDK workspace | 功能相关，保留               |
| `.npmrc`                  | pnpm/npm 行为配置                       | 与 Node workspace 同进退     |
| `.prettierrc.toml`        | Prettier 格式配置                       | 保留文档和脚本格式验证时保留 |
| `.prettierignore`         | Prettier 排除规则                       | 与 Prettier 配置同进退       |
| `.markdownlint-cli2.yaml` | Markdown lint 配置                      | 保留文档验证时保留           |
| `.codespellrc`            | 拼写检查配置                            | 基础验证相关，建议保留       |
| `.codespellignore`        | 拼写检查例外                            | 与 `.codespellrc` 同进退     |

### 第一批可清理信息

| 文件                    | 原因                                                             | 清理注意事项                                |
| ----------------------- | ---------------------------------------------------------------- | ------------------------------------------- |
| `CHANGELOG.md`          | 只有 GitHub Releases 跳转，对源码学习没有本地信息                | execpolicy 示例引用了它，删除时同步调整示例 |
| `SECURITY.md`           | 上游漏洞报告流程，不参与构建或运行                               | `docs/contributing.md` 引用了它             |
| `announcement_tip.toml` | 上游通过 GitHub 托管的公告内容；本地代码读取的是 OpenAI 上游 URL | 不影响当前 fork 的本地运行，可删除          |

## `docs/` 第一批整理建议

### 可直接列入清理计划

- `docs/CLA.md`：贡献者许可协议。
- `docs/contributing.md`：上游贡献、Issue 和社区流程。
- `docs/open-source-fund.md`：资助项目宣传。

删除前应同步移除 `README.md`、Issue 模板和 CLA workflow 中的链接。

### 先合并链接，再删除跳转文件

以下文件大多只有指向官方文档的链接。技术主题仍有参考价值，但没有必要各占一个文件。建议将仍需的链接集中到 `docs/learning/README.md` 的参考资料区，再删除原跳转文件：

- `docs/agents_md.md`
- `docs/authentication.md`
- `docs/example-config.md`
- `docs/exec.md`
- `docs/execpolicy.md`
- `docs/getting-started.md`
- `docs/license.md`
- `docs/sandbox.md`
- `docs/skills.md`
- `docs/slash_commands.md`

### 当前保留

- `docs/install.md`：本地构建、运行和测试入口。
- `docs/config.md`：包含链接之外的生命周期 Hook 配置说明。
- `docs/learning/`：学习导航、盘点和后续笔记。

## 推荐清理顺序

1. 删除上游贡献、宣传、变更和安全流程文档，并修复对应链接。
2. 将有价值的技术文档链接集中到学习 README，再删除单行跳转文档。
3. 审计 `.github/`，保留基础构建测试工作流，删除 CLA、Issue、发布和上游治理配置。
4. 根据本机工作方式决定是否删除 `.devcontainer/`、`.vscode/` 和 Nix 文件。
5. 最后才评估 Bazel、SDK、npm wrapper 和跨平台构建材料；每次删除后验证 Cargo/`just` 构建与关键测试。
