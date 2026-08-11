# VSP Codex 0.145.0-vsp.11.1-alpha.7

This Linux-first hotfix restores the official OpenAI-compatible `spawn_agent` wire contract and
fixes the stable 502 regression introduced in Alpha 6 when connecting to VSPLab `/responses`.

## Fixes

- Removed the VSP-private `routing_class` parameter from the provider-facing `spawn_agent` tool
  schema. Worker Routing remains client-internal and defaults to the trusted `standard` policy.
- Restored the simple model-facing spawn result instead of exposing nested routing diagnostics.
- Normalized full-history requests when internal semantic routing changes the child model: Exec
  uses a no-history spawn, while registered interactive threads use the recent-turn default.

## Install Or Update

```shell
VSP_CODEX_RELEASE_CHANNEL=alpha vsp-codex update
```

For a fresh Linux alpha installation:

```shell
curl -fsSL https://gitlab.vsplab.cn/heyx/Codex/-/raw/main/scripts/install/install.sh | VSP_CODEX_RELEASE_CHANNEL=alpha sh
```

## Validation

- `spawn_agent` schema tests passed 6/6, Multi-Agent V2 spawn tests passed 19/19, and V2 prompt
  assembly tests passed 2/2.
- A real VSPLab V2 root smoke completed with `ok` after the official tool shape was restored.
- A real Subagent smoke resolved `standard-luna-max` internally, initialized a Luna child session,
  completed without 502 or spawn errors, and returned `root-ok`.

## Known Limitations

- This emergency release publishes Linux x86-64 GNU only. macOS ARM64 and Windows x86-64 MSVC
  remain on Alpha 6 and do not block this release.
- Explicit six-class routing selection is not exposed to the model in Alpha 7. A future release
  must provide a client-internal intent channel without adding private provider-facing fields.
