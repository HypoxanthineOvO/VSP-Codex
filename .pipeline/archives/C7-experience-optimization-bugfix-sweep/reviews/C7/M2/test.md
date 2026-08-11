# C7/M2 Test Worker Report

## Role

`test` worker for M2 Subagent Identity, Role Policy, And Name Leases.

## Scope

Added focused red tests only. No production behavior, docs, release files, workflow state, progress, or logs were edited.

## Changed Files

- `codex-rs/tui/src/chatwidget/tests/app_server.rs`
  - `collab_lifecycle_history_uses_same_fallback_names_as_active_panel`
  - `active_subagent_name_leases_avoid_collisions_and_reuse_released_names`
- `codex-rs/core/src/tools/handlers/multi_agents_tests.rs`
  - `multi_agent_v2_spawn_role_allows_explicit_reasoning_effort_override`
- `.pipeline/reviews/C7/M2/test.md`

## Commands And Results

- `cargo test -p codex-tui collab_lifecycle_history_uses_same_fallback_names_as_active_panel -- --nocapture`
  - Result: failed as expected.
  - Failure: lifecycle history rows render raw thread IDs instead of fallback display names:
    `Spawned 019cff70-2599-75e2-af72-b90000000100`, wait details with raw IDs, and `Closed 019cff70-2599-75e2-af72-b90000000100`.

- `cargo test -p codex-tui active_subagent_name_leases_avoid_collisions_and_reuse_released_names -- --nocapture`
  - Result: failed as expected.
  - Failure: active panel assigns the same configured fallback name to two active agents:
    both rows render `娄鑫 · GPT 5 high · 运行中 · 0s`.

- `cargo test -p codex-core multi_agent_v2_spawn_role_allows_explicit_reasoning_effort_override -- --nocapture`
  - Result: failed as expected.
  - Failure: role plus explicit reasoning effort is rejected before role defaults apply:
    `Reasoning effort 'low' is not supported for model 'gpt-5.5'. Supported reasoning efforts:`.

- `rustfmt codex-rs/tui/src/chatwidget/tests/app_server.rs codex-rs/core/src/tools/handlers/multi_agents_tests.rs`
  - Result: succeeded; emitted existing repo warning about unstable `imports_granularity = Item`.

- `cargo fmt --check`
  - Result: succeeded; emitted existing repo warning about unstable `imports_granularity = Item`.

- `git diff --check -- codex-rs/tui/src/chatwidget/tests/app_server.rs codex-rs/core/src/tools/handlers/multi_agents_tests.rs .pipeline/reviews/C7/M2/test.md`
  - Result: passed.

## Expected Red Failures

1. Lifecycle history rows do not use fallback display names when no explicit nickname metadata exists.
2. Active Subagents fallback selection is hash-based per thread and can collide while unused candidates remain.
3. No shared resolver/lease state currently makes active panel and history rows agree.
4. Terminal agents do not release leased names for deterministic reuse.
5. Core spawn handling validates explicit `reasoning_effort` against the parent/current model before applying the role's model defaults.

## Implementation Guidance

- Add one authoritative display-name resolver keyed by `ThreadId` and use it from both lifecycle history rendering and the Active Subagents panel.
- Lease names from explicit nickname, role-specific candidates, configured candidates, channel defaults, and built-ins in deterministic order. Avoid collisions while active candidates are available.
- Release names when agents reach terminal states such as completed, shutdown, close, or not found, while preserving enough resolved display metadata for already-rendered/history rows.
- Keep raw thread IDs available in detail/debug/search surfaces, but do not use them as primary labels when a fallback display name can be resolved.
- In spawn policy, apply role config defaults before explicit spawn overrides. Then validate and apply explicit `model` and `reasoning_effort` unless role lock semantics are introduced and explicitly forbid them.

## Fix-Round Red Tests

Added focused red tests for the two audit `needs_changes` items. No production code was edited.

### Changed Files

- `codex-rs/tui/src/chatwidget/tests/app_server.rs`
  - `active_subagent_fallback_avoids_active_explicit_nickname_collision`
- `codex-rs/core/src/tools/handlers/multi_agents_tests.rs`
  - `spawn_agent_explicit_default_service_tier_uses_no_tier_semantics`
- `.pipeline/reviews/C7/M2/test.md`

### Commands And Results

- `rustfmt codex-rs/tui/src/chatwidget/tests/app_server.rs codex-rs/core/src/tools/handlers/multi_agents_tests.rs`
  - Result: succeeded; emitted existing repo warning about unstable `imports_granularity = Item`.

- `cargo test -p codex-tui active_subagent_fallback_avoids_active_explicit_nickname_collision -- --nocapture`
  - Result: failed as expected.
  - Failure: fallback display name collides with active explicit nickname:
    `娄鑫 [audit] · GPT 5 high · 运行中 · 0s` and `娄鑫 · GPT 5 high · 运行中 · 0s`.

- `cargo test -p codex-core spawn_agent_explicit_default_service_tier_uses_no_tier_semantics -- --nocapture`
  - Result: failed as expected.
  - Failure: explicit `service_tier = "default"` is rejected as an unsupported catalog tier:
    `Service tier `default` is not supported for model `gpt-5.4`. Supported service tiers: priority`.
