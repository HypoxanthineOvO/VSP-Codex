# C4/M4 Audit - VSCode-Terminal-Safe Keymap + Composer Prominence

## verdict

pass

## reviewed_refs

- `.pipeline/prompts/004-vscode-terminal-safe-keymap-composer.md`
- `.pipeline/reports/004-vscode-terminal-safe-keymap-composer.md`
- `.pipeline/reviews/C4/M4/test.md`
- `.pipeline/reviews/C4/M4/implement.md`
- `codex-rs/tui/src/keymap.rs`
- `codex-rs/tui/src/bottom_pane/request_user_input/mod.rs`
- `codex-rs/tui/src/bottom_pane/request_user_input/render.rs`
- `codex-rs/tui/src/bottom_pane/request_user_input/snapshots/codex_tui__bottom_pane__request_user_input__tests__request_user_input_multi_question_last.snap`
- `codex-rs/tui/src/bottom_pane/chat_composer.rs`
- `codex-rs/tui/src/bottom_pane/footer.rs`
- `codex-rs/tui/src/bottom_pane/snapshots/codex_tui__bottom_pane__chat_composer__tests__footer_mode_shortcut_overlay.snap`
- `codex-rs/tui/src/bottom_pane/snapshots/codex_tui__bottom_pane__footer__tests__footer_shortcuts_collaboration_modes_enabled.snap`
- `codex-rs/tui/src/bottom_pane/snapshots/codex_tui__bottom_pane__footer__tests__footer_shortcuts_shift_and_esc.snap`

Read-only commands used:

- `git status --short`
- `find .pipeline -maxdepth 5 -type f | sort`
- `sed -n ...` on M4 prompt/report/review artifacts
- `git diff -- ...` on scoped TUI files and M4 snapshot
- `git diff --numstat -- ...` on scoped TUI files
- `rg -n ...` over scoped TUI keymap, Ask, composer, footer, and snapshots
- `nl -ba ... | sed -n ...` for cited evidence windows

## checks

- Default list movement no longer requires the originally reported VSCode-terminal conflicts: `list.move_up` is `Up`/`k`, and `list.move_down` is `Down`/`j` in `codex-rs/tui/src/keymap.rs:832-834`.
- `request_user_input` ranking prompts advertise `shift+up/down` instead of `Ctrl+J/K`, including zh-CN text in `codex-rs/tui/src/bottom_pane/request_user_input/mod.rs:651-657` and `:701-706`.
- Free-text multi-question navigation now advertises `tab / shift+tab change question` instead of `Ctrl+P/N` in `codex-rs/tui/src/bottom_pane/request_user_input/mod.rs:685-690`; the updated snapshot also reflects this.
- Handlers provide safe default paths: `Tab`/`BackTab` navigate free-text questions in `codex-rs/tui/src/bottom_pane/request_user_input/mod.rs:1497-1536`, and `Shift+Up/Down` reorders ranking options in `:1586-1590`.
- Legacy `Ctrl+J/K` ranking reorder remains accepted in `codex-rs/tui/src/bottom_pane/request_user_input/mod.rs:1592-1596`, but it is not advertised by the Ask footer; this is acceptable compatibility, not a required/default path.
- Composer focused/unfocused render states are visibly distinct: unfocused composer style is dimmed and focused prompt uses accent styling in `codex-rs/tui/src/bottom_pane/chat_composer.rs:4632-4655`.
- Worker-separation artifacts are mostly coherent: the implement report claims production-only M4 edits, and the test retry report claims old assertion/snapshot updates only. Current diff is noisy across the same files, but the M4-specific changed sections match those reports.

## findings

### Blocking - Footer shortcut overlay still advertises high-conflict default shortcuts

The M4 prompt explicitly included reviewing the footer shortcut overlay and removing high-risk keys as required actions. The production shortcut overlay still includes:

- `Ctrl+C` as `to exit` in `codex-rs/tui/src/bottom_pane/footer.rs:1225-1232`.
- Alt-only reasoning shortcuts in `codex-rs/tui/src/bottom_pane/footer.rs:1252-1268`.

Existing accepted snapshots still render these in the shortcut overlay, for example:

- `codex-rs/tui/src/bottom_pane/snapshots/codex_tui__bottom_pane__chat_composer__tests__footer_mode_shortcut_overlay.snap` shows `ctrl + c to exit` and `⌥ + ,` / `⌥ + .`.
- `codex-rs/tui/src/bottom_pane/snapshots/codex_tui__bottom_pane__footer__tests__footer_shortcuts_collaboration_modes_enabled.snap` and `footer_shortcuts_shift_and_esc.snap` show the same pattern.

The new footer test does not catch this because it searches for compact strings like `ctrl+c` and `alt+`, while the rendered UI uses `ctrl + c` and the Option symbol `⌥ +`. This leaves the visible default help overlay still promoting a VSCode-terminal-conflicting exit shortcut and Alt-only shortcuts.

Required fix: remove or demote these high-conflict shortcuts from the default visible shortcut overlay, or make the overlay clearly present safe alternatives such as slash commands as the primary path. Update the footer snapshots/tests so they assert against the actual rendered spellings (`ctrl + c`, `⌥ +`, and equivalent variants), not only compact ASCII forms.

## retry_verdict - 2026-05-28

pass

Retry read-only commands used:

- `git status --short`
- `git diff --stat`
- `sed -n ...` on `.pipeline/reviews/C4/M4/audit.md`, M4 test/implement reports, and targeted footer snapshots
- `git diff -- ...` on scoped M4 TUI files
- `rg -n ...` over scoped footer/keymap/Ask/composer files and targeted snapshots
- `nl -ba ... | sed -n ...` for cited evidence windows

Retry checks:

- The first audit blocker is resolved. `ShortcutDescriptor::overlay_entry` now filters the default visible overlay entries for `Ctrl+J` newline fallback, `Ctrl+C` quit, and Alt-only reasoning down/up before line construction in `codex-rs/tui/src/bottom_pane/footer.rs:1096-1152`. The overlay input is compacted with `filter(|line| line.width() > 0)` before column layout in `footer.rs:928-944`, so hidden entries do not leave empty cells.
- Targeted accepted snapshots no longer render the blocked visible paths. `footer_mode_shortcut_overlay.snap`, `footer_shortcuts_collaboration_modes_enabled.snap`, and `footer_shortcuts_shift_and_esc.snap` contain no `ctrl + j`, `ctrl + c`, `⌥ + ,`, or `⌥ + .`; the safe `shift + enter for newline` path remains visible when enhanced keys select it.
- M4 list movement defaults remain safe: `list.move_up` is `Up` plus plain `k`, and `list.move_down` is `Down` plus plain `j` in `codex-rs/tui/src/keymap.rs:832-834`.
- Ask ranking and multi-question safe hints/handlers remain intact: ranking footer text advertises `shift+↑/↓ to reorder` in `request_user_input/mod.rs:651-657`; free-text multi-question navigation advertises `tab / shift+tab change question` in `:685-690`; zh-CN equivalents advertise the same safe paths in `:701-735`. Handlers accept `Tab`/`BackTab` for free-text question navigation in `:1497-1536`, and `Shift+Up/Down` for ranking reorder in `:1581-1590`.
- Compatibility is preserved. Legacy `Ctrl+J/K` ranking reorder is still accepted but no longer advertised in `request_user_input/mod.rs:1592-1596`. Alt reasoning defaults remain in the runtime keymap in `codex-rs/tui/src/keymap.rs:680-682`, and `ChatComposer::set_keymap_bindings` still forwards runtime/custom keymap bindings into footer key hints in `chat_composer.rs:655-674`; the retry only suppresses specific default visible overlay entries.
- Composer focus visual distinction remains covered by production rendering and a test: focused state uses the normal/accent prompt while unfocused state dims the composer/prompt in `chat_composer.rs:4632-4655`; the test asserts focused and unfocused renders differ in `:4937-4970`.
- Test/report artifacts are sufficient for pass. The M4 test report records the strengthened red footer assertion for `ctrl + j`, `ctrl + c`, and `⌥ +`; the implement retry report records the production presentation-layer filtering and accepted footer snapshots. The main-agent verification reported `just test -p codex-tui footer`, `chat_composer`, `keymap`, and `request_user_input` all passing, plus no pending snapshots. This audit did not rerun tests per the read-only retry-worker constraint.

## residual_risks

- The current working tree has broad accumulated changes in `request_user_input` beyond M4, including zh-CN card rendering, multi-choice/ranking payload behavior, and secret masking. These appear to be earlier C3/C4 work rather than M4 implement changes, but the same-file diff makes worker attribution partly artifact-dependent.
- The main agent reported the three requested filtered `codex-tui` runs and `cargo insta pending-snapshots` as green. I did not rerun tests because this audit worker was asked to stay read-only and only write the audit artifact.
- Editor-internal optional bindings such as `Ctrl+K` kill-line remain outside the blocker scope unless they are promoted as required visible workflow. The footer overlay issue above is different because it is visible shortcut-help UI.
