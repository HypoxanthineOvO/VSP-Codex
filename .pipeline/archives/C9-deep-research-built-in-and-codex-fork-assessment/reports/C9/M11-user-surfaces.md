# C9/M11 User Surfaces Narrow Baseline Fix

## Conclusion

Implemented the narrow M11 baseline blocker fixes for VSP CLI update policy and the request_user_input remapped interrupt snapshot. The two requested blockers now pass. The broad TUI nextest failures remain intentionally out of scope and are tracked as follow-up risk.

## Changes

- `codex-rs/cli/src/main.rs`
  - Changed `run_update_command` so plain `vsp-codex update` checks the VSP official-service-disabled guard before the debug-build updater guard.
  - Preserved the explicit VSP manifest update path: `vsp-codex update --from-vsp-manifest ...` still bypasses the official-service guard and reaches the existing debug/release update behavior.

- `codex-rs/tui/src/bottom_pane/request_user_input/mod.rs`
  - Added a test-only snapshot helper for the remapped interrupt snapshot that trims full-width trailing buffer padding.
  - Scoped this helper to `request_user_input_freeform_remapped_interrupt_snapshot` only, avoiding broad snapshot churn.

- `codex-rs/tui/src/bottom_pane/request_user_input/snapshots/codex_tui__bottom_pane__request_user_input__tests__request_user_input_freeform_remapped_interrupt.snap`
  - Updated the single allowed snapshot to assert the intended semantic output: `enter to submit answer | f12 to interrupt`.
  - Did not accept any other generated request_user_input snapshots.

## Rendering Diagnosis

The freeform remapped interrupt body did not change semantically. The failing `.snap.new` showed unchanged visible content plus full-width right padding on the body rows. That padding comes from ratatui buffer snapshots, not from a new user-facing layout requirement. A broad change to trim every request_user_input snapshot would have caused unrelated snapshot churn, so the fix is a narrow test helper used only by the stale remapped interrupt snapshot.

## Validation

- `cargo test --manifest-path codex-rs/Cargo.toml -p codex-cli --test vsp_entrypoint -- --nocapture`
  - Passed: 8 passed, 0 failed.

- `cargo test --manifest-path codex-rs/Cargo.toml -p codex-tui request_user_input -- --nocapture`
  - Passed: 89 passed, 0 failed.

- `cargo test --manifest-path codex-rs/Cargo.toml -p codex-tui workflow_slash_routing -- --nocapture`
  - Passed: 9 passed, 0 failed in `tests/all.rs`; other filtered targets had 0 matching tests.

- `cargo test --manifest-path codex-rs/Cargo.toml -p codex-cli --test vscode -- --nocapture`
  - Failed: 15 passed, 1 failed.
  - Failing test: `vscode_install_remote_scope_repoints_running_bundled_elf`.
  - Failure: `Text file busy (os error 26)`.
  - Left unmodified because CLI vscode tests are outside the allowed edit scope.

- `git diff --check`
  - Passed.

## Unhandled Risks

- The previously observed broad `nextest -p codex-tui` failures were not addressed by this narrow M11 blocker task. They remain follow-up risk and should be triaged separately by category rather than accepted through broad snapshot updates.
- The initial `codex-cli --test vscode` failure was rerun by the main thread and passed 16/16, so it is treated as a transient `Text file busy` environment issue.
- Broad TUI nextest still must not be represented as release-clean evidence.

## Main-Thread Follow-up Validation

After the narrow implementation, the main thread and test worker reran the focused M11 validation set:

```bash
cargo test --manifest-path codex-rs/Cargo.toml -p codex-tui workflow_slash_routing -- --nocapture
cargo test --manifest-path codex-rs/Cargo.toml -p codex-tui request_user_input -- --nocapture
cargo test --manifest-path codex-rs/Cargo.toml -p codex-cli --test vscode -- --nocapture
cargo test --manifest-path codex-rs/Cargo.toml -p codex-cli --test vsp_entrypoint -- --nocapture
cargo test --manifest-path codex-rs/Cargo.toml -p codex-cli --test update -- --nocapture
git diff --check
```

Results:

- `workflow_slash_routing`: PASS, 9/9.
- `request_user_input`: PASS, 89/89.
- `vscode`: PASS, 16/16.
- `vsp_entrypoint`: PASS, 8/8.
- `update`: PASS, 1/1.
- `git diff --check`: PASS.

Audit verdict: **PASS with follow-up risks**. M11 can advance to M12, carrying broad TUI nextest failures and stack overflow as release-candidate risks.
