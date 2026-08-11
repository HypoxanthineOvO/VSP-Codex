# C6 M4 Audit

Verdict: needs_changes

Auditor: `019e730a-640c-7151-ac39-fd217c0ef036`

## Blocking Findings

1. `codex-rs/core/src/workflow_conversation_capture.rs` exposes `capture_completed_turn()` but the production runtime does not call `ConversationCapture`.
   - Ordinary root Codex turns are not automatically recorded after `TurnComplete`.
   - Current tests call the helper directly, so they do not prove passive ordinary conversation capture.

2. Inbox and Mini-Cycle records are not sufficiently bounded.
   - Inbox deduplicates by source turn but has no max item/window limit.
   - Mini-Cycle state has no total/window control.

3. Secret redaction is too narrow.
   - It only handles a few whitespace-delimited `starts_with` tokens.
   - It misses common forms such as bearer authorization headers, cookies, passwords, and `ghp_` tokens.

4. Replay dedupe is unstable for long IDs.
   - Existing detection compares stored bounded/redacted ids with raw incoming ids.
   - If `turn_id` or `completion_event_id` is truncated, replay can append duplicates.

## Passing Observations

- Compact fallback already includes `.pipeline/chat/journal.yaml` and `.pipeline/inbox/items.yaml`.
- Writes are scoped to `.pipeline/chat`, `.pipeline/inbox`, and `.pipeline/chats`; no direct mutation of `.pipeline/state.yaml`, `.pipeline/cycle.yaml`, or `.pipeline/rules.yaml` was observed.

## Read-Only Validation

- `cargo test -p codex-core workflow_conversation_capture`: 5 passed
- `cargo fmt --check`: exited 0 with existing stable rustfmt warnings
- `git diff --check`: passed

## Reaudit After Fix

Verdict: pass

Reaudit worker: `019e7326-6e06-7850-8a38-8941d801ef8b`

Result:

- Root/non-subagent `TurnComplete` events are now passively captured for Workflow projects with `.pipeline/`.
- Non-terminal, aborted, partial, subagent, and internal turns are not treated as ordinary root chat.
- Long ID replay dedupe uses a stable hash before display-field truncation.
- Redaction covers the tested bearer auth, cookie, password, `ghp_`, and API-key shapes.
- Journal, Inbox, and Mini-Cycle files are bounded by the recent-window option.
- Capture writes stay under `.pipeline/chat`, `.pipeline/inbox`, and `.pipeline/chats`; formal Cycle files are not directly mutated.
- Compact fallback still includes Journal and Inbox sources.
- The `turn_context.cwd` deprecated warning is non-blocking and can be cleaned separately.

Validation observed:

- `cargo test -p codex-core workflow_conversation_capture -- --nocapture`: 9 passed
- `cargo fmt --check`: exited 0 with existing stable rustfmt warnings
- `git diff --check`: passed
