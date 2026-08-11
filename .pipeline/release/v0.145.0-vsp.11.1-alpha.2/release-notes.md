# VSP Codex 0.145.0-vsp.11.1-alpha.2

This local alpha combines both C22 delivery parts:

- Long-running local compaction preserves the active user request and newest context while moving older completed tool interactions into the checkpoint summary when the replacement budget is exhausted.
- Hypo-Workflow Host UX consumes verified manifest-declared commands, including `/hw:experiment`, recognizes semantic workspaces, and reads the current Host Contract v1 shape without taking Workflow write authority.

The package targets `x86_64-unknown-linux-gnu` and remains on the `alpha` channel.

## Install Or Update

```shell
VSP_CODEX_RELEASE_CHANNEL=alpha vsp-codex update
```

For a fresh alpha installation:

```shell
curl -fsSL https://gitlab.vsplab.cn/heyx/Codex/-/raw/main/scripts/install/install.sh | VSP_CODEX_RELEASE_CHANNEL=alpha sh
```

## Verification

- Host Contract compatibility: 7 focused tests passed.
- Semantic and legacy workspace routing: 7 focused tests passed.
- Manifest routing and palette integration: 11 focused tests passed.
- The command palette snapshot includes all ten public commands.
- The release binary, package metadata, archive SHA-256, and local standalone installation all report `0.145.0-vsp.11.1-alpha.2`.
