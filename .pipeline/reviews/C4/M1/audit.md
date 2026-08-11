# C4/M1 Audit Worker Review

verdict: pass

## reviewed_refs

- `.pipeline/prompts/001-release-spine-vscode-hijack.md`
- `.pipeline/reviews/C4/M1/test.md`
- `.pipeline/reviews/C4/M1/implement.md`
- `.pipeline/reviews/C4/M1/audit.md`
- `.pipeline/reports/001-release-spine-vscode-hijack.md`
- `codex-rs/cli/tests/vscode.rs`
- `codex-rs/cli/src/main.rs`
- `codex-rs/cli/src/vscode_cmd.rs`

## findings

- No blocking findings remain.

- The previous rollback scope-safety finding is closed. `rollback --yes` now resolves the active settings scope before selecting a backup, and `latest_backup_for_scope(&scope)` only considers regular files whose names match the active scope prefix (`user-` or `vscode-server-`) and the `.settings.json` suffix. This prevents a newer backup from the other supported scope in a shared `$CODEX_HOME/vscode-settings-backups/` directory from being selected for the current scope. Evidence: `codex-rs/cli/src/vscode_cmd.rs:182`, `codex-rs/cli/src/vscode_cmd.rs:285`, `codex-rs/cli/src/vscode_cmd.rs:296`.

- The retry test directly guards the regression that caused the first audit failure. It creates a user install and backup, creates a newer VSCode Server install and backup using the same `CODEX_HOME`, then rolls back from the user scope and asserts the user settings restore to the original user JSON rather than the newer server backup. Evidence: `codex-rs/cli/tests/vscode.rs:265`.

- The rollback safety fix is intentionally filename-scope based rather than metadata based. That is acceptable for M1 because the backup writer uses the same controlled filename convention (`{scope}-{timestamp}.settings.json`), and the selection now filters both prefix and suffix. Evidence: `codex-rs/cli/src/vscode_cmd.rs:274`, `codex-rs/cli/src/vscode_cmd.rs:285`.

- Settings write behavior remains conservative for strict JSON: unrelated keys are preserved, `chatgpt.cliExecutable` is set, `chatgpt.localeOverride` is only written when absent or empty, and rollback restores the pre-install file contents. Evidence: `codex-rs/cli/src/vscode_cmd.rs:101`, `codex-rs/cli/src/vscode_cmd.rs:113`, `codex-rs/cli/src/vscode_cmd.rs:128`, `codex-rs/cli/tests/vscode.rs:144`, `codex-rs/cli/tests/vscode.rs:201`, `codex-rs/cli/tests/vscode.rs:232`.

- Dry-run, status JSON, VSCode Server fallback, and missing-scope diagnostics match the test-worker contract. Evidence: `codex-rs/cli/src/vscode_cmd.rs:120`, `codex-rs/cli/src/vscode_cmd.rs:139`, `codex-rs/cli/src/vscode_cmd.rs:200`, `codex-rs/cli/tests/vscode.rs:313`, `codex-rs/cli/tests/vscode.rs:357`, `codex-rs/cli/tests/vscode.rs:398`.

## validation

- Implement retry reports the required scoped validation passed from `codex-rs`:
  - `just fmt`
  - `just test -p codex-cli vscode` with 8/8 tests passing
  - `just fix -p codex-cli`

- This audit worker did not rerun validation commands because the assignment is strict read-only review and explicitly forbids formatting/fix commands. Review was performed by reading the prompt, worker reports, release report, and the scoped implementation/test files.

## residual risks

- JSONC/comment support is a documented residual gap, not an M1 blocker. The implementation parses settings with `serde_json::from_str` and rewrites with `serde_json::to_string_pretty`, so comments, trailing commas, original formatting, and ordering are not preserved. Because parse failure happens before backup/write, the main safety requirement is not violated, but release notes or quickstart text should make the strict-JSON limitation explicit before broader preview distribution. Evidence: `codex-rs/cli/src/vscode_cmd.rs:247`, `codex-rs/cli/src/vscode_cmd.rs:263`, `.pipeline/reviews/C4/M1/implement.md`.

- Extension presence probing is a residual release-evidence gap, not an M1 blocker. The prompt technical route included `code --list-extensions` and optional `code --install-extension openai.chatgpt`, but the implemented CLI only configures VSCode settings and the tests only provide a fake `code` binary. This is acceptable for the current M1 scope if recorded as deferred release validation; it should not be mistaken for proof that the official extension is installed on an end-user machine. Evidence: `.pipeline/prompts/001-release-spine-vscode-hijack.md`, `codex-rs/cli/tests/vscode.rs:89`, `codex-rs/cli/src/vscode_cmd.rs`.

- The app-server initialize smoke remains synthetic in this M1 implementation. `VSP_CODEX_VSCODE_PROBE_LOG` records the expected official-extension initialize contract and VSP response fields, but it does not spawn a real `vsp-codex app-server --analytics-default-enabled --listen stdio://` process. This is sufficient for the test-worker contract and not a blocker for the rollback retry, but a real initialize smoke should be collected as release evidence before final preview signoff. Evidence: `codex-rs/cli/src/vscode_cmd.rs:335`, `codex-rs/cli/tests/vscode.rs:414`, `.pipeline/reports/001-release-spine-vscode-hijack.md`.

- Rollback still restores the latest backup for the currently resolved scope, not the exact historical settings path that produced a backup. If a user changes `HOME` or `XDG_CONFIG_HOME` while reusing the same `CODEX_HOME`, rollback can target the current scope path. This is an acceptable M1 residual because the reported and tested failure mode was cross-scope selection inside one shared backup directory, which is now covered.

## scope boundary

- Worker separation is preserved. The test worker owned `codex-rs/cli/tests/vscode.rs` and `.pipeline/reviews/C4/M1/test.md`; the implement worker owned `codex-rs/cli/src/main.rs`, `codex-rs/cli/src/vscode_cmd.rs`, `.pipeline/reports/001-release-spine-vscode-hijack.md`, and `.pipeline/reviews/C4/M1/implement.md`; this audit worker only rewrote `.pipeline/reviews/C4/M1/audit.md`.

- No official VSCode extension code, app-server implementation, app-server protocol/schema files, release binaries, tests, or implementation files were modified by this audit retry.

- The reviewed implementation scope remains limited to adding the `vsp-codex vscode` CLI surface in `codex-rs/cli/src/main.rs` and the new VSCode settings command module in `codex-rs/cli/src/vscode_cmd.rs`.
