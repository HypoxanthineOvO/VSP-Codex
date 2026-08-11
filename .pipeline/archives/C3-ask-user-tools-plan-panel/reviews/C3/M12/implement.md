# C3/M12 Implement Worker Report

## Changed files

- `codex-rs/tui/src/chatwidget/active_subagents.rs`
  - Changed active Subagent `display_name()` to return the unified label contract:
    `<nickname> [<role>]` when role is present, otherwise `<nickname>`.
  - Kept fallback nickname behavior when metadata has no nickname.
  - Removed the separate title-cased role segment from compact row summaries so rows no longer show duplicated role information such as `Robie ... Explorer`.
  - Detail popup item names use the same `display_name()` path, while descriptions still include the `role: ...` line.
- `codex-rs/tui/src/chatwidget/plan_panel.rs`
  - Changed collapsed Plan rendering to detect `completed == total && total > 0` before selecting an active/first item.
  - All-completed collapsed state now renders `<title>: all steps completed (<completed>/<total>)`.
  - Expanded Plan rendering was left unchanged.

## Implementation strategy

- Kept the production change scoped to the two allowed TUI files.
- Reused the existing active Subagent nickname fallback and role metadata instead of adding new plumbing.
- Made collapsed Plan completion a narrow resilience branch in `collapsed_lines`, preserving the existing active item/subplan summary logic for incomplete plans.

## Commands

- `cd codex-rs && just test -p codex-tui active_subagents_ctrl_o_opens_detail_popup_and_esc_closes`
  - Passed: `1 test run: 1 passed, 2676 skipped`.
- `cd codex-rs && just test -p codex-tui active_subagents_refresh_existing_row_when_metadata_arrives_late`
  - Passed: `1 test run: 1 passed, 2676 skipped`.
- `cd codex-rs && just test -p codex-tui collapsed_plan_panel_summarizes_all_completed_state`
  - Passed: `1 test run: 1 passed, 2676 skipped`.

## Notes

- Did not modify tests, snapshots, fixtures, assertions, or `.pipeline/reviews/C3/M12/test.md`.
- Did not run `fmt` or `fix`, per implement worker separation instructions.
