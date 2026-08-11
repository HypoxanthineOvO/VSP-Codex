# C7/M6 Test Worker Report

## Changed Files

- `codex-rs/tui/src/chatwidget/tests/plan_mode.rs`
- `codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__workflow_cycle_plan_surface_split_progress.snap`
- `codex-rs/core/tests/suite/workflow_routing.rs`
- `codex-rs/tui/tests/suite/workflow_slash_routing.rs`

## Tests Added

- `workflow_cycle_plan_surface_shows_current_subplan_and_root_progress_separately`
  - Builds a C7 PlanState with root milestones M1-M8.
  - Expects current M6 SubPlan expanded, future M7/M8 SubPlans collapsed.
  - Expects split progress text `1/5 | 5/8`.
  - Rejects mixed progress such as a combined root+SubPlan count.
- `workflow_runtime_guidance_specializes_questions_plan_and_subagents_without_bypassing_gates`
  - Tightened existing guidance assertion so Workflow `update_plan` guidance must name phase, worker, validation, and final response freshness boundaries.
- `session_workflow_guidance_uses_selected_turn_environment_cwd_without_deprecated_allowance`
  - Red test for removing deprecated `TurnContext::cwd` allowances from model-visible session prompt paths.
- Workflow slash routing tests for `/hw:status`, unknown `/hw:*`, and ordinary slash non-capture were added and currently pass.

## Commands Run

- `cargo fmt --check`
  - Passes after test formatting adjustments.
- `cargo test -p codex-tui workflow_cycle_plan_surface_shows_current_subplan_and_root_progress_separately -- --nocapture`
  - Fails as expected.
- `cargo test -p codex-core workflow_routing -- --nocapture`
  - Fails as expected.
- `cargo test -p codex-tui workflow_slash_routing -- --nocapture`
  - Passes: 8 passed.
- `git diff --check -- codex-rs/core/tests/suite/workflow_routing.rs codex-rs/tui/src/chatwidget/tests/plan_mode.rs codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__workflow_cycle_plan_surface_split_progress.snap codex-rs/tui/tests/suite/workflow_slash_routing.rs`
  - Passes.

## Expected Failure Points

- TUI Plan panel currently renders mixed progress `6/15` and expands future M7/M8 SubPlans. M6 expects split progress `1/5 | 5/8` and only the current M6 SubPlan expanded.
- Core Workflow guidance currently says only `For visible planning discipline, use update_plan`; it does not require phase, worker, validation, and final response freshness triggers.
- Build/test output still reports deprecated `TurnContext::cwd` usage in `core/src/session/mod.rs`; the red test also flags remaining deprecated allowances in model-visible prompt construction.

## GREEN Snapshot Closure - 2026-05-30T06:31:58+08:00

- Reviewed `.snap.new` for `workflow_cycle_plan_surface_split_progress`; it preserved `Cycle 7 Plan (1/5 | 5/8)`, expanded current M6 SubPlan content, and collapsed future M7/M8 SubPlans.
- Chose test-side snapshot cropping over accepting the composer tail so the snapshot remains focused on the plan surface while full render assertions still guard the behavior.
- Command: `cd codex-rs && cargo test -p codex-tui workflow_cycle_plan_surface_shows_current_subplan_and_root_progress_separately -- --nocapture`
- Result: GREEN, 1 passed; no remaining `.snap.new` for this snapshot.
