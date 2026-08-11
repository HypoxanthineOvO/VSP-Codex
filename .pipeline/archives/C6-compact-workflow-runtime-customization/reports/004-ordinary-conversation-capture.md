# M4 Ordinary Conversation Capture Report

Status: completed

## Summary

M4 adds lightweight ordinary conversation capture for Hypo-Workflow projects. Completed root Codex turns in a workspace with `.pipeline/` are now captured into Workflow files without making every chat a formal Cycle.

Implemented behavior:

- Root/non-subagent `TurnComplete` events passively write ordinary chat capture.
- Journal entries are stored under `.pipeline/chat/journal.yaml`.
- Conservative actionable/reusable items are stored under `.pipeline/inbox/items.yaml`.
- Repeated long-topic continuity can create Mini-Cycle state under `.pipeline/chats/<id>/state.yaml`.
- Replay dedupe is stable for long turn/event IDs by using a pre-truncation hash key.
- Captured text is bounded and redacts common secret shapes.
- Journal, Inbox, and Mini-Cycle outputs keep a recent bounded window.
- Compact fallback includes recent Journal and Inbox capture through existing fallback sources.

## Files Changed

- `codex-rs/core/src/workflow_conversation_capture.rs`
- `codex-rs/core/src/workflow_context.rs`
- `codex-rs/core/src/session/mod.rs`
- `codex-rs/core/src/lib.rs`
- `codex-rs/core/tests/suite/workflow_conversation_capture.rs`
- `codex-rs/core/tests/suite/mod.rs`

## Validation

Final validation passed:

- `cargo test -p codex-core workflow_conversation_capture -- --nocapture`: 9 passed
- `cargo fmt --check`: exited 0 with existing stable rustfmt warnings
- `git diff --check`: passed

## Review Outcome

Initial audit returned `needs_changes` because capture was only an API, not wired into real root turn completion, and because bounds/redaction/long-id dedupe needed hardening. Fix-round tests and implementation addressed those findings. Reaudit passed.

Known non-blocking follow-up:

- `session/mod.rs` still emits one deprecated `turn_context.cwd` warning in the new capture path. Reaudit accepted it as non-blocking because behavior is tested and guarded; it can be cleaned up separately by using the selected turn environment cwd.
