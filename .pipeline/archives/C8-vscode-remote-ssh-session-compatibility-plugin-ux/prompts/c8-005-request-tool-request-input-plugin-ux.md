# M5 Request Tool And Request Input UX For Plugin

## Objective

Improve VSCode plugin interaction for Workflow decisions and tool-style questions by routing them through native `requestUserInput` capability when supported, and through polished Markdown fallback when not supported.

## Source Of Truth

- User wants plugin-visible Request Tool/Input style interaction without changing the plugin by default.
- Required Workflow decisions must use `request_user_input` when available.
- C7 established card semantics: plan confirmation, phase transition, scope selection, revision/rejection, authorization-intent, and release/update confirmation. True permission grants remain separate.
- Known code: app-server protocol includes `item/tool/requestUserInput` and tests cover native/fallback behavior.

## Subworker Assignment Plan

Status: authorized, strict worker separation.

- `test`
  - Scope: owns app-server/TUI/protocol tests for native request input, markdown fallback, authorization-intent separation, and ordinary chat non-forcing behavior.
  - Write boundary: tests/snapshots/fixtures and `.pipeline/reviews/C8/M5/test.md` only.
- `implement`
  - Scope: owns production routing, protocol item construction, fallback rendering, and minimal docs/help text.
  - Must not create, edit, rewrite, or satisfy tests owned by `test`.
  - Expected artifact: `.pipeline/reviews/C8/M5/implement.md`.
- `audit`
  - Scope: read-only review of decision routing, safety boundaries, plugin compatibility, and fallback readability.
  - Expected artifact: `.pipeline/reviews/C8/M5/audit.md`.

Workers are not alone in the codebase. Do not revert user edits or another worker's edits; adapt around them and stop on scope conflicts.

Required user decisions must use `request_user_input` when available. authorization-intent is only intent capture; real permission grants still require the existing approval or permission mechanism.

## Technical Solution

Create a plugin-facing decision routing contract. When the client supports native `item/tool/requestUserInput`, emit structured questions with choice metadata. When unsupported, emit Markdown cards that render cleanly in the plugin thread and preserve answer controls. Workflow required decisions use this path. Ordinary non-Workflow chat questions should remain ordinary unless the runtime explicitly marks them as required Workflow decisions.

## Technical Route

1. Use M2 capability matrix to identify native versus fallback support.
2. Inspect app-server request input protocol code and existing fallback tests.
3. Add tests for the six Workflow decision card types in plugin-facing protocol or fallback output.
4. Add tests proving authorization-intent does not bypass real permission/approval gates.
5. Add tests proving ordinary non-Workflow clarification is not forcibly carded unless marked required.
6. Implement routing and fallback rendering in the shared app-server/Workflow interaction boundary.
7. Ensure Markdown fallback includes enough structure for choices without pretending to be an approval overlay.

Non-goals: plugin source modification, permission bypass, or release/update execution.

## Research Required

- M2 capability matrix is a dependency.
- Existing request input tests should be reused instead of retyping broad protocol fixtures.

## Risks And Alternatives

- Risk: plugin ignores native request input items. Mitigation: fallback Markdown path remains available.
- Risk: users confuse authorization intent with real permission. Mitigation: label intent cards and keep approval mechanisms separate.
- Alternative: plain prose questions only. Rejected for required Workflow decisions because C7 established structured card behavior.

## Validation Path

Required validation:

- app-server request input native/fallback focused tests.
- Workflow interaction guidance/routing tests.
- Markdown fallback snapshots or fixtures for plugin-facing thread items.
- `cargo fmt --check`
- `git diff --check`

Manual acceptance:

- In Remote SSH plugin, trigger at least one Workflow confirmation or scope question and verify the plugin shows a readable structured interaction or fallback card.

Pass signal: required Workflow decisions become structured and safe, while ordinary chat remains natural.

Fail signal: authorization-intent acts as a permission grant, or plugin fallback is unreadable raw protocol/debug text.

## Audit Focus

- Verify safety boundaries around permissions and destructive/external actions.
- Verify native/fallback behavior is capability-driven.
- Verify no broad plugin changes were made without confirmation.

