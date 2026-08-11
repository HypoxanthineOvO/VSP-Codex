# M4 VSCode-Terminal-Safe Keymap + Composer Prominence

## Objective

修复 VSCode 集成终端中的必要快捷键冲突，并让输入框更像当前可输入的主操作面。

## Source Of Truth

- `.plan-state/decompose.yaml` M4
- User clarified the main keyboard pain is VSCode terminal conflicts such as `Ctrl+J/K`
- Existing Ask UI currently has `ctrl+j/k to reorder` and `ctrl+p/n change question`

## Subworker Assignment Plan

Status: authorized, strict worker separation.

- `test`
  - Owns keymap constraint tests, Ask overlay snapshots, and composer focused/unfocused snapshots.
  - Expected artifact: `.pipeline/reviews/C4/M4/test.md`
- `implement`
  - Owns default keymap/hint updates and composer visual changes.
  - Expected artifact: `.pipeline/reviews/C4/M4/implement.md`
- `audit`
  - Checks forbidden required chords, terminal-native copy behavior, and scope boundaries.
  - Expected artifact: `.pipeline/reviews/C4/M4/audit.md`

## Technical Solution

Adjust default required bindings and contextual hints to avoid VSCode-terminal-reserved chords. Keep `/keymap` customizability, but default workflows must use arrows, Tab/Shift+Tab, Enter/Esc, and slash commands. Composer visual prominence is a small render/style change, not a full focus-stack redesign.

## Technical Route

1. Add tests asserting essential defaults do not require `Ctrl+J/K`, `Ctrl+P`, `Ctrl+Shift+P`, `Alt`, or `Ctrl+C`.
2. Update `request_user_input` ranking reorder and multi-question navigation hints/handlers away from Ctrl chords.
3. Review footer shortcut overlay and remove high-risk keys as required actions.
4. Preserve custom keymap support for users who explicitly remap.
5. Strengthen composer focused/unfocused border, subtle accent/background, and focus hint.
6. Update snapshots for Ask ranking, multi-question navigation, shortcut overlay, and composer.

## Research Required

Resolved: current conflicting Ask UI and footer hints were found by grep.

## Risks And Alternatives

- Risk: removing Ctrl+J newline surprises users. Mitigate with safe defaults and configurable keymap.
- Risk: Tab may also be intercepted in some hosts. Keep Enter/Esc/slash fallbacks.
- Deferred: complete command palette, focus stack, and keyboard copy/selection system.

## Validation Path

Commands:

- `just test -p codex-tui keymap`
- `just test -p codex-tui request_user_input`
- `just test -p codex-tui chat_composer`

Pass signal: forbidden chords are not required/default for essential actions; Ask UI no longer instructs Ctrl+J/K or Ctrl+P/N; composer is visually clearer.

Fail signal: VSCode-conflicting keys remain required, Ctrl+C becomes copy primary, or composer still blends into history.

## Audit Focus

- User's VSCode terminal conflict pain is solved first.
- No broad keybinding rewrite.
- Mouse selection/copy is not made worse by new visuals.
