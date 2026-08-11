Role: `implement`

Scope:
- Production/runtime TUI fix for M1 Resume And Scrollback Stability.
- No tests, fixtures, snapshots, docs, release files, or protected pipeline state files were edited by this worker.

Changed files:
- `codex-rs/tui/src/app/resize_reflow.rs`
- `codex-rs/tui/src/transcript_reflow.rs`
- `.pipeline/reviews/C7/M1/implement.md`

Implementation summary:
- Added an observed terminal-size key to `TranscriptReflowState` for pending resize reflows.
- Added explicit duplicate coalescing in `App::handle_draw_size_change`: when the terminal reports the same observed size and an equivalent resize reflow is already pending, keep the existing pending deadline instead of re-scheduling and postponing it.
- Preserved real resize behavior by continuing to schedule reflow when the observed height or width actually changes, and by keeping width-targeted pending reflow semantics intact.
- Immediate stream-consolidation reflow and pending-clear paths now clear the observed-size key along with pending width/deadline state.

Commands and results:
- `cargo test -p codex-tui duplicate_height_settling_does_not_keep_postponing_resize_reflow -- --nocapture` from repository root
  - Result: failed to start because `/home/heyx/Codex-VSP` is not the Cargo workspace root.
- `cargo test -p codex-tui duplicate_height_settling_does_not_keep_postponing_resize_reflow -- --nocapture` from `codex-rs`
  - Result: passed. `1 passed; 0 failed`. Existing `codex-core` deprecation warning for `TurnContext::cwd`.
- `cargo test -p codex-tui height_shrink_schedules_resize_reflow -- --nocapture` from `codex-rs`
  - Result: passed. `1 passed; 0 failed`. Existing `codex-core` deprecation warning for `TurnContext::cwd`.
- `cargo test -p codex-tui resize_reflow -- --nocapture` from `codex-rs`
  - Result: passed. Unit-filtered tests: `12 passed; 0 failed`; tmux resize smoke tests were discovered but ignored as configured.
- `cargo fmt --check` from `codex-rs`
  - Result: failed because it reported formatting differences in the test-owned `codex-rs/tui/src/app/tests.rs`, which this worker is forbidden to edit. It also reported production formatting differences that were fixed manually.
- `rustfmt --check tui/src/app/resize_reflow.rs tui/src/transcript_reflow.rs` from `codex-rs`
  - Result: passed for changed production files. Rustfmt emitted the repository's existing stable-toolchain warning about `imports_granularity = Item`.

Residual risks / manual acceptance notes:
- This fix targets duplicate same-size pending reflow scheduling. It does not add terminal scroll-position observability.
- Manual acceptance from the source prompt still applies: resume a long session in VS Code Terminal and macOS Terminal over SSH, verify at most one initial settling pass, then scroll upward and confirm refresh/reflow activity does not jump to the conversation start.
- The worktree already contained modified protected pipeline files and the test-owned `codex-rs/tui/src/app/tests.rs`; this worker did not modify or revert them.
