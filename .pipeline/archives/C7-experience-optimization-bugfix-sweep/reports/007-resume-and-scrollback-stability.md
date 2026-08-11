# C7/M1 Report - Resume And Scrollback Stability

Status: completed

## Summary

M1 fixed the deterministic startup-settling path behind long-session resume flicker and scrolled-up refresh jumps. The TUI resize reflow scheduler now tracks the observed terminal size for pending reflows and coalesces duplicate same-size startup-settling draws instead of postponing the pending debounce deadline again.

## Changed Files

- `codex-rs/tui/src/app/tests.rs`
- `codex-rs/tui/src/app/resize_reflow.rs`
- `codex-rs/tui/src/transcript_reflow.rs`
- `.pipeline/reviews/C7/M1/test.md`
- `.pipeline/reviews/C7/M1/implement.md`
- `.pipeline/reviews/C7/M1/audit.md`

## Validation

- `cargo test -p codex-tui duplicate_height_settling_does_not_keep_postponing_resize_reflow -- --nocapture`: passed.
- `cargo test -p codex-tui height_shrink_schedules_resize_reflow -- --nocapture`: passed.
- `cargo test -p codex-tui resize_reflow -- --nocapture`: passed; 12 unit-filtered tests passed and 3 tmux smoke tests remained ignored by configuration.
- `cargo fmt --check`: passed with the repository's existing stable rustfmt warning for `imports_granularity = Item`.
- `git diff --check`: passed.

## Worker Evidence

- `test`: 孟轲, `019e746f-2631-7830-a3af-54a128686d03`, completed and closed.
- `implement`: 孔丘 the 2nd, `019e747a-dad7-7320-bca2-e72282435cec`, completed and closed.
- `test-fix`: 孟轲 the 2nd, `019e7481-d25d-74a1-a516-9fd473afb288`, completed and closed.
- `audit`: 孟轲 the 3rd, `019e7485-d808-7d11-b39d-27cd2a551ca0`, pass and closed.

## Residual Risk

Terminal scroll position is not directly observable from all target terminals. The implementation covers the internal duplicate reflow scheduling proxy; manual terminal acceptance remains required for VS Code Terminal and macOS Terminal over SSH.
