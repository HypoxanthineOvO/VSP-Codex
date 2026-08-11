# C8/M6 Test Worker Report

Status: red tests added.

## Changed Files

- `codex-rs/app-server/tests/suite/v2/mod.rs`
- `codex-rs/app-server/tests/suite/v2/workflow_markdown_surface.rs`
- `.pipeline/reviews/C8/M6/acceptance-checklist.md`
- `.pipeline/reviews/C8/M6/test.md`

## Coverage Mapping

1. Markdown Workflow surface helper/contract:
   - `m6_workflow_markdown_surface_is_plugin_readable_without_protocol_clutter` expects `codex_app_server::workflow_markdown_surface` and checks readable Markdown headings, summaries, code fences, paths, command text, and forbidden raw protocol/debug clutter.

2. Subagent prompt/report Markdown:
   - Same test asserts separate `Prompt Original`, `Report Original`, and `Derived Translation` labels while preserving original report evidence, paths, commands, and code fences.

3. Plan/SubPlan progress:
   - `m6_plan_progress_surface_keeps_subplan_and_root_counts_separate` expects text like `Plan progress: M6 implement worker 2/5 | C8 5/6` and rejects mixed `7/11` counts.
   - Existing TUI coverage already includes `workflow_cycle_plan_surface_shows_current_subplan_and_root_progress_separately`.

4. Tool output summary:
   - The M6 surface test expects a concise `Tool Output Summary`, omitted-line count, and rejects duplicated translation/transcript-looking markers.

5. Request Input fallback integration:
   - The M6 surface test embeds the M5 Markdown fallback shape with `## Workflow Decision` and `**Answer controls**`.
   - Existing M5 app-server tests cover native Ask gating and fallback Markdown behavior directly.

6. Remote SSH acceptance:
   - Added `.pipeline/reviews/C8/M6/acceptance-checklist.md` with commands and observations for final validation. It does not claim release, publish, tag, VSIX packaging, or plugin source changes.

## Commands Run

### Focused red test

Command:

```bash
cd /home/heyx/Codex-VSP/codex-rs
cargo test -p codex-app-server --test all m6_workflow_markdown_surface -- --nocapture
```

Result: failed as expected.

Key output:

```text
error[E0432]: unresolved import `codex_app_server::workflow_markdown_surface`
 --> app-server/tests/suite/v2/workflow_markdown_surface.rs:1:23
  |
1 | use codex_app_server::workflow_markdown_surface::{
  |                       ^^^^^^^^^^^^^^^^^^^^^^^^^ could not find `workflow_markdown_surface` in `codex_app_server`

error: could not compile `codex-app-server` (test "all") due to 1 previous error
```

This is the intended focused compile-red signal when the production M6 Markdown Workflow surface helper/contract has not been added yet.

### Formatting

Command:

```bash
cd /home/heyx/Codex-VSP/codex-rs
cargo fmt --check --package codex-app-server
```

Result: passed, with existing stable rustfmt warnings:

```text
Warning: can't set `imports_granularity = Item`, unstable features are only available in nightly channel.
```

### Whitespace

Command:

```bash
cd /home/heyx/Codex-VSP
git diff --check -- codex-rs/app-server/tests/suite/v2/mod.rs codex-rs/app-server/tests/suite/v2/workflow_markdown_surface.rs
```

Result: passed.

## Assumptions

- Production code is intentionally untouched by this test worker.
- The new app-server helper is expected to be public enough for integration tests, or the implement worker should expose an equivalent plugin-facing contract and adjust the test import accordingly.
- Existing dirty files in `.pipeline/`, M1-M5 production code, and other tests were treated as other-worker/user edits and were not reverted.

## Audit-Fix Red Coverage: Live Plugin-Facing Emission Path

Status: red test added for the M6 audit finding.

### Changed Files

- `codex-rs/app-server/tests/suite/v2/request_user_input.rs`
- `.pipeline/reviews/C8/M6/test.md`

### Coverage Mapping

- Extended `workflow_required_decisions_without_native_ask_ui_render_polished_markdown_cards` to exercise the live unsupported native Ask fallback path with `native_ask_ui: false`.
- The test starts a real app-server thread/turn, reads the emitted plugin-facing `ThreadItem::AgentMessage` through `item/started`, and now requires the fallback to be wrapped in the M6 Workflow Markdown surface.
- The new expectation checks for `## Workflow Surface`, `### Request Input`, and `Request Input fallback for plugin clients without native Ask UI.` while preserving existing M5 fallback expectations such as `## Workflow Decision`, `**Answer controls**`, Workflow decision labels, option text, and authorization-intent safety copy.
- This maps directly to the audit finding that `workflow_markdown_surface` must be wired into a live app-server/plugin-facing thread item emission path, not only exported as a pure helper.

### Focused Red Command

Command:

```bash
cd /home/heyx/Codex-VSP/codex-rs
cargo test -p codex-app-server --test all workflow_required_decisions_without_native_ask_ui_render_polished_markdown_cards -- --nocapture
```

Result: failed as expected.

Key output:

```text
thread 'suite::v2::request_user_input::workflow_required_decisions_without_native_ask_ui_render_polished_markdown_cards' panicked at app-server/tests/suite/v2/request_user_input.rs:454:9:
live plugin-facing request input fallback should be wrapped in the M6 Workflow Markdown surface and contain "## Workflow Surface"; got:
## Workflow Decision
...
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 574 filtered out
```

The observed emitted item text contains the M5 fallback body directly and does not contain the M6 surface wrapper, which is the intended red signal for the implement worker.

### Assumptions

- No production code was edited by this test worker.
- The exact wrapper wording is based on the current helper-level M6 contract and the audit direction to expose a request-input surface heading. If the implement worker chooses an equivalent title, update the assertion to the accepted surface contract while keeping this as a live `ThreadItem::AgentMessage` test.
