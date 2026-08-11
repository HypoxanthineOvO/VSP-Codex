# M6 集成回归与实际验收报告

## 结果

M6 集成回归主体已完成，final audit worker 发现的 P2 已修复。C2 的核心体验可以组合运行：全局 `vsp-codex` 指向当前仓库 debug binary，交互式 TUI 默认进入 VSP YOLO 权限，启动时不再出现 OAuth/API Key/Base URL 初始化页，普通远程插件官方同步噪声不再出现。

本阶段还修复了两个实际 smoke 暴露的问题：

- Provider Debug 日志会打印 inline `experimental_bearer_token`。已改为 `ModelProviderInfo` Debug 默认脱敏敏感字段，并新增脱敏测试。
- 默认 VSP 配置下仍会触发官方 remote plugin startup sync，导致 ChatGPT auth warning。已把 startup sync、featured ids、remote installed cache/bundle sync、legacy remote sync 按 `remote_plugin` feature 硬门控。
- `transcript_only` hidden reasoning 仍可能被 Hypo-Translate 处理，或通过 Replace 翻译 wrapper 在 transcript overlay 中露出。已阻止 hidden reasoning 生成翻译源，并让 transcript overlay 对 hidden reasoning 保持空输出。

## 主要改动

- `codex-rs/tui/src/lib.rs`：交互式 TUI 默认使用 `approval_policy=never` 和 `sandbox_mode=danger-full-access`，但保留 CLI/config 显式覆盖。
- `codex-rs/model-provider-info/src/lib.rs`：为 `ModelProviderInfo` 实现脱敏 `Debug`。
- `codex-rs/model-provider-info/src/model_provider_info_tests.rs`：新增 inline token/header secret 脱敏测试。
- `codex-rs/core-plugins/src/manager.rs`：远程插件 startup/background/featured/installed/sync 路径按 `remote_plugin_enabled` 跳过。
- `codex-rs/core-plugins/src/startup_remote_sync.rs`：startup remote sync 入口增加 feature gate。
- `codex-rs/core-plugins/src/manager_tests.rs`、`startup_remote_sync_tests.rs`：新增 remote plugin 禁用时不发请求的回归测试，并补充显式启用场景确保原 remote sync 行为仍可用。
- `codex-rs/tui/src/history_cell/messages.rs`、`base.rs`、`tests.rs`：修复 hidden reasoning 翻译/Transcript 泄露边界，并补充回归测试。

## 测试与验证

- M6 test worker：
  - `just test -p codex-cli --test vsp_entrypoint`：8 passed。
  - `just test -p codex-exec`：116 passed。
  - focused `codex-core` tests：4 passed，2548 skipped。
  - `cargo insta pending-snapshots --manifest-path tui/Cargo.toml`：No pending snapshots。
  - 当时全局 `vsp-codex --version` 和跨目录 `vsp-codex --version`：`vsp-codex 0.134.0-vsp.1`；C2 release prep 后已重新验证为 `vsp-codex 0.134.0-vsp.2`，见 `.pipeline/reports/007-release-prep-runtime.md`。
- M6 implement worker：
  - `just test -p codex-tui default_vsp_interactive_permissions*`：4 passed。
  - `just test -p codex-cli --test vsp_entrypoint`：8 passed。
- 主代理：
  - `just test -p codex-model-provider-info`：21 passed。
  - `just test -p codex-core-plugins startup_remote_plugin_sync`：2 passed。
  - `just test -p codex-core-plugins` remote-plugin gated negative tests：3 passed。
  - `just test -p codex-core-plugins` remote-plugin positive sync/featured tests：7 passed。
  - `just test -p codex-core-plugins remote_installed_marketplaces_skip_fetch_when_remote_plugin_disabled`：1 passed。
  - `cargo build -p codex-cli --bin vsp-codex`：首次 180s timeout，延长后通过。
  - `cargo insta pending-snapshots --manifest-path codex-rs/tui/Cargo.toml`：No pending snapshots。
  - `git diff --check`：通过。
  - `just fix -p codex-model-provider-info`：通过。
  - `just fix -p codex-core-plugins`：通过；保留既有 `manifest.rs` `large_enum_variant` warning。
  - audit follow-up `just test -p codex-tui translated_history_cell transcript_only_reasoning_is_not_translated_or_revealed translated_replace_cell_does_not_reveal_fully_hidden_inner_cell_in_transcript reasoning_summary_block*`：10 passed。
  - audit follow-up `just fix -p codex-tui`：通过。
  - final binary `cargo build -p codex-cli --bin vsp-codex`：通过。

## 实际启动 Smoke

命令使用全新日志目录 `/tmp/vsp-codex-smoke-m6`，`timeout 8s script -q -c 'vsp-codex -c log_dir="..."'` 启动真实 TUI。

结果：

- 退出码 `124` 是 timeout 结束交互式 TUI，符合 smoke 预期。
- 启动捕获文本未匹配 `OAuth`、`API Key`、`Sign in`、`login`、`Base URL`、`panic`、`error`、`failed`。
- 日志确认 `approval_policy=never`。
- 日志确认 `sandbox_policy=danger-full-access`。
- 日志确认 `experimental_bearer_token` 只显示 `Some("<redacted>")`。
- 日志未匹配 raw provider token 模式 `experimental_bearer_token: Some("sk-...")`。
- 日志未匹配 `startup remote plugin sync failed`、`chatgpt authentication required to sync remote plugins`、`/plugins/featured`、`/plugins/list`、`/ps/plugins/installed`。

## 已知说明

- 没有运行完整 workspace `just test`；本轮按用户“不设 Gate”连续推进，且 workspace full test 较慢。已覆盖所有本轮新增/改动 crate 的 focused tests、TUI smoke 和 clippy fix。
- `just fix -p codex-core-plugins` 输出一个既有 `large_enum_variant` warning，位置在 `core-plugins/src/manifest.rs`，不是本轮 remote plugin gate 变更引入。
- 默认禁用的是官方 remote plugin 同步/目录/featured/installed 远程访问；本地插件、Skills、MCP、marketplace 本地路径仍保留。
