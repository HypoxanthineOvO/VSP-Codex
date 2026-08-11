# Official DB And Backend Hijack Repair

Timestamp: 2026-06-27T21:24:48+08:00

## Conclusion

The remaining official `~/.codex/state_5.sqlite` blank `model_provider` row was repaired, and a persistent local repair job was installed so official Codex updates or Volta shim regeneration can be corrected back to VSP-Codex routing.

The official DB row was set to `openai`, not `vsp`, because it belongs to the official `~/.codex` namespace. VSP-owned runtime data remains under `~/.vsp-codex`.

## Changes

- Backed up the official DB to `/home/heyx/.codex/state_5.sqlite.backup-provider-fix-20260627212226`.
- Updated official DB rows where `trim(model_provider)=''` to `model_provider='openai'`.
- Added `/home/heyx/.local/bin/vsp-codex-repair-backends`.
- Added user systemd units:
  - `/home/heyx/.config/systemd/user/vsp-codex-repair.service`
  - `/home/heyx/.config/systemd/user/vsp-codex-repair.timer`
- Replaced `/home/heyx/.volta/bin/codex` with a VSP wrapper because the active `PATH` resolves `codex` through Volta before `/home/heyx/.local/bin`.

## Repair Strategy

The repair script is idempotent and intentionally avoids killing active official processes. Each run:

- ensures `/home/heyx/.local/bin/codex` points to the VSP standalone binary with `CODEX_HOME=/home/heyx/.vsp-codex`;
- ensures `/home/heyx/.local/bin/vsp-codex` points to `/home/heyx/.vsp-codex/packages/standalone/current/bin/vsp-codex`;
- ensures `/home/heyx/.volta/bin/codex` is also a VSP wrapper, preventing Volta from retaking the default `codex` command after updates;
- runs `vsp-codex vscode repair --yes --quiet` to repair VSCode extension bundled `codex` shims;
- repairs any future blank provider rows in the official DB to `openai`;
- removes stale official `~/.codex/app-server-control/*.sock` files only when no process is listening on them.

The timer is enabled and runs after user-manager startup and then every 5 minutes.

## Validation

- `codex --version` resolves through `/home/heyx/.volta/bin/codex` and returns `vsp-codex 0.140.0-vsp.8.1`.
- `vsp-codex --version` returns `vsp-codex 0.140.0-vsp.8.1`.
- Official `~/.codex/state_5.sqlite` blank provider count is `0`.
- VSP `~/.vsp-codex/state_5.sqlite` blank provider count is `0`.
- `vsp-codex-repair.timer` is enabled, active, and scheduled.
- The timer triggered automatically at 2026-06-27T21:24:17+08:00 and completed successfully.
- Active app-server sockets are only under `/home/heyx/.vsp-codex/app-server-control`.
- `vsp-codex vscode status --json` reports configured remote settings and the expected local-UI scope warning.

## Risks And Follow-Up

- If a future official installer force-replaces `/home/heyx/.volta/bin/codex`, the timer should restore the VSP wrapper within 5 minutes.
- The VSCode status warning about `chatgpt.cliExecutable` and `chatgpt.localeOverride` is a VSCode setting-scope limitation for remote Machine settings; local VSCode User settings still need to carry those values on the UI host.
- Existing already-running UI/app-server processes may still need reload/restart to pick up repaired launchers.
