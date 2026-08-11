# M4 VSCode-Terminal-Safe Keymap + Composer Prominence

## Implementation Summary

- Removed VSCode-terminal-conflicting Ctrl aliases from the built-in required `list.move_up` / `list.move_down` defaults. Defaults now use arrow keys and plain Vim-style `k` / `j`.
- Updated Ask/request-user-input production hints so ranking reorder advertises `shift+↑/↓`, and free-text multi-question navigation advertises `tab / shift+tab`.
- Updated Ask/request-user-input production navigation so free-text multi-question prompts can move with `Tab` and `Shift+Tab`/`BackTab`; option prompts retain horizontal arrow navigation.
- Added safe ranking reorder handling on `Shift+Up` / `Shift+Down`. Legacy `Ctrl+J/K` remains accepted as an optional alias for compatibility, but it is no longer shown or required.
- Made the chat composer render a stable focused/unfocused visual difference: focused input uses the shared accent prompt, while unfocused input dims the composer area and prompt.
- No footer production changes were needed.
- No snapshots were accepted or edited.

## Files Changed

- `codex-rs/tui/src/keymap.rs`
- `codex-rs/tui/src/bottom_pane/request_user_input/mod.rs`
- `codex-rs/tui/src/bottom_pane/chat_composer.rs`
- `.pipeline/reports/004-vscode-terminal-safe-keymap-composer.md`
- `.pipeline/reviews/C4/M4/implement.md`

## Validation

- `cd codex-rs && just fmt`
  - Passed.
- `cd codex-rs && just test -p codex-tui keymap`
  - Failed: 107 passed, 1 failed.
  - M4 red test `keymap::tests::default_workflow_bindings_do_not_require_vscode_terminal_conflicts` passed.
  - Remaining failure is pre-existing/old expectation `keymap::tests::defaults_include_list_page_and_jump_actions`, which still expects `list.move_up` / `list.move_down` defaults to include `Ctrl+P/K/N/J`. That expectation conflicts with this milestone's requested safe defaults.
- `cd codex-rs && just test -p codex-tui request_user_input`
  - Failed: 79 passed, 2 failed.
  - M4 red test `bottom_pane::request_user_input::tests::ranking_reorder_and_multi_question_navigation_avoid_ctrl_chords` passed.
  - Remaining failures are old Ctrl-hint artifacts:
    - `freeform_shows_ctrl_p_and_ctrl_n_question_navigation_tip` still expects the old `ctrl + p / ctrl + n change question` hint.
    - `request_user_input_multi_question_last_snapshot` still snapshots the old Ctrl hint.
- `cd codex-rs && just test -p codex-tui chat_composer`
  - Passed: 186 passed.
- `cd codex-rs && cargo insta pending-snapshots -p codex-tui`
  - Not supported by this installed `cargo-insta`; it exits with `unexpected argument '-p'`.
- `cd codex-rs && cargo insta pending-snapshots`
  - Passed after removing the test-generated `.snap.new` file; no pending snapshots remain.

## Remaining Risk

- The implementation changes are in place, but full filtered `keymap` and `request_user_input` runs remain red until the test/snapshot worker updates old expectations that still require or snapshot VSCode-conflicting Ctrl hints.
- `Ctrl+J/K` is still accepted as a non-advertised legacy ranking reorder alias. This preserves compatibility while keeping the default documented path VSCode-terminal safe.

## Retry 2 Implementation Update

- Updated `codex-rs/tui/src/bottom_pane/footer.rs` production rendering so the default shortcut overlay no longer renders high-conflict/default-only visible paths:
  - `ctrl + j for newline` is hidden when it is only the fallback newline hint.
  - `ctrl + c to exit` is hidden from the overlay; the transient quit reminder behavior is unchanged.
  - Alt-only reasoning effort hints (`⌥ + ,` / `⌥ + .`) are hidden from the overlay.
- Kept underlying keybindings and runtime/custom keymap plumbing intact; this retry changes presentation of default overlay help only.
- Preserved safe overlay entries, including `/`, `!`, `Tab`, `@`, paste image, external editor, Esc edit previous, history search, transcript, and `Shift+Tab` mode change where applicable.
- Kept `Shift+Enter for newline` visible when enhanced keys are supported.
- Accepted only footer shortcut overlay snapshots directly caused by this production UI change:
  - `codex-rs/tui/src/bottom_pane/snapshots/codex_tui__bottom_pane__footer__tests__footer_shortcuts_shift_and_esc.snap`
  - `codex-rs/tui/src/bottom_pane/snapshots/codex_tui__bottom_pane__footer__tests__footer_shortcuts_collaboration_modes_enabled.snap`
  - `codex-rs/tui/src/bottom_pane/snapshots/codex_tui__bottom_pane__chat_composer__tests__footer_mode_shortcut_overlay.snap`

## Retry 2 Validation

- `cd codex-rs && just fmt`
  - Passed.
- `cd codex-rs && just test -p codex-tui footer`
  - Passed: 42 passed.
- `cd codex-rs && just test -p codex-tui chat_composer`
  - Passed: 186 passed.
- `cd codex-rs && cargo insta pending-snapshots`
  - Passed: no pending snapshots.

## Retry 2 Remaining Risk

- This retry intentionally does not remove the underlying `Ctrl+J`, `Ctrl+C`, or Alt-only reasoning bindings; it only removes those default visible paths from shortcut overlay help.
- Custom keymaps can still surface custom bindings in other keymap-oriented UI. That is expected and remains outside this footer overlay presentation fix.
