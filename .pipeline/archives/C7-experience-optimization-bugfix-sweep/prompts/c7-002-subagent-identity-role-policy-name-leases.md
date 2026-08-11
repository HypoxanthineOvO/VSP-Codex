# M2 Subagent Identity, Role Policy, And Name Leases

## Objective

Make Subagents appear as stable named collaborators instead of UUID-backed internal threads, and make role plus explicit reasoning effort compose predictably.

## Source Of Truth

- P1 issues: Issue 3 channel-specific Subagent name pools; Issue 6 lifecycle rows expose UUIDs, role/effort overrides conflict, and names are not leased.
- P2 technical solution: introduce one authoritative Subagent identity resolver shared by lifecycle history rows and the active Subagents panel; role configs supply defaults and explicit spawn values override unless locked.
- Relevant modules: `codex-rs/tui/src/multi_agents.rs`, `codex-rs/tui/src/chatwidget/active_subagents.rs`, `codex-rs/core/src/tools/handlers/multi_agents/spawn.rs`, `codex-rs/core/src/tools/handlers/multi_agents_v2/spawn.rs`, and `codex-rs/core/src/tools/handlers/multi_agents_common.rs`.

## Subworker Assignment Plan

Status: authorized, strict worker separation.

- `test`
  - Scope: owns failing tests, fixtures, assertions, and snapshots for lifecycle labels, active-panel names, name leases, release/reuse policy, and role/model/reasoning precedence.
  - Write boundary: test/snapshot files and `.pipeline/reviews/C7/M2/test.md` only.
  - Expected artifact: `.pipeline/reviews/C7/M2/test.md`.
- `implement`
  - Scope: owns production TUI/core code for display-name resolver, lease table, role default/lock semantics, and channel default name hooks.
  - Must not create, edit, rewrite, or satisfy tests owned by `test`.
  - Expected artifact: `.pipeline/reviews/C7/M2/implement.md`.
- `audit`
  - Scope: read-only review of name fallback behavior, lease release behavior, role override semantics, user nickname priority, and internal ID visibility.
  - Must reject fake tests that only instantiate resolver helpers without lifecycle-row or spawn-policy coverage.
  - Expected artifact: `.pipeline/reviews/C7/M2/audit.md`.

Workers must not revert user or other-worker edits. Scope violations stop the worker and must be reported. Required user decisions must use `request_user_input` when available. Destructive operations, system-level dependency installation, network/remote side effects, and release publication require explicit confirmation/authorization.

## Technical Solution

Create a shared Subagent identity resolver keyed by `ThreadId`. On first sight, assign a stable display name from explicit metadata, role nickname candidates, user-configured candidates, release-channel defaults, or built-ins. Lease display names while active and release them on terminal lifecycle states according to a documented policy. Keep raw thread IDs available only in detail/debug/search contexts. In core spawn handling, apply role defaults first, then explicit `model`/`reasoning_effort` overrides unless the role marks those fields locked.

## Technical Route

1. Extract or centralize fallback nickname logic currently split between `multi_agents.rs` and `active_subagents.rs`.
2. Add a lease table keyed by `ThreadId`, with deterministic fallback when the candidate pool is exhausted.
3. Release names on close/shutdown/not_found and define conservative behavior for completed-but-still-inspectable agents.
4. Replace UUID fallback in lifecycle rows for spawned, sent input, waiting, completed, and closed states with leased display names.
5. Preserve raw IDs in detail views and debug contexts.
6. Add role default/lock fields or equivalent semantics in core spawn config application.
7. Change spawn application order to role defaults first, explicit `model`/`reasoning_effort` second when allowed.
8. Update tests and tool descriptions that currently claim role model/reasoning settings cannot be changed.
9. Add channel/default nickname-pool selection hooks without hardcoding private internal lists into generic core behavior.

Non-goals: full Subagent prompt/report viewer, which is M4. Full-history fork override relaxation remains deferred unless explicitly requested.

## Research Required

- Resolved by local evidence: display-name fallback and role override code paths have been identified.
- Internal VSP Lab full name pool is not required for implementation; use configurable/build-channel candidates and fixture names from P1 such as `娄鑫`, `李超凡`, and `陈泉宇`.
- Full-history fork keeps inherited-context override restrictions in this milestone unless a later audit finds a direct bug.

## Risks And Alternatives

- Risk: stale names after abnormal shutdown. Mitigation: release on terminal states and use deterministic fallback if the pool is exhausted.
- Risk: role lock semantics break existing configs. Mitigation: default roles to default-overridable unless explicit lock fields are present, then update descriptions.
- Rejected alternative: hide UUIDs only in one row. That would leave active panel/history consistency broken.

## Validation Path

Required validation:

- `cargo test -p codex-tui multi_agents active_subagents -- --nocapture` with new snapshots for spawn/wait/close labels.
- `cargo test -p codex-core subagent_notifications -- --nocapture` or narrower spawn-policy tests covering explicit reasoning effort with roles.
- `cargo fmt --check`.
- `git diff --check`.

Manual acceptance:

- Spawn several Subagents with roles and explicit reasoning efforts.
- Verify display names stay stable across lifecycle rows.
- Verify no UUID appears as the primary label when fallback names exist.
- Verify names are reused after release.

Pass signal: lifecycle rows and active panel consistently use stable display names; role defaults compose with explicit reasoning effort according to lock policy; configured/user names override channel defaults.

Fail signal: UUIDs remain primary labels, names collide while candidates are available, explicit reasoning effort is silently discarded, or internal IDs disappear from intended debug/detail surfaces.

## Audit Focus

- Verify display name never becomes the thread ID in primary UI when fallback candidates exist.
- Verify explicit reasoning effort is not silently discarded.
- Verify user-configured nicknames override role/channel defaults.
- Verify internal IDs remain available in intended detail/debug contexts.
