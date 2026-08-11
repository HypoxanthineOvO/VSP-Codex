# C8/M5 Request Tool And Request Input UX For Plugin

Status: completed

## Summary

M5 made plugin-facing request input safer and more readable:

- Native `item/tool/requestUserInput` now preserves explicit Workflow metadata from `request_user_input` tool args.
- Unsupported native Ask clients receive readable Markdown fallback cards instead of raw protocol/debug-looking output.
- Authorization-intent cards remain intent-only and do not grant command, file, or permission approval.
- Ordinary non-Workflow clarification remains ordinary chat.

## Implementation

- Added optional explicit metadata fields to the core request input model:
  - `workflowDecisionType`
  - `intentOnly`
  - `grantsApproval`
- Propagated those fields into app-server v2 request input questions.
- Updated v2 serialization so Workflow metadata is emitted only when explicit metadata exists.
- Updated Markdown fallback rendering for Workflow decisions with a `## Workflow Decision` heading, readable decision labels, answer controls, and authorization-intent safety wording.
- Updated existing MCP dependency/tool approval prompts to keep Workflow metadata unset.

## Validation

- `cargo test -p codex-app-server --test all request_user_input -- --nocapture`: 11 passed.
- `cargo fmt --check --package codex-app-server --package codex-app-server-protocol --package codex-protocol --package codex-core`: passed with the existing stable rustfmt warning.
- `git diff --check`: passed for M5 production/report files.

## Review

- Initial audit returned `needs_changes` because metadata was derived from reserved question ids.
- Audit-fix tests added coverage for explicit non-reserved ids, reserved-id collisions, and fallback behavior.
- Audit recheck returned `pass` with no findings.
