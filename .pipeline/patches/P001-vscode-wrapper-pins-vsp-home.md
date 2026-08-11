# P001: VSCode wrapper pins VSP home
- 严重级: normal
- 状态: closed
- 发现于: Patch
- 创建时间: 2026-06-07 17:14 CST
- 修复时间: 2026-06-07 17:15 CST
- 改动: codex-rs/cli/src/vscode_cmd.rs — VSCode install now writes a persistent wrapper that exports CODEX_HOME=$HOME/.vsp-codex before execing the real VSP CLI.
- 测试: ok — codex-cli VSCode integration tests 14/14 passed; focused wrapper regression passed; fmt and diff checks passed.
- 关联: vscode/empty_model_provider_resume
- resolved_by: patch
- related: []
- supersedes: []
- worker_separation: test=019ea153-5646-7173-84f9-ba7822329fea, implement=019ea156-30f7-7ba1-8721-84eeba394ee2, audit=019ea159-7396-7fc2-bb7c-04f61a67110a
- worker_lifecycle: test=requested/started/completed/closed, implement=requested/started/completed/closed, audit=requested/started/completed/closed
- commit: enclosing `fix(P001): pin VSCode launch to VSP home` commit

## Completion

- 改动摘要: 修复官方 VSCode ChatGPT 插件启动 VSP CLI 时继承 `CODEX_HOME=$HOME/.codex`，导致 VSP 读错配置目录并可能再次触发 `Model provider `` not found` 的路径。
- 技术思路: 保持 `chatgpt.cliExecutable` 仍为单一可执行路径，但将它指向 `$HOME/.vsp-codex/vscode/vsp-codex` wrapper。wrapper 使用 POSIX shell 安全 quoting，先导出 `CODEX_HOME=$HOME/.vsp-codex`，再 `exec` 真实 `vsp-codex` 和原始参数。
- 修改文件/模块: `codex-rs/cli/src/vscode_cmd.rs`; `codex-rs/cli/tests/vscode.rs`.
- 测试设计: test worker 先添加回归测试，模拟 VSCode 继承 `$HOME/.codex` 并断言 install 写入 wrapper 而不是真实 CLI 路径；implement worker 只改生产逻辑；audit worker 只读复审 shell quoting、dry-run、status/rollback 语义和测试覆盖。
- 验证结果: `cargo test --manifest-path codex-rs/Cargo.toml -p codex-cli --test vscode -- --nocapture` 14/14 passed; `cargo test --manifest-path codex-rs/Cargo.toml -p codex-cli vscode_install_writes_wrapper_that_pins_default_vsp_codex_home --test vscode -- --nocapture` passed; `cargo fmt --manifest-path codex-rs/Cargo.toml --check --package codex-cli` passed with existing stable rustfmt warnings; `git diff --check -- codex-rs/cli/src/vscode_cmd.rs codex-rs/cli/tests/vscode.rs` passed; `cargo build --manifest-path codex-rs/Cargo.toml --bin vsp-codex` passed.
- 预期结果: 新生成的 VSCode app-server process 通过 wrapper 启动后使用 `/home/heyx/.vsp-codex`，不会再因为官方插件传入的 `/home/heyx/.codex` home 而读到错误 provider config。
- 遇到的问题: 最新 VSCode extension logs 未复现今天的新 `Model provider `` not found` 行；诊断证据来自历史日志、当前进程环境和现有 VSCode setting。
- 风险/后续: wrapper 是 POSIX shell；当前 Remote SSH/Linux 场景适用。原生 Windows 若需要 `vsp-codex vscode install`，后续应补 `.cmd` 或 PowerShell wrapper。
