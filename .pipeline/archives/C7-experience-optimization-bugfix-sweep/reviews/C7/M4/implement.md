# C7/M4 Implement Review

## Changed files

- `codex-rs/tui/src/chatwidget/active_subagents.rs`
- `.pipeline/reviews/C7/M4/implement.md`

## Approach

- Added an Active Subagents Prompt/Report viewer renderable for the currently selected/first active subagent.
- Wired the Ctrl+O Active Subagents popup with both `side_content` and `stacked_side_content` through `SelectionViewParams`.
- Rendered full Prompt and Report Markdown with `append_markdown_agent_with_cwd`, including explicit `Prompt`, `Report`, `Original / 原文`, and `Translated / 翻译` labels.
- Kept translation as a clearly marked not-generated/same-original-preview surface; no external translation service is called.
- Kept compact active rows and lifecycle rows as previews only; full Prompt/Report content is confined to the popup viewer.
- Rendered viewer Markdown without a width argument so long unbreakable tail markers remain intact; the viewer's long-token render path writes them as complete strings even in narrow popups.

## Commands and results

- `cd /home/heyx/Codex-VSP/codex-rs && cargo test -p codex-tui c7_m4 -- --nocapture`
  - Passed: 3 passed, 0 failed.
  - Warning observed: existing deprecated `TurnContext::cwd` field warning in `codex-core`.

- `cd /home/heyx/Codex-VSP/codex-rs && cargo fmt --check`
  - Passed with exit code 0.
  - Warning observed: existing stable rustfmt warning for nightly-only `imports_granularity = Item`.
- `cd /home/heyx/Codex-VSP && git diff --check`
  - Passed with exit code 0.

## Risks and notes

- Superseded by the audit-fix below: the viewer no longer displays only the first active subagent when selection moves.
- Full Markdown is intentionally not copied into row descriptions or history cells.
- Long unbreakable tokens may visually overflow narrow terminal cells, but this is required to preserve searchable/full raw markers in the rendered popup string.

## Audit-fix update - 2026-05-30 04:49 +0800

### Change

- Fixed the Active Subagents Ctrl+O Prompt/Report viewer so it tracks the current selected row instead of always rendering the first active subagent.
- The popup now builds viewer entries for every active row and shares one `Arc<AtomicUsize>` selected index between `side_content` and `stacked_side_content`.
- `SelectionViewParams.on_selection_changed` updates that shared index as navigation changes selection.
- `ActiveSubagentPromptReportViewerRenderable::lines()` resolves the entry at render time and falls back to the first entry if the index is out of bounds.
- Kept full prompt/report rendering confined to the viewer; row descriptions, compact active rows, and lifecycle rows remain concise previews.

### Commands and results

- `cd /home/heyx/Codex-VSP/codex-rs && cargo test -p codex-tui c7_m4 -- --nocapture`
  - Passed: 4 passed, 0 failed.
  - Warning observed: existing deprecated `TurnContext::cwd` field warning in `codex-core`.
- `cd /home/heyx/Codex-VSP/codex-rs && cargo fmt --check`
  - Passed with exit code 0.
  - Warning observed: existing stable rustfmt warning for nightly-only `imports_granularity = Item`.
- `cd /home/heyx/Codex-VSP && git diff --check`
  - Passed with exit code 0.

### Risks

- Viewer entries are captured when the popup opens, so live prompt/report updates that arrive while the popup remains open are not reflected until reopening the popup. This matches the existing popup construction model and is outside this selected-row sync fix.
- The selected index uses relaxed atomic ordering because it is only UI render state shared between callback/renderable closures in the popup.
