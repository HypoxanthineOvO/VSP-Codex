# C9 Debug: VSCode Remote Bundled Codex Root Cause

## 改动摘要

用户复测新建 VSP-Codex 会话后，VSCode 插件仍报 `Failed to resume chat: failed to load configuration: Model provider `` not found`。本轮确认前一次修复只覆盖了历史 session/provider 数据和 VSP wrapper 目标，真正的 VSCode Remote 插件入口仍在调用官方 bundled `codex`。

已修复 VSCode Remote 安装路径：当 `vsp-codex vscode install` 发现远端 Machine scope 时，会扫描 `~/.vscode-server/extensions/openai.chatgpt-*`，把官方扩展的 `bin/linux-x86_64/codex` 替换为指向 `/home/heyx/.vsp-codex/vscode/vsp-codex` 的 shim，并保留原始 ELF 备份。

## 技术思路

排序假设与验证：

1. 历史数据仍坏。排除。目标会话 `019edae7-f810-7ed3-a4c1-5b8168fb88f8` 的 JSONL 首行和 state DB 已有 `model_provider: vsp`。
2. VSCode 插件仍启动官方/bundled Codex。确认成立。远端插件路径 `/home/heyx/.vscode-server/extensions/openai.chatgpt-26.609.30741-linux-x64/bin/linux-x86_64/codex` 最初仍是 ELF，可执行备份不存在。
3. 仅写 VSCode Remote Machine settings 不足以让插件使用 VSP CLI。确认成立。`chatgpt.cliExecutable` 是 application-scoped setting，写入远端 Machine settings 会被 VSCode 忽略。
4. 运行中的 bundled ELF 会阻止直接覆盖。确认成立。第一次安装时报 `Text file busy (os error 26)`。

## 修改文件/模块

- `codex-rs/cli/src/vscode_cmd.rs`：远端 Machine scope 安装时扫描官方 VSCode ChatGPT extension 的 bundled `codex`，备份原始文件并安装 VSP shim。
- `codex-rs/cli/src/vscode_cmd.rs`：shim 安装改为写临时文件、设置可执行权限、再 `rename` 到目标路径，避免运行中 ELF 触发 `Text file busy`。
- `codex-rs/cli/tests/vscode.rs`：新增远端 bundled `codex` repoint 测试。
- `codex-rs/cli/tests/vscode.rs`：新增运行中 ELF 场景测试，用 `/bin/sleep` 复现 ETXTBSY 后验证 rename 替换可成功。

## 运行态修复

- 重新构建 release `vsp-codex`。
- 原子替换 `/home/heyx/.vsp-codex/packages/standalone/current/bin/vsp-codex`。
- 执行 `/home/heyx/.vsp-codex/packages/standalone/current/bin/vsp-codex vscode install --yes --cli /home/heyx/.vsp-codex/packages/standalone/current/bin/vsp-codex`。
- 远端插件 bundled 入口现在是 shell shim：
  - `/home/heyx/.vscode-server/extensions/openai.chatgpt-26.609.30741-linux-x64/bin/linux-x86_64/codex`
  - 内容为 `exec '/home/heyx/.vsp-codex/vscode/vsp-codex' "$@"`
- 原始官方 ELF 已备份：
  - `/home/heyx/.vscode-server/extensions/openai.chatgpt-26.609.30741-linux-x64/bin/linux-x86_64/codex.vsp-backup`

## 验证结果

通过：

- `cargo test --manifest-path codex-rs/Cargo.toml -p codex-cli --test vscode vscode_install_remote_scope_repoints -- --nocapture`：2 passed。
- `cargo build --manifest-path codex-rs/Cargo.toml -p codex-cli --release`：通过。
- `cargo fmt --manifest-path codex-rs/Cargo.toml --all -- --check`：通过，仅稳定 rustfmt 输出 `imports_granularity = Item` warning。
- `git diff --check`：通过。
- 插件 bundled 路径 `--version`：`vsp-codex 0.140.0-vsp.0`。
- 运行态 smoke：从 `/home/heyx/.vscode-server/extensions/openai.chatgpt-26.609.30741-linux-x64/bin/linux-x86_64/codex app-server --listen stdio://` 启动，`initialize` 返回 `codexHome=/home/heyx/.vsp-codex`，`thread/resume` 目标会话 `019edae7-f810-7ed3-a4c1-5b8168fb88f8` 成功，返回 `modelProvider=vsp`、`thread.modelProvider=vsp`、`model=gpt-5.5`、`cwd=/home/heyx/HUINA/JINGMAO`。

## 预期结果

VSCode Remote 插件后续从 bundled `codex` 启动时会进入 VSP wrapper，并使用 `/home/heyx/.vsp-codex` 配置。新建或恢复 VSP 会话不应再因为官方 Codex 不认识 `vsp` provider 而报 `Model provider `` not found` 或 `Model provider `vsp` not found`。

## 遇到的问题

- 第一次安装失败于 `Text file busy (os error 26)`，原因是官方 bundled ELF 正在被 VSCode 插件进程执行；已通过临时文件 + `rename` 替换修复。
- 已经运行中的 VSCode/app-server 进程不会自动变成新入口；需要 VSCode reload window 或重启插件进程后才会使用新的 bundled shim。

## 风险/后续

这次没有强杀用户的 VSCode 或 app-server 进程。若 VSCode 里仍复现旧错误，优先执行 `Developer: Reload Window` 或重启 Remote SSH 窗口，让插件重新启动 bundled 入口。
