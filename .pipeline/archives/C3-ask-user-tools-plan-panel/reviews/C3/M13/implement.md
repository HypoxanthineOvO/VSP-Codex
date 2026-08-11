# C3/M13 Implement Worker Handoff

## Scope

Worker role: implement only.

I modified production/runtime/config files only. I did not intentionally modify tests, snapshots, fixtures, assertions, or `.pipeline/reviews/C3/M13/test.md`. I did not run `fmt` or `fix`.

## Changed Production Files

- `codex-rs/core/src/config/mod.rs`
- `codex-rs/core/src/config/agent_roles.rs`
- `codex-rs/core/src/agent/role.rs`
- `codex-rs/core/src/agent/control.rs`
- `codex-rs/tui/src/chatwidget/active_subagents.rs`
- `codex-rs/tui/src/chatwidget/rendering.rs`

Notes:

- `codex-rs/config/src/config_toml.rs` currently only has whitespace churn in the diff from an attempted approach; no functional config shape remains there.
- The worktree already contains many unrelated changes from other workers. I did not revert them.

## Implementation Strategy Applied

- Added resolved `Config.agent_nickname_candidates: Option<Vec<String>>`.
- Started resolving top-level `[agents].nickname_candidates` from the raw effective TOML in `ConfigLayerStack`, because adding it directly to `AgentsToml` broke existing struct literal construction in current tests.
- Added `remove_agents_nickname_candidates` so the raw top-level key is stripped before normal `AgentsToml` deserialization, preventing serde `flatten` from treating `nickname_candidates` as an agent role.
- Reused `normalize_agent_role_nickname_candidates` for top-level nickname validation.
- Relaxed nickname validation by removing ASCII-only validation and keeping rejection of empty lists, trim-empty names, duplicate trimmed names, and control characters.
- Replaced public built-in role nickname pools with the M13 ancient Chinese names:
  `孔丘, 孟轲, 墨翟, 庄周, 屈原, 司马迁, 张衡, 蔡伦, 祖冲之, 郦道元, 李白, 杜甫, 韩愈, 柳宗元, 苏轼, 沈括, 李清照, 辛弃疾, 关汉卿, 徐霞客, 宋应星, 曹雪芹`.
- Updated core spawn candidate precedence to:
  1. user-defined role-level `nickname_candidates`
  2. resolved top-level `Config.agent_nickname_candidates`
  3. built-in role candidates
  4. built-in ancient Chinese default pool
- Removed the old TUI private-name fallback list and wired active subagent fallback rendering to `config.agent_nickname_candidates`.
- Fixed the TUI no-config panic risk by adding a TUI built-in ancient Chinese fallback pool when the configured/effective pool is absent or empty.

## Commands Run And Results

Command:

```text
cd /home/heyx/Codex-VSP/codex-rs
just test -p codex-core load_config_accepts_top_level_agent_nickname_candidates load_config_accepts_chinese_agent_role_nickname_candidates load_config_rejects_invalid_top_level_agent_nickname_candidates spawn_thread_subagent_uses_top_level_nickname_candidates_without_role_pool spawn_thread_subagent_uses_role_specific_nickname_candidates built_in_roles_use_public_ancient_chinese_nickname_candidates
```

Result: failed during compile.

First failure after the initial implementation:

```text
error[E0063]: missing field `nickname_candidates` in initializer of `AgentsToml`
```

I then removed the direct `AgentsToml.nickname_candidates` production field and moved top-level parsing to raw effective TOML.

Command rerun:

```text
cd /home/heyx/Codex-VSP/codex-rs
just test -p codex-core load_config_accepts_top_level_agent_nickname_candidates load_config_accepts_chinese_agent_role_nickname_candidates load_config_rejects_invalid_top_level_agent_nickname_candidates spawn_thread_subagent_uses_top_level_nickname_candidates_without_role_pool spawn_thread_subagent_uses_role_specific_nickname_candidates built_in_roles_use_public_ancient_chinese_nickname_candidates
```

Result: failed during compile.

Failure:

```text
error[E0277]: the trait bound `Vec<std::string::String>: From<toml::Value>` is not satisfied
```

I changed the conversion from `Vec::<String>::try_from` to `TomlValue::try_into`.

Command rerun:

```text
cd /home/heyx/Codex-VSP/codex-rs
just test -p codex-core load_config_accepts_top_level_agent_nickname_candidates load_config_accepts_chinese_agent_role_nickname_candidates load_config_rejects_invalid_top_level_agent_nickname_candidates spawn_thread_subagent_uses_top_level_nickname_candidates_without_role_pool spawn_thread_subagent_uses_role_specific_nickname_candidates built_in_roles_use_public_ancient_chinese_nickname_candidates
```

Result: failed during compile, and the user then instructed me to stop and hand off.

Current compile error:

```text
error[E0283]: type annotations needed for `std::option::Option<_>`
    --> core/src/config/mod.rs:1938:9
     |
1938 |     let nickname_candidates = config_layer_stack
     |         ^^^^^^^^^^^^^^^^^^^
...
1944 |         .map(TomlValue::try_into)
     |              ------------------- type must be known at this point
```

Focused TUI command from the test worker was not rerun after the latest changes because core focused tests still did not compile and the user requested handoff.

## Remaining Risks / Known Issues

- Current production code does not compile yet. The immediate known fix is likely to add an explicit type annotation:
  `let nickname_candidates: Option<Vec<String>> = ...`
  in `resolve_agent_nickname_candidates`.
- `codex-rs/tui/src/chatwidget/rendering.rs` diff includes an existing plan-panel block relative to git base. I did not author that behavior intentionally as part of M13; it appears to be pre-existing worktree drift or a concurrent change. I did not revert it.
- Top-level `[agents].nickname_candidates` is being handled outside `AgentsToml`, so generated config schema may not include the new field. Schema generation was not run per worker constraints and uncertainty.
- TUI currently duplicates the built-in ancient Chinese pool locally because `codex-tui` does not depend directly on `codex-core`; this avoids the old private-name fallback and fixes no-config panic, but it is not a single shared constant.
- `codex-rs/core/src/agent/agent_names.txt` remains in the tree but is no longer referenced by the modified `control.rs`.
