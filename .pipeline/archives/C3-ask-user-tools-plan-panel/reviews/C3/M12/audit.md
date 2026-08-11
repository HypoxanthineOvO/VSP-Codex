# C3/M12 Audit Worker Report

## verdict

Pass.

No blocking findings were identified in the M12 Subagent/Plan UI consistency scope. The reviewed code and snapshots satisfy the requested behavior:

- Active Subagent compact and detail identity surfaces use `<nickname> [<role>]` when metadata is available.
- Active compact rows no longer render role as a separate summary segment.
- Raw thread ids remain a metadata-missing history fallback; active panel rows use deterministic nickname fallback instead of exposing raw ids.
- Collapsed Plan all-completed rendering only triggers for `completed == total && total > 0`.
- Tests and snapshots cover spawn history, active compact/detail, late metadata refresh, and collapsed Plan completion.

## reviewed_refs

- `.pipeline/prompts/012-subagent-plan-ui-consistency.md`
- `.pipeline/reviews/C3/M12/test.md`
- `.pipeline/reviews/C3/M12/implement.md`
- `codex-rs/tui/src/chatwidget/active_subagents.rs`
  - `ActiveSubagentRow::summary`: removes separate role summary segment.
  - `ActiveSubagentRow::display_name`: renders `<nickname> [<role>]`, falling back to deterministic active-panel nickname.
  - `ActiveSubagentRow::detail_description`: keeps `role: ...` as detail metadata, not as a duplicated compact summary segment.
- `codex-rs/tui/src/chatwidget/plan_panel.rs`
  - `collapsed_lines`: all-completed branch guarded by `completed == total && total > 0`.
  - `collapsed_plan_panel_summarizes_all_completed_state`: regression test.
- `codex-rs/tui/src/chatwidget/tests/app_server.rs`
  - `collab_spawn_end_shows_requested_model_and_effort`
  - `active_subagents_ctrl_o_opens_detail_popup_and_esc_closes`
  - `active_subagents_refresh_existing_row_when_metadata_arrives_late`
  - `live_app_server_collab_spawn_completed_renders_requested_model_and_effort`
- `codex-rs/tui/src/multi_agents.rs`
  - `agent_label_spans`: history labels prefer nickname, append `[role]`, and only fall back to thread id when nickname is absent.
- Snapshots:
  - `codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__app_server_collab_spawn_completed_renders_requested_model_and_effort.snap`
  - `codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__active_subagents_ctrl_o_detail_popup.snap`
  - `codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__plan_panel__tests__collapsed_plan_panel_summarizes_all_completed_state.snap`

## findings

No Critical, Warning, or Info findings.

Specific audit checks:

- Active Subagent compact rows now build summary parts from `display_name`, model, status, and elapsed time only. The removed standalone `role_label()` summary segment prevents duplicated role display such as `Robie ... Explorer`.
- Active Subagent detail item names use `display_name()`, so the popup selection row shows `Robie [explorer]`. The detail description still includes `role: Explorer`; this is acceptable because it is metadata in the detail body, not a separate summary segment.
- Active panel metadata-missing fallback is `fallback_subagent_name(thread_id)`, so raw ids are not exposed in active compact rows before metadata arrives.
- History label rendering in `multi_agents.rs` still uses raw thread id only when nickname metadata is unavailable, and appends `[role]` when role metadata exists.
- Plan collapsed rendering evaluates the all-completed branch before selecting active/first item, and the branch is guarded by both `completed == total` and `total > 0`. Incomplete plans with pending or in-progress work continue through active item selection and should not be hidden by the completed summary.

## residual_risks

- This audit worker did not rerun tests or formatting/lint commands because the assignment was strictly read-only. Validation is based on reviewed source, snapshots, and the command results reported by the main agent and test/implement workers.
- The active detail body still title-cases the role in `role: Explorer` while the identity label shows raw role as `[explorer]`. This is pre-existing detail formatting and not a behavior mismatch for the audited summary-label contract.
- The repository worktree contains many unrelated modified/untracked files outside M12 scope. They were not reviewed except where directly relevant to the requested Subagent/Plan UI diff.

## validation_evidence

Main agent reported successful validation:

- `just test -p codex-tui active_subagents_ctrl_o_opens_detail_popup_and_esc_closes active_subagents_refresh_existing_row_when_metadata_arrives_late live_app_server_collab_spawn_completed_renders_requested_model_and_effort collapsed_plan_panel_summarizes_all_completed_state`
- `just test -p codex-tui collab_events_snapshot active_subagents plan_panel`
- `cargo insta pending-snapshots --workspace`
- `just fmt`
- `just fix -p codex-tui`
- `cargo build --bin vsp-codex`

Reviewed snapshot evidence:

- Spawn completed history renders `Spawned 李宇轩 [test] (gpt-5 high)`.
- Active compact/detail snapshot renders `Robie [explorer] · GPT 5 high · 运行中 · 0s` and popup item `Robie [explorer]`.
- Completed collapsed Plan snapshot renders `Plan: all steps completed (4/4)` and the regression assertion rejects fallback to `Prepare failing tests`.
