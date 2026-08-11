# C8/M6 Markdown Workflow Surface And Remote SSH Acceptance

Status: completed

## Summary

M6 made plugin fallback output more coherent for Workflow Request Input and added the Markdown surface contract for future Workflow/Subagent rendering paths. The final audit blocker was fixed by wiring the Request Input Markdown surface into live app-server plugin-facing fallback emission.

## Implementation

- Added `codex_app_server::workflow_markdown_surface` with a Markdown-first surface contract for:
  - Workflow summary headings
  - separate Plan/SubPlan progress counts
  - Subagent prompt/report sections with original and derived translation labels
  - concise tool output summaries
  - Request Input fallback Markdown
- Added `workflow_surface_fallback_prompt(params)` to preserve the existing M5 fallback body and wrap it with:
  - `## Workflow Surface`
  - `### Request Input`
  - `Request Input fallback for plugin clients without native Ask UI.`
- Wired live unsupported native Ask fallback in `bespoke_event_handling.rs` to emit the wrapped Markdown as `ThreadItem::AgentMessage`.
- Wired recovered pending unsupported Ask fallback replay in `thread_lifecycle.rs` to emit the same wrapped Markdown.
- Added `.pipeline/reviews/C8/M6/acceptance-checklist.md` for real Remote SSH manual acceptance evidence collection.

## Validation

- `cargo test -p codex-app-server --test all workflow_required_decisions_without_native_ask_ui_render_polished_markdown_cards -- --nocapture`: 1 passed.
- `cargo test -p codex-app-server --test all request_user_input -- --nocapture`: 11 passed.
- `cargo test -p codex-app-server --test all workflow_markdown_surface -- --nocapture`: 2 passed.
- `cargo fmt --check --package codex-app-server`: passed with the existing stable rustfmt warning.
- Scoped `git diff --check`: passed.

## Review

- Initial audit returned `needs_changes` because the helper was not wired into live app-server/plugin thread emission.
- Audit-fix tests added live `ThreadItem::AgentMessage` coverage for unsupported native Ask fallback.
- Audit-fix implementation wired both live fallback and recovered pending fallback replay to the M6 wrapper.
- Audit recheck returned `pass`.

## Manual Acceptance Note

Remote SSH manual acceptance is not claimed as completed by this report. The checklist exists at `.pipeline/reviews/C8/M6/acceptance-checklist.md` and should be filled with observed Remote SSH plugin evidence before any release or external acceptance claim.
