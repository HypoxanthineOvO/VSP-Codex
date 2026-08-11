# C3/M13 Test Worker Evidence

## Scope

Worker role: test only.

Changed files:

- `codex-rs/core/src/config/config_tests.rs`
- `codex-rs/core/src/agent/role_tests.rs`
- `codex-rs/core/src/agent/control_tests.rs`
- `codex-rs/tui/src/chatwidget/tests/app_server.rs`
- `.pipeline/reviews/C3/M13/test.md`

No production logic, generated schema, fmt, or fix commands were changed/run by this worker.

## Tests Added Or Adjusted

- Config:
  - `[agents].nickname_candidates = ["陈阳宾", "尹一帆"]` must load into resolved `Config`.
  - Chinese role-level `nickname_candidates` must be accepted and trimmed.
  - Top-level blank/empty, duplicate-after-trim, and control-character candidates must be rejected.
  - Role-level control-character candidates must be rejected without ASCII-only rejection.
- Agent roles:
  - Built-in public defaults must use the M13 ancient Chinese public pool.
  - Previous private VSP names must not appear in built-in public defaults.
- Core spawn:
  - Role-level candidates still override a configured top-level default pool.
  - Missing role-level candidates fall back to the top-level configured pool.
- TUI active fallback:
  - Active subagent fallback rows must use the configured nickname pool instead of the old hardcoded `SUBAGENT_FALLBACK_NAMES`.

## Commands And Red Evidence

Command:

```text
cd /home/heyx/Codex-VSP/codex-rs
just test -p codex-core load_config_accepts_top_level_agent_nickname_candidates load_config_accepts_chinese_agent_role_nickname_candidates load_config_rejects_invalid_top_level_agent_nickname_candidates spawn_thread_subagent_uses_top_level_nickname_candidates_without_role_pool spawn_thread_subagent_uses_role_specific_nickname_candidates built_in_roles_use_public_ancient_chinese_nickname_candidates
```

Result: failed during compile, as expected for current implementation.

Failure summary:

```text
error[E0609]: no field `agent_nickname_candidates` on type `config::Config`
  core/src/agent/control_tests.rs:1767:20
  core/src/agent/control_tests.rs:1813:20
  core/src/config/config_tests.rs:7540:16
```

This proves the resolved top-level nickname candidate pool has not yet been added to `Config`, so top-level config acceptance and spawn fallback cannot pass.

Command:

```text
cd /home/heyx/Codex-VSP/codex-rs
just test -p codex-tui active_subagents_fallback_uses_configured_nickname_candidates
```

Result: failed during compile, as expected for current implementation.

Failure summary:

```text
error[E0609]: no field `agent_nickname_candidates` on type `codex_app_server_client::legacy_core::config::Config`
  tui/src/chatwidget/tests/app_server.rs:784:17
```

This proves the TUI has no effective configured nickname pool available yet and therefore cannot use it for active subagent fallback.

## Risks / Gaps

- The new tests currently stop at compile failure because the production `Config.agent_nickname_candidates` field does not exist. After implement adds the field, rerun the focused tests to expose the next expected runtime failures: ASCII-only role validation, old built-in defaults, core fallback ignoring top-level pool, and TUI fallback still using `SUBAGENT_FALLBACK_NAMES`.
- I did not update snapshots because the focused TUI red test does not reach snapshot generation.
- I did not run `just fmt` or `just fix` per Worker Separation instructions.
