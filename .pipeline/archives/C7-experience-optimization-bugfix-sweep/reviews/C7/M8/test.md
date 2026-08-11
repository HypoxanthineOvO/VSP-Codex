# C7/M8 Test Worker Report

## Changed Files

- `codex-rs/tui/src/slash_command.rs`
- `codex-rs/tui/src/theme_picker.rs`
- `codex-rs/tui/src/app/config_persistence.rs`
- `.pipeline/reviews/C7/M8/test.md`

## Tests Added

- `c7_m8_theme_command_describes_unified_appearance_entry`
  - Expects `/theme` to be described as the unified Appearance entry.
  - Requires the description to mention both UI palette and syntax theme access.
- `c7_m8_ui_theme_command_remains_palette_shortcut`
  - Expects `/ui-theme` to remain explicitly scoped to the UI palette.
  - Requires the label/description to make shortcut semantics clear.
- `c7_m8_theme_entry_opens_appearance_picker_with_palette_and_syntax_actions`
  - Compile-red test for the expected unified Appearance picker helper:
    `build_appearance_picker_params(current_ui_theme, current_syntax_theme, codex_home, terminal_width)`.
  - Expects separate UI palette and Syntax theme actions/sections from the `/theme` entry.
- `c7_m8_sync_tui_ui_theme_selection_does_not_change_syntax_theme`
  - Locks `[tui].ui_theme` changes to UI palette state only.
  - Verifies app config and chat-widget config keep the existing `[tui].theme`.
- `c7_m8_sync_tui_theme_selection_does_not_change_ui_palette`
  - Locks `[tui].theme` changes to syntax theme state only.
  - Verifies app config and chat-widget config keep the existing `[tui].ui_theme`.

## Commands Run

```sh
cd /home/heyx/Codex-VSP/codex-rs
cargo test -p codex-tui c7_m8 --no-run
```

Expected result: red.

Actual result: red at compile time:

```text
error[E0425]: cannot find function `build_appearance_picker_params` in this scope
   --> tui/src/theme_picker.rs:662:22
```

The command also emitted an unrelated existing deprecation warning from `codex-core`.

## Expected Failure Points

1. `theme_picker.rs` does not yet expose `build_appearance_picker_params(...)`.
2. Once the helper exists, `/theme` command description is still expected to fail until it is changed from syntax-only wording to a unified Appearance/Theme description.
3. `/ui-theme` command description is expected to fail until it explicitly presents the route as a UI palette shortcut.

## Minimal Production Work For Implement Worker

1. Add a unified Appearance picker builder, or equivalent helper/API, matching the tested intent:
   - current UI palette input
   - current syntax theme input
   - optional `codex_home`
   - optional terminal width
   - returns `SelectionViewParams`
2. Route `/theme` through the unified Appearance picker instead of directly opening the syntax-only picker.
3. Keep `/ui-theme` routed directly to the UI palette picker.
4. Update slash command descriptions so:
   - `/theme` is the primary Appearance/Theme entry and mentions UI palette plus syntax theme.
   - `/ui-theme` is a UI palette shortcut and does not imply syntax theme selection.
5. Preserve separate persistence for `[tui].theme` and `[tui].ui_theme`.
