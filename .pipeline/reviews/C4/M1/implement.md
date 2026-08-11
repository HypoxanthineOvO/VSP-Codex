# C4/M1 Implement Worker Review

## Scope

Implemented the production CLI side of `.pipeline/prompts/001-release-spine-vscode-hijack.md` and the test worker contract in `.pipeline/reviews/C4/M1/test.md`.

Changed only implementation-owned files:

- `codex-rs/cli/src/main.rs`
- `codex-rs/cli/src/vscode_cmd.rs`
- `.pipeline/reports/001-release-spine-vscode-hijack.md`
- `.pipeline/reviews/C4/M1/implement.md`

No test files, fixtures, snapshots, app-server files, app-server protocol/schema files, official VSCode extension files, or test/audit worker artifacts were modified.

## Implementation

Added `vsp-codex vscode` with:

- `install --yes [--cli PATH]`
- `install --dry-run`
- `status --json`
- `rollback --yes`

Behavior implemented:

- Resolves Linux VSCode user settings at `$XDG_CONFIG_HOME/Code/User/settings.json`.
- Falls back to VSCode Server settings at `$HOME/.vscode-server/data/Machine/settings.json` when local user settings are absent and server settings exist.
- Creates new local user settings when `$XDG_CONFIG_HOME` is a usable directory and no settings file exists.
- Writes `chatgpt.cliExecutable` to the provided `--cli` path or current `vsp-codex` executable.
- Sets `chatgpt.localeOverride` to `zh-CN` only when absent or empty.
- Preserves unrelated JSON settings.
- Writes pre-change backups to `$CODEX_HOME/vscode-settings-backups/`.
- Restores the latest backup on rollback.
- Emits diagnostics containing `VSCode settings` and `chatgpt.cliExecutable` when settings scope or parsing fails.
- Writes structured probe evidence when `VSP_CODEX_VSCODE_PROBE_LOG` is set.

## Validation

Commands run from `codex-rs`:

```sh
cargo check -p codex-cli
just fmt
just test -p codex-cli vscode
```

Results:

- `cargo check -p codex-cli`: passed after fixing the backup path type conversion.
- `just fmt`: passed.
- `just test -p codex-cli vscode`: passed, 7 tests run and 7 passed.

## Residual Risks

- Settings are parsed and written as strict JSON; JSONC comments/trailing commas are not preserved.
- `status --json` reports configured when `chatgpt.cliExecutable` is a non-empty string; it does not verify the target executable still exists.
- The probe log records the structured initialize contract required by the test worker and does not launch a long-running app-server process.

## Retry: Scope-Bound Rollback

Audit follow-up fixed rollback backup selection in `codex-rs/cli/src/vscode_cmd.rs`.

`rollback --yes` now resolves the current VSCode settings scope first and restores only the latest backup whose filename matches that scope prefix (`user-` or `vscode-server-`) and `.settings.json` suffix. This preserves the existing user/server fallback behavior while preventing a newer backup from another scope in the same `CODEX_HOME` from being selected.

Validation rerun from `codex-rs`:

```sh
just fmt
just test -p codex-cli vscode
just fix -p codex-cli
```

Results:

- `just fmt`: passed.
- `just test -p codex-cli vscode`: passed, 8 tests run and 8 passed.
- `just fix -p codex-cli`: passed. Per repo instructions, tests were not rerun after `fix`.

Residual risk added by the retry:

- Rollback scope isolation still depends on the current filename convention rather than a richer backup metadata file. This is intentionally small and matches the current backup writer.
