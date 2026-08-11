# C7/M2 Implement Worker Report

## Role

`implement` worker for M2 Subagent Identity, Role Policy, And Name Leases.

## Scope

Production/runtime code only, plus this report. Tests, snapshots, docs, release files, and protected workflow state were not edited.

## Changed Files

- `codex-rs/tui/src/chatwidget/active_subagents.rs`
- `codex-rs/tui/src/chatwidget.rs`
- `codex-rs/tui/src/chatwidget/constructor.rs`
- `codex-rs/core/src/tools/handlers/multi_agents/spawn.rs`
- `codex-rs/core/src/tools/handlers/multi_agents_v2/spawn.rs`
- `codex-rs/core/src/tools/handlers/multi_agents_common.rs`
- `.pipeline/reviews/C7/M2/implement.md`

## Implementation Summary

- Added a shared TUI subagent display-name resolver on `ChatWidget`.
- The resolver keeps stable fallback names by `ThreadId`, leases active fallback names to avoid collisions, and releases active leases for terminal agent states and close completions.
- Routed both lifecycle history rendering and the Active Subagents panel through the same resolver, so fallback labels are used as primary names instead of raw thread IDs.
- Changed v1 and v2 spawn config ordering for non-full-history forks so role defaults apply before explicit model/reasoning overrides.
- Updated reasoning-only override validation to inspect the resolved child model config after role defaults. Empty model reasoning metadata is treated as unknown rather than rejecting all explicit efforts.
- Kept full-history fork override rejection in place before role/default/override application.

## Commands And Results

- `cargo test -p codex-tui collab_lifecycle_history_uses_same_fallback_names_as_active_panel -- --nocapture`
  - Result: passed.
  - Test result: `1 passed; 0 failed; 2700 filtered out`.

- `cargo test -p codex-tui active_subagent_name_leases_avoid_collisions_and_reuse_released_names -- --nocapture`
  - Result: passed.
  - Test result: `1 passed; 0 failed; 2700 filtered out`.

- `cargo test -p codex-core multi_agent_v2_spawn_role_allows_explicit_reasoning_effort_override -- --nocapture`
  - Result: passed.
  - Test result: `1 passed; 0 failed; 1821 filtered out`.

## Residual Risks

- Focused validation only was run per the convergence instruction. Broader `multi_agents active_subagents`, `multi_agent`, and formatting checks were not run in this final pass.
- Commands emitted the existing core warning about deprecated `TurnContext::cwd`; no new warning remained after the final focused reruns.

## Validation Fix Round 2

### Root Cause

- The failing `spawn_agent_service_tier_inheritance_preserves_supported_or_configured_tiers` path overflowed while the child session startup was being polled inline from the current test task. `gdb` showed the overflow at `ConfigLayerStack::clone` through `Session::build_per_turn_config`, reached from `AgentControl::spawn_agent_with_metadata` during child startup.
- I did not keep or reintroduce the previous `std::thread::Builder + stack_size + runtime_handle.block_on` approach. No `stack_size` workaround remains in the touched spawn files.
- The current fix uses a Tokio task boundary around `spawn_agent_with_metadata` so the deep child-session startup future is not polled as another nested frame chain on the caller's current task stack. This is intended as a root-cause fix for the inline async poll-stack overflow, not a larger OS-thread stack workaround.

### Service Tier / Config Recursion Check

- I checked `apply_spawn_agent_service_tier`, `apply_requested_spawn_agent_model_overrides`, `Config::to_models_manager_config`, `AgentControl::spawn_agent_with_metadata`, child thread startup, and the child `Session::build_per_turn_config` clone path.
- I also tried a local `compact_spawn_agent_config_layer_stack` approach that stripped layer TOML and then normalized requirements. It did not fix the overflow, which pointed away from a service-tier decision recursion and toward the nested child startup poll stack.
- I still tightened service-tier ordering: explicit requested tier is validated against the resolved child model and then applied directly; without an explicit request, the role/current tier is tried first and parent/config tier is used as fallback only if supported.

### Command Results

- `cargo test -p codex-core spawn_agent_service_tier_inheritance_preserves_supported_or_configured_tiers -- --nocapture`
  - Initial result: failed with stack overflow / SIGABRT.
  - After partial Tokio task edit: failed to compile because the spawned future borrowed local `agent_control`; I fixed this by moving the `spawn_agent_with_metadata` call inside the `async move` block.
  - Not rerun after the final lifetime fix because the user issued a stop-and-return instruction.
- `gdb -q -batch ... spawn_agent_service_tier_inheritance_preserves_supported_or_configured_tiers --nocapture`
  - Result: captured stack showing `ConfigLayerStack::clone` via `Session::build_per_turn_config` during child session startup.
- `cargo fmt --check`
  - Result: failed after my intermediate edits, showing only formatting diffs in `multi_agents/spawn.rs`, `multi_agents_v2/spawn.rs`, and `multi_agents_common.rs`; I manually applied those rustfmt diffs. Not rerun after the stop instruction.

### Residual Risk

- Final code was not revalidated after the last lifetime-fix edit because work was stopped on request.
- The Tokio task boundary is narrower than the prior OS-thread stack workaround, but it still changes scheduling behavior around spawn completion and should be validated with the required focused tests before acceptance.

## Validation Fix Round 4

### Root Cause And Fix

- First layer: `codex-rs/core/src/session/session.rs` now returns `(sess, initial_history, session_start_source)` from the large session-init future and calls `record_initial_history(initial_history)` only after that future has unwound. This keeps the initial-history turn/config work from running on top of the already deep child session startup stack.
- Remaining layer: `codex-rs/core/src/goals.rs` now treats non-root agent turns as not supporting persisted thread-goal runtime accounting. `TurnStarted`, progress accounting, and usage-limit runtime paths return before opening the thread-goal state DB for spawned subagent turns.
- This avoids the observed child initial prompt path `TurnStarted -> state_db_for_thread_goals -> try_ensure_rollout_materialized -> live_thread.persist -> update_thread_metadata/read_thread`, which synchronously materialized rollout state during subagent startup and exhausted the default stack.

### Semantics

- The change keeps root-thread goal runtime behavior intact.
- Spawned subagent turns are not the root thread-goal owner, so they should not perform root-thread goal lookup/materialization during turn-start. Local turn accounting is cleared when a non-root turn is skipped.
- M2 spawn role, model, reasoning-effort, service-tier, and explicit override semantics are unchanged by the Round 4 `goals.rs` fix.

### Verification

- Main-thread validation: `cargo test -p codex-core spawn_agent_service_tier_inheritance_preserves_supported_or_configured_tiers -- --nocapture`
  - Result: passed on the default stack; no stack overflow.
- Worker validation before the stop instruction:
  - `cargo test -p codex-core spawn_agent_service_tier_inheritance_preserves_supported_or_configured_tiers -- --nocapture`: passed.
  - `cargo test -p codex-core multi_agent_v2_spawn_role_allows_explicit_reasoning_effort_override -- --nocapture`: passed.
  - `cargo fmt --check`: passed, with existing rustfmt nightly-only configuration warnings.
  - `git diff --check`: passed.

### Residual Risk

- The fix intentionally scopes persisted thread-goal runtime to root turns. If product requirements later allow independent goals inside spawned subagent threads, that should be implemented as a separate subagent-owned goal model rather than by reusing root-thread materialization on child startup.

## Audit Fix Round 5

### Bugs Fixed

- Active explicit nicknames now reserve their trimmed display names in the active subagent lease map before fallback rows are resolved. Fallback leases therefore skip active explicit names while preserving resolved fallback history and normal release behavior.
- Explicit spawn `service_tier = "default"` now uses the existing default/no-tier sentinel semantics. It bypasses catalog service-tier support validation and clears the child config's concrete `service_tier`, so snapshots and requests carry no concrete child tier where default semantics require none.

### Verification

- `cargo test -p codex-tui active_subagent_fallback_avoids_active_explicit_nickname_collision -- --nocapture`
  - Result: passed.
  - Test result: `1 passed; 0 failed; 2701 filtered out`.
- `cargo test -p codex-core spawn_agent_explicit_default_service_tier_uses_no_tier_semantics -- --nocapture`
  - Result: passed.
  - Test result: `1 passed; 0 failed; 1822 filtered out`.
- `cargo fmt --check`
  - Result: passed.
  - Note: emitted existing stable rustfmt warnings for nightly-only `imports_granularity = Item`.
- `git diff --check`
  - Result: passed.

### Residual Risk

- Validation was focused on the requested red tests and formatting checks only. The commands still emit the existing `TurnContext::cwd` deprecated warning during core compilation.
