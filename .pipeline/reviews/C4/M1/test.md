# C4/M1 Test Worker Review

## Scope

Owned test work for `.pipeline/prompts/001-release-spine-vscode-hijack.md`.

Changed only:

- `codex-rs/cli/tests/vscode.rs`
- `.pipeline/reviews/C4/M1/test.md`

No production code, app-server code, release docs, or other worker artifacts were modified.

## Coverage Added

Added CLI integration tests for the planned `vsp-codex vscode` command group:

- `vscode install --yes` writes `chatgpt.cliExecutable` into VSCode settings.
- Existing unrelated VSCode settings are preserved.
- Empty or absent `chatgpt.localeOverride` becomes `zh-CN`.
- Non-empty existing `chatgpt.localeOverride` is not overwritten.
- Install creates a backup under `CODEX_HOME/vscode-settings-backups`.
- `vscode rollback --yes` restores the latest backup and removes installed keys when they were absent before install.
- `vscode rollback --yes` must restore only the backup for the current resolved settings scope; when a shared `CODEX_HOME` contains an older user backup and a newer VSCode Server backup, rollback from user scope must ignore the server backup.
- `vscode install --dry-run` does not mutate settings or create backups.
- `vscode status --json` reports settings path and installed/configured state.
- VSCode Server settings fallback uses `~/.vscode-server/data/Machine/settings.json` when local user settings are absent.
- Missing/unresolvable settings scope reports actionable diagnostics mentioning VSCode settings and `chatgpt.cliExecutable`.
- App-server initialize probe contract records:
  - `app-server --analytics-default-enabled --listen stdio://`
  - official extension client name `openai.chatgpt`
  - roots capability
  - response fields including VSP `codexHome` and `userAgent`

The tests isolate `CODEX_HOME`, `HOME`, `XDG_CONFIG_HOME`, and `PATH`, and provide a fake `code` executable that supports `--list-extensions` and `--install-extension`.

## Validation Run

Command run from `codex-rs`:

```sh
just fmt
just test -p codex-cli vscode
```

Result:

- `just fmt`: passed.
- `just test -p codex-cli vscode`: failed as expected on the rollback safety red test.

Current failure signal:

```text
TRY 2 FAIL codex-cli::vscode vscode_rollback_ignores_newer_server_backup_when_current_scope_is_user

assertion failed: `(left == right)`: rollback in user scope must restore the user backup, not the newer server backup

Diff < left / right > :
 Object {
     "files.autoSave": String("afterDelay"),
<    "chatgpt.cliExecutable": String("/home/heyx/Codex-VSP/codex-rs/target/debug/vsp-codex"),
<    "chatgpt.localeOverride": String("zh-CN"),
 }
```

The other 7 `codex-cli::vscode` tests passed. The failing test demonstrates that rollback selected the newer server-scope backup from the shared backup directory and left the user-scope install mutations in place, instead of restoring the user backup for the current resolved scope.

## Notes For Implement Worker

Expected command surface from tests:

- `vsp-codex vscode install --yes`
- `vsp-codex vscode install --dry-run`
- `vsp-codex vscode status --json`
- `vsp-codex vscode rollback --yes`

Expected settings keys:

- `chatgpt.cliExecutable`
- `chatgpt.localeOverride`

Expected primary settings path on Linux:

- `$XDG_CONFIG_HOME/Code/User/settings.json`

Expected VSCode Server fallback:

- `$HOME/.vscode-server/data/Machine/settings.json`

The app-server probe test currently expects install to emit or record structured probe evidence through the test contract in `VSP_CODEX_VSCODE_PROBE_LOG`. If implementation chooses a different structured evidence surface, update the test worker contract before accepting the implementation.
