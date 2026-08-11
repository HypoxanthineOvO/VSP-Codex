# C3/M13 Audit Follow-up: Subagent Nickname Candidate Pool Configuration

verdict: no blocking findings

Read-only follow-up of the implement audit fix for the M13 active-subagent nickname fallback warning. I inspected the audit notes, implementation notes, TUI fallback implementation, and focused TUI tests. No production code, tests, schemas, or workflow state were modified.

## Findings

No blocking findings.

## Verification Notes

- Role-level configured candidates are now checked before top-level candidates in `configured_fallback_subagent_nickname` at `codex-rs/tui/src/chatwidget/active_subagents.rs:556`-`:565`.
- Top-level `agent_nickname_candidates` remain the second fallback tier at `codex-rs/tui/src/chatwidget/active_subagents.rs:567`-`:572`.
- Built-in role fallback candidates are checked after configured role/top-level candidates at `codex-rs/tui/src/chatwidget/active_subagents.rs:574`-`:587`, including `explorer`/`simple` mapping to `墨翟`/`庄周`.
- When no metadata/configured/built-in role pool applies, rendering still falls back to the non-empty ancient Chinese `BUILT_IN_FALLBACK_NAMES` list via `fallback_subagent_name` at `codex-rs/tui/src/chatwidget/active_subagents.rs:592`-`:599`, avoiding empty-list modulo panic.
- Metadata refresh and active tool-call sync both recompute the configured fallback nickname through `configured_fallback_subagent_nickname`, so late metadata updates use the same precedence.
- Focused tests cover top-level fallback, role-specific fallback winning over top-level config, and built-in role fallback:
  - `active_subagents_fallback_uses_configured_nickname_candidates`
  - `active_subagents_fallback_uses_role_specific_nickname_candidates`
  - `active_subagents_fallback_uses_built_in_role_nickname_candidates`

## Validation Reviewed

The implement follow-up reports successful runs for:

- `just test -p codex-tui active_subagents_fallback_uses_configured_nickname_candidates active_subagents_fallback_uses_role_specific_nickname_candidates`
- `cargo insta pending-snapshots --workspace`
- `just fmt`
- `just fix -p codex-tui`
- `cargo build --bin vsp-codex`

I did not rerun commands during this read-only follow-up.
