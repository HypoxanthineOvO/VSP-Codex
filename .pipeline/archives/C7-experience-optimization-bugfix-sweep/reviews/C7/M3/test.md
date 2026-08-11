# C7/M3 Test Report - Tool And Collaboration Output Rendering Boundaries

## Changed files

- `codex-rs/tui/src/history_cell/tests.rs`
- `.pipeline/reviews/C7/M3/test.md`

## Test intent

- `translated_wrapping_skips_completed_exec_tool_output`: proves completed `ExecCell` output should stay literal technical output and should not be wrapped in `TranslatedHistoryCell` or append a translated `  └ ...` block.
- `translated_wrapping_skips_completed_mcp_tool_output`: proves completed `McpToolCallCell` results should stay literal MCP output and should not be wrapped in `TranslatedHistoryCell` or append translated raw results.
- `collab_lifecycle_status_lines_are_not_append_translated`: proves collaboration lifecycle/status rows such as `Waiting for ...` and `Ran sed ...` should remain status output, not append translated duplicates.
- `reasoning_summary_still_uses_translation_wrapper`: positive guard proving intended reasoning/prose translation still calls the translator and renders the translated prose, so an implementation cannot satisfy the boundary by globally disabling translation.

## Commands and results

- `cargo test -p codex-tui translated_wrapping_skips_completed_exec_tool_output -- --nocapture`
  - Result: failed as expected.
  - Red reason: current `wrap_translated` returns `TranslatedHistoryCell` for completed `ExecCell` with output.

- `cargo test -p codex-tui translated_wrapping_skips_completed_mcp_tool_output -- --nocapture`
  - Result: failed as expected.
  - Red reason: current `wrap_translated` returns `TranslatedHistoryCell` for completed `McpToolCallCell` with result content.

- `cargo test -p codex-tui collab_lifecycle_status_lines_are_not_append_translated -- --nocapture`
  - Result: failed as expected.
  - Red reason: current `wrap_collab_translated` always creates a `TranslatedHistoryCell` for the provided `PlainHistoryCell`.

- `cargo test -p codex-tui reasoning_summary_still_uses_translation_wrapper -- --nocapture`
  - Result: passed.
  - Purpose: confirms reasoning summary prose translation is still active.

Note: cargo emitted an existing deprecation warning in `core/src/session/mod.rs` about `TurnContext::cwd`; it is unrelated to these tests.

## Red light summary

M3 has real red coverage. The current production behavior blindly routes completed exec output, MCP result output, and collaboration lifecycle/status rows through append translation. The new tests expect those cells to remain unwrapped/literal while preserving the existing reasoning/prose translation path.

## Uncovered risk

- The tests cover direct history-cell wrapping boundaries, not every chatwidget event path that calls these wrappers.
- The MCP case covers text result content only, not image/audio/resource variants.
- The exec case covers a single completed agent command, not grouped exploring cells or user shell commands.
- No snapshot updates were made.
