# Plugin And Remote APP Session Visibility Debug

## Conclusion

The user-visible symptom was reproduced: VSCode Plugin and Remote APP could not see VSP-Codex sessions because both runtime surfaces had drifted back to official Codex startup paths.

Root cause:

- VSCode Remote ChatGPT extension directories contained official bundled `codex` ELF binaries (`codex-cli 0.142.0` / `0.142.2`) instead of VSP shims.
- Running VSCode extension app-server processes were started from those official bundled binaries with `CODEX_HOME` unset, so they defaulted to `~/.codex`.
- Remote APP app-server/proxy processes were started through Volta `@openai/codex`, also with `CODEX_HOME` unset, and explicitly used `/home/heyx/.codex/app-server-control/desktop-ssh-websocket-v0.sock`.
- `~/.codex/state_5.sqlite` and `~/.vsp-codex/state_5.sqlite` are distinct state databases, so an app-server running under `~/.codex` cannot be expected to show VSP-only thread state.

Applied fix:

- Re-ran the existing VSP VSCode installer against the current standalone binary. It backed up and replaced all detected remote bundled extension `codex` executables with shell shims pointing to `/home/heyx/.vsp-codex/vscode/vsp-codex`.
- Added a user-local `/home/heyx/.local/bin/codex` wrapper so Remote APP startup scripts that put `${CODEX_INSTALL_DIR:-$HOME/.local/bin}` first in `PATH` resolve `codex` to VSP-Codex instead of Volta official Codex.
- Added repository diagnostics so `vsp-codex vscode status --json` now detects remote bundled extension shim drift instead of reporting configured solely because `chatgpt.cliExecutable` is present in remote Machine settings.

Current runtime caveat:

- Existing official app-server/proxy processes were not killed because the confirmation Ask returned no explicit answer. The file/startup path is fixed for new launches, but currently connected VSCode Plugin / Remote APP windows may need reload/restart before they stop using the old official processes.

## Technical Approach

Hypotheses were checked in this order:

1. VSCode Plugin launches official bundled Codex instead of VSP wrapper.
2. Remote APP launches official Volta `@openai/codex` instead of VSP wrapper.
3. State DB divergence makes official and VSP app-servers show different session inventories.
4. A stale VSP app-server process exists but is not the active Plugin/Remote control path.

Evidence confirmed hypotheses 1, 2, and 3. Hypothesis 4 is true but secondary: a VSP app-server process exists, but the active plugin/remote paths are official Codex processes.

Strict worker separation was used:

- Repro/test worker independently confirmed process, binary, socket, and state DB evidence.
- Implement worker changed only `codex-rs/cli/src/vscode_cmd.rs`.
- Test worker added the focused regression test in `codex-rs/cli/tests/vscode.rs`.
- Main orchestrator integrated, fixed one over-specific test assertion, applied local runtime shims, and validated.

## Modified Files And Runtime Surfaces

Repository files:

- `codex-rs/cli/src/vscode_cmd.rs`
  - `vscode status` now inspects remote `~/.vscode-server/extensions/openai.chatgpt-*/bin/linux-x86_64/codex`.
  - In remote scope, `configured` now requires both non-empty settings and healthy bundled extension shims.
  - Warnings now include bundled Codex shim drift and inspection errors.
- `codex-rs/cli/tests/vscode.rs`
  - Added `vscode_status_json_warns_when_remote_bundled_codex_is_not_vsp_shim`.
  - Updated an existing server-scope status test to construct a valid remote bundled shim before asserting `configured=true`.

Local runtime surfaces:

- `/home/heyx/.vscode-server/extensions/openai.chatgpt-26.616.81150-linux-x64/bin/linux-x86_64/codex`
- `/home/heyx/.vscode-server/extensions/openai.chatgpt-26.623.30605-linux-x64/bin/linux-x86_64/codex`
- `/home/heyx/.vscode-server/extensions/openai.chatgpt-26.623.31921-linux-x64/bin/linux-x86_64/codex`
  - Replaced by VSP shims through `vsp-codex vscode install --yes --cli /home/heyx/.vsp-codex/packages/standalone/current/bin/vsp-codex`.
- `/home/heyx/.local/bin/codex`
  - Added a VSP wrapper that sets `CODEX_HOME=/home/heyx/.vsp-codex` when unset and execs the current standalone `vsp-codex`.

No M12 release metadata was changed. The user's preferred future release number remains `v0.140.0-vsp.8.1` in context, but this debug did not apply that version bump.

## Test Design

Reproduction and validation focused on startup identity rather than reading session contents:

- Check extension bundled binaries using `file`, `readlink -f`, and `--version`.
- Map active app-server/proxy processes to `/proc/<pid>/exe` and `CODEX_HOME`.
- Compare state DB thread counts only, without dumping thread contents.
- Confirm VSCode installer rewrites bundled binaries to VSP shims.
- Confirm `codex` resolution through `$HOME/.local/bin` now returns VSP-Codex.
- Add a regression test proving `vscode status` reports bundled executable drift.

## Validation Results

Passed:

- `cargo test -p codex-cli --test vscode vscode_status_json_warns_when_remote_bundled_codex_is_not_vsp_shim -- --nocapture`
- `cargo test -p codex-cli --test vscode vscode_status_json_prefers_server_scope_when_user_and_server_settings_exist -- --nocapture`
- `cargo test -p codex-cli --test vscode vscode_install_remote_scope_repoints_running_bundled_elf -- --nocapture`
- `cargo test -p codex-cli --test vscode -- --test-threads=1`
- `git diff --check -- codex-rs/cli/src/vscode_cmd.rs codex-rs/cli/tests/vscode.rs`
- `/home/heyx/.vsp-codex/packages/standalone/current/bin/vsp-codex vscode status --json`
- `PATH=/home/heyx/.local/bin:$PATH codex --version`

Important observed outputs:

- All three current remote VSCode extension bundled `codex` paths now report `vsp-codex 0.140.0-vsp.8`.
- `vscode status --json` reports `configured:true`; its remaining warning is the expected VSCode limitation that `chatgpt.cliExecutable` in remote Machine settings is application-scoped and only takes effect in local client User settings. No bundled executable drift warning remains.
- `PATH=/home/heyx/.local/bin:$PATH codex --version` reports `vsp-codex 0.140.0-vsp.8`.

One full parallel `cargo test -p codex-cli --test vscode` run initially failed `vscode_install_remote_scope_repoints_running_bundled_elf` with `Text file busy`; the same test passed when rerun alone, and the full suite passed with `--test-threads=1`. This is recorded as a test concurrency/file-lock timing risk, not a product regression introduced by this fix.

## Expected Result

After VSCode Plugin and Remote APP reconnect or restart:

- VSCode Plugin app-server should launch through the bundled shim and report VSP-Codex.
- Remote APP startup scripts should resolve `codex` from `/home/heyx/.local/bin/codex`, set `CODEX_HOME=/home/heyx/.vsp-codex`, and read VSP state.
- VSP-Codex sessions should again be visible to Plugin/Remote APP surfaces that use the refreshed startup paths.

## Problems Encountered

- Existing official Codex app-server/proxy processes remain alive because no explicit restart confirmation was received.
- Remote Machine `chatgpt.cliExecutable` remains intrinsically limited by VSCode application-scope semantics; the durable remote fix is the bundled executable shim.
- The current installed standalone directory name still resolves through `releases/0.140.0-vsp.0-...`, but the binary itself reports `vsp-codex 0.140.0-vsp.8`; this is an existing hotfix layout artifact and was not changed here.

## Risks And Follow-Up

- User should reload VSCode extension host / reconnect Remote SSH / restart Remote APP, or explicitly authorize terminating the old heyx official app-server/proxy PIDs, before expecting current UI windows to reflect the fix.
- Current old official heyx processes observed before this report included VSCode extension app-server PIDs `2445159`, `2673912`, `2789888`, `3684717`, `3706843` and Remote APP official app-server/proxy chain PIDs `4018684`, `4018687`, `4018711`, `4019685`, `4019687`, `4019695`.
- If future OpenAI ChatGPT extension updates install a new `openai.chatgpt-*` directory, `vsp-codex vscode status --json` should now expose the drift; running `vsp-codex vscode install --yes` should re-shim the new bundled binary.
