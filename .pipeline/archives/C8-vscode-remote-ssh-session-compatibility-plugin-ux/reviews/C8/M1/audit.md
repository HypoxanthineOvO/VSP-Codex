# C8 M1 Audit Worker Report

## Verdict

pass

## Blocking Findings

None.

## Non-Blocking Observations

- `codex-rs/cli/src/vscode_cmd.rs:200` keeps settings selection centralized in `resolve_settings_scope()`. `install`, `status`, and `rollback` all call this resolver, so the Remote SSH priority is not duplicated across command handlers.
- `codex-rs/cli/src/vscode_cmd.rs:214` now prefers an existing `.vscode-server/data/Machine/settings.json` before local user settings, and `codex-rs/cli/src/vscode_cmd.rs:221` preserves local user fallback when server Machine settings are absent.
- `codex-rs/cli/src/vscode_cmd.rs:274` and `codex-rs/cli/src/vscode_cmd.rs:285` preserve rollback semantics through `scope.name`; backups remain prefixed by the selected scope, and lookup filters by that same selected scope.
- `codex-rs/cli/src/vscode_cmd.rs:136` reports `settingsPath`, `settingsScope`, and configured `chatgpt.cliExecutable` in status output. This is sufficient to identify the selected settings path and scope for M1 diagnostics.
- Residual risk remains filesystem-driven: a stale `.vscode-server/data/Machine/settings.json` on a local-only machine will be preferred over local user settings. This was already identified in the M1 prompt and implement report and is non-blocking for the Remote SSH priority requirement.

## Evidence Checked

- Requirement prompt: `.pipeline/prompts/c8-001-remote-ssh-vscode-integration-probe.md`.
- Test worker report: `.pipeline/reviews/C8/M1/test.md`.
- Implement worker report: `.pipeline/reviews/C8/M1/implement.md`.
- Current diff for `codex-rs/cli/src/vscode_cmd.rs` and `codex-rs/cli/tests/vscode.rs`.
- Both-exist Remote SSH priority coverage:
  - `codex-rs/cli/tests/vscode.rs:359` verifies install selects server Machine settings, leaves local user settings unchanged, and writes the server settings.
  - `codex-rs/cli/tests/vscode.rs:425` verifies `status --json` reports `settingsScope: "vscode-server"`, the server settings path, and the remote CLI executable.
- User-only fallback coverage:
  - `codex-rs/cli/tests/vscode.rs:313` verifies status remains on `settingsScope: "user"` with only user settings.
  - Existing user-scope install and rollback tests still exercise local user settings behavior.
- Server-only fallback coverage:
  - `codex-rs/cli/tests/vscode.rs:471` verifies install/status use `settingsScope: "vscode-server"` when only server Machine settings exist.
- Rollback scope separation coverage:
  - `codex-rs/cli/tests/vscode.rs:264` verifies rollback in user scope does not restore a newer server-scope backup.
- Worker separation evidence:
  - Test worker report lists only `codex-rs/cli/tests/vscode.rs` and `.pipeline/reviews/C8/M1/test.md`.
  - Implement worker report lists only `codex-rs/cli/src/vscode_cmd.rs` and `.pipeline/reviews/C8/M1/implement.md`.
  - Audit worker performed read-only inspection and wrote only `.pipeline/reviews/C8/M1/audit.md`.
- Validation rerun from `/home/heyx/Codex-VSP/codex-rs`:
  - `cargo test -p codex-cli vscode -- --nocapture`: passed, 10 vscode tests passed; existing `codex-core` deprecation warning observed.
  - `cargo fmt --check --package codex-cli`: passed; existing stable-channel `imports_granularity = Item` warnings observed.
  - `git diff --check` from `/home/heyx/Codex-VSP`: passed.
