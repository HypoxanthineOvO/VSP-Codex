# Milestone 3: `vsp-codex` 外部入口与 VSP home

## Execution Boundary

本提示只执行 Milestone 3。目标是把用户实际启动入口切到 `vsp-codex`，并把默认配置目录切到 `~/.vsp-codex`。

不要开始以下实现：

- `/provider` 命令、provider picker 或 Base URL/API Key 管理。
- Cloud、OAuth、ChatGPT app、feedback、update 等官方服务入口隐藏或禁用。
- 全面品牌替换、汉化、美化或高频 UI 文案清理。
- crate/package 全量重命名；内部 crate 名称仍保持 `codex-*` 以降低后续合并成本。

本 Milestone 允许的产品行为：

- 外部二进制入口改为 `vsp-codex`，普通用户不再需要运行 `codex`。
- 默认 home 从 `~/.codex` 改为 `~/.vsp-codex`。
- 保留 `CODEX_HOME` override，作为测试隔离和低成本兼容入口。
- TUI 首次启动时，如果默认 `~/.vsp-codex/config.toml` 不存在而同级 `~/.codex/config.toml` 存在，显示选择界面询问是否导入旧配置目录。

## Subworker Assignment Plan

Worker Separation: `strict`。用户已授权 Codex 在 `/hw:start` 和 `/hw:resume` 中使用执行 Subagent，角色必须分离。

### test

- Role status: authorized.
- Ownership: 先写 M3 回归测试和快照，只允许测试/快照/测试注册改动。
- Allowed write paths:
  - `codex-rs/cli/tests/*.rs`
  - `codex-rs/tui/src/app/tests.rs`
  - `codex-rs/tui/src/app/tests/vsp_home_import.rs`
  - `codex-rs/tui/src/app/snapshots/*.snap`
  - `codex-rs/utils/home-dir/src/lib.rs` 中的 `#[cfg(test)]` 测试段落
  - `.pipeline/reviews/003-vsp-entrypoint-home-test.md`
- Must not edit production behavior, manifests, package wrappers, justfile, Bazel files, install scripts, or non-test Rust code outside the allowed test registration line.
- Expected evidence:
  - CLI tests use `cargo_bin("vsp-codex")` and include a help/version entrypoint assertion.
  - home-dir 默认目录测试期望 `~/.vsp-codex`，`CODEX_HOME` override 仍通过。
  - TUI legacy-home import detection/copy/prompt render 有 focused test 或 snapshot 覆盖。
  - 先记录 red/compile-fail 状态或说明无法单独运行的原因。

### implement

- Role status: authorized.
- Ownership: 只实现 M3 生产/运行时改动，不创建或修改测试断言。
- Allowed write paths:
  - `codex-rs/cli/Cargo.toml`
  - `codex-rs/cli/src/main.rs`
  - `codex-rs/cli/BUILD.bazel`
  - `codex-rs/utils/home-dir/src/lib.rs` 中非 `#[cfg(test)]` 生产段落
  - `codex-rs/tui/src/lib.rs`
  - `codex-rs/tui/src/vsp_home_import.rs`
  - `justfile`
  - `codex-cli/package.json`
  - `codex-cli/bin/codex.js`
  - `codex-cli/scripts/build_npm_package.py`
  - `scripts/debug-codex.sh`
  - `scripts/test-remote-env.sh`
  - `scripts/start-codex-exec.sh`
  - `scripts/run_tui_with_exec_server.sh`
  - `.pipeline/reviews/003-vsp-entrypoint-home-implement.md`
- Must not edit tests, fixtures, snapshots, or `.pipeline/reviews/*-test.md`.
- Expected evidence:
  - Cargo binary target is `vsp-codex`.
  - Clap top-level help/usage uses `vsp-codex`.
  - just recipe supports `just vsp-codex --version`; `exec` recipe uses `vsp-codex`.
  - npm wrapper resolves native `vsp-codex` binary.
  - default home uses `~/.vsp-codex`; `CODEX_HOME` remains honored.
  - TUI import prompt copies from sibling `~/.codex` into `~/.vsp-codex` only when the new config is absent and env override is not set.

### audit

- Role status: authorized.
- Ownership: 只读审查最终 diff、测试证据、CI 配置影响、旧入口残留和 merge 成本。
- Allowed write paths:
  - `.pipeline/reviews/003-vsp-entrypoint-home-audit.md`
- Must not edit source, tests, manifests, scripts, snapshots, or workflow state outside its audit artifact.

### main agent

- 编排 worker 生命周期，集成返回结果，运行验证命令，更新 lifecycle 状态。
- 不得本地承担 test、implement、audit 任一角色；只做必要的集成冲突处理、验证和提交。

## Technical Solution

采用低 merge 成本改造：外部入口和用户 home 改为 VSP 形态，内部 crate 名称和大部分内部 type/module 名称保持 Codex 原状。

首次导入不做自动迁移。只在默认 VSP home 下、旧 `~/.codex/config.toml` 存在且新 `~/.vsp-codex/config.toml` 不存在时，在 TUI 启动早期显示选择界面。用户选择导入后递归复制旧 home 内容到新 home，已存在文件不覆盖；用户跳过则继续使用新 home 默认配置。

## Validation Path

必须运行：

```bash
just fmt
just test -p codex-utils-home-dir
just test -p codex-cli
just test -p codex-tui vsp_home_import
cargo build --bin vsp-codex
./target/debug/vsp-codex --version
just vsp-codex --version
git diff --check
```

若 TUI snapshot 有更新：

```bash
cargo insta pending-snapshots -p codex-tui
cargo insta accept -p codex-tui
```

如果 Bazel target 同步改动，至少运行：

```bash
bazel query //codex-rs/cli:vsp-codex
```

不要在未再次获得用户确认前运行完整 workspace `just test`。

## Audit Focus

- 用户可见入口是否已经是 `vsp-codex`，不是只改标题。
- 是否保留内部 crate 名称，避免无谓 merge 冲突。
- 默认 home 是否不再写 `~/.codex`，同时 `CODEX_HOME` override 仍可隔离测试。
- 首次导入是否不会覆盖 `~/.vsp-codex` 已有文件，不会在 env override 下误弹。
- npm wrapper、just、Cargo、Bazel 的入口名是否一致，脚本旧名是否被解释或修复。
- 是否没有提前做 M4-M6 的 provider、官方服务隐藏或汉化工作。

## Completion Criteria

- M3 入口/home/import 行为已合入当前工作树。
- `.pipeline/reviews/003-vsp-entrypoint-home-test.md`、`003-vsp-entrypoint-home-implement.md`、`003-vsp-entrypoint-home-audit.md` 已生成。
- 必需验证命令通过，或阻塞项被明确记录并由用户确认。
- 当前分支可提交、推送到 GitLab，并检查 GitLab CI 不回退到错误状态。
