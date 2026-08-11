# C8 M1 Remote SSH VSCode Integration Probe Report

完成时间：2026-05-30T14:30:43+08:00

## 结果

M1 已完成，audit 结论为 `pass`。`vsp-codex vscode install/status/rollback` 现在通过同一个 `resolve_settings_scope()` 优先选择远端 VSCode server Machine settings：`~/.vscode-server/data/Machine/settings.json`。当 server settings 不存在时，本地 `Code/User/settings.json` fallback 仍保持可用。

## 变更

- `codex-rs/cli/src/vscode_cmd.rs`
  - 调整 settings scope 解析优先级：server Machine settings 优先于 local user settings。
  - 保持 `install`、`status`、`rollback` 共用同一 resolver。
- `codex-rs/cli/tests/vscode.rs`
  - 新增 both-exist 场景 red/green 覆盖。
  - 补强 user-only status scope 断言。

## 验证

- `cargo test -p codex-cli vscode -- --nocapture`：10 项通过。
- `cargo fmt --check --package codex-cli`：通过，仅有既有 stable rustfmt `imports_granularity` warning。
- `git diff --check`：通过。
- M1 audit worker 复跑上述验证并通过。

## 风险

非阻塞残余风险：当前优先级是 filesystem-driven。如果本地-only 机器上残留 `.vscode-server/data/Machine/settings.json`，会优先选择该 server settings。该行为符合本 Cycle 的 Remote SSH 主路径要求，后续如需更细粒度可增加显式 scope 参数或环境判断。

