# C3/M13 Implement Retry Handoff

## Scope

Worker role: implement retry only.

I stopped implementation after the user interruption. I did not modify tests, snapshots, fixtures, assertions, or `.pipeline/reviews/C3/M13/test.md`. I did not run `fmt` or `fix`.

## Files Changed By This Retry

- `codex-rs/core/src/config/mod.rs`
- `codex-rs/config/src/config_toml.rs`
- `.pipeline/reviews/C3/M13/implement-retry.md`

The worktree already contained broad unrelated changes and prior M13 implementation edits. I did not revert them.

## Fixes Applied

- Fixed the immediate `TomlValue::try_into` inference error in `core/src/config/mod.rs` by adding the explicit type:
  `let nickname_candidates: Option<Vec<String>> = ...`.
- Adjusted `resolve_agent_nickname_candidates` so `Config.agent_nickname_candidates` represents only configured top-level `[agents].nickname_candidates`, instead of injecting the built-in default during config resolution. This preserves the intended core spawn precedence:
  role-level configured candidates > top-level configured candidates > built-in role candidates > built-in default.
- Removed the now-unused `PUBLIC_ANCIENT_CHINESE_NICKNAME_CANDIDATES` import from `core/src/config/mod.rs`.
- Added a custom `Deserialize` impl for `AgentsToml` in `codex-rs/config/src/config_toml.rs` that consumes top-level `nickname_candidates` and keeps it out of the flattened `roles` map. This was intended to prevent `[agents].nickname_candidates` from being parsed as an `AgentRoleToml` role entry while avoiding a stored `AgentsToml.nickname_candidates` field that would break existing struct literals in tests.

## Static Checks Performed

- Confirmed the existing normalization path in `core/src/config/agent_roles.rs` allows Unicode names and rejects:
  - empty candidate lists
  - blank names after trim
  - duplicates after trim
  - control characters
- Confirmed current core spawn candidate selection in `core/src/agent/control.rs` is ordered as:
  - `config.agent_roles[role].nickname_candidates`
  - `config.agent_nickname_candidates`
  - `resolve_role_config(...).nickname_candidates`
  - `PUBLIC_ANCIENT_CHINESE_NICKNAME_CANDIDATES`
- Confirmed current TUI fallback in `tui/src/chatwidget/active_subagents.rs` checks configured candidates for non-empty before indexing, then falls back to `BUILT_IN_FALLBACK_NAMES`, so the no-config/empty-list path should not panic from modulo by zero.

## Commands Run

Command:

```text
cd /home/heyx/Codex-VSP/codex-rs
just test -p codex-core load_config_accepts_top_level_agent_nickname_candidates load_config_accepts_chinese_agent_role_nickname_candidates load_config_rejects_invalid_top_level_agent_nickname_candidates spawn_thread_subagent_uses_top_level_nickname_candidates_without_role_pool spawn_thread_subagent_uses_role_specific_nickname_candidates built_in_roles_use_public_ancient_chinese_nickname_candidates
```

Result before retry fixes: failed.

Observed result:

- 4 passed:
  - `load_config_accepts_chinese_agent_role_nickname_candidates`
  - `built_in_roles_use_public_ancient_chinese_nickname_candidates`
  - `spawn_thread_subagent_uses_top_level_nickname_candidates_without_role_pool`
  - `spawn_thread_subagent_uses_role_specific_nickname_candidates`
- 2 failed:
  - `load_config_accepts_top_level_agent_nickname_candidates`
  - `load_config_rejects_invalid_top_level_agent_nickname_candidates`

Failure summary:

```text
invalid length 2, expected struct AgentRoleToml with 3 elements
invalid length 0, expected struct AgentRoleToml with 3 elements
```

Interpretation: top-level `[agents].nickname_candidates` was still being captured by the `AgentsToml.roles` flatten map and parsed as an agent role.

Command rerun after retry fixes:

```text
cd /home/heyx/Codex-VSP/codex-rs
just test -p codex-core load_config_accepts_top_level_agent_nickname_candidates load_config_accepts_chinese_agent_role_nickname_candidates load_config_rejects_invalid_top_level_agent_nickname_candidates spawn_thread_subagent_uses_top_level_nickname_candidates_without_role_pool spawn_thread_subagent_uses_role_specific_nickname_candidates built_in_roles_use_public_ancient_chinese_nickname_candidates
```

Result: interrupted by user request before completion. At the time of interruption it had compiled through `codex-core` and was compiling `core_test_support`; no final pass/fail result was captured.

The focused TUI command was not run by this retry worker:

```text
cd /home/heyx/Codex-VSP/codex-rs
just test -p codex-tui active_subagents_fallback_uses_configured_nickname_candidates
```

## Current Remaining Unknowns / Risks

- The latest retry changes have not completed a focused core test run, so there may still be compile or behavior failures after the custom `AgentsToml` deserializer.
- The focused TUI test has not been run after the retry changes.
- Because `AgentsToml` now has a manual `Deserialize` implementation while deriving `JsonSchema`, schema generation may still need audit attention later. I did not run schema generation.
- I did not verify whether the interrupted `just test` process fully exited; I stopped polling after the user asked for handoff and did not start any further long commands.
