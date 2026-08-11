# C6/M4 Implement Report - Ordinary Conversation Capture

role: implement
agent: replacement
status: completed
verdict: pass_with_scope_note
milestone: C6/M4 Ordinary Conversation Capture

## Changed Paths

- `codex-rs/core/src/workflow_conversation_capture.rs`
- `codex-rs/core/src/lib.rs`
- `codex-rs/core/src/workflow_context.rs`
- `.pipeline/reviews/C6/M4/implement.md`

No additional production code edits were required in this replacement pass. The existing production changes were inspected and validated.

## Implemented Behavior

- Adds `workflow_conversation_capture` as a public core module.
- Provides `ConversationCapture::open` and `capture_completed_turn` for ordinary Codex conversation capture under a project-local `.pipeline/` directory.
- Writes bounded recent ordinary chat turns to `.pipeline/chat/journal.yaml`.
- Deduplicates completed-turn replay by `(turn_id, completion_event_id)`.
- Creates conservative Inbox items in `.pipeline/inbox/items.yaml` for actionable or reusable conversation content.
- Auto-upgrades repeated long-topic ordinary chat into a Mini-Cycle state at `.pipeline/chats/<mini-cycle-id>/state.yaml`.
- Bounds captured text and applies basic secret redaction for common API-token shapes.
- Extends compact fallback context to include recent ordinary conversation journal and Inbox sources.

## Validation

Run from `codex-rs/`:

- `cargo test -p codex-core workflow_conversation_capture --no-run`
  - Result: passed.
- `cargo test -p codex-core workflow_conversation_capture`
  - Result: passed, 5 passed / 0 failed / 0 ignored for the focused suite.
- `cargo fmt --check`
  - Result: failed.
  - Cause: formatting diffs are in test-owned files outside this implement worker scope:
    - `codex-rs/core/tests/suite/mod.rs`
    - `codex-rs/core/tests/suite/workflow_conversation_capture.rs`
  - The command also emitted existing stable-toolchain warnings for nightly-only `imports_granularity = Item`.

## Limitations

- This worker did not edit tests, fixtures, snapshots, state, log, progress, or unrelated files per ownership constraints.
- The remaining `cargo fmt --check` failure requires a test-scope formatting pass by the test worker or orchestrator.
- The capture API is production-side plumbing; broader automatic invocation from every ordinary Codex turn is not asserted by this focused implementation report unless separately wired by a later integration milestone.

## Audit-Fix Implementation - 2026-05-29

role: needs_changes implement-fix
status: completed
verdict: pass

### Changed Paths

- `codex-rs/core/src/workflow_conversation_capture.rs`
- `codex-rs/core/src/session/mod.rs`
- `.pipeline/reviews/C6/M4/implement.md`

### Implemented Fixes

- Wired root/non-subagent `TurnComplete` events into passive Workflow conversation capture when the turn cwd resolves to a project with `.pipeline/`.
- Captures the latest user message from session history plus the final assistant message from `TurnComplete`, and skips empty or non-root/subagent/internal turns.
- Added stable hashed dedupe keys before display-field truncation, so long `turn_id` / `completion_event_id` replay does not append duplicates.
- Broadened secret redaction for bearer auth, cookies, password assignments, `ghp_` tokens, and existing API-key shapes.
- Bounded Inbox and Mini-Cycle outputs using the configured recent-window limit.

### Validation

- `cargo test -p codex-core workflow_conversation_capture -- --nocapture`
  - Result: passed, 9 passed / 0 failed.
- `cargo fmt --check`
  - Result: passed.
  - Note: emitted existing stable-toolchain warnings for nightly-only `imports_granularity = Item`.
- `git diff --check -- codex-rs/core/src/workflow_conversation_capture.rs codex-rs/core/src/session/mod.rs .pipeline/reviews/C6/M4/implement.md`
  - Result: passed.

### Limitations

- Capture intentionally remains conservative: only completed root turns with non-empty final assistant text are recorded.
- This fix does not record subagent, internal, aborted, or partial/interrupted turns as ordinary root chat.
