# C6/M5 Test Worker Report

## Scope

Role: test worker for Milestone M5 `Workflow Command And UX Polish`.

Allowed scope observed:

- Edited test-owned file: `codex-rs/tui/tests/suite/workflow_slash_routing.rs`
- Wrote this report: `.pipeline/reviews/C6/M5/test.md`
- Did not edit production code, docs, changelog, state, log, or progress files.

## Surfaces Inspected

- `codex-rs/tui/src/slash_command.rs`
  - Built-in slash registry currently contains ordinary Codex commands such as `/status`, `/debug-config`, `/plan`, `/compact`.
  - No first-class `/hw:*` command entry is present in the built-in registry.
- `codex-rs/tui/src/bottom_pane/slash_commands.rs`
  - Command popup/palette is built from `built_in_slash_commands()` plus service-tier commands.
  - No Workflow project detection flag or Workflow-specific command list exists in this seam.
- `codex-rs/tui/src/workflow_slash_routing.rs`
  - Existing public seam parses `/hw:*` and `$hypo-workflow:*` while rejecting unrelated slash commands.
  - This is the most stable executable seam available for a narrow M5 test without touching private widget internals.
- `codex-rs/tui/src/debug_config.rs`
  - `/debug-config` has unit-testable render helpers for config layers and requirements.
  - No Workflow status/debug rendering hook currently exists there.
- `codex-rs/tui/tests/suite/workflow_slash_routing.rs`
  - Existing integration tests already validate M3 route parsing.
- Docs/reports:
  - `.pipeline/reports/001-compact-fallback-context-pack.md` verifies compact fallback behavior.
  - `.pipeline/reports/003-native-workflow-runtime-routing.md` verifies guarded Workflow runtime guidance and route parsing.
  - `.pipeline/reports/004-ordinary-conversation-capture.md` verifies ordinary conversation capture.
  - `README.md`, `docs/vsp-codex.md`, `docs/install.md`, and `CHANGELOG.md` currently do not describe the new M1-M4 Workflow-native behavior, so docs polish remains implementation-owned.

## Tests Added

Added two focused red tests to `codex-rs/tui/tests/suite/workflow_slash_routing.rs`:

- `workflow_commands_are_discoverable_only_for_workflow_projects`
  - Requires a stable UX model function `workflow_command_palette_entries(is_workflow_project: bool)`.
  - Workflow projects should advertise native `/hw:status` and `/hw:plan` entries with Hypo-Workflow descriptions.
  - Non-Workflow projects should not be cluttered with active `/hw:*` command entries.
- `workflow_status_summary_reports_verified_runtime_features`
  - Requires a stable status/debug summary function `workflow_status_summary(is_workflow_project: bool)`.
  - Workflow projects should report:
    - `Workflow project: detected`
    - `compact fallback: enabled`
    - `ordinary capture: enabled`
    - `Subagent prompt audit: enabled by default`
  - Non-Workflow projects should report `Workflow project: not detected` and must not claim compact fallback or ordinary capture are enabled.

These tests intentionally target a small public seam instead of brittle command-palette snapshots. The implementation worker can either implement these helpers and wire them into actual TUI surfaces, or replace the seam with an equivalent stable test-owned API while preserving the same assertions.

## Validation

Command run from `codex-rs/`:

```bash
cargo test -p codex-tui workflow_slash_routing --no-run
```

Result: red / compile-fail as expected.

Relevant failure summary:

```text
error[E0425]: cannot find function `workflow_command_palette_entries` in module `codex_tui::workflow_slash_routing`
error[E0425]: cannot find function `workflow_status_summary` in module `codex_tui::workflow_slash_routing`
error: could not compile `codex-tui` (test "all") due to 4 previous errors
```

Observed existing warning during compile:

```text
warning: use of deprecated field `session::turn_context::TurnContext::cwd`
```

Whitespace check:

```bash
git diff --check -- codex-rs/tui/tests/suite/workflow_slash_routing.rs
```

Result: passed.

## Coverage Status

Red tests now cover the M5 UX contract at the smallest stable executable seam found:

- Discoverability exists for Workflow projects.
- Non-Workflow projects are not misleadingly shown active Workflow commands.
- Status/debug summary names M1-M4 verified runtime features.
- Docs/release checks are not directly executable here; implementation/audit should verify docs do not claim beyond the completed M1-M4 reports.

## Test-Fix Evidence

Scope: M5 test-fix worker. Edited only:

- `codex-rs/tui/tests/suite/workflow_slash_routing.rs`
- `.pipeline/reviews/C6/M5/test.md`

Fix:

- Applied the `cargo fmt --check` expected formatting for the `workflow_status_summary(true).join("\n")` assignment in the M5 test file.

Validation run from `codex-rs/`:

```bash
cargo fmt --check
```

Result: passed with exit code 0. Output still includes the existing stable-toolchain warning:

```text
Warning: can't set `imports_granularity = Item`, unstable features are only available in nightly channel.
```

```bash
cargo test -p codex-tui workflow_slash_routing
```

Result: passed. Focused test result:

```text
running 6 tests
test suite::workflow_slash_routing::hw_plan_slash_command_is_recognized_as_native_workflow_route ... ok
test suite::workflow_slash_routing::hypo_workflow_dollar_command_is_recognized_as_native_workflow_route ... ok
test suite::workflow_slash_routing::plain_text_is_not_a_workflow_route ... ok
test suite::workflow_slash_routing::unrelated_slash_commands_are_not_workflow_routes ... ok
test suite::workflow_slash_routing::workflow_commands_are_discoverable_only_for_workflow_projects ... ok
test suite::workflow_slash_routing::workflow_status_summary_reports_verified_runtime_features ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 12 filtered out
```

Observed non-blocking warning during test compile:

```text
warning: use of deprecated field `session::turn_context::TurnContext::cwd`: use the selected turn environment cwd instead
```
