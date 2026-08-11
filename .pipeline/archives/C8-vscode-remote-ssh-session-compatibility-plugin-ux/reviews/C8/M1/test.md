# C8 M1 Test Worker Report

## Files Changed

- `codex-rs/cli/tests/vscode.rs`
- `.pipeline/reviews/C8/M1/test.md`

## Tests Added

- `vscode_install_prefers_server_settings_when_user_and_server_settings_exist`
  - Creates both local user settings and remote `.vscode-server/data/Machine/settings.json`.
  - Runs `vsp-codex vscode install --yes --cli <remote path>`.
  - Expects install diagnostics to name the selected server settings path.
  - Expects local user settings to remain byte-for-byte unchanged.
  - Expects server Machine settings to receive `chatgpt.cliExecutable` and default locale.

- `vscode_status_json_prefers_server_scope_when_user_and_server_settings_exist`
  - Creates both local user settings and remote server Machine settings.
  - Runs `vsp-codex vscode status --json`.
  - Expects `settingsScope: "vscode-server"`.
  - Expects `settingsPath` to equal the server Machine settings path.
  - Expects `cliExecutable` to come from the server settings.

## Existing Coverage Preserved

- `vscode_install_falls_back_to_server_settings_when_user_settings_are_absent` still covers server-only fallback.
- `vscode_dry_run_and_status_report_without_writing_settings_or_backup` still covers user-only status behavior and now also asserts `settingsScope: "user"`.

## Commands Run

- `cargo test -p codex-cli vscode -- --nocapture` from `/home/heyx/Codex-VSP`
  - Result: failed before running because no `Cargo.toml` exists at repository root.

- `cargo test -p codex-cli vscode -- --nocapture` from `/home/heyx/Codex-VSP/codex-rs`
  - Result: expected red failure.
  - Observed: 8 passed, 2 failed.
  - Failing tests:
    - `vscode_install_prefers_server_settings_when_user_and_server_settings_exist`
    - `vscode_status_json_prefers_server_scope_when_user_and_server_settings_exist`
  - Failure signal: current resolver selects `/tmp/.../Code/User/settings.json` with `settingsScope: "user"` when both user and server settings exist.

- `cargo fmt --check --package codex-cli` from `/home/heyx/Codex-VSP`
  - Result: failed before running because no `Cargo.toml` exists at repository root.

- `cargo fmt --check --package codex-cli` from `/home/heyx/Codex-VSP/codex-rs`
  - Result: passed.
  - Notes: emitted existing stable-channel rustfmt warnings about `imports_granularity = Item`.

- `git diff --check -- codex-rs/cli/tests/vscode.rs .pipeline/reviews/C8/M1/test.md` from `/home/heyx/Codex-VSP`
  - Result: passed.

## Expected Status

- Red until production resolver changes Remote SSH priority from local-first to server Machine settings first when both settings files exist.
- No production code was edited.

## Assumptions

- The Remote SSH probe should be filesystem-driven for this milestone: when both settings files exist, `.vscode-server/data/Machine/settings.json` is the selected integration scope.
- Stable diagnostics for tests are limited to selected settings path and JSON fields already exposed by `status --json`.
