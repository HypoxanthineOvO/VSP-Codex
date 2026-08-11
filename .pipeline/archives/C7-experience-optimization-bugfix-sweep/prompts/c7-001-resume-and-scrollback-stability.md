# M1 Resume And Scrollback Stability

## Objective

Stabilize long-session resume and terminal scrollback behavior so `vsp-codex resume` and in-TUI resume settle coherently, without repeated startup flicker or jumping to the conversation start while the user is reading scrolled-up history.

## Source Of Truth

- P1 issues: Issue 1 long-session resume flicker; Issue 2 scrolled-up terminal jumps to conversation start after refresh/reflow.
- P2 technical solution: treat resume/reflow as a bounded replay transaction; coalesce startup redraw/reflow; avoid unconditional full scrollback clear/rebuild when the user may be reading terminal scrollback.
- Relevant modules: `codex-rs/tui/src/chatwidget/replay.rs`, `codex-rs/tui/src/app/thread_routing.rs`, `codex-rs/tui/src/app/resize_reflow.rs`, `codex-rs/tui/src/insert_history.rs`, `codex-rs/tui/src/custom_terminal.rs`, `codex-rs/tui/src/tui.rs`, and resize smoke tests under `codex-rs/tui/tests/suite/`.

## Subworker Assignment Plan

Status: authorized, strict worker separation.

- `test`
  - Scope: owns failing tests, fixtures, assertions, and snapshots for initial history replay buffering, resize/reflow scheduling, duplicate scrollback clear suppression, and deterministic proxy coverage for user-scrolled history.
  - Write boundary: test files and `.pipeline/reviews/C7/M1/test.md` only.
  - Expected artifact: `.pipeline/reviews/C7/M1/test.md`.
- `implement`
  - Scope: owns production TUI code for replay/reflow scheduling, scrollback rebuild guardrails, and startup redraw coalescing.
  - Must not create, edit, rewrite, or satisfy tests owned by `test`.
  - Expected artifact: `.pipeline/reviews/C7/M1/implement.md`.
- `audit`
  - Scope: read-only review of final diff, test evidence, real resize behavior, scrollback clear policy, and manual acceptance notes.
  - Must reject fake tests that only assert helper strings without exercising replay/reflow scheduling or destructive clear behavior.
  - Expected artifact: `.pipeline/reviews/C7/M1/audit.md`.

Workers are not alone in the codebase. Do not revert user edits or another worker's edits; adapt around them and stop on scope conflicts. Lifecycle evidence must record requested, started, completed/failed/blocked, and closed/close_failed for all roles.

Required user decisions must use `request_user_input` when available. Destructive operations, system-level dependency installation, remote side effects, and release publication require explicit confirmation/authorization.

## Technical Solution

Introduce a bounded replay/reflow transaction for resume startup. Initial resume should buffer replayed history and flush once after the transcript tail is stable. Post-resume refreshes should not repeatedly clear and rebuild terminal scrollback just because delayed replay, transient rows, or terminal-size settling occurred. Reflow must still work for real terminal resize; the fix should suppress duplicate startup/replay-triggered reflows and guard destructive scrollback rebuilds when viewport anchoring cannot be preserved.

## Technical Route

1. Inspect current initial history replay, resize reflow, scrollback insertion, and terminal clear paths.
2. Add test hooks or counters for replay-buffer boundaries, full scrollback clears, and resize/reflow scheduling.
3. Write red tests for long initial replay that currently schedules repeated full clears or redraw/reflow bursts after the visible tail is already correct.
4. Write a deterministic proxy test for "user reading history + refresh/reflow occurs" by verifying destructive rebuilds are skipped, delayed, or bounded when anchoring cannot be preserved.
5. Implement one-shot initial replay flushing and debounce/suppression for duplicate replay-settling reflows.
6. Preserve real resize behavior and existing composer/input anchoring tests.
7. Document manual acceptance in the implementation/report path for VS Code Terminal and macOS Terminal over SSH.

Non-goals: transcript storage redesign, terminal backend replacement, and removing scrollback insertion entirely.

## Research Required

- Resolved by local evidence: relevant TUI replay/reflow modules and existing resize tests are in the repository.
- Implementation-time investigation remains for exact terminal-scroll detection limits. If terminal scroll position cannot be observed directly, use deterministic scheduling/clear-count proxy tests plus manual terminal acceptance.

## Risks And Alternatives

- Risk: suppressing reflow can leave stale wrapping after real terminal resize. Mitigation: only suppress duplicate startup/replay reflows and preserve real resize signals.
- Risk: terminal scroll state is not observable from all terminals. Mitigation: test internal destructive-clear scheduling and require manual acceptance.
- Rejected alternative: keep clearing and rebuilding scrollback because it is simpler. The user-visible flicker/jump is the target bug.

## Validation Path

Required validation:

- Focused TUI tests around replay buffering and resize reflow, for example `cargo test -p codex-tui resize_reflow -- --nocapture` plus any new replay/reflow tests.
- `cargo fmt --check`.
- `git diff --check`.

Manual acceptance:

- Resume a long session in VS Code Terminal and macOS Terminal over SSH.
- Observe at most one initial settling pass.
- Scroll upward in a long session, wait through refresh/reflow activity, and verify the view does not jump to the conversation start.

Pass signal: startup redraw/reflow is bounded, destructive full scrollback rebuilds are not repeated during resume settling, and real resize behavior remains correct.

Fail signal: resume still flickers 5-10 times, or a refresh while reading scrolled-up history can still jump to the beginning.

## Audit Focus

- Verify no real resize regression.
- Verify no repeated full-screen clear/rebuild loop after resume.
- Verify destructive scrollback rebuild behavior is covered by tests or clearly documented manual evidence.
- Verify manual acceptance covers both resume flicker and scrolled-up jump.

