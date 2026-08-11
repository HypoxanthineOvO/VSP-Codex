# C4/M4 Test Worker Report

## Scope

Role: test worker only.

Allowed files touched:
- `codex-rs/tui/src/keymap.rs` test module
- `codex-rs/tui/src/bottom_pane/request_user_input/mod.rs` test module
- `codex-rs/tui/src/bottom_pane/chat_composer.rs` test module
- `codex-rs/tui/src/bottom_pane/footer.rs` test module
- `.pipeline/reviews/C4/M4/test.md`

No production runtime implementation was added by this worker.

## Added Coverage

- `keymap::tests::default_workflow_bindings_do_not_require_vscode_terminal_conflicts`
  - Locks essential/default workflow bindings against required `Ctrl+J/K`, `Ctrl+P/N`, `Ctrl+Shift+P`, `Ctrl+C`, and Alt-only chords.
- `request_user_input::tests::ranking_reorder_and_multi_question_navigation_avoid_ctrl_chords`
  - Locks Ask ranking reorder and multi-question navigation away from required Ctrl chords in hints/handlers.
- `chat_composer::tests::focused_and_unfocused_composer_render_distinct_visual_state`
  - Locks composer focused/unfocused rendering to have a stable visual difference.
- `footer::tests::footer_snapshots`
  - Added stable assertion that shortcut overlay text does not present high-risk keys or Alt-only chords as necessary actions.

## Command Results

- `cd codex-rs && just fmt`
  - Passed.
- `cd codex-rs && just test -p codex-tui keymap`
  - Red: 107 passed, 1 failed.
  - Failing test: `keymap::tests::default_workflow_bindings_do_not_require_vscode_terminal_conflicts`.
  - Signal: `list.move_up` still includes `Ctrl+P` and `Ctrl+K`.
- `cd codex-rs && just test -p codex-tui request_user_input`
  - Red: 80 passed, 1 failed.
  - Failing test: `bottom_pane::request_user_input::tests::ranking_reorder_and_multi_question_navigation_avoid_ctrl_chords`.
  - Signal: ranking footer still says `ctrl+j/k to reorder`.
- `cd codex-rs && just test -p codex-tui chat_composer`
  - Red: 185 passed, 1 failed.
  - Failing test: `bottom_pane::chat_composer::tests::focused_and_unfocused_composer_render_distinct_visual_state`.
  - Signal: focused and unfocused composer render identically.
- `cd codex-rs && just test -p codex-tui footer`
  - Green: 42 passed.
  - Signal: current shortcut overlay assertion is satisfied.
- `cd codex-rs && cargo insta pending-snapshots`
  - Green: no pending snapshots.

## Notes

The red tests are intentional for the implement worker. They lock the user-reported VSCode terminal conflict pain before production changes are made.

## Retry - 2026-05-28

Scope: test retry worker only. Updated stale assertions/snapshots to match the M4 implementation without editing production logic.

Changes:
- Updated `keymap::tests::defaults_include_list_page_and_jump_actions` so default `list.move_up/down` expects arrows plus plain `k/j`, while page/jump actions remain asserted separately.
- Renamed and updated the freeform multi-question hint test to expect `tab / shift+tab change question`.
- Updated `request_user_input_multi_question_last` snapshot for the same free-text navigation hint.

Command results:
- `cd codex-rs && just fmt` - passed.
- `cd codex-rs && just test -p codex-tui keymap` - passed: 108 passed.
- `cd codex-rs && just test -p codex-tui request_user_input` - passed: 81 passed.
- `cd codex-rs && just test -p codex-tui chat_composer` - passed: 186 passed.
- `cd codex-rs && cargo insta pending-snapshots` - passed: no pending snapshots.

Remaining red items in this retry scope: none observed.

## Retry 2 - 2026-05-28

Scope: second-round test retry worker only. Edited only footer test assertions and this report; no production logic was changed.

Changes:
- Strengthened `bottom_pane::footer::tests::footer_snapshots` so the default shortcut overlay guard checks the actual visible rendering format, including `ctrl + c` and `⌥ +`.
- Added compact normalized matching that removes whitespace and maps `⌥` to `alt`, covering `ctrl+c`, `alt+`, and related variants.
- Kept the assertion focused on the M4 requirement: default visible shortcut help should not present VSCode-terminal high-conflict keys or Alt-only chords as necessary/main paths.

Command results:
- `cd codex-rs && just fmt` - passed.
- `cd codex-rs && just test -p codex-tui footer` - red as expected: 41 passed, 1 failed.

Red failure:
- `bottom_pane::footer::tests::footer_snapshots`
  - Failure begins at the visible-path assertion for `ctrl + j`.
  - Rendered overlay still includes `ctrl + j for newline`, `ctrl + c to exit`, `⌥ + , reasoning down`, and `⌥ + . reasoning up`, confirming the audit blocker remains exposed by the strengthened test.

Snapshot status:
- No related `.snap.new` files were generated.
