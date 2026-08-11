# C7/M3 Audit Report

## Verdict

pass

Auditor: 孟轲 the 10th (`019e7540-1f9f-7261-8db1-e92686da729e`)

Timestamp: 2026-05-30T03:41:55+08:00

## Checked Scope

- `codex-rs/tui/src/history_cell/base.rs`
- `codex-rs/tui/src/history_cell/mcp.rs`
- `codex-rs/tui/src/hypo_translate.rs`
- `codex-rs/tui/src/history_cell/tests.rs`
- TUI translation call sites and usages for `TranslationSource`, `TranslationScope`, `TranslationDisplay`, `wrap_translated`, and `wrap_collab_translated`.

## Evidence

- `wrap_translated` now only wraps `ReasoningSummaryCell` when `translation_source()` exists; exec and MCP paths no longer create `TranslatedHistoryCell`.
- `wrap_collab_translated` returns the original `PlainHistoryCell`, so collaboration lifecycle/status rows are not append-translated.
- `McpToolCallCell` no longer exposes `translation_source`; display/raw paths still render literal `CallToolResult` content through `render_content_block`.
- `TranslationScope` and `TranslationDisplay` now only contain `Reasoning` and `Replace`, matching the intended prose-only translation boundary.
- Tests cover the prior failure modes for completed exec output, completed MCP output, and collaboration status rows, and include a positive guard that reasoning summaries still translate.

## Validation Reviewed

- `cargo test -p codex-tui translated_wrapping_skips_completed_exec_tool_output -- --nocapture`
- `cargo test -p codex-tui translated_wrapping_skips_completed_mcp_tool_output -- --nocapture`
- `cargo test -p codex-tui collab_lifecycle_status_lines_are_not_append_translated -- --nocapture`
- `cargo test -p codex-tui reasoning_summary_still_uses_translation_wrapper -- --nocapture`
- `cargo test -p codex-tui hypo_translate -- --nocapture`
- `cargo test -p codex-tui translated -- --nocapture`
- `cargo fmt --check`
- `git diff --check`

## Non-Blocking Notes

- The wider `cargo test -p codex-tui history_cell -- --nocapture` filter still fails on `session_info_availability_nux_tooltip_snapshot`, but the failure is unrelated to M3. The snapshot pins `v0.134.0-vsp.4`, while `codex-rs/Cargo.toml` uses `0.134.0-vsp.5`.
- The new negative tests assert rich rendering and wrapper absence, not separate raw/transcript lines. Because no translation wrapper is created, raw/transcript output remains literal by construction.
- Reasoning summaries can still contain technical literals and may be translated as part of intended reasoning prose; this is consistent with the M3 boundary.
