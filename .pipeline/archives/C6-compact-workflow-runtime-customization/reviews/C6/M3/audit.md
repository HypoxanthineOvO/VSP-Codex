# C6 M3 Audit

Verdict: needs_changes

Auditor:

- Replacement audit worker: `019e72ec-efee-7180-ad37-da7f4b0d8bb0`
- Original audit worker `019e72eb-5fb4-7d83-9164-6e0308ae2bb9` became `not_found` before writing an artifact and is not counted as complete lifecycle evidence.

## Findings

1. `codex-rs/tui/src/workflow_slash_routing.rs` currently parses unrelated input as Hypo-Workflow routing.
   - `parse_workflow_slash_command` uses `unwrap_or(trimmed)` and always returns `WorkflowSlashRoute`.
   - `WorkflowSlashRoute::is_hypo_workflow()` always returns `true`.
   - As a result, strings such as `/review foo`, `/compact`, or `hello` can be falsely classified as Hypo-Workflow commands unless every caller adds an external guard.
   - Audit did not find existing caller-side protection.
   - Required fix: return no route for non-Workflow prefixes, and add negative tests.

## Passed Checks

- Activation guard: core runtime guidance is guarded by `.pipeline/` ancestor detection.
- Prompt size: injected guidance is concise and does not copy large Skill bodies.
- Permission gates: guidance preserves confirmation requirements for destructive operations, system-level dependency installation, remote side effects, and configured approval boundaries.
- Test authenticity: core tests exercise real first-turn request capture and do not depend on skipped remote-env tests.
- Worker Separation evidence: original test/implement lifecycles were incomplete but recorded honestly; replacement test/implement evidence is sufficient to continue after the routing fix.

## Validation Observed

- `cargo test -p codex-core workflow_routing -- --nocapture`: 4 passed
- `cargo test -p codex-tui hw_plan_slash_command_is_recognized_as_native_workflow_route --no-run`: passed
- `cargo fmt --check`: exited 0 with existing stable rustfmt warnings
- `git diff --check`: passed

## Reaudit After Fix

Verdict: pass

Reaudit worker: `019e72f4-a8ba-7f51-9254-5ab207c5b0e5`

Result:

- `codex-rs/tui/src/workflow_slash_routing.rs` now returns `Option<WorkflowSlashRoute>`.
- Only `/hw:` and `$hypo-workflow:` prefixes return `Some(route)`.
- Unrelated slash commands, plain text, and empty input return `None`.
- TUI tests cover both positive Workflow routes and negative unrelated input.
- Core Workflow runtime guidance remains guarded by `.pipeline/`, stays concise, and preserves Request Questions, Plan, Subagent, destructive-operation, system-level dependency, remote side-effect, and approval-boundary guidance.
- Original worker lifecycle gaps are recorded honestly; replacement and fix-round worker evidence is sufficient for M3 acceptance.

Validation observed:

- `cargo test -p codex-core workflow_routing -- --nocapture`: 4 passed
- `cargo test -p codex-tui workflow_slash_routing`: 4 passed
- `cargo fmt --check`: exited 0 with existing stable rustfmt warnings
- `git diff --check`: passed
