# M3 Native Workflow Runtime Routing Report

Status: completed

## Summary

M3 makes Hypo-Workflow behavior visible to the runtime when the current workspace is a Workflow project and adds a small native route parser for Workflow slash forms.

Implemented behavior:

- Detect `.pipeline/` by walking cwd ancestors.
- Inject concise Hypo-Workflow runtime guidance into first-turn context for Workflow projects only.
- Keep non-Workflow projects free of Workflow-specific guidance.
- Prefer `request_user_input` for required user decisions and `update_plan` for visible planning discipline.
- Encourage Subagent decomposition for suitable Workflow roles while preserving destructive-operation, system-level dependency, remote side-effect, and approval-boundary gates.
- Parse `/hw:*` and `$hypo-workflow:*` as Workflow routes in TUI helper code.
- Reject unrelated slash commands, ordinary text, and empty input as non-Workflow routes.

## Files Changed

- `codex-rs/core/src/workflow_routing.rs`
- `codex-rs/core/src/session/mod.rs`
- `codex-rs/core/src/lib.rs`
- `codex-rs/core/tests/suite/workflow_routing.rs`
- `codex-rs/core/tests/suite/mod.rs`
- `codex-rs/tui/src/workflow_slash_routing.rs`
- `codex-rs/tui/src/lib.rs`
- `codex-rs/tui/tests/suite/workflow_slash_routing.rs`
- `codex-rs/tui/tests/suite/mod.rs`

## Validation

Final validation passed:

- `cargo test -p codex-core workflow_routing -- --nocapture`: 4 passed
- `cargo test -p codex-tui workflow_slash_routing`: 4 passed
- `cargo fmt --check`: exited 0 with existing stable rustfmt warnings
- `git diff --check`: passed

## Review Outcome

Initial replacement audit returned `needs_changes` because the TUI parser falsely classified unrelated input as a Workflow route. The fix changed the parser to return `Option<WorkflowSlashRoute>` and added negative tests. Reaudit passed.

Worker Separation evidence:

- Original M3 test worker produced usable tests but was closed while running; replacement test worker completed.
- Original M3 implement worker left production changes but was closed while running; replacement implement worker completed.
- Original M3 audit worker became `not_found`; replacement audit returned `needs_changes`.
- Fix-round test, implement, and reaudit workers completed and closed.

The lifecycle gaps are recorded in state/log/progress and did not hide any acceptance risk after replacement and fix-round evidence.
