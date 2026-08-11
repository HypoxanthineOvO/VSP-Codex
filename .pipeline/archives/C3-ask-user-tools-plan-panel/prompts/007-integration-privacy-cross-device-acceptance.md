# M7 集成隐私回归与真实跨设备验收

## Objective

把 Ask、fallback、Plan/SubPlan、TUI panel 和 app-server state 串成可交付闭环，并完成最终隐私与跨设备验收。

## Source Of Truth

- P2: `.pipeline/plan-state/decompose.yaml` -> `M7`
- Depends on M1-M6.

## Subworker Assignment Plan

Status: authorized, strict worker separation.

- `test`
  - Owns final automated validation matrix and smoke scenarios.
  - Must run or explicitly document skipped focused commands.
  - Must prepare real cross-device acceptance steps for the user.
- `implement`
  - Owns integration fixes discovered by tests only.
  - Must not claim final acceptance without user cross-device test.
- `audit`
  - Checks final diff, test evidence, schema/snapshot status, secret leakage, worker separation, and manual acceptance gate.
  - Must reject if C3 is accepted before real cross-device resume succeeds.

## Technical Solution

M7 is an integration and acceptance milestone. It should not add new feature surface unless integration reveals a blocker. It verifies schema, core behavior, TUI behavior, privacy, app-server exposure, fallback, and final manual cross-device resume.

## Technical Route

1. Run schema generation and protocol tests.
2. Run focused core tests.
3. Run TUI tests and snapshot review.
4. Run formatting and scoped fix.
5. Perform local smoke:
   - Ask single/multi/free/ranking.
   - Secret answer masking.
   - Plan/SubPlan update and panel display.
   - Pending Ask replay/fallback path where locally simulatable.
6. Prepare manual acceptance steps for user:
   - Start pending Ask on one computer.
   - Resume on another computer.
   - Confirm native Ask UI reappears if supported.
   - Confirm unsupported client falls back to numbered chat.
7. If manual cross-device resume fails, extend C3 with a repair milestone. Do not accept.

## Validation Path

Automated:

- `just fmt`
- `just write-app-server-schema --experimental`
- `just test -p codex-app-server-protocol`
- `just test -p codex-core`
- `just test -p codex-tui`
- `cargo insta pending-snapshots -p codex-tui`

Manual:

- User performs real cross-device pending Ask resume.

Pass signal:

- Automated validations pass.
- Secret/private answers are masked in visible and raw user-facing records.
- User confirms real cross-device resume and fallback behavior.

Fail signal:

- Any automated validation fails.
- Secret leaks.
- Cross-device resume or fallback fails.

## Completion Criteria

- `.pipeline/reports/007-integration-privacy-cross-device-acceptance.md` records all automated evidence.
- Manual cross-device result is recorded.
- If failed, C3 remains active and gets an extend milestone.
