# C6/M3 Implement Report

## Scope

Replacement implement worker for M3 runtime Workflow routing and TUI slash routing.

Allowed production scope:

- `codex-rs/core/src/**` runtime routing related code
- `codex-rs/tui/src/**` slash routing parser/export related code
- `.pipeline/reviews/C6/M3/implement.md`

No test, Cargo, fixture, or snapshot files were edited by this worker.

## Production State Reviewed

- `codex-rs/core/src/workflow_routing.rs`
  - Adds `.pipeline/` workspace detection.
  - Injects model-visible Hypo-Workflow runtime guidance only when a `.pipeline/` directory exists in the current workspace ancestry.
  - Guidance explicitly covers `Hypo-Workflow`, `.pipeline`, `/hw:plan`, `$hypo-workflow:plan`, `request_user_input`, `update_plan`, `Subagent`, destructive operation confirmation, system-level dependency confirmation, and remote/external side-effect boundaries.

- `codex-rs/core/src/session/mod.rs`
  - Adds workflow runtime guidance to developer sections during turn context construction.
  - Uses the primary turn environment cwd when present, falling back to the legacy turn cwd.

- `codex-rs/tui/src/workflow_slash_routing.rs`
  - Adds a native parser/export surface for `/hw:` and `$hypo-workflow:` style Workflow slash routes.
  - Current parser accepts `/hw:plan refine` and returns command `plan`, arguments `refine`, and `is_hypo_workflow() == true`.

- `codex-rs/tui/src/lib.rs`
  - Publicly exports `workflow_slash_routing`.

## Changes Made By This Worker

No production changes were required in this replacement pass. Existing M3 production changes satisfied the requested tests and M3 behavior contract.

This worker only created this implementation report.

## Verification

Commands run from `/home/heyx/Codex-VSP/codex-rs`:

```bash
cargo test -p codex-core workflow_routing -- --nocapture
```

Result: passed.

- `workflow_project_gets_native_hypo_workflow_runtime_guidance`: ok
- `non_workflow_project_omits_hypo_workflow_runtime_guidance`: ok
- `workflow_runtime_guidance_routes_hw_plan_as_native_command`: ok
- `workflow_runtime_guidance_specializes_questions_plan_and_subagents_without_bypassing_gates`: ok
- Overall: 4 passed, 0 failed

```bash
cargo test -p codex-tui hw_plan_slash_command_is_recognized_as_native_workflow_route --no-run
```

Result: passed. Test targets compiled successfully.

```bash
cargo fmt --check
```

Result: failed due to a formatting diff in `codex-rs/tui/tests/suite/mod.rs` only:

```diff
-mod workflow_slash_routing;
 mod vt100_history;
 mod vt100_live_commit;
+mod workflow_slash_routing;
```

That file is a test file and is outside this worker's allowed edit scope, so this worker did not modify it. The command also emitted the repository's existing stable-toolchain warnings about nightly-only `imports_granularity = Item`.

## Remaining Issues

- `cargo fmt --check` remains non-green until `codex-rs/tui/tests/suite/mod.rs` is reordered by a worker authorized to edit test files.
- No known M3 production behavior issue remains from this replacement implement pass.

## Needs Changes Fix - Parser Prefix Guard

Timestamp: `2026-05-29T16:56:11+08:00`

Changed paths:

- `codex-rs/tui/src/workflow_slash_routing.rs`

Fix:

- Changed `parse_workflow_slash_command` to return `Option<WorkflowSlashRoute>`.
- The parser now returns `Some(route)` only for explicit `/hw:` and `$hypo-workflow:` prefixes.
- Unrelated slash commands, plain text, and empty input now return `None` instead of being classified as Hypo-Workflow routes.

Validation commands run from `/home/heyx/Codex-VSP/codex-rs`:

```bash
cargo test -p codex-tui workflow_slash_routing --no-run
```

Result: passed. Test targets compiled successfully.

```bash
cargo test -p codex-tui workflow_slash_routing
```

Result: passed.

- `hw_plan_slash_command_is_recognized_as_native_workflow_route`: ok
- `hypo_workflow_dollar_command_is_recognized_as_native_workflow_route`: ok
- `plain_text_is_not_a_workflow_route`: ok
- `unrelated_slash_commands_are_not_workflow_routes`: ok
- Overall focused suite: 4 passed, 0 failed
