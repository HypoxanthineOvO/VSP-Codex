# C8/M5 Test Worker Report

## Files Changed

- `codex-rs/app-server/tests/suite/v2/request_user_input.rs`

## Commands Run

- `rustfmt app-server/tests/suite/v2/request_user_input.rs`
  - Result: formatted test file.
  - Warning: `can't set imports_granularity = Item`, existing stable rustfmt warning.
- `cargo test -p codex-app-server --test all request_user_input -- --nocapture`
  - Result: red as intended.
  - Final summary: `6 passed; 2 failed; 0 ignored; 562 filtered out`.
- `cargo test -p codex-app-server --test all workflow_required_decisions_with_native_ask_ui_carry_card_metadata -- --nocapture`
  - Result: red as intended, captured native metadata assertion.
- `git diff --check -- codex-rs/app-server/tests/suite/v2/request_user_input.rs`
  - Result: passed.

## Red Output

Focused request_user_input run fails on the two new Workflow UX expectations:

```text
failures:
    suite::v2::request_user_input::workflow_required_decisions_with_native_ask_ui_carry_card_metadata
    suite::v2::request_user_input::workflow_required_decisions_without_native_ask_ui_render_polished_markdown_cards

test result: FAILED. 6 passed; 2 failed; 0 ignored; 0 measured; 562 filtered out
```

Native Ask UI metadata failure:

```text
assertion `left == right` failed: native Ask UI payload should preserve Workflow card metadata for every required decision type
left: []
right: ["plan_confirmation", "phase_transition", "scope_selection", "revision_rejection", "authorization_intent", "release_update_confirmation"]
```

Fallback Markdown failure:

```text
polished Markdown fallback should contain "## Workflow Decision"; got:
我需要你回答几个问题后继续。当前客户端不支持原生 Ask UI，请按编号回复：

Workflow decision required
Hypo-Workflow needs structured input before continuing.

1. [single_choice] Confirm the current plan.
...
```

## Coverage Mapping

- Requirement 1: `workflow_required_decisions_with_native_ask_ui_carry_card_metadata` verifies native `item/tool/requestUserInput` routing when `native_ask_ui: true`.
- Requirement 2: `workflow_required_decisions_without_native_ask_ui_render_polished_markdown_cards` verifies unsupported clients get polished Markdown, not raw/debug protocol text or generic `[single_choice]` tags.
- Requirement 3: both Workflow tests cover all six card types: plan confirmation, phase transition, scope selection, revision/rejection, authorization-intent, release/update confirmation.
- Requirement 4: native assertions require `intentOnly: true` and `grantsApproval: false`; fallback assertions require visible wording that authorization-intent is not command/file/permission approval and existing approval mechanisms still apply.
- Requirement 5: `ordinary_non_workflow_clarification_remains_agent_message_even_with_native_ask_ui` proves a normal assistant clarification remains an `AgentMessage` and does not produce `ToolRequestUserInput`.

## Notes

- Existing dirty files were present before this worker task. I did not edit production/runtime/docs or protected pipeline state files.
- The current implementation drops unknown Workflow metadata from request_user_input questions and renders the existing generic fallback prompt, so these tests should fail until the implement worker adds the M5 contract.

## Audit-Fix Red Tests - 2026-05-30 21:28:34 CST

### Files Changed

- `codex-rs/app-server/tests/suite/v2/request_user_input.rs`
- `.pipeline/reviews/C8/M5/test.md`

### Commands Run

- `rustfmt app-server/tests/suite/v2/request_user_input.rs`
  - Result: formatted test file.
  - Warning: `can't set imports_granularity = Item`, existing stable rustfmt warning.
- `cargo test -p codex-app-server --test all request_user_input -- --nocapture`
  - Result: red as intended.
  - Final summary: `8 passed; 3 failed; 0 ignored; 562 filtered out`.
- `cargo test -p codex-app-server --test all workflow_explicit_metadata_survives_native_ask_ui_for_non_reserved_question_id -- --nocapture --test-threads=1`
  - Result: red as intended.
- `cargo test -p codex-app-server --test all reserved_question_id_without_explicit_metadata_stays_non_workflow_in_native_ask_ui -- --nocapture --test-threads=1`
  - Result: red as intended.
- `cargo test -p codex-app-server --test all fallback_markdown_uses_explicit_workflow_metadata_not_reserved_question_ids -- --nocapture --test-threads=1`
  - Result: red as intended.
- `git diff --check -- codex-rs/app-server/tests/suite/v2/request_user_input.rs`
  - Result: passed.

### Red Output

Focused request_user_input run fails only on the three new audit-fix expectations:

```text
failures:
    suite::v2::request_user_input::fallback_markdown_uses_explicit_workflow_metadata_not_reserved_question_ids
    suite::v2::request_user_input::reserved_question_id_without_explicit_metadata_stays_non_workflow_in_native_ask_ui
    suite::v2::request_user_input::workflow_explicit_metadata_survives_native_ask_ui_for_non_reserved_question_id

test result: FAILED. 8 passed; 3 failed; 0 ignored; 0 measured; 562 filtered out
```

Explicit non-reserved id metadata is dropped in native Ask UI:

```text
assertion `left == right` failed: native Ask UI must preserve explicit Workflow metadata from tool args even when the question id is not reserved
left: Null
right: "release_update_confirmation"
```

Reserved id without explicit metadata is over-labeled in native Ask UI:

```text
native Ask UI must not synthesize workflowDecisionType for a non-Workflow question just because its id is reserved
"id": "authorization_intent",
"intentOnly": true,
"grantsApproval": false,
"workflowDecisionType": "authorization_intent"
```

Fallback Markdown ignores explicit metadata for the non-reserved id:

```text
fallback Markdown must use explicit Workflow metadata for non-reserved ids
got:
I need your answers before continuing. This client does not support the native Ask UI, so reply by number:

Workflow release confirmation
Confirm release handling before continuing.

**Answer controls**

1. Choose how to handle this release.
```

### Coverage Mapping

- Requirement 1: `workflow_explicit_metadata_survives_native_ask_ui_for_non_reserved_question_id` sends `id: "confirm_release"` plus explicit `workflowDecisionType: "release_update_confirmation"` and asserts native serialization preserves `workflowDecisionType`.
- Requirement 2: `reserved_question_id_without_explicit_metadata_stays_non_workflow_in_native_ask_ui` sends an ordinary question with `id: "authorization_intent"` and no explicit Workflow metadata, then asserts native serialization omits `workflowDecisionType`, `intentOnly`, and `grantsApproval`.
- Requirement 3: `fallback_markdown_uses_explicit_workflow_metadata_not_reserved_question_ids` asserts fallback Markdown uses explicit release/update metadata for `confirm_release` and does not show authorization-intent labeling or intent-only safety copy for the arbitrary reserved-id question.
