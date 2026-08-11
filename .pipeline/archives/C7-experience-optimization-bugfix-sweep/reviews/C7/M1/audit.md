# C7/M1 Audit - Resume And Scrollback Stability

Role: `audit`

Verdict: `pass`

No blocking findings.

## Reviewed Refs

- `.pipeline/prompts/c7-001-resume-and-scrollback-stability.md`
- `.pipeline/reviews/C7/M1/test.md`
- `.pipeline/reviews/C7/M1/implement.md`
- `codex-rs/tui/src/app/resize_reflow.rs`
- `codex-rs/tui/src/transcript_reflow.rs`
- `codex-rs/tui/src/app/tests.rs`

## Audit Result

- The production change coalesces duplicate same-size startup-settling resize scheduling. `handle_draw_size_change` skips only when `has_equivalent_pending_reflow` matches the same observed width/height and compatible target width; otherwise it keeps the existing schedule path for real width/height changes.
- The test is behavior-oriented. It drives `App::handle_draw_size_change`, observes the pending debounce deadline, and proves a duplicate same-size settling draw no longer postpones the pending rebuild.
- Pending state transitions are coherent: observed-size key is set on debounced scheduling, checked for equivalence, cleared on immediate stream reflow and pending clear, and actual rebuilt width remains tracked by `mark_reflowed_width`.
- Width-targeted and height-only pending reflows are not incorrectly collapsed. A width-targeted pending reflow can cover a height-only duplicate for the same observed size, but a new width or height observation fails the same-size key and reschedules normally.
- Worker separation evidence is sufficient: distinct `test`, `implement`, `test-fix`, and read-only `audit` workers are recorded with separate ids and scopes.

## Residual Risk / Manual Acceptance

This is a bounded internal scheduling fix, not direct terminal scroll-position observability. Manual acceptance still applies:

- Resume a long session in VS Code Terminal and macOS Terminal over SSH.
- Confirm at most one initial settling pass.
- Scroll upward and verify refresh/reflow activity does not jump to the conversation start.
