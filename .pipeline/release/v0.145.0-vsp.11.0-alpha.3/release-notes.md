# v0.145.0-vsp.11.0-alpha.3

VSP 11.0 alpha.3 adds dedicated Multi-Agent V2 TUI support, editable pending Steer messages, and the finalized blockquote layout. The default `lab` channel remains on 9.5; this release is available through the explicit `alpha` channel.

## Changes

- Multi-Agent V2 activity now renders as a canonical-path tree with a dot-only root, per-agent model, reasoning effort, status, current task, and elapsed time.
- `Ctrl+O` opens the agent detail viewer; pressing Enter on a V2 agent switches to its real thread for interaction.
- V2 runtime metadata and lifecycle updates tolerate out-of-order thread/activity notifications.
- Pending normal-conversation Steer messages can be pulled back with the queued-message edit shortcut (`Alt+Up`, with the terminal fallback binding where configured), interrupting the turn and restoring the message to the composer.
- Markdown blockquotes now include rail-only lines above and below the quote. The subtle green background begins at the rail, leaving list indentation uncolored.
- V1 subagent rendering and detail navigation remain unchanged.

## Install Or Update

```shell
VSP_CODEX_RELEASE_CHANNEL=alpha vsp-codex update
```

For a fresh alpha installation:

```shell
curl -fsSL https://gitlab.vsplab.cn/heyx/Codex/-/raw/main/scripts/install/install.sh | VSP_CODEX_RELEASE_CHANNEL=alpha sh
```

## Verification

- Release binary reports `vsp-codex 0.145.0-vsp.11.0-alpha.3`.
- Blockquote rendering and streaming tests: 40 passed.
- Active-subagent panel, viewer, V1 compatibility, and V2 interaction tests: 16 passed.
- Pending Steer edit/restore regression: 1 passed.
- Package builder validated the archive layout, target, metadata, and embedded binary version.
- Archive SHA-256: `96b70ca70e09d2272f5df6a1f018066b18a0ff7e7d707fa1dcf632a7ff72477e`.
