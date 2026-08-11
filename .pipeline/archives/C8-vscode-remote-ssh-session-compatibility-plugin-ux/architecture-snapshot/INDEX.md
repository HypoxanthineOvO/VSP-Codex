# Codex-VSP 架构基线

初始化时间：2026-05-24T18:35:13+08:00

## 项目概览

Codex-VSP 是 OpenAI Codex CLI 的本地源码仓库副本，主体是 `codex-rs/` 下的 Rust Cargo workspace，外层包含 npm 分发包装、SDK、安装脚本、Bazel 配置、CI 配置和文档。项目目标是提供本地运行的编码 Agent，包括交互式 TUI、非交互执行、代码审查、MCP 客户端/服务端、app-server JSON-RPC 接口、沙箱执行、插件/技能、远程环境和 Codex Cloud 任务能力。

## 技术栈

- Rust 2024 edition：核心业务、CLI、TUI、app-server、协议、沙箱、MCP、SDK runtime。
- Cargo workspace：`codex-rs/Cargo.toml` 管理约 87 个 workspace crates。
- Bazel：根目录 `MODULE.bazel`、`BUILD.bazel` 和 `.bazelversion` 提供 CI/release 构建路径，Bazel 版本为 9.0.0。
- Node/pnpm：根目录 `package.json` 用于维护脚本，`codex-cli/` 是 npm 包 `@openai/codex` 的 JS 启动包装。
- TypeScript/Python SDK：`sdk/typescript/` 和 `sdk/python/` 面向外部程序嵌入 Codex。
- just：根目录 `justfile` 是推荐开发命令入口，默认工作目录切到 `codex-rs`。

## 顶层结构

- `codex-rs/`：Rust workspace 主体，包含 `core`、`cli`、`tui`、`exec`、`app-server`、`protocol`、`codex-mcp`、`mcp-server`、`exec-server`、`sandboxing`、`skills`、`plugin`、`state`、`thread-store` 等 crates。
- `codex-cli/`：npm 包入口，`bin/codex.js` 根据平台定位并启动原生 `codex` 二进制。
- `sdk/`：TypeScript SDK 和 Python SDK，面向应用集成和自动化脚本。
- `docs/`：用户安装、配置、认证、贡献和执行策略文档。
- `scripts/`：发布打包、安装、Bazel lock 检查、远程测试辅助脚本。
- `.github/workflows/`：Rust、Bazel、SDK、release、V8 canary 等 CI workflow。
- `patches/`、`third_party/`、`tools/`：依赖补丁、第三方资源和工程工具。

## 架构文档索引

- `features.md`：按底层服务、接口/API、用户功能、外部功能拆分的功能列表。
- `code-architecture.md`：crate 分层、模块边界、核心执行链路。
- `data-flow.md`：交互式 TUI、非交互 exec、app-server、MCP、exec-server、插件/技能的数据流。
- `build-and-run.md`：编译、测试、实际启动二进制和环境注意事项。
- `decisions.md`：初始化期间形成的架构判断、约束和风险记录。
- `c2-foundation-experience.md`：C2 基础体验优化的 Milestone 架构计划、范围边界和验证基线。
- `c3-ask-user-tools-plan-panel.md`：C3 Ask User Tools 与 Plan/SubPlan 常驻窗口的架构计划、决策和验证基线。
- `c4-preview-release-experience.md`：C4 体验优化与预览版发布闭环的架构计划、范围边界和顺延项。
- `c7-experience-optimization-bugfix-sweep.md`：C7 体验优化与 Bugfix Sweep 的架构方向、Milestone map、结构化交互卡片类型和验证约束。
- `c8-vscode-remote-ssh-session-compatibility-plugin-ux.md`：C8 Remote SSH VSCode 插件兼容、Session discovery、Request Tool/Input 和 Markdown Workflow surface 的架构方向。

## 已验证事实

- 当前目录是 Git worktree。
- 当前没有预先存在的 `.pipeline/`，本次按已有大型项目初始化。
- 已补齐 Rust 1.93.0 pinned toolchain 的 `clippy`、`rustfmt`、`rust-src` 等组件。
- 已安装并验证 `just 1.51.0` 与 `cargo-nextest 0.9.136`。
- `cargo metadata --no-deps --format-version=1` 已在 `codex-rs/` 下成功运行。
- 已重新构建并运行 debug 二进制 `codex-rs/target/debug/codex`，`--version` 输出 `codex-cli 0.0.0`，`--help` 可正常显示命令。
- `just codex --version` 可从仓库根目录运行；注意 root `justfile` 会自动切到 `codex-rs/`。
- 本地 Codex 状态库启动故障已通过备份旧 `~/.codex/state_5.sqlite*` 并让 Codex 重建状态库修复；`codex doctor --summary` 结果为 `12 ok · 1 idle · 3 notes · 1 warn · 0 fail degraded`。
- Volta 已安装 Node 22.22.3，当前 Codex 会话因 `_VOLTA_TOOL_RECURSION=1` 和 PATH 中的 Node 20 image，普通 `node` 仍显示 `v20.20.0`；需要 Node 22 的仓库脚本可用 `volta run --node 22 ...` 运行。

## 关键证据文件

- `README.md`
- `docs/install.md`
- `docs/config.md`
- `docs/authentication.md`
- `codex-rs/README.md`
- `codex-rs/Cargo.toml`
- `codex-rs/core/README.md`
- `codex-rs/cli/src/main.rs`
- `codex-rs/cli/Cargo.toml`
- `codex-rs/app-server/README.md`
- `codex-rs/app-server-protocol/src/protocol/v2/`
- `codex-rs/exec-server/README.md`
- `codex-rs/codex-mcp/src/lib.rs`
- `codex-cli/package.json`
- `codex-cli/bin/codex.js`
- `sdk/typescript/README.md`
- `sdk/python/README.md`
