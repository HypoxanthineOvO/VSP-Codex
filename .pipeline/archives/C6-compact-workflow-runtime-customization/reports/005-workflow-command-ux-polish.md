# M5 Workflow Command And UX Polish Report

Status: completed

## Summary

M5 adds a small Workflow command/status helper surface and updates documentation for the C6 Workflow-native behavior without overpromising full UI wiring.

Implemented behavior:

- `parse_workflow_slash_command` continues to parse only `/hw:*` and `$hypo-workflow:*`.
- `workflow_command_palette_entries(true)` returns `/hw:status`, `/hw:plan`, and `/hw:resume` discoverability entries.
- `workflow_command_palette_entries(false)` returns no entries for non-Workflow projects.
- `workflow_status_summary(true)` lists only verified runtime behavior.
- `workflow_status_summary(false)` states Workflow is not detected and does not claim enabled features.
- README, VSP docs, slash command docs, and changelog now describe C6 behavior and limitations honestly.

## Files Changed

- `codex-rs/tui/src/workflow_slash_routing.rs`
- `codex-rs/tui/tests/suite/workflow_slash_routing.rs`
- `README.md`
- `docs/vsp-codex.md`
- `docs/slash_commands.md`
- `CHANGELOG.md`

## Validation

Final validation passed:

- `cargo test -p codex-tui workflow_slash_routing`: 6 passed
- `cargo fmt --check`: exited 0 with existing stable rustfmt warnings
- `git diff --check`: passed

## Review Outcome

M5 audit returned `pass`. The docs explicitly state that `/hw:*` command palette discoverability is helper-level and that full command palette wiring remains a later integration step.
