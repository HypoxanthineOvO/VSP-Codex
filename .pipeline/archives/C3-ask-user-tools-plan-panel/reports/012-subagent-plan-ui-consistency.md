# M12 Subagent 与 Plan UI 一致性修复报告

## 结果

状态：completed

M12 修复了两个 M11 后暴露的 UI 一致性问题：

- Active Subagent compact row 和 detail popup item name 统一显示 `<nickname> [<role>]`。
- Plan 折叠态在所有步骤完成后显示完成摘要，不再 fallback 到第一个旧步骤。

## 代码变更

- `codex-rs/tui/src/chatwidget/active_subagents.rs`
  - `ActiveSubagentRow::display_name()` 现在输出统一身份 label。
  - compact row 不再额外追加单独的 title-cased role 段，避免 `Robie ... Explorer` 和 `Robie [explorer]` 两套显示并存。
  - detail popup name 复用同一 label；description 中保留 `role: ...` 明细。
- `codex-rs/tui/src/chatwidget/plan_panel.rs`
  - `collapsed_lines()` 增加 `completed == total && total > 0` 分支。
  - 完成态显示 `<title>: all steps completed (<completed>/<total>)`。
  - expanded rendering 不变。
- `codex-rs/tui/src/chatwidget/tests/app_server.rs`
  - 覆盖 spawn history nickname+role。
  - 覆盖 active compact/detail nickname+role。
  - 覆盖 late metadata refresh 后的 active row label。
- `codex-rs/tui/src/chatwidget/snapshots/*`
  - 更新 active Subagent detail popup snapshot。
  - 更新 app-server spawn completed snapshot。
  - 新增 Plan all-completed collapsed snapshot。

## Worker Separation

严格分离已执行：

- test worker：`杨超`
  - 写红色测试与 snapshots。
  - 报告：`.pipeline/reviews/C3/M12/test.md`
- implement worker：`李萌`
  - 只改生产代码。
  - 报告：`.pipeline/reviews/C3/M12/implement.md`
- audit worker：`李萌 the 2nd`
  - 只读审计。
  - 报告：`.pipeline/reviews/C3/M12/audit.md`
  - verdict：Pass

## 验证

已通过：

- `cd codex-rs && just test -p codex-tui active_subagents_ctrl_o_opens_detail_popup_and_esc_closes active_subagents_refresh_existing_row_when_metadata_arrives_late live_app_server_collab_spawn_completed_renders_requested_model_and_effort collapsed_plan_panel_summarizes_all_completed_state`
- `cd codex-rs && just test -p codex-tui collab_events_snapshot active_subagents plan_panel`
- `cd codex-rs && cargo insta pending-snapshots --workspace`
- `cd codex-rs && just fmt`
- `cd codex-rs && just fix -p codex-tui`
- `cd codex-rs && cargo build --bin vsp-codex`

说明：按仓库规则，`just fix -p codex-tui` 之后没有重复跑 tests。

## 残余风险

- M12 修复的是 TUI resilience 和显示一致性；Plan 源状态仍由最新 `update_plan` 驱动，不读取 `.pipeline/state.yaml`。
- C3 仍不能最终验收，因为 M7 的真实跨设备 pending Ask resume 验收仍未完成。
