---
kind: prompt-candidate
cycle: C28-prompt-stack-quality-redesign
stone: S5
surface: root-agents
status: proposed
updated: 2026-08-10T22:24:25+08:00
---

# Root AGENTS Candidate

## English target

```markdown
# Repository instructions

## Scope and precedence

These instructions apply repository-wide. Before changing a file, read every applicable `AGENTS.md` from the repository root through that file's directory. The initial prompt only guarantees instructions on the session-cwd ancestry, so discover deeper files explicitly for targets below the cwd. Repeat this for every disjoint target path.

More-specific `AGENTS.md` files override broader repository rules within their directory tree. Direct system, developer, and user instructions take precedence. If applicable rules conflict and precedence does not resolve them, explain the conflict before mutating files.

## Workspace safety

- Preserve existing and unexpected working-tree changes. Do not revert, overwrite, or discard work whose ownership is unknown.
- Work with overlapping changes carefully and leave unrelated changes alone. Ask only when safe integration is materially ambiguous.
- Do not use destructive Git or filesystem operations unless the user explicitly authorizes the named operation and scope.
- Keep changes focused on the requested behavior. Avoid unrelated refactors, generated churn, and metadata changes.
- Use non-interactive Git commands when practical. Do not commit, push, publish, install, deploy, or restart services unless the request authorizes that action.

## Repository-wide engineering rules

- Support Linux, macOS, and Windows unless a feature is explicitly platform-specific. Connected app-server and exec-server environments may use different operating systems.
- Never add or modify behavior involving `CODEX_SANDBOX_NETWORK_DISABLED_ENV_VAR` or `CODEX_SANDBOX_ENV_VAR`.
- Preserve public/private ownership boundaries. Prefer private modules and explicitly exported crate APIs.
- Do not add general product or user documentation under `docs/`; official Codex documentation is maintained elsewhere. App-server API documentation follows its scoped guidance.
- When Rust dependencies change, run `just bazel-lock-update` and include `MODULE.bazel.lock`. When `ConfigToml` or nested config types change, run `just write-config-schema`.
- Build-time source reads such as `include_str!`, `include_bytes!`, `sqlx::migrate!`, or similar macros must also be declared in the crate's Bazel data.
- Keep modules maintainable: prefer a new focused module over growing a high-touch or already-large module. Move related tests and docs with extracted ownership.
- Be patient with Rust commands and lock contention. Do not kill a Rust process by PID merely because it is waiting for a build lock.

## Domain routing

- Any change under `codex-rs/` requires `codex-rs/AGENTS.md`.
- TUI changes also require `codex-rs/tui/AGENTS.md`; bottom-pane state-machine changes additionally require `codex-rs/tui/src/bottom_pane/AGENTS.md`.
- App-server changes require `codex-rs/app-server/AGENTS.md`.
- App-server protocol changes require `codex-rs/app-server-protocol/AGENTS.md`.
- Python changes require the nearest applicable Python-root `AGENTS.md` and the closest `pyproject.toml` runtime contract.
- Work under `.pipeline/` follows the active installed Hypo-Workflow capability. Repository AGENTS do not own Workflow lifecycle semantics.

## Validation

- Before running tests, inspect `scripts/vsp_test_groups.json` or use `just test-groups`, then select the smallest behavior group that protects the changed contract.
- Do not use unfiltered workspace-wide test commands as routine validation. A selected failing group becomes part of the current work until the failure is fixed or proven unrelated with a focused reproduction.
- Report the major logic changes, user-facing behaviors, selected behavior groups, and the contract established by each result.
```

English injected body size is validated separately and must remain at or below 4,000 characters.

## 中文对照

```markdown
# 仓库指令

## 作用域与优先级

本指令适用于整个仓库。修改文件前，读取从仓库根到该文件目录之间所有适用的 `AGENTS.md`。初始提示只保证包含 session cwd 祖先路径上的规则，因此 cwd 以下的目标必须显式发现更深层规则。多个互不相同的目标路径需要分别执行这一过程。

更具体的 `AGENTS.md` 在其目录树内覆盖更宽泛的仓库规则。直接 system、developer 和 user 指令优先。如果适用规则冲突且优先级无法解决，在修改文件前说明冲突。

## 工作区安全

- 保留现有和意外的工作树改动，不还原、覆盖或丢弃所有权未知的工作。
- 谨慎整合重叠改动，忽略无关改动；只有安全整合存在实质歧义时才询问。
- 未经用户明确授权具体操作与范围，不执行破坏性 Git 或文件系统操作。
- 改动聚焦于请求行为，避免无关重构、生成噪声和元数据变化。
- 尽量使用非交互 Git。除非请求已授权，不 commit、push、publish、install、deploy 或重启服务。

## 全仓工程规则

- 除非功能明确限定平台，支持 Linux、macOS 和 Windows；app-server 与 exec-server 可能运行在不同操作系统。
- 绝不新增或修改涉及 `CODEX_SANDBOX_NETWORK_DISABLED_ENV_VAR` 或 `CODEX_SANDBOX_ENV_VAR` 的行为。
- 保持公私有 ownership 边界，优先私有模块与显式导出的 crate API。
- 不在 `docs/` 新增通用产品或用户文档；官方 Codex 文档在其他位置维护。app-server API 文档遵循其局部规则。
- Rust 依赖变化运行 `just bazel-lock-update` 并包含 `MODULE.bazel.lock`；`ConfigToml` 或嵌套配置类型变化运行 `just write-config-schema`。
- `include_str!`、`include_bytes!`、`sqlx::migrate!` 等构建期源码读取也必须声明在 crate 的 Bazel data 中。
- 保持模块可维护；高频或已很大的模块优先拆出聚焦模块，并把相关测试与文档移动到新的 ownership。
- Rust 命令和锁竞争需要耐心，不能因为等待 build lock 就按 PID 杀进程。

## 领域路由

- `codex-rs/` 下的改动必须读取 `codex-rs/AGENTS.md`。
- TUI 还需读取 `codex-rs/tui/AGENTS.md`；bottom-pane 状态机还需读取现有 leaf AGENTS。
- app-server 与 app-server-protocol 分别读取对应目录的 AGENTS。
- Python 改动读取最近的 Python-root AGENTS 和最近 `pyproject.toml` 的运行时合同。
- `.pipeline/` 工作遵循当前安装的 Hypo-Workflow capability；仓库 AGENTS 不拥有 Workflow 生命周期语义。

## 验证

- 测试前读取 `scripts/vsp_test_groups.json` 或使用 `just test-groups`，选择保护本次行为的最小 group。
- 日常验证不运行无过滤的 workspace 全量测试。已选择 group 的失败由本次工作负责，直到修复或用最小复现证明无关。
- 报告主要逻辑变化、用户可见行为、选择的 behavior groups 和每个结果证明的合同。
```
