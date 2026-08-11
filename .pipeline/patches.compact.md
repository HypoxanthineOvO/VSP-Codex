# Patches Compact

- P001: VSCode wrapper pins VSP home: codex-rs/cli/src/vscode_cmd.rs — VSCode install now writes a persistent wrapper that exports CODEX_HOME=$HOME/.vsp-codex before execing the real VSP CLI.
