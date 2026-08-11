# C8 Cycle Summary

Status: completed

## Summary

C8 completed the VSCode Remote SSH Session Compatibility And Plugin UX cycle without publishing, tagging, releasing, pushing, or modifying VSCode plugin source. The cycle focused on making VSP-Codex sessions usable from Remote SSH VSCode plugin contexts, preserving official Codex compatibility options, and improving Request Input / Workflow Markdown fallback behavior.

## Completed Milestones

- M1 Remote SSH VSCode integration probe: added `vsp-codex vscode` status/install behavior and tests for Remote SSH server Machine settings priority.
- M2 VSCode plugin protocol capability map: documented plugin protocol capability and no-plugin-fork constraints.
- M3 provider and resume compatibility contract: fixed resume behavior so empty provider metadata does not break plugin session resume.
- M4 official Codex session discovery strategy: added sanitized export support for official Codex discovery without raw/symlink sharing of VSP session data.
- M5 Request Tool / Request Input plugin UX: added explicit Workflow metadata propagation and readable Markdown fallback for unsupported native Ask clients.
- M6 Markdown Workflow surface and Remote SSH acceptance: added Markdown surface contracts and wired Request Input fallback emission into live plugin-facing `ThreadItem::AgentMessage` output.

## Final Validation

- `cargo test -p codex-app-server --test all request_user_input -- --nocapture`: 11 passed.
- `cargo test -p codex-app-server --test all workflow_markdown_surface -- --nocapture`: 2 passed.
- `cargo test -p codex-app-server --test all thread_resume -- --nocapture`: 37 passed.
- `cargo test -p codex-rollout --test official_codex_discovery`: 8 passed.
- `cargo test -p codex-cli --test vscode`: 10 passed.
- `cargo fmt --check --package codex-app-server --package codex-app-server-protocol --package codex-protocol --package codex-core --package codex-rollout --package codex-cli --package codex-tui`: passed.
- `git diff --check`: passed.

Known non-blocking warnings remained unchanged: stable rustfmt reports `imports_granularity = Item` as nightly-only, `codex-core` still warns about deprecated `TurnContext::cwd`, and app-server tests emit expected temporary trust/mock-model/file-watcher warnings.

## Review And Fixes

- M4 audit found overclaiming and sanitization risks in official Codex export; audit-fix tests and implementation addressed sanitized titles, duplicate basenames, symlink traversal, and rollout-reader compatibility.
- M5 audit found Workflow metadata derived from reserved ids; audit-fix implementation made metadata explicit from tool args through core/app-server/protocol.
- M6 audit found Markdown helper not wired into live emission; audit-fix implementation wired live unsupported native Ask fallback and recovered pending fallback replay to the M6 Markdown wrapper.
- Final validation found a TUI compile regression from new `ToolRequestUserInputQuestion` fields; final fix added `None` metadata defaults to TUI fixtures/helpers and passed independent audit.

## Manual Acceptance Note

Remote SSH manual acceptance remains a checklist artifact, not a completed external release claim. Use `.pipeline/reviews/C8/M6/acceptance-checklist.md` to record real VSCode Remote SSH plugin observations before any release or external acceptance statement.
