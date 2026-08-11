# C4 Preview Release Experience Architecture

## Intent

C4 focuses on a usable preview release path for VSP Codex through both TUI and the official VSCode extension. The scope is intentionally bounded: release spine, VSCode hijack, canonical Rename, real Thinking capability, VSCode-terminal-safe keyboard defaults, Markdown color refinement, low-risk performance evidence, and final release validation.

## Key Decisions

- Use the official VSCode extension through `chatgpt.cliExecutable`; do not fork, patch, or redistribute the extension in this Cycle.
- Add first-party `vsp-codex vscode` CLI commands for install/status/rollback.
- Treat session/thread name as canonical metadata. Rename must be visible through TUI resume list, app-server `thread/list/read`, and VSCode.
- Thinking content must come from real reasoning summary/raw events. No synthetic Thinking from tools or progress state.
- Default required keyboard flows must avoid VSCode integrated terminal conflicts such as `Ctrl+J/K`, `Ctrl+P`, `Ctrl+Shift+P`, `Alt` chords, and `Ctrl+C`.
- Existing Markdown renderer is the base. C4 refines color/style; it does not rebuild Markdown parsing.
- Performance work is evidence plus low-risk startup deferral only. Caching systems are deferred.

## Primary Touchpoints

- `codex-rs/cli/src/main.rs`
- New `codex-rs/cli/src/vscode_cmd.rs`
- `codex-rs/app-server`
- `codex-rs/app-server-protocol`
- `codex-rs/tui/src/keymap.rs`
- `codex-rs/tui/src/bottom_pane/request_user_input/`
- `codex-rs/tui/src/bottom_pane/chat_composer.rs`
- `codex-rs/tui/src/markdown_render.rs`
- `codex-rs/tui/src/history_cell/`
- `codex-rs/rollout/src/session_index.rs`

## Deferred Architecture

- Full Color Theme picker and persistent theme groups.
- Complete command palette and focus stack.
- Keyboard copy/selection system beyond preserving terminal-native selection.
- Resume metadata cache and progressive stale-cache verification.
- Custom VSCode extension or official extension UI patching.
