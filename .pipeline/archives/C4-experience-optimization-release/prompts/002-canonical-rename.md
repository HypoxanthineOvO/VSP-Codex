# M2 Canonical Rename

## Objective

让 Rename 真正修改 canonical session/thread name，而不是 TUI-only alias，并在 TUI、app-server 和 VSCode history/list 可见。

## Source Of Truth

- `.plan-state/decompose.yaml` M2
- Existing app-server `thread/name/set` and `thread/name/updated`
- Existing rollout thread-name compatibility index

## Subworker Assignment Plan

Status: authorized, strict worker separation.

- `test`
  - Owns failing/coverage tests for loaded rename, stored rename, app-server list/read, restart persistence, and TUI snapshot.
  - Expected artifact: `.pipeline/reviews/C4/M2/test.md`
- `implement`
  - Owns canonical metadata write/read fixes across app-server/TUI/rollout.
  - Expected artifact: `.pipeline/reviews/C4/M2/implement.md`
- `audit`
  - Checks transactional behavior, no transcript pollution, duplicate-name safety, and VSCode-visible notification path.
  - Expected artifact: `.pipeline/reviews/C4/M2/audit.md`

## Technical Solution

Use thread id as identity. `thread/name/set` writes canonical thread metadata and appends rollout compatibility name records. TUI and app-server display paths read canonical metadata. Duplicate names remain allowed; lookup paths that need identity must use thread id.

## Technical Route

1. Trace current app-server `thread/name/set` processor and `ThreadMetadataPatch.name`.
2. Verify state DB thread metadata and rollout `append_thread_name` update behavior.
3. Fix loaded and stored/unloaded thread rename so both persist.
4. Emit `thread/name/updated` for connected app-server clients.
5. Update TUI `/rename`, terminal title/status, resume hint, and resume picker path as needed.
6. Ensure `thread/list` and `thread/read` return the updated canonical name.
7. Keep archived rename and resume picker action as should-have; do not let them block the core path.

## Research Required

Resolved by P1: relevant APIs and stores already exist. Implementation must verify exact failure gaps before editing.

## Risks And Alternatives

- Risk: state DB succeeds but rollout compatibility append fails. Do not report success if visible compatibility write fails.
- Risk: duplicate names confuse lookup. Use thread id for identity.
- Rejected: new alias field or TUI-only alias.

## Validation Path

Commands:

- `just test -p codex-app-server thread_name`
- `just test -p codex-tui rename`
- `just test -p codex-rollout thread_name`

Pass signal: loaded/stored rename persists across restart; TUI/app-server/list/read show the new name; notification is emitted.

Fail signal: only current UI changes, resume list keeps old name, empty names save, or transcript gets a rename turn/item.

## Audit Focus

- Canonical metadata is source of truth.
- Rollout compatibility remains coherent.
- Failure paths keep editor open or restore old visible name.
