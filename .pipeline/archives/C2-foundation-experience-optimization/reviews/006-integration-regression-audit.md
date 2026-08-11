# M6 集成回归与实际验收审核

## Verdict

pass after follow-up

## Reviewed Refs

- `codex-rs/tui/src/lib.rs`
- `codex-rs/model-provider-info/src/lib.rs`
- `codex-rs/model-provider-info/src/model_provider_info_tests.rs`
- `codex-rs/core-plugins/src/manager.rs`
- `codex-rs/core-plugins/src/manager_tests.rs`
- `codex-rs/core-plugins/src/startup_remote_sync.rs`
- `codex-rs/core-plugins/src/startup_remote_sync_tests.rs`
- M1-M5 reports and M6 smoke evidence

## Current Findings

最终只读 audit worker `019e6134-eb1c-7212-88db-e8ac3353d2cb` 未发现 P1，发现 1 个 P2。该 P2 已修复并通过 focused tests。

已修复的 M6 问题：

- P1/security：Provider Debug 日志泄露 inline token。已用脱敏 `Debug` 修复，并用真实 TUI smoke 确认 raw token 不再出现。
- P2/product noise：默认 VSP 启动仍触发官方 remote plugin sync 并打印 ChatGPT auth warning。已按 `remote_plugin` feature 门控，并用单元测试和真实 smoke 确认默认不发起远程插件请求。
- P2/privacy：`transcript_only` hidden reasoning 可被 Hypo-Translate 处理，并可能通过 Replace 翻译 wrapper 在 transcript overlay 中显示。已修复为 hidden reasoning 不生成翻译源、`transcript_only` transcript lines 为空、完全 hidden inner cell 的 Replace 翻译不在 transcript overlay 输出。

## Residual Risks

- 未运行 full workspace `just test`。当前证据覆盖本轮实际修改 crate 和真实 TUI 启动；完整 suite 可在提交前或 CI 中补跑。
- 远程插件 feature gate 会让默认 VSP 不再显示官方远程插件 featured/installed 数据，这是符合 C1/C2 “清理 Cloud/OAuth/Remote 官方部分”的预期；如未来需要重新启用，需要显式设置 `remote_plugin=true` 并接受 ChatGPT auth 依赖。
- Provider `Debug` 仍保留 `env_key`、`env_http_headers` 的名称信息，但不打印实际 token/header value。当前判断为可接受，因为这些是变量名/配置键，不是 secret value。

## Validation

- `just test -p codex-model-provider-info`：21 passed。
- `just test -p codex-core-plugins startup_remote_plugin_sync`：2 passed。
- `just test -p codex-core-plugins` remote-plugin gated negative tests：3 passed。
- `just test -p codex-core-plugins` remote-plugin positive sync/featured tests：7 passed。
- `cargo build -p codex-cli --bin vsp-codex`：passed。
- Actual TUI smoke：默认 YOLO、无 OAuth/API Key/Base URL 启动页、Provider token 脱敏、远程插件 auth 噪声消失。
- `cargo insta pending-snapshots --manifest-path codex-rs/tui/Cargo.toml`：No pending snapshots。
- `git diff --check`：通过。
- `just fix -p codex-model-provider-info`：通过。
- `just fix -p codex-core-plugins`：通过，保留既有 warning。
- audit follow-up `just test -p codex-tui translated_history_cell transcript_only_reasoning_is_not_translated_or_revealed translated_replace_cell_does_not_reveal_fully_hidden_inner_cell_in_transcript reasoning_summary_block*`：10 passed。
- audit follow-up `just fix -p codex-tui`：通过。
- audit follow-up `cargo insta pending-snapshots --manifest-path codex-rs/tui/Cargo.toml`：No pending snapshots。
- audit follow-up `git diff --check`：通过。
- final binary `cargo build -p codex-cli --bin vsp-codex`：通过。
