# C7/M2 Audit Report

## Verdict

needs_changes

## Findings

1. Active fallback names can collide with active explicit nicknames.
   - Evidence: `codex-rs/tui/src/chatwidget/active_subagents.rs` removes the active lease when a row has an explicit nickname, while fallback candidate selection only avoids names present in `active_leases`.
   - Impact: an active agent explicitly named `娄鑫` does not prevent another active fallback agent from leasing the same primary display name.

2. Explicit spawn `service_tier = "default"` can be rejected as unsupported.
   - Evidence: spawn service-tier validation checks catalog support before applying explicit requests, but existing model/session semantics treat `default` as the explicit no-tier sentinel.
   - Impact: callers can request the standard/default service tier and receive an unsupported-tier error even though `default` is a valid request value.

## Checked Scope

- TUI display-name resolver and lease release behavior.
- Lifecycle history versus Active Subagents display-name consistency.
- Spawn role/default/explicit override and service-tier behavior.
- Default-stack child startup fixes in session startup and goal runtime.
- M2 test, implement, and worker lifecycle evidence.

## Required Fix

- Reserve active explicit nicknames so fallback leases avoid them while the explicit-nickname agent is active.
- Treat explicit `service_tier = "default"` as a valid no-tier/default request instead of catalog-validating it as a model tier.

## Recheck

verdict: pass

Timestamp: 2026-05-30T03:12:48+08:00

Auditor: 孔丘 the 8th (`019e7524-5156-7550-af44-8362dc88ab7c`)

Checked scope:

- `codex-rs/tui/src/chatwidget/active_subagents.rs`: active explicit nicknames are reserved in the active lease map through `reserve_active_explicit_metadata` / `resolve_active_metadata`, so fallback selection skips those names while the explicit-nickname row is active. Terminal states release active leases while preserving resolved fallback history.
- `codex-rs/core/src/tools/handlers/multi_agents_common.rs` and related v1/v2 spawn paths: explicit `service_tier = "default"` is handled before catalog validation as the default/no-tier sentinel by clearing concrete child `service_tier`.
- M2 original semantics remain intact: lifecycle history and the active panel share display-name resolution, role defaults compose before explicit model/reasoning overrides, and the default-stack child startup repair remains the session-initial-history unwind plus non-root goal-runtime skip rather than a `stack_size` workaround.

Evidence reviewed:

- `cargo test -p codex-tui active_subagent_fallback_avoids_active_explicit_nickname_collision -- --nocapture`
- `cargo test -p codex-core spawn_agent_explicit_default_service_tier_uses_no_tier_semantics -- --nocapture`
- `cargo test -p codex-core spawn_agent_service_tier_inheritance_preserves_supported_or_configured_tiers -- --nocapture`
- `cargo test -p codex-core multi_agent_v2_spawn_role_allows_explicit_reasoning_effort_override -- --nocapture`
- `cargo test -p codex-tui collab_lifecycle_history_uses_same_fallback_names_as_active_panel -- --nocapture`
- `cargo test -p codex-tui active_subagent_name_leases_avoid_collisions_and_reuse_released_names -- --nocapture`
- `cargo test -p codex-tui active_subagents -- --nocapture`
- `cargo fmt --check`
- `git diff --check`

Residual non-blocking risk:

- The recheck was source-review based and did not rerun tests itself; main-agent validation already reran the focused and broader M2 regression set.
- If the fallback candidate pool is exhausted, duplicate deterministic fallback remains a current acceptable degradation rather than a blocker for this milestone.
