# C8/M5 Audit Worker Report

Verdict: needs_changes

## Blocking Findings

1. Blocking: Workflow card metadata is synthesized from `question.id` instead of preserving explicit tool-argument metadata.
   - Evidence: `codex-rs/protocol/src/request_user_input.rs:28`-`54` has no fields for `workflowDecisionType`, `intentOnly`, or `grantsApproval`, so the explicit metadata inserted by the test fixture at `codex-rs/app-server/tests/suite/v2/request_user_input.rs:764`-`824` is not represented in the core request model. `codex-rs/app-server/src/request_user_input.rs:20`-`41` then maps only the core fields into the app-server protocol question. Finally, `codex-rs/app-server-protocol/src/protocol/v2/item.rs:1447`-`1470` re-creates metadata by matching reserved ids in `workflow_decision_type_for_question` at `codex-rs/app-server-protocol/src/protocol/v2/item.rs:1476`-`1488`.
   - Impact: this does not actually preserve explicit Workflow card metadata from the tool args. A valid Workflow caller using a stable id like `confirm_release` with `workflowDecisionType: "release_update_confirmation"` would lose the metadata. Conversely, an arbitrary non-Workflow question with id `authorization_intent` would be serialized as an intent-only Workflow card, which misrepresents arbitrary questions and changes plugin-facing semantics.
   - Why blocking for M5: the M5 focus explicitly requires native `item/tool/requestUserInput` payloads to be capability-driven and safe for VSCode/plugin clients, and asks whether deriving metadata from ids is acceptable. For this milestone, it is a contract mismatch because the tests claim metadata is preserved, but production derives it. The fix should carry explicit metadata through the core request model and app-server protocol, then only use id-based inference if the Workflow contract intentionally defines those ids as canonical and documents/tests that behavior.

## Non-Blocking Notes

- Native versus fallback routing is capability-driven. New ask requests use `first_native_ask_ui_connection_for_thread` before delivery at `codex-rs/app-server/src/bespoke_event_handling.rs:656`-`684`, and transport skips `ToolRequestUserInput` for connections without `native_ask_ui` at `codex-rs/app-server/src/transport.rs:110`-`114`. Recovered pending asks follow the same native/fallback split at `codex-rs/app-server/src/request_processors/thread_lifecycle.rs:761`-`815`.
- Markdown fallback is readable and avoids raw protocol/debug markers for the covered Workflow path. The renderer emits `## Workflow Decision`, `**Answer controls**`, readable decision labels, and explicit option text at `codex-rs/app-server/src/request_user_input_fallback.rs:13`-`108`. The focused test rejects raw markers such as `ToolRequestUserInputParams`, `ServerRequest`, `workflowDecisionType`, and kind tags at `codex-rs/app-server/tests/suite/v2/request_user_input.rs:380`-`392`.
- Authorization-intent remains intent-only in the implemented surfaces. Native serialization emits `intentOnly: true` and `grantsApproval: false` at `codex-rs/app-server-protocol/src/protocol/v2/item.rs:1468`-`1470`; fallback text says it does not grant command, file, or permission approvals and that existing approval mechanisms still apply at `codex-rs/app-server/src/request_user_input_fallback.rs:54`-`59`. I found no code path that converts request_user_input answers into command/file permission approvals.
- Ordinary non-Workflow clarification remains natural chat in the covered case. The regression test asserts a normal assistant clarification is emitted as an `AgentMessage` and no native ask request is buffered at `codex-rs/app-server/tests/suite/v2/request_user_input.rs:409`-`429`.
- Worker separation evidence is sufficient. The prompt assigns strict separated `test`, `implement`, and `audit` roles; `test.md` reports only test ownership and red/green expectations; `implement.md` reports only production/fallback changes and its own report. This audit was read-only except for this file.

## Validation Evidence Reviewed

- Test worker reported initial red coverage: `cargo test -p codex-app-server --test all request_user_input -- --nocapture` with `6 passed; 2 failed`.
- Implement worker reported final focused pass: `cargo test -p codex-app-server --test all request_user_input -- --nocapture` with `8 passed; 0 failed; 562 filtered out`.
- Implement worker reported formatting and whitespace checks:
  - `cargo fmt --package codex-app-server --package codex-app-server-protocol`
  - `cargo fmt --check --package codex-app-server --package codex-app-server-protocol --package codex-protocol --package codex-core`
  - `git diff --check -- codex-rs/app-server/src/request_user_input_fallback.rs codex-rs/app-server-protocol/src/protocol/v2/item.rs .pipeline/reviews/C8/M5/implement.md`

## Residual Risk

The focused tests pass the currently implemented behavior because the fixture uses question ids that match the derived metadata map. They do not protect explicit metadata preservation or prevent accidental over-labeling of arbitrary questions with those ids. Add tests for both cases when fixing the blocking issue.

## Audit Recheck - 2026-05-30 21:48:16 CST

Verdict: pass

## Findings

None.

## Recheck Evidence

- Explicit `workflowDecisionType`, `intentOnly`, and `grantsApproval` metadata now exists on the core request model and is deserialized from request_user_input args at `codex-rs/protocol/src/request_user_input.rs:55`-`78`.
- App-server conversion preserves those explicit fields into `ToolRequestUserInputQuestion` at `codex-rs/app-server/src/request_user_input.rs:41`-`43`.
- Native v2 serialization emits metadata only from the explicit fields and no longer derives Workflow labels from question ids at `codex-rs/app-server-protocol/src/protocol/v2/item.rs:1427`-`1502`.
- Fallback Markdown is also driven by `workflow_decision_type_for_question`, which now returns only explicit metadata; reserved ids without metadata do not become Workflow cards at `codex-rs/app-server/src/request_user_input_fallback.rs:49`-`60` and `:234`-`:242`.
- Authorization-intent remains intent-only in the surfaced contract: the explicit Workflow fixture asserts `intentOnly: true` and `grantsApproval: false`, fallback text says it does not grant command, file, or permission approvals, and I found no path that converts request_user_input answers into real approvals.
- Existing MCP dependency and MCP tool approval prompts set all new metadata fields to `None`, preserving them as non-Workflow prompts at `codex-rs/core/src/mcp_skill_dependencies.rs:252`-`254` and `codex-rs/core/src/mcp_tool_call.rs:1580`-`1582`.

## Validation Evidence

- Main-agent validation reported `cargo test -p codex-app-server --test all request_user_input -- --nocapture`: 11 passed.
- Main-agent validation reported `cargo fmt --check --package codex-app-server --package codex-app-server-protocol --package codex-protocol --package codex-core`: passed with the existing stable rustfmt warning.
- Main-agent validation reported `git diff --check -- ...`: passed.
- Test coverage directly exercises the prior blocker: non-reserved explicit metadata preservation, reserved-id collision without metadata, fallback explicit-metadata behavior, authorization-intent intent-only flags, and ordinary non-Workflow clarification routing in `codex-rs/app-server/tests/suite/v2/request_user_input.rs`.
