# C3/M13 Implement Audit Fix

## Summary

- Updated TUI active subagent fallback naming so rows without nickname metadata keep a computed fallback nickname.
- Fallback selection now prefers `chat.config.agent_roles[role].nickname_candidates`, then `chat.config.agent_nickname_candidates`, then the existing built-in ancient Chinese fallback path.
- Added focused coverage for role-specific fallback winning over the top-level nickname pool.
- Did not change core config/schema or snapshots.

## Validation

- `cd codex-rs && just fmt`
  - Passed.
- `cd codex-rs && just test -p codex-tui active_subagents_fallback_uses_configured_nickname_candidates active_subagents_fallback_uses_role_specific_nickname_candidates`
  - Passed: 2 tests run, 2 passed, 2677 skipped.
