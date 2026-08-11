# v0.145.0-vsp.11.0-alpha.2

VSP 11.0 alpha.2 improves long-session resume behavior and refreshes Markdown quote rendering. The default `lab` channel remains on 9.5; this release is available through the explicit `alpha` channel.

## Changes

- Resume replay now restores the complete transcript in memory while rendering only the newest bounded tail to terminal scrollback in one batch.
- Redraw requests emitted during saved-history replay are coalesced to prevent rapid terminal flicker on long sessions.
- Markdown blockquotes now render with a green vertical rail and a subtle terminal-aware green background instead of repeated `>` markers.
- Includes the post-alpha.1 CI bootstrap and mirror fixes already merged to `main`.

## Install Or Update

```shell
VSP_CODEX_RELEASE_CHANNEL=alpha vsp-codex update
```

For a fresh alpha installation:

```shell
curl -fsSL https://gitlab.vsplab.cn/heyx/Codex/-/raw/main/scripts/install/install.sh | VSP_CODEX_RELEASE_CHANNEL=alpha sh
```

## Verification

- Release binary reports `vsp-codex 0.145.0-vsp.11.0-alpha.2`.
- Quote rendering tests: 39 passed.
- Resume replay buffer tests: 4 passed.
- Resume history replay test: 1 passed.
- Package builder validated the archive layout and embedded binary version.
- Archive SHA-256: `6f5cafb26823e202c88988039c4eb918fdb2b88e80fb7f762d6aba8117da7e95`.
