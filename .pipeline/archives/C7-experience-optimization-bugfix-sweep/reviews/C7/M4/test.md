# C7/M4 Test Worker Report

## Changed Files

- `codex-rs/tui/src/chatwidget/tests/app_server.rs`
- `.pipeline/reviews/C7/M4/test.md`

## Tests Added

- `c7_m4_active_subagents_ctrl_o_viewer_exposes_full_prompt_and_report_markdown`
  - Opens the real Active Subagents `Ctrl+O` popup from `ChatWidget`.
  - Expects full prompt and report Markdown content, including tail markers beyond preview length.
  - Expects original/translated labels and no raw triple-backtick fence leakage.
  - Verifies compact active row does not leak full prompt/report tails.
- `c7_m4_active_subagents_ctrl_o_viewer_preserves_core_content_at_narrow_width`
  - Renders the same `Ctrl+O` surface at width 48.
  - Expects identity, Prompt/Report entry points, original/translated labels, headings, and full-content tail markers to remain available.
- `c7_m4_collab_lifecycle_rows_keep_prompt_and_report_concise`
  - Exercises real lifecycle history rows for spawn/wait events.
  - Confirms ordinary lifecycle rows stay concise and do not leak full prompt/report Markdown or raw code fences.

## Red Command

```bash
cargo test -p codex-tui c7_m4 -- --nocapture
```

## Red Failure Reason

Current production code still renders Active Subagents `Ctrl+O` as the existing detail selection popup. It shows only truncated `recent input` and `message` previews:

- Full prompt tail marker `C7_M4_FULL_PROMPT_TAIL_MARKER_VISIBLE_ONLY_IN_VIEWER` is missing.
- Full report tail marker `C7_M4_FULL_REPORT_TAIL_MARKER_VISIBLE_ONLY_IN_VIEWER` is missing.
- There is no explicit original/translated mode label in the popup.
- Narrow width has the same limitation: wrapped truncated previews, not a full viewer.

Observed result:

- `c7_m4_collab_lifecycle_rows_keep_prompt_and_report_concise` passed.
- `c7_m4_active_subagents_ctrl_o_viewer_exposes_full_prompt_and_report_markdown` failed as expected.
- `c7_m4_active_subagents_ctrl_o_viewer_preserves_core_content_at_narrow_width` failed as expected.

## Risks / Notes

- No production code was changed.
- No snapshots were added or accepted.
- Existing unrelated worktree changes were left untouched.

## Audit-Fix Test: Selected Active Subagent Viewer Binding

### Tests Added

- `c7_m4_active_subagents_ctrl_o_viewer_tracks_selected_row`
  - Constructs two active subagents through real app-server `ItemStarted` notifications.
  - Uses distinct names and roles: `Robie [explorer]` and `Ada [worker]`.
  - Uses distinct full prompt/report markers for each agent.
  - Opens the real Active Subagents `Ctrl+O` popup, sends a real list navigation key event (`Down`), then renders through `render_bottom_popup`.
  - Asserts the second row is selected and the viewer renders Ada's full prompt/report markers, not Robie's.

### Red Command

```bash
cd /home/heyx/Codex-VSP/codex-rs && cargo test -p codex-tui c7_m4 -- --nocapture
```

### Red Result

- Failed as expected.
- Filtered result: 4 `c7_m4` tests ran; 3 passed, 1 failed.
- Failing test: `chatwidget::tests::app_server::c7_m4_active_subagents_ctrl_o_viewer_tracks_selected_row`.

### Failure Reason

The popup list selection moved to `Ada [worker]` (`› 2. Ada [worker]` was visible), but the prompt/report viewer still rendered `Robie [explorer]` and Robie's full prompt/report markers. This confirms the audit finding: the Active Subagents `Ctrl+O` viewer is bound to the first active row instead of the currently selected row.
