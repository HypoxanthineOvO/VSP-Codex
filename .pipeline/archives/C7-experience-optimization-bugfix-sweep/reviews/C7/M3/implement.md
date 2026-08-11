# C7/M3 Implement Report

## Changed files

- `codex-rs/tui/src/history_cell/base.rs`
- `codex-rs/tui/src/history_cell/mcp.rs`
- `codex-rs/tui/src/hypo_translate.rs`

## Implementation

- Restricted `wrap_translated` to only wrap `ReasoningSummaryCell` with `TranslatedHistoryCell`.
- Kept collaboration lifecycle/status rows literal by making `wrap_collab_translated` return the original `PlainHistoryCell`.
- Removed MCP tool-output translation source plumbing so completed MCP tool output cannot be blind append translated.
- Removed now-unused `CollabTranscript`, `ToolOutput`, and `Append` translation policy variants. Runtime translation remains available for reasoning/prose through replace rendering.

## Command results

- `cargo test -p codex-tui translated_wrapping_skips_completed_exec_tool_output -- --nocapture` - passed.
- `cargo test -p codex-tui translated_wrapping_skips_completed_mcp_tool_output -- --nocapture` - passed.
- `cargo test -p codex-tui collab_lifecycle_status_lines_are_not_append_translated -- --nocapture` - passed.
- `cargo test -p codex-tui reasoning_summary_still_uses_translation_wrapper -- --nocapture` - passed.
- `cargo test -p codex-tui hypo_translate -- --nocapture` - passed.
- `cargo test -p codex-tui history_cell -- --nocapture` - failed on pre-existing snapshot version drift: expected `v0.134.0-vsp.4`, rendered `v0.134.0-vsp.5` in `session_info_availability_nux_tooltip_snapshot`. No snapshot was updated; generated `.snap.new` was removed.
- `cargo fmt --check` - passed, with existing stable-rust warnings about ignored unstable `imports_granularity = Item`.
- `git diff --check` - passed.

## Remaining risk

- Future non-reasoning prose translation surfaces will need an explicit policy/viewer path instead of using the removed blind append mode.
- The broader `history_cell` filter remains blocked by repository version snapshot drift unrelated to this M3 change.
