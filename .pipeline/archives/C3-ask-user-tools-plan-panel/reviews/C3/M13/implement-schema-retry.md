# C3/M13 Implement Schema Retry Handoff

## Scope

Worker role: implement schema retry only.

I only worked on production/config schema convergence. I did not modify tests, snapshots, fixtures, assertions, or `.pipeline/reviews/C3/M13/test.md`. I did not run `fmt` or `fix`.

## Files Changed

- `codex-rs/config/src/config_toml.rs`
- `codex-rs/core/config.schema.json`
- `.pipeline/reviews/C3/M13/implement-schema-retry.md`

I did not change `codex-rs/core/src/config/mod.rs` in this retry.

## Schema Exposure Fix

Problem observed before this retry:

- Generated `codex-rs/core/config.schema.json` exposed `AgentRoleToml.nickname_candidates`.
- Generated `AgentsToml` did not expose top-level `[agents].nickname_candidates`.
- Runtime parsing already had a custom `Deserialize` path for `AgentsToml` that consumes top-level `nickname_candidates` and keeps it out of the flattened `roles` map.

Implemented approach:

- Kept runtime `AgentsToml` shape without a stored `nickname_candidates` field, so existing struct literals should not need broad updates.
- Removed `JsonSchema` derive from runtime `AgentsToml`.
- Added a private schema-only proxy struct `AgentsTomlSchema` with the same fixed fields plus:
  - `nickname_candidates: Option<Vec<String>>`
  - flattened `roles: BTreeMap<String, AgentRoleToml>`
- Implemented `JsonSchema` for `AgentsToml` by delegating to `AgentsTomlSchema` while preserving schema name `AgentsToml`.
- Added `#[allow(dead_code)]` only on the schema proxy because its fields are intentionally consumed by the derive macro.

Generated schema now shows `AgentsToml.properties.nickname_candidates` with type `array` and `string` items, while `additionalProperties` remains `AgentRoleToml` for role-level tables.

## Commands Run And Results

Command:

```text
cd /home/heyx/Codex-VSP/codex-rs
just write-config-schema
```

Result: failed after first local schema-proxy edit.

Failure summary:

```text
error: cannot find attribute `schemars` in this scope
```

Cause: after removing `JsonSchema` derive from runtime `AgentsToml`, its remaining `#[schemars(...)]` attributes were invalid. I moved schema attributes to `AgentsTomlSchema`.

Command:

```text
cd /home/heyx/Codex-VSP/codex-rs
just write-config-schema
```

Result: passed, but emitted a dead-code warning for `AgentsTomlSchema` fields.

Follow-up: added `#[allow(dead_code)]` to the schema proxy.

Command:

```text
cd /home/heyx/Codex-VSP/codex-rs
just write-config-schema
```

Result: passed with no warning observed.

Command:

```text
cd /home/heyx/Codex-VSP
sed -n '30,65p' codex-rs/core/config.schema.json
```

Result: confirmed `AgentsToml.properties.nickname_candidates` appears in generated schema.

Command:

```text
cd /home/heyx/Codex-VSP
rg -n '"AgentsToml"|"nickname_candidates"' codex-rs/core/config.schema.json
```

Result: confirmed both role-level and top-level schema occurrences:

```text
24:        "nickname_candidates": {
34:    "AgentsToml": {
61:        "nickname_candidates": {
```

Command:

```text
cd /home/heyx/Codex-VSP/codex-rs
just test -p codex-core load_config_accepts_top_level_agent_nickname_candidates load_config_rejects_invalid_top_level_agent_nickname_candidates
```

Result: no final result captured. The command was still waiting on the cargo artifact directory lock when the user interrupted and requested handoff. I did not poll further after that interruption.

## Remaining Risks

- The focused core config test command did not complete in this retry, so there is no fresh pass/fail result from this worker after the final schema-proxy change.
- A previous `just test` process may still have been waiting/running after the tool interaction was interrupted; I did not inspect or kill it because the user requested no further work.
- The schema fix is intentionally schema-only and relies on the existing custom `Deserialize` implementation to keep top-level `[agents].nickname_candidates` out of the flattened role map.
- No `fmt` or `fix` was run by request.
