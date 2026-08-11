# VSP Codex 0.145.0-vsp.11.1-alpha.3

This alpha combines the closed C23 and C24 deliveries on the 11.1 line.

## Highlights

- TUI quote and Markdown rendering now preserve heading hierarchy, stream-finalize refresh,
  resize replay anchoring, and intentional CJK/Latin whitespace.
- A standalone `$CODEX_HOME/model-providers.toml` can define multiple OpenAI Responses-compatible
  providers without adding third-party definitions or defaults to official `config.toml`.
- `/model` presents provider tabs and switches provider, model, and reasoning effort as one
  transaction. Missing credentials stay discoverable but disabled.
- Connected TUI clients use app-server-owned provider catalog and persistence APIs, so remote
  configuration is read and written on the server host.
- Provider-local model metadata is preserved across turns, resume, fork, and manual compact.

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

- App-server protocol: 271 unit tests and 2 schema fixture tests passed.
- Core addon contract: 13 tests passed; provider metadata and provider-specific compact regressions
  passed.
- App-server public provider list/write integration passed.
- Provider catalog, transaction rollback, grouped popup, narrow layout, resize replay, and CJK
  whitespace TUI regressions passed with reviewed snapshots.
- Release binary, package metadata, archive SHA-256, and three-host standalone deployments report
  `0.145.0-vsp.11.1-alpha.3` after publication.

## Stable Promotion

After the alpha soak period, promote the same reviewed source through a new immutable formal version
such as `0.145.0-vsp.11.1`. Do not move this alpha tag; rebuild so the binary reports the formal
version.
