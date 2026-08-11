# C7/M1 Test Review - Resume And Scrollback Stability

Role: `test`

Scope: Focused red test only. No production behavior changes.

Changed files:
- `codex-rs/tui/src/app/tests.rs`
- `.pipeline/reviews/C7/M1/test.md`

Commands and results:
- `cargo test -p codex-tui duplicate_height_settling_does_not_keep_postponing_resize_reflow --no-run`
  - Result: passed compile. `codex-core` emitted an existing deprecation warning for `TurnContext::cwd`.
- `cargo test -p codex-tui duplicate_height_settling_does_not_keep_postponing_resize_reflow -- --nocapture`
  - Result: failed as expected.
  - Failure: `app::tests::duplicate_height_settling_does_not_keep_postponing_resize_reflow` panicked because the second same-size startup-settling draw postponed the pending reflow deadline:
    `second=...982115748` was greater than `first=...979477520`.

Expected red failure:
- `App::handle_draw_size_change` currently treats a repeated draw with the same observed terminal size but the same stale `last_known_screen_size` as another resize/reflow scheduling event.
- That duplicate call pushes the pending `TranscriptReflowState` debounce deadline later, which is a deterministic proxy for repeated startup settling work that can keep delaying or repeating destructive scrollback rebuild/clear behavior.
- The desired behavior is that once a resize/reflow is already pending for the settled observed size, duplicate same-size settling draws must not keep postponing or re-arming the rebuild.

Implementation guidance:
- Coalesce duplicate resize/reflow scheduling when the observed terminal size has not changed and an equivalent reflow is already pending.
- Preserve the existing real resize path: first height shrink/growth and true width changes should still schedule resize reflow.
- Keep the fix near `App::handle_draw_size_change` / `TranscriptReflowState` scheduling state so the policy is explicit and does not weaken resize coverage.

## Test-fix note - 2026-05-30 00:13:29 CST

Scope: mechanical formatting only in `codex-rs/tui/src/app/tests.rs`.

Commands and results:
- `rustfmt tui/src/app/tests.rs`
  - Result: passed; rustfmt emitted the existing stable-channel warning for `imports_granularity = Item`.
- `cargo fmt --check`
  - Result: passed; same rustfmt warning emitted.
- `cargo test -p codex-tui duplicate_height_settling_does_not_keep_postponing_resize_reflow -- --nocapture`
  - Result: passed. `codex-core` emitted an existing deprecation warning for `TurnContext::cwd`; 1 matching `codex_tui` test passed, remaining tests were filtered out.
