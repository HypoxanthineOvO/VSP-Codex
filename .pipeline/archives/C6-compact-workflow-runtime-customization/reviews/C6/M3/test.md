# C6/M3 Test Worker Report

## Scope

Milestone: M3 `Native Workflow Runtime Routing`

Role: test worker

Production code touched: no

## Changed Files

- `codex-rs/core/tests/suite/workflow_routing.rs`
- `codex-rs/core/tests/suite/mod.rs`
- `codex-rs/tui/tests/suite/workflow_slash_routing.rs`
- `codex-rs/tui/tests/suite/mod.rs`
- `.pipeline/reviews/C6/M3/test.md`

## Tests Added

- `workflow_project_gets_native_hypo_workflow_runtime_guidance`
  - Builds a real first turn request in a temp workspace containing `.pipeline/`.
  - Expects model-visible runtime context to include concise native Hypo-Workflow guidance.

- `non_workflow_project_omits_hypo_workflow_runtime_guidance`
  - Builds the same real first turn request in a workspace without `.pipeline/`.
  - Expects no Workflow-specific runtime context.

- `workflow_runtime_guidance_routes_hw_plan_as_native_command`
  - Submits `/hw:plan refine this change` through a real turn request in a `.pipeline/` workspace.
  - Expects Workflow runtime guidance to make `/hw:plan` or `$hypo-workflow:plan` routing visible.

- `workflow_runtime_guidance_specializes_questions_plan_and_subagents_without_bypassing_gates`
  - Builds a real first turn request in a `.pipeline/` workspace.
  - Expects Workflow mode guidance for `request_user_input`, `update_plan`, Subagent use, and preservation of destructive/system-level dependency confirmation gates.

- `hw_plan_slash_command_is_recognized_as_native_workflow_route`
  - Lightweight TUI integration red test.
  - Expects a native `codex_tui::workflow_slash_routing::parse_workflow_slash_command` API that recognizes `/hw:plan refine`.

## Commands

```bash
cargo test -p codex-core workflow_routing --no-run
```

Result: passed compile.

```bash
cargo test -p codex-core workflow_routing -- --nocapture
```

Result: red as expected.

- `non_workflow_project_omits_hypo_workflow_runtime_guidance`: passed.
- `workflow_project_gets_native_hypo_workflow_runtime_guidance`: failed because `.pipeline/` projects do not inject native Hypo-Workflow guidance.
- `workflow_runtime_guidance_routes_hw_plan_as_native_command`: failed because `/hw:plan` routing guidance is absent.
- `workflow_runtime_guidance_specializes_questions_plan_and_subagents_without_bypassing_gates`: failed because Workflow-specific `request_user_input`, `update_plan`, Subagent, and gate-preservation guidance is absent.

```bash
cargo test -p codex-tui hw_plan_slash_command_is_recognized_as_native_workflow_route --no-run
```

Result: red as expected with `E0433`: `could not find workflow_slash_routing in codex_tui`.

## Expected Red Failure Cause

The current runtime only exposes ordinary permissions, skills list, and environment context. It does not yet detect `.pipeline/` workspaces as Workflow-aware projects, inject native Hypo-Workflow guidance, or expose a native `/hw:*` routing surface in TUI.

## Notes

- The core context tests are real request-level tests, not static string scans over source files.
- The TUI test is intentionally minimal and compile-red to define the routing API/registry seam without editing `codex-rs/tui/src/**`.
- No production/runtime files were edited by this worker.

## Replacement Validation

Replacement worker: C6/M3 test worker lifecycle closeout

Production code touched: no

Test files adjusted: no

Report file touched: yes, this replacement validation section only

### Commands Re-run

```bash
cargo test -p codex-core workflow_routing -- --nocapture
```

Run from `/home/heyx/Codex-VSP`: failed before test execution because this repository's Cargo workspace is under `codex-rs/`, not the repository root:

```text
error: could not find `Cargo.toml` in `/home/heyx/Codex-VSP` or any parent directory
```

Run from `/home/heyx/Codex-VSP/codex-rs`: red as expected.

- `non_workflow_project_omits_hypo_workflow_runtime_guidance`: passed.
- `workflow_project_gets_native_hypo_workflow_runtime_guidance`: failed because `.pipeline/` projects still do not inject native Hypo-Workflow runtime guidance.
- `workflow_runtime_guidance_routes_hw_plan_as_native_command`: failed because `/hw:plan` routing guidance is still absent.
- `workflow_runtime_guidance_specializes_questions_plan_and_subagents_without_bypassing_gates`: failed because Workflow-specific `request_user_input`, `update_plan`, Subagent, and gate-preservation guidance is still absent.

```bash
cargo test -p codex-tui hw_plan_slash_command_is_recognized_as_native_workflow_route --no-run
```

Run from `/home/heyx/Codex-VSP`: failed before compilation for the same missing root `Cargo.toml` reason.

Run from `/home/heyx/Codex-VSP/codex-rs`: red as expected with `E0433`:

```text
could not find `workflow_slash_routing` in `codex_tui`
```

### Replacement Conclusion

The existing M3 red tests are syntactically registered and credible. No test correction was needed. The observed failures match the intended missing production behavior: native Workflow runtime guidance and TUI `/hw:*` routing API have not been implemented yet.

## Test Fix Validation

Worker: C6/M3 test-fix worker

Production code touched: no

Files edited:

- `codex-rs/core/tests/suite/workflow_routing.rs`
- `.pipeline/reviews/C6/M3/test.md`

### Fix

Changed the core `workflow_routing` request harness from `build_with_remote_env(&server)` to the ordinary local `build(&server)`. The tests still use the same mock SSE server and still inspect real model-visible request messages, but they no longer depend on `CODEX_TEST_REMOTE_ENV` or `CODEX_TEST_REMOTE_EXEC_SERVER_URL`.

Assertions were not weakened. The four core tests still check:

- `.pipeline/` projects get native Hypo-Workflow runtime guidance.
- Non-Workflow projects omit Workflow-specific guidance.
- `/hw:plan` route guidance is visible.
- `request_user_input`, `update_plan`, Subagent guidance, and destructive/system-level dependency confirmation gates remain visible.

### Command

Run from `/home/heyx/Codex-VSP/codex-rs`:

```bash
cargo test -p codex-core workflow_routing -- --nocapture
```

Result: passed.

- `workflow_project_gets_native_hypo_workflow_runtime_guidance`: ok
- `non_workflow_project_omits_hypo_workflow_runtime_guidance`: ok
- `workflow_runtime_guidance_routes_hw_plan_as_native_command`: ok
- `workflow_runtime_guidance_specializes_questions_plan_and_subagents_without_bypassing_gates`: ok

No `Skipping test because CODEX_TEST_REMOTE_ENV is not set` output appeared.

## Needs Changes Red Tests

Worker: C6/M3 needs_changes test-fix worker

Production code touched: no

Files edited:

- `codex-rs/tui/tests/suite/workflow_slash_routing.rs`
- `.pipeline/reviews/C6/M3/test.md`

### Added Tests

- `hypo_workflow_dollar_command_is_recognized_as_native_workflow_route`
  - Expects `$hypo-workflow:*` inputs to continue routing as native Hypo-Workflow commands.
- `unrelated_slash_commands_are_not_workflow_routes`
  - Expects `/review foo`, `/compact`, and `/model gpt-5` to return no Workflow route.
- `plain_text_is_not_a_workflow_route`
  - Expects ordinary text and empty input to return no Workflow route.

The existing `/hw:plan` positive test was updated to expect an optional route, matching the intended contract that only `/hw:*` and `$hypo-workflow:*` are classified as Workflow routes.

### Command

Run from `/home/heyx/Codex-VSP/codex-rs`:

```bash
cargo test -p codex-tui workflow_slash_routing --no-run
```

Result: red as expected.

Output summary:

- `E0599`: no method named `expect` found for struct `WorkflowSlashRoute`
- `E0599`: no method named `is_none` found for struct `WorkflowSlashRoute`
- `could not compile codex-tui (test "all") due to 4 previous errors`

### Expected Red Failure Cause

`parse_workflow_slash_command` currently returns `WorkflowSlashRoute` for every input and falls back to the trimmed input when neither `/hw:` nor `$hypo-workflow:` is present. The tests intentionally require an `Option`-like API so unrelated slash commands and plain text cannot be falsely classified as Hypo-Workflow routes.
