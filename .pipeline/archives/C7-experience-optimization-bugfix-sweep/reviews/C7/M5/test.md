# C7/M5 Test Worker Report

## Changed Files

- `codex-rs/core/tests/suite/workflow_routing.rs`
  - Added focused M5 red tests for Workflow interaction-gate guidance.
  - Note: this file was already untracked in the worktree before this worker; this worker only appended M5-related tests.
- `codex-rs/tui/src/bottom_pane/request_user_input/mod.rs`
  - Added focused M5 red tests for zh-CN Workflow Request User Input card rendering.
- `.pipeline/reviews/C7/M5/test.md`
  - This report.

## Tests Added

- Core:
  - `workflow_interaction_required_decisions_must_use_request_user_input_not_prefer_it`
  - `workflow_interaction_guidance_separates_confirmation_authorization_intent_and_permission_grants`
  - `workflow_interaction_guidance_lists_all_required_card_semantic_categories`
  - `ordinary_non_workflow_chat_is_not_forced_into_workflow_request_user_input_cards`

- TUI:
  - `workflow_plan_confirmation_card_zh_cn_renders_revision_and_cancel_paths`
  - `workflow_authorization_intent_card_zh_cn_states_it_is_not_a_permission_grant`
  - `workflow_release_update_card_zh_cn_keeps_core_options_at_narrow_width`

## Red Commands

- `cd /home/heyx/Codex-VSP/codex-rs && cargo test -p codex-core workflow_interaction -- --nocapture`
  - Result: red.
  - Failures:
    - required Workflow decisions still say `prefer Request Questions via request_user_input` instead of mandatory `must use request_user_input`.
    - guidance does not define confirmation / authorization-intent / real permission grant boundaries with `request_permissions` or approval overlays.
    - guidance does not list the six required semantic card categories.

- `cd /home/heyx/Codex-VSP/codex-rs && cargo test -p codex-tui workflow_ -- --nocapture`
  - Result: red.
  - Failure: compile-red with `E0433` because `request_user_input::workflow_cards` does not exist.
  - Missing helpers asserted by tests:
    - `workflow_cards::plan_confirmation_zh_cn`
    - `workflow_cards::authorization_intent_zh_cn`
    - `workflow_cards::release_update_confirmation_zh_cn`

## Failure Reason

Current production code has generic Workflow guidance and generic Request User Input rendering, but lacks an explicit Workflow interaction-gate contract and reusable card templates. The existing guidance still frames required user decisions as preferred, not mandatory, and does not clearly separate ordinary confirmation, authorization intent, and real permission grants.

## Notes

- No production code was edited.
- No snapshots were added because the TUI tests intentionally fail before rendering until Workflow card template helpers exist.
- No `.pipeline/state.yaml`, `.pipeline/log.yaml`, `.pipeline/PROGRESS.md`, implement report, or audit report files were edited.

## GREEN Snapshot Closure

- Reviewed and accepted generated TUI Workflow card snapshots:
  - `workflow_plan_confirmation_card_zh_cn`: includes approve, request revision, and cancel paths.
  - `workflow_authorization_intent_card_zh_cn`: states the card is not a permission grant and points real permissions to `request_permissions` / approval overlay handling.
  - `workflow_release_update_card_zh_cn_narrow`: preserves immediate update, skip this time, skip this version, view notes, and cancel options at narrow width.
- Accepted the three `.snap.new` files as `.snap` baselines and removed the `.snap.new` files.
- Verification command:
  - `cd /home/heyx/Codex-VSP/codex-rs && cargo test -p codex-tui workflow_ -- --nocapture`
- Result: GREEN.
  - `codex_tui` unit workflow tests: 4 passed, 0 failed.
  - `tests/all.rs` workflow routing tests: 6 passed, 0 failed.
  - Remaining filtered/empty test binaries completed without failures.
