# C3/M13 Audit: Subagent Nickname Candidate Pool Configuration

verdict: pass

Read-only re-audit of the M13 finding after the TUI fallback fix. Scope was limited to the current workspace implementation, schema/tests evidence, and this audit artifact. No production code, tests, or schema files were modified during this review.

## Re-audit Result

The previous finding is fixed. TUI active-subagent fallback now follows the required effective-pool order for the checked path:

1. configured role-level candidates
2. top-level configured candidates
3. built-in role-specific candidates
4. default public fallback

Evidence:

- `codex-rs/tui/src/chatwidget/active_subagents.rs:541`-`:580` defines `configured_fallback_subagent_nickname`.
- `codex-rs/tui/src/chatwidget/active_subagents.rs:556`-`:566` checks `metadata.agent_role` against `config.agent_roles[role].nickname_candidates` first.
- `codex-rs/tui/src/chatwidget/active_subagents.rs:567`-`:573` then checks `config.agent_nickname_candidates`.
- `codex-rs/tui/src/chatwidget/active_subagents.rs:574`-`:577` then uses `built_in_role_fallback_names(metadata.agent_role.as_deref())`.
- `codex-rs/tui/src/chatwidget/active_subagents.rs:582`-`:590` maps built-in roles to role-specific pools: `supervisor`, `explorer`/`simple`, `worker`, and `default`.
- If no stored fallback is available, the render-time fallback remains the default public pool via `fallback_subagent_name` at `codex-rs/tui/src/chatwidget/active_subagents.rs:592`-`:600`.

## Metadata Late Arrival

Pass. Metadata arriving after an active row exists now recomputes and stores the fallback nickname.

Evidence:

- `ChatWidget::set_collab_agent_metadata` stores the latest metadata at `codex-rs/tui/src/chatwidget.rs:891`-`:902`.
- It recomputes fallback with `active_subagents::configured_fallback_subagent_nickname(&self.config, thread_id, &metadata)` at `codex-rs/tui/src/chatwidget.rs:903`-`:907`.
- It then updates any existing active row through `active_subagents.update_metadata(thread_id, metadata, fallback_nickname)` at `codex-rs/tui/src/chatwidget.rs:908`-`:913`.
- `ActiveSubagentPanelState::update_metadata` writes both metadata and `fallback_nickname` at `codex-rs/tui/src/chatwidget/active_subagents.rs:200`-`:215`.
- The existing late-metadata test still exercises row refresh after metadata arrives in `codex-rs/tui/src/chatwidget/tests/app_server.rs:728`-`:779`.

## Focused Test Coverage

Pass. There is now a focused test for the previously missing explorer built-in role fallback.

Evidence:

- `active_subagents_fallback_uses_built_in_role_nickname_candidates` is defined at `codex-rs/tui/src/chatwidget/tests/app_server.rs:881`-`:924`.
- The test sets metadata with `agent_role = Some("explorer")` and no nickname at `codex-rs/tui/src/chatwidget/tests/app_server.rs:887`.
- It asserts the rendered active row contains either `墨翟 [explorer]` or `庄周 [explorer]`, matching the explorer built-in role pool, at `codex-rs/tui/src/chatwidget/tests/app_server.rs:920`-`:923`.
- Existing adjacent tests cover top-level configured fallback at `codex-rs/tui/src/chatwidget/tests/app_server.rs:783`-`:825` and configured role-specific fallback winning over top-level at `codex-rs/tui/src/chatwidget/tests/app_server.rs:828`-`:878`.

## Residual Notes

- The TUI still mirrors the core built-in candidate pools locally rather than sharing the exact core resolver. The checked mappings match the current built-in roles, but future built-in role changes could drift unless tests or shared code keep them aligned.
- I did not run tests in this re-audit; the conclusion is based on static inspection of the current implementation and test code.
