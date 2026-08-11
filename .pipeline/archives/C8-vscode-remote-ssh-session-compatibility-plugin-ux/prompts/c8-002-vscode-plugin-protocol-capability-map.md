# M2 VSCode Plugin Protocol Capability Map

## Objective

Build a precise black-box capability map for the official VSCode plugin as used from Remote SSH, so later milestones can route Request Input, approvals, Markdown, tool output, and Workflow surface through supported app-server protocol paths or safe fallbacks.

## Source Of Truth

- User preference: optimize plugin experience without modifying the plugin itself if possible.
- P2 technical solution: inspect VSIX/manifest/logs and app-server protocol behavior as a black-box client.
- Known protocol evidence: app-server protocol contains experimental `item/tool/requestUserInput`, and tests cover request-user-input native/fallback behavior.

## Subworker Assignment Plan

Status: authorized, strict worker separation.

- `test`
  - Scope: owns black-box probe scripts/fixtures, protocol logging setup, capability assertions, and `.pipeline/reviews/C8/M2/test.md`.
  - Write boundary: tests/probe fixtures/review artifact only. Do not modify plugin source.
- `implement`
  - Scope: owns production-side diagnostic hooks or capability-report helpers if needed, plus generated capability report.
  - Must not create, edit, rewrite, or satisfy tests owned by `test`.
  - Expected artifact: `.pipeline/reviews/C8/M2/implement.md`.
- `audit`
  - Scope: read-only review of capability matrix, evidence quality, black-box boundary, and unsupported-feature fallback claims.
  - Expected artifact: `.pipeline/reviews/C8/M2/audit.md`.

Workers are not alone in the codebase. Do not revert user edits or another worker's edits; adapt around them and stop on scope conflicts.

Required user decisions must use `request_user_input` when available. Do not install system dependencies, fetch external artifacts, change plugin source, or create remote side effects without explicit confirmation.

## Technical Solution

Treat the VSCode extension as a protocol client. Produce a capability matrix that separates: native plugin-supported items, app-server-supported but plugin-unknown items, safe Markdown fallback items, and unsupported items requiring plugin changes. Use local VSIX/manifest/extension logs when present, existing protocol type definitions, and controlled app-server tests/probes.

## Technical Route

1. Locate installed VSCode extension artifacts and logs in the Remote SSH environment if present.
2. Inspect extension manifest and activation/contribution points without modifying the extension.
3. Read app-server protocol types and tests for `item/tool/requestUserInput`, approvals, permission requests, dynamic tool calls, plan updates, and markdown thread items.
4. If feasible, capture one local app-server protocol trace from a Remote SSH plugin session.
5. Create a capability matrix covering:
   - request input questions and choices
   - tool approvals and permission requests
   - plan/subplan updates
   - subagent lifecycle, prompt, and report display
   - markdown thread item rendering
   - tool output and transcript expansion
6. Record explicit unknowns instead of guessing.

Non-goals: changing VSCode plugin source, publishing or installing a VSIX, or implementing the UX changes from M5/M6.

## Research Required

- Required: local inspection of available VSCode extension files/logs and app-server protocol code.
- If VSIX files are unavailable, record that limitation and rely on protocol tests plus observable plugin logs.

## Risks And Alternatives

- Risk: plugin behavior differs between desktop and Remote SSH extension host. Mitigation: prefer Remote SSH logs and clearly label local-only evidence.
- Risk: overfitting to undocumented protocol behavior. Mitigation: keep unsupported/unknown states explicit and covered by fallback design.
- Alternative: modify plugin source now. Rejected until capability evidence proves server-side protocol/output shaping is insufficient.

## Validation Path

Required validation:

- Produce `.pipeline/reviews/C8/M2/test.md` with probe evidence or explicit unavailable-artifact notes.
- Produce a capability matrix in `.pipeline/reviews/C8/M2/implement.md` or a dedicated report.
- `git diff --check`

Optional focused tests:

- app-server protocol tests around request input/native fallback if new diagnostic helpers are added.

Manual acceptance:

- Confirm the matrix names which UX improvements can be made without changing the plugin.

Pass signal: M3-M6 can use the matrix to choose native versus fallback behavior without guessing.

Fail signal: capability claims are unsupported by protocol code, logs, or repeatable probes.

## Audit Focus

- Verify black-box boundary is preserved.
- Verify every capability claim has evidence or is marked unknown.
- Verify no plugin modification, remote publication, or external side effect occurred.

