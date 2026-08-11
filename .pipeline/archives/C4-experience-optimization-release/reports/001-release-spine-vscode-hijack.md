# M1 Preview VSCode Hijack Quickstart

## Install

Configure the official VSCode ChatGPT extension to launch the preview `vsp-codex` CLI:

```sh
vsp-codex vscode install --yes
```

To point VSCode at a specific preview binary:

```sh
vsp-codex vscode install --yes --cli /absolute/path/to/vsp-codex
```

Preview the settings change without writing files or creating a backup:

```sh
vsp-codex vscode install --dry-run
```

The command writes `chatgpt.cliExecutable` and only sets `chatgpt.localeOverride` to `zh-CN` when that setting is absent or empty. Existing non-empty locale values are preserved. Before writing, it creates a backup in:

```text
$CODEX_HOME/vscode-settings-backups/
```

## Status

Check the resolved settings scope and whether the extension is configured:

```sh
vsp-codex vscode status --json
```

The JSON includes at least:

```json
{
  "configured": true,
  "settingsPath": "/path/to/settings.json",
  "settingsScope": "user"
}
```

On Linux the preferred settings path is `$XDG_CONFIG_HOME/Code/User/settings.json`. If that file is absent and VSCode Server settings exist, the command uses `$HOME/.vscode-server/data/Machine/settings.json`.

## Rollback

Restore the latest settings backup created by install:

```sh
vsp-codex vscode rollback --yes
```

Rollback restores the original settings file content from the newest backup and targets the same settings scope recorded in the backup file name.

## Probe Evidence

For test and release smoke collection, set `VSP_CODEX_VSCODE_PROBE_LOG` during install:

```sh
VSP_CODEX_VSCODE_PROBE_LOG=/tmp/vsp-codex-vscode-probe.json \
  vsp-codex vscode install --yes
```

The probe file records the official extension initialize contract for:

```text
app-server --analytics-default-enabled --listen stdio://
```

