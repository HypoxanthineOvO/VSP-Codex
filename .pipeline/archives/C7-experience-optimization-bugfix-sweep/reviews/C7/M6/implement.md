# C7/M6 Implement Report

## Handoff

I took over from a stopped implement worker that left a candidate production diff without an implementation report. I preserved the existing C6/C7 conversation-capture changes in `session/mod.rs` and avoided protected Workflow state/log/progress files and test/snapshot edits.

## Final Changes

- `codex-rs/tui/src/chatwidget/plan_panel.rs`
  - Split Workflow Cycle progress as current SubPlan progress plus root Plan progress, e.g. `(1/5 | 5/8)`.
  - Kept future Workflow milestone SubPlans collapsed by expanding only the active root item.
  - Restored non-Workflow plan progress to aggregate counts, so existing generic plan panel samples continue to show `2/4`.
  - Removed the candidate `reserved_bottom_rows` / tail guard behavior from the plan panel.
- `codex-rs/core/src/workflow_routing.rs`
  - Kept the candidate Workflow runtime guidance update requiring `update_plan` refreshes at Workflow phase changes, worker handoffs, validation boundaries, and final response boundaries.
- `codex-rs/core/src/session/mod.rs`
  - Switched model-visible permissions, Workflow guidance, and user-instructions cwd resolution to the selected turn environment cwd with config cwd fallback.
  - Did not roll back the pre-existing Workflow conversation capture diff.

## Verification

- PASS: `cargo test -p codex-core workflow_routing -- --nocapture` (`9 passed`)
- PASS: `cargo test -p codex-tui workflow_slash_routing -- --nocapture` (`8 passed`, reported by main-agent validation)
- PASS: `cargo test -p codex-tui plan_panel -- --nocapture` (`5 passed`)
- PASS: `cargo fmt --check`
- PASS: `git diff --check`
- FAIL: `cargo test -p codex-tui workflow_cycle_plan_surface_shows_current_subplan_and_root_progress_separately -- --nocapture`
  - Logic assertions pass, including split progress and collapsed future SubPlans.
  - Remaining failure is snapshot-only: current 24-row `TestBackend` output includes bottom composer/blank rows after the 20-row plan surface once plan-panel reserved rows are removed. I did not edit tests or snapshots per instructions.

## Residual Risk

The production plan-panel behavior is narrowed to the requested semantics, but the focused Workflow snapshot still needs either an accepted snapshot update or a test-side trim/capture adjustment to reconcile the 24-row backend output with the 20-row checked-in snapshot.
