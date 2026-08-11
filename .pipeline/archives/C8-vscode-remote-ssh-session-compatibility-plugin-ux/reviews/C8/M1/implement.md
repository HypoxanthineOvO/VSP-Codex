# C8 M1 Implement Worker Report

## Files Changed

- `codex-rs/cli/src/vscode_cmd.rs`
- `.pipeline/reviews/C8/M1/implement.md`

## Implementation Summary

- Updated the single authoritative `resolve_settings_scope()` priority order.
- When `.vscode-server/data/Machine/settings.json` exists, `vscode install/status/rollback` now select the `vscode-server` scope before local user settings.
- Preserved the local user settings fallback when no server Machine settings file exists.
- Preserved rollback backup lookup semantics because backups still use the selected scope name.
- Existing status/install diagnostics continue to print or serialize the selected settings path and selected scope.

## Commands Run

- `cargo test -p codex-cli vscode -- --nocapture` from `/home/heyx/Codex-VSP/codex-rs`
  - Result: passed.
  - Observed: `tests/vscode.rs` ran 10 tests, all passed.
  - Note: emitted an existing `codex-core` deprecation warning for `TurnContext::cwd`.

- `cargo fmt --check --package codex-cli` from `/home/heyx/Codex-VSP/codex-rs`
  - Result: passed.
  - Note: emitted existing stable-channel rustfmt warnings about `imports_granularity = Item`.

- `git diff --check -- codex-rs/cli/src/vscode_cmd.rs .pipeline/reviews/C8/M1/implement.md` from `/home/heyx/Codex-VSP`
  - Result: passed.

## Residual Risks

- The priority is filesystem-driven: any existing `.vscode-server/data/Machine/settings.json` under `HOME` now takes precedence over local user settings. This matches the M1 requirement and red tests, but stale server settings on a local-only machine could still be selected.
