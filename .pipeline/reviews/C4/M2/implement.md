# C4/M2 Implement Worker Review

## Scope

Reviewed the production rename path for `.pipeline/prompts/002-canonical-rename.md` against the test worker report in `.pipeline/reviews/C4/M2/test.md`.

This implementation pass is a no-op for production code. The canonical rename behavior is already present in the runtime paths, and the focused tests added by the test worker are green.

Changed only implementation-owned artifacts:

- `.pipeline/reports/002-canonical-rename.md`
- `.pipeline/reviews/C4/M2/implement.md`

No production runtime files, tests, fixtures, snapshots, audit artifacts, or M1 artifacts were modified.

## Production Evidence

- App-server `thread/name/set` normalizes and rejects empty names before writing metadata. Evidence: `codex-rs/app-server/src/request_processors/thread_processor.rs` `thread_set_name_response_inner`.
- The app-server write uses `StoreThreadMetadataPatch { name: Some(Some(name.clone())) }`, so rename updates canonical thread metadata rather than a TUI-only alias. Evidence: `codex-rs/app-server/src/request_processors/thread_processor.rs` `thread_set_name_response_inner`.
- The app-server sends `thread/name/updated` only after the metadata update succeeds. Evidence: `codex-rs/app-server/src/request_processors/thread_processor.rs` `thread_set_name`.
- Thread read/list paths attach names from stored thread metadata through `attach_thread_name`, preserving canonical metadata visibility for stored and loaded threads. Evidence: `codex-rs/app-server/src/request_processors/thread_processor.rs` `attach_thread_name` and its read/list call sites.
- Thread-store metadata updates mark name changes as needing rollout compatibility, then call `apply_thread_name`, which updates the state DB title and appends the compatibility index entry. Evidence: `codex-rs/thread-store/src/local/update_thread_metadata.rs` `needs_rollout_compatibility_update` and `apply_thread_name`.
- Rollout batch lookup ignores blank latest entries and keeps the last non-empty indexed name for each requested thread id. Evidence: `codex-rs/rollout/src/session_index.rs` `find_thread_names_by_ids`.
- TUI `/rename` and rename prompt normalize non-empty names and send `AppCommand::SetThreadName`; app routing forwards that to app-server `thread/name/set`. Evidence: `codex-rs/tui/src/chatwidget/slash_dispatch.rs`, `codex-rs/tui/src/chatwidget/interaction.rs`, and `codex-rs/tui/src/app/thread_routing.rs`.
- TUI handles `ThreadNameUpdated` by validating the thread id, updating `chat.thread_name`, refreshing status surfaces, requesting redraw, and continuing queued input. Evidence: `codex-rs/tui/src/chatwidget/protocol.rs` and `codex-rs/tui/src/chatwidget/session_flow.rs`.
- Resume picker rows prefer `Thread.name` when present and map app-server thread list rows into `thread_name`. Evidence: `codex-rs/tui/src/resume_picker.rs`.

## No-op Rationale

The requested production behavior is already implemented across the canonical metadata source of truth, app-server visibility, notification delivery, TUI update path, and rollout compatibility path. Adding code would duplicate existing behavior or increase churn without closing a demonstrated gap.

The test worker added focused regression coverage for the previously risky cases:

- app-server restart persistence for `thread/name/set`
- rejection of empty names without metadata or transcript pollution
- rollout `find_thread_names_by_ids` blank-latest-entry compatibility
- existing TUI rename and resume-hint coverage

Because those focused checks passed, this worker did not make Rust production changes and therefore did not run `just fmt` or `just fix`.

## Validation

Reused test worker validation from `.pipeline/reviews/C4/M2/test.md`:

- `cd codex-rs && just fmt`: passed.
- `cd codex-rs && just test -p codex-app-server thread_name`: passed, 5/5.
- `cd codex-rs && just test -p codex-tui rename`: passed, 9/9.
- `cd codex-rs && just test -p codex-rollout thread_name`: passed, 3/3.
- `cd codex-rs && just test -p codex-tui live_app_server_thread_name_update_shows_resume_hint`: passed, 1/1.

No additional tests were run by this implement worker because the production implementation was unchanged.
