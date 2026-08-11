# M3 Subagent 常驻卡片与键盘查看报告

## 结果

M3 已完成。VSP Codex 现在会在 TUI composer 上方显示 active Subagent 常驻卡片，只展示结构化事件中仍处于 `PendingInit` 或 `Running` 的 Subagent；completed、closed、not found 等终态不会常驻占位。用户可以通过 `Alt+A` 打开详情弹窗查看当前 active Subagent 的 id、状态、角色、model/reasoning、最近工具调用、最近输入和结构化消息。

## 主要改动

- `codex-rs/tui/src/chatwidget/active_subagents.rs`：新增 active Subagent panel 状态与渲染模块，使用结构化 `ThreadItem::CollabAgentToolCall` 更新 rows。
- `codex-rs/tui/src/chatwidget/tool_lifecycle.rs`：在处理 collab tool call 时同步 active Subagent panel。
- `codex-rs/tui/src/chatwidget/rendering.rs`：只有存在 active Subagent 时才在 composer 上方插入 panel，避免空状态影响现有布局。
- `codex-rs/tui/src/chatwidget/interaction.rs`：新增 `Alt+A` 打开 active Subagent 详情弹窗。
- `codex-rs/tui/src/chatwidget.rs`：当后到的 Subagent nickname/role metadata 抵达时，刷新已有 active row，避免长期显示 fallback thread id。
- `codex-rs/tui/src/app.rs` 与 TUI tests：补充 test-only panel render hook。

## 测试与验证

- `just fmt`：通过。
- `just test -p codex-tui active_subagent_panel_lists_only_running_and_waiting_agents_snapshot active_subagent_panel_truncates_rows_at_narrow_width_snapshot active_subagents_alt_a_opens_detail_popup_and_esc_closes active_subagents_refresh_existing_row_when_metadata_arrives_late`：通过。
- `just test -p codex-tui`：2634 passed, 4 skipped。
- `cargo insta pending-snapshots --manifest-path tui/Cargo.toml`：No pending snapshots。
- `git diff --check`：通过。
- `just fix -p codex-tui`：通过。

## Worker 证据

- test worker：`019e608a-bc5d-7121-97db-73e9ef6e3bc5`，补充 active panel snapshot 测试。
- implement worker：`019e608a-f945-72e3-a6ce-2f3fab808277`，实现 active panel、详情弹窗和事件接线。
- test follow-up：`019e6094-5f09-7bd0-b5a8-65f02d54763c`，补充 `Alt+A` 详情弹窗交互测试。
- audit worker：`019e60a2-0fce-7042-b542-2cac7d8e0b56`，指出 metadata late update 和 elapsed snapshot flake 两个问题，均已修复并验证。

## 已知说明

M3 只接入结构化 Subagent 状态与详情。Subagent transcript 内容翻译归 M4；静态 UI 大范围汉化归 M5。
