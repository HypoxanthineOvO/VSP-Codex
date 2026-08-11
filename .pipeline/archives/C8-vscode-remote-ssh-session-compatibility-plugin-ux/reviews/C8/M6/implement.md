# C8/M6 Implement Worker Report

Status: implemented and validated.

## Changed Files

- `codex-rs/app-server/src/workflow_markdown_surface.rs`
- `codex-rs/app-server/src/lib.rs`
- `.pipeline/reviews/C8/M6/implement.md`

## Implementation

- Added public `codex_app_server::workflow_markdown_surface`.
- Added the M6 surface contract types:
  - `WorkflowMarkdownSurface`
  - `PlanProgressSurface`
  - `SubagentMarkdownSurface`
  - `ToolOutputSummarySurface`
  - `RequestInputFallbackMarkdown`
- Added `render_workflow_markdown_surface` for readable Markdown fallback output.
- Rendered Plan/SubPlan progress as separate counts:
  - `Plan progress: <subplan> <x>/<y> | <root> <a>/<b>`
- Rendered Subagent sections with `Prompt Original`, `Report Original`, and `Derived Translation` labels while preserving caller-provided Markdown, code fences, paths, and commands.
- Rendered concise `Tool Output Summary` with command, summary, excerpt, and omitted raw output line count.
- Appended request input fallback Markdown as provided to preserve M5 fallback shape.

## Commands Run

```bash
cd /home/heyx/Codex-VSP/codex-rs
cargo test -p codex-app-server --test all m6_workflow_markdown_surface -- --nocapture
```

Result: passed, 1 test passed, 574 filtered out.

```bash
cd /home/heyx/Codex-VSP/codex-rs
cargo test -p codex-app-server --test all request_user_input -- --nocapture
```

Result: passed, 11 tests passed, 564 filtered out.

```bash
cd /home/heyx/Codex-VSP/codex-rs
cargo test -p codex-app-server --test all workflow_markdown_surface -- --nocapture
```

Result: passed, 2 tests passed, 573 filtered out.

```bash
cd /home/heyx/Codex-VSP/codex-rs
cargo fmt --check --package codex-app-server
```

Result: passed. Existing rustfmt warning remained: `imports_granularity = Item` requires nightly.

```bash
cd /home/heyx/Codex-VSP
git diff --check -- codex-rs/app-server/src/workflow_markdown_surface.rs codex-rs/app-server/src/lib.rs .pipeline/reviews/C8/M6/implement.md
```

Result: passed.

## Notes

- I first attempted `cargo test -p codex-app-server --test all request_user_input workflow_markdown_surface -- --nocapture`; Cargo rejected the command because it accepts only one test-name filter. I reran the filters separately.
- The validation output included existing warnings from `codex-core` about deprecated `TurnContext::cwd` and app-server test logs about temporary untrusted project config. These were not introduced by this change.

## Residual Risks

- This implements the minimal helper/contract required by the M6 tests. It does not yet wire the helper into a live plugin thread emission path.
- Remote SSH manual acceptance remains to be completed by the acceptance owner using `.pipeline/reviews/C8/M6/acceptance-checklist.md`.

## Audit-Fix Implementation: Live Request Input Surface Wiring

Status: implemented and validated.

### Changed Files

- `codex-rs/app-server/src/bespoke_event_handling.rs`
- `codex-rs/app-server/src/request_processors/thread_lifecycle.rs`
- `codex-rs/app-server/src/request_user_input_fallback.rs`
- `codex-rs/app-server/src/workflow_markdown_surface.rs`
- `.pipeline/reviews/C8/M6/implement.md`

### Implementation

- Added `render_request_input_workflow_markdown_surface` so Request Input fallbacks can use the M6 surface wrapper directly.
- Added `workflow_surface_fallback_prompt`, preserving the existing M5 fallback body from `fallback_prompt` and wrapping it with:
  - `## Workflow Surface`
  - `### Request Input`
  - `Request Input fallback for plugin clients without native Ask UI.`
- Wired the live unsupported native Ask fallback in `bespoke_event_handling.rs` to emit the wrapped Markdown as `ThreadItem::AgentMessage`.
- Wired recovered pending unsupported Ask fallback replay in `thread_lifecycle.rs` to emit the same wrapped Markdown.
- Native Ask UI delivery remains on the existing `ToolRequestUserInput` request path and is not wrapped as an agent message.

### Commands Run

```bash
cd /home/heyx/Codex-VSP/codex-rs
cargo test -p codex-app-server --test all workflow_required_decisions_without_native_ask_ui_render_polished_markdown_cards -- --nocapture
```

Result: passed, 1 test passed, 574 filtered out.

```bash
cd /home/heyx/Codex-VSP/codex-rs
cargo test -p codex-app-server --test all request_user_input -- --nocapture
```

Result: passed, 11 tests passed, 564 filtered out.

```bash
cd /home/heyx/Codex-VSP/codex-rs
cargo test -p codex-app-server --test all workflow_markdown_surface -- --nocapture
```

Result: passed, 2 tests passed, 573 filtered out.

```bash
cd /home/heyx/Codex-VSP/codex-rs
cargo fmt --check --package codex-app-server
```

Result: passed. Existing rustfmt warning remained: `imports_granularity = Item` requires nightly.

```bash
cd /home/heyx/Codex-VSP
git diff --check -- codex-rs/app-server/src/bespoke_event_handling.rs codex-rs/app-server/src/request_processors/thread_lifecycle.rs codex-rs/app-server/src/request_user_input_fallback.rs codex-rs/app-server/src/workflow_markdown_surface.rs .pipeline/reviews/C8/M6/implement.md
```

Result: passed before this report append; rerun after append also passed.

### Notes

- The focused live-emission test observed the plugin-facing `ThreadItem::AgentMessage` containing the M6 wrapper and the preserved M5 fallback body.
- Validation output included existing warnings from `codex-core` about deprecated `TurnContext::cwd`, stable rustfmt warnings, temporary untrusted project config test logs, unknown mock model warnings, and file watcher cleanup warnings. These were not introduced by this change.
- The working tree had pre-existing dirty files from other workers. I did not revert or normalize unrelated changes.

### Residual Risks

- Remote SSH manual acceptance remains to be completed by the acceptance owner using `.pipeline/reviews/C8/M6/acceptance-checklist.md`.
