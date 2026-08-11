# C6/M4 Test Worker Report

## Scope

Milestone: M4 `Ordinary Conversation Capture`

Edited files:

- `codex-rs/core/tests/suite/workflow_conversation_capture.rs`
- `codex-rs/core/tests/suite/mod.rs`
- `.pipeline/reviews/C6/M4/test.md`

## Added Red Tests

- `ordinary_root_chat_appends_bounded_journal_entries`
  - Expects ordinary completed turns to append bounded entries to `.pipeline/chat/journal.yaml`.
- `actionable_content_creates_conservative_inbox_items`
  - Expects actionable/reusable ordinary chat content to create conservative items in `.pipeline/inbox/items.yaml` with source-turn evidence.
- `repeated_long_topic_auto_upgrades_to_mini_cycle_state`
  - Expects repeated long-topic continuity to create a Mini-Cycle state under `.pipeline/chats/<id>/state.yaml`.
- `completed_turn_replay_does_not_append_duplicates`
  - Expects replaying the same completed turn id/event id to avoid duplicate journal entries.
- `compact_fallback_includes_recent_ordinary_conversation_capture`
  - Exercises the existing compact fallback integration seam and expects recent ordinary journal/inbox capture to be visible.

## Red Validation

Command run from `codex-rs/`:

```bash
cargo test -p codex-core workflow_conversation_capture --no-run
```

Result: red / compile-fail as expected.

Failure summary:

- `codex_core::workflow_context` is currently private, so the fallback integration helper is not accessible from integration tests.
- `codex_core::workflow_conversation_capture` does not exist yet, so the intended public capture API is missing.

This failure matches M4's missing production behavior: there is no public turn-completion conversation capture API yet, no journal/inbox writer, no Mini-Cycle auto-upgrade API, and no replay dedupe surface available to the core integration tests.

## Test-Fix Validation

Worker: M4 test-fix

Edited files:

- `codex-rs/core/tests/suite/mod.rs`
- `codex-rs/core/tests/suite/workflow_conversation_capture.rs`
- `.pipeline/reviews/C6/M4/test.md`

Fix summary:

- Reordered `workflow_conversation_capture`, `workflow_files`, and `workflow_routing` modules to match rustfmt ordering.
- Expanded the bounded journal `assert_eq!` to rustfmt's multi-line shape.

Commands run from `codex-rs/`:

```bash
cargo fmt --check
cargo test -p codex-core workflow_conversation_capture
```

Results:

- `cargo fmt --check`: passed with exit code 0; output only contained the existing stable-toolchain warning for nightly-only `imports_granularity = Item`.
- `cargo test -p codex-core workflow_conversation_capture`: passed; 5 tests passed, 0 failed.

## Needs-Changes Test-Fix Validation

Worker: M4 needs_changes test-fix

Edited files:

- `codex-rs/core/tests/suite/workflow_conversation_capture.rs`
- `.pipeline/reviews/C6/M4/test.md`

Added red coverage for audit findings:

- `ordinary_root_session_turn_passively_writes_workflow_capture_files`
  - Builds a real root Codex session with local mock SSE, creates `.pipeline`, submits ordinary user input, waits through `submit_turn`/`TurnComplete`, then asserts `.pipeline/chat/journal.yaml` and `.pipeline/inbox/items.yaml` exist and contain ordinary conversation capture.
- `completed_turn_replay_with_long_ids_does_not_append_duplicates`
  - Replays a turn whose `turn_id` and `completion_event_id` exceed `max_entry_bytes`, expecting dedupe to happen before truncation.
- `captured_text_redacts_common_secret_shapes`
  - Captures text containing `Authorization: Bearer ...`, `cookie=...`, `password=...`, and `ghp_...`, expecting secrets not to appear in journal output.
- `inbox_and_mini_cycle_outputs_remain_bounded`
  - Appends multiple actionable long-topic turns, expecting Inbox and Mini-Cycle outputs to remain bounded instead of growing without limit.

Commands run from `codex-rs/`:

```bash
cargo test -p codex-core workflow_conversation_capture --no-run
cargo test -p codex-core workflow_conversation_capture -- --nocapture
cargo fmt --check
```

Results:

- `cargo test -p codex-core workflow_conversation_capture --no-run`: passed; tests compile against the current public API.
- `cargo test -p codex-core workflow_conversation_capture -- --nocapture`: red as expected; 5 passed, 4 failed.
- `cargo fmt --check`: passed with exit code 0; output only contained the existing stable-toolchain warning for nightly-only `imports_granularity = Item`.

Red failure summary:

- `ordinary_root_session_turn_passively_writes_workflow_capture_files`: failed because `.pipeline/chat/journal.yaml` was not created after a real ordinary root session `TurnComplete`, confirming capture is not yet wired into the session completion path.
- `completed_turn_replay_with_long_ids_does_not_append_duplicates`: failed with 2 journal entries instead of 1, confirming replay dedupe compares against truncated stored ids and duplicates long-id events.
- `captured_text_redacts_common_secret_shapes`: failed because `bearer-secret-123` and related common secret shapes remained in journal output.
- `inbox_and_mini_cycle_outputs_remain_bounded`: failed because Inbox retained 8 items, confirming the output is not bounded.
