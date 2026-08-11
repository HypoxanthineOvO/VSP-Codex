# M3 Subagent 常驻卡片与键盘查看审核

## Verdict

pass

## Reviewed Refs

- `codex-rs/tui/src/chatwidget/active_subagents.rs`
- `codex-rs/tui/src/chatwidget.rs`
- `codex-rs/tui/src/chatwidget/constructor.rs`
- `codex-rs/tui/src/chatwidget/rendering.rs`
- `codex-rs/tui/src/chatwidget/tool_lifecycle.rs`
- `codex-rs/tui/src/chatwidget/interaction.rs`
- `codex-rs/tui/src/app.rs`
- `codex-rs/tui/src/app/tests.rs`
- `codex-rs/tui/src/chatwidget/tests/app_server.rs`
- M3 active Subagent snapshots

## Findings

初始审计发现两个问题，均已修复：

- P2：后到的 Subagent metadata 不会刷新已有 active row。已让 `set_collab_agent_metadata` 同步更新 `active_subagents`，并补充 `active_subagents_refresh_existing_row_when_metadata_arrives_late`。
- P3：详情弹窗 snapshot 中 active panel elapsed 可能因测试暂停从 `0s` 变为 `1s`。已增加 test-only 时间固定钩子，避免 snapshot flaky。

## Residual Risks

- active panel 目前只使用结构化 collab tool call 中已有的 prompt/message/model/status 字段，不解析 Subagent 内部 transcript。M4 会处理 Subagent transcript 翻译与查看体验。
- panel 行只保留 active 状态，completed/closed agent 仍可通过既有 `/agent`/thread replay 路径查看。

## Validation

- `just test -p codex-tui active_subagent_panel_lists_only_running_and_waiting_agents_snapshot active_subagent_panel_truncates_rows_at_narrow_width_snapshot active_subagents_alt_a_opens_detail_popup_and_esc_closes active_subagents_refresh_existing_row_when_metadata_arrives_late`：通过。
- `just test -p codex-tui`：2634 passed, 4 skipped。
- `cargo insta pending-snapshots --manifest-path tui/Cargo.toml`：No pending snapshots。
- `git diff --check`：通过。
- `just fix -p codex-tui`：通过。
