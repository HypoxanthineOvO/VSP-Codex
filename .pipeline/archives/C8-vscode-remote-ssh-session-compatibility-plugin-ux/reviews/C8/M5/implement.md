# C8/M5 Implement Worker Report

## Files Changed

- `codex-rs/app-server-protocol/src/protocol/v2/item.rs`
- `codex-rs/app-server/src/request_user_input_fallback.rs`
- `.pipeline/reviews/C8/M5/implement.md`

## Implementation

- Added plugin-facing serialization for Workflow decision metadata on `ToolRequestUserInputQuestion`:
  - `workflowDecisionType` for the six Workflow decision ids.
  - `intentOnly: true` and `grantsApproval: false` for `authorization_intent`.
- Added polished Markdown fallback rendering for unsupported native Ask clients:
  - `## Workflow Decision` heading for Workflow decisions.
  - `**Answer controls**` section.
  - Human-readable decision labels.
  - No raw kind tokens such as `[single_choice]`.
  - Explicit authorization-intent safety text:
    - `Authorization intent only; this does not grant command, file, or permission approvals.`
    - `Existing approval mechanisms still apply.`
- Kept ordinary non-Workflow fallback as an agent message and did not add broad carding/routing logic.

## Commands Run

- `cargo fmt --package codex-app-server --package codex-app-server-protocol`
  - Result: passed.
  - Warning: stable rustfmt reported existing `imports_granularity = Item` nightly-only config warning.
- `cargo test -p codex-app-server --test all request_user_input -- --nocapture`
  - First run result after initial implementation: `7 passed; 1 failed`; remaining failure was Workflow fallback option marker formatting.
- `cargo fmt --package codex-app-server --package codex-app-server-protocol && cargo test -p codex-app-server --test all request_user_input -- --nocapture`
  - Final result: passed, `8 passed; 0 failed; 562 filtered out`.
  - Warning: existing deprecated `TurnContext::cwd` warning from `codex-core`.
- `cargo fmt --check --package codex-app-server --package codex-app-server-protocol --package codex-protocol --package codex-core`
  - Result: passed.
  - Warning: stable rustfmt reported existing `imports_granularity = Item` nightly-only config warning.
- `git diff --check -- codex-rs/app-server/src/request_user_input_fallback.rs codex-rs/app-server-protocol/src/protocol/v2/item.rs .pipeline/reviews/C8/M5/implement.md`
  - Result before and after report creation: passed.

## Residual Risk

- Workflow metadata is derived from the established Workflow question ids at the v2 app-server protocol boundary. If future tool callers use different ids while still sending Workflow metadata, the current production path will need a core-model metadata field expansion across all constructors.
- I did not modify tests, fixtures, protected pipeline state files, docs, or release files.

## Audit-Fix Attempt - 2026-05-30 21:46:00 CST

### Files Changed

- `codex-rs/protocol/src/request_user_input.rs`
- `codex-rs/app-server/src/request_user_input.rs`
- `codex-rs/app-server-protocol/src/protocol/v2/item.rs`
- `codex-rs/core/src/tools/handlers/request_user_input_spec.rs`
- `.pipeline/reviews/C8/M5/implement.md`

### Implementation Attempt

- Added optional explicit metadata fields to the core `RequestUserInputQuestion` model:
  - `workflowDecisionType`
  - `intentOnly`
  - `grantsApproval`
- Propagated those fields from core request events into app-server `ToolRequestUserInputQuestion`.
- Changed v2 app-server question serialization helpers to use explicit metadata fields instead of deriving Workflow metadata from reserved question ids.
- Extended the `request_user_input` tool schema so explicit Workflow metadata fields are accepted in question args.

### Commands Run

- `cargo test -p codex-app-server --test all workflow_explicit_metadata_survives_native_ask_ui_for_non_reserved_question_id -- --nocapture --test-threads=1`
  - Result: blocked at compile time.
  - Error: `RequestUserInputQuestion` struct literals outside the allowed write scope now require the new metadata fields:
    - `codex-rs/core/src/mcp_skill_dependencies.rs:227`
    - `codex-rs/core/src/mcp_tool_call.rs:1571`

### Blocker

The requested core model expansion cannot compile without updating additional `RequestUserInputQuestion` struct literals outside this worker's allowed write scope. Per strict worker separation, I stopped instead of editing those paths.

### Residual Risks

- The current working tree contains the in-scope partial audit-fix edits listed above, but validation is blocked until the out-of-scope constructors are updated or the allowed write scope is expanded.
- Full required validation was not run because the focused compile failed first.

## Replacement Audit-Fix Completion - 2026-05-30 21:42:21 CST

### Files Changed

- `codex-rs/protocol/src/request_user_input.rs`
- `codex-rs/app-server/src/request_user_input.rs`
- `codex-rs/app-server/src/request_user_input_fallback.rs`
- `codex-rs/app-server-protocol/src/protocol/v2/item.rs`
- `codex-rs/core/src/tools/handlers/request_user_input_spec.rs`
- `codex-rs/core/src/mcp_skill_dependencies.rs`
- `codex-rs/core/src/mcp_tool_call.rs`
- `.pipeline/reviews/C8/M5/implement.md`

### Implementation

- Completed the explicit metadata path for `request_user_input` questions:
  - `workflowDecisionType`
  - `intentOnly`
  - `grantsApproval`
- Propagated explicit metadata from the core request model into v2 app-server Ask UI questions.
- Serialized v2 metadata only when those explicit fields are present.
- Kept fallback Markdown driven by explicit metadata only; reserved question ids no longer synthesize Workflow card metadata.
- Updated existing non-Workflow MCP dependency and MCP tool approval prompts to set all three metadata fields to `None`.

### Commands Run

- `cargo test -p codex-app-server --test all request_user_input -- --nocapture`
  - Result: passed, `11 passed; 0 failed; 562 filtered out`.
  - Warning: existing deprecated `TurnContext::cwd` warning from `codex-core`.
- `cargo fmt --check --package codex-app-server --package codex-app-server-protocol --package codex-protocol --package codex-core`
  - First result: failed on one line-wrap diff in `codex-rs/app-server-protocol/src/protocol/v2/item.rs`.
  - Final result after formatting fix: passed.
  - Warning: stable rustfmt reported existing `imports_granularity = Item` nightly-only config warning.
- `git diff --check -- codex-rs/protocol/src/request_user_input.rs codex-rs/app-server/src/request_user_input.rs codex-rs/app-server/src/request_user_input_fallback.rs codex-rs/app-server-protocol/src/protocol/v2/item.rs codex-rs/core/src/tools/handlers/request_user_input_spec.rs codex-rs/core/src/mcp_skill_dependencies.rs codex-rs/core/src/mcp_tool_call.rs .pipeline/reviews/C8/M5/implement.md`
  - Result: passed.

### Residual Risks

- I did not edit tests, fixtures, docs, release files, or protected pipeline lifecycle files.
- The focused app-server request_user_input suite covers the explicit metadata behavior, but broader workspace tests were not requested or run.
