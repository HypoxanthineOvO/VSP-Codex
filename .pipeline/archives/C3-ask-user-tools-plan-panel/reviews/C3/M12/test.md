# C3/M12 Test Worker Report

## Changed files

- `codex-rs/tui/src/chatwidget/tests/app_server.rs`
  - Added active Subagent compact panel assertion requiring `nickname [role]`.
  - Tightened active Subagent detail popup assertion to require `nickname [role]`.
  - Updated late-metadata active row assertion to require `nickname [role]`.
  - Seeded spawn-completed metadata so the app-server snapshot covers nickname + role instead of raw UUID.
- `codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__app_server_collab_spawn_completed_renders_requested_model_and_effort.snap`
  - Expected spawn-completed history line now shows `李宇轩 [test]`.
- `codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__active_subagents_ctrl_o_detail_popup.snap`
  - Expected compact and popup rows now show `Robie [explorer]`.
- `codex-rs/tui/src/chatwidget/plan_panel.rs`
  - Added a test-only completed Plan fixture and collapsed-state regression test.
- `codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__plan_panel__tests__collapsed_plan_panel_summarizes_all_completed_state.snap`
  - Added expected completed collapsed Plan summary snapshot.

## Commands

- `cd codex-rs && just test -p codex-tui collab_spawn_end_shows_requested_model_and_effort`
  - Passed. Current worktree already renders metadata in this direct app-server spawn assertion.
- `cd codex-rs && just test -p codex-tui live_app_server_collab_spawn_completed_renders_requested_model_and_effort`
  - First run failed on snapshot mismatch:
    - old: `• Spawned 019cff70-2599-75e2-af72-b91781b41a8e (gpt-5 high)`
    - new: `• Spawned 李宇轩 [test] (gpt-5 high)`
  - After updating the allowed snapshot, rerun passed.
- `cd codex-rs && just test -p codex-tui active_subagents_ctrl_o_opens_detail_popup_and_esc_closes`
  - Failed as intended.
- `cd codex-rs && just test -p codex-tui active_subagents_refresh_existing_row_when_metadata_arrives_late`
  - Failed as intended.
- `cd codex-rs && just test -p codex-tui collapsed_plan_panel_summarizes_all_completed_state`
  - Failed as intended.

## Red evidence

- Active Subagent compact panel still renders role as a separate title-cased field:
  - expected: `Robie [explorer] · GPT 5 high · 运行中 · 0s`
  - actual: `Robie · GPT 5 high · Explorer · 运行中 · 0s`
- Late metadata refresh has the same compact panel issue:
  - expected: `杨超 [audit] · GPT 5 high · 运行中 · 0s`
  - actual: `杨超 · GPT 5 high · Audit · 运行中 · 0s`
- Plan collapsed all-completed state still falls back to the first completed item:
  - expected: `• Plan: all steps completed (4/4)`
  - actual: `• Plan: Prepare failing tests (4/4)`

## Risks / gaps

- I did not change production/runtime logic and did not run `fmt` or `fix`, per worker instructions.
- `codex-rs/tui/src/chatwidget/plan_panel.rs` and its existing plan-panel snapshots were already untracked in this worktree before this report; I only added the new completed-state test/snapshot on top of that current state.
- Spawn-completed nickname+role behavior appears already implemented in the current worktree, so the red signal for that surface is snapshot coverage rather than a currently failing runtime assertion.
