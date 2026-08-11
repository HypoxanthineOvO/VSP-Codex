# M2 Canonical Rename

## Status

Implemented as a production no-op after review. The canonical rename behavior required by `.pipeline/prompts/002-canonical-rename.md` is already present in the current runtime code and covered by the M2 test worker.

## Runtime Behavior

- `thread/name/set` is the canonical rename entry point.
- Empty or whitespace-only names are rejected before metadata writes.
- Successful rename writes `ThreadMetadataPatch.name` through thread-store metadata.
- Thread-store updates the canonical state DB title and appends the rollout compatibility thread-name index.
- `thread/name/updated` is emitted only after the metadata write succeeds.
- `thread/read`, `thread/list`, and resume picker display paths read the canonical thread name.
- TUI `/rename` and the rename prompt call app-server `thread/name/set` and update visible UI only from `thread/name/updated`.
- Rollout compatibility lookup preserves the latest non-empty indexed name by thread id, so blank latest entries do not hide a valid name.

## Evidence

- App-server canonical write and notification: `codex-rs/app-server/src/request_processors/thread_processor.rs`
- Thread-store metadata and rollout compatibility update: `codex-rs/thread-store/src/local/update_thread_metadata.rs`
- Rollout batch name lookup: `codex-rs/rollout/src/session_index.rs`
- TUI rename request path: `codex-rs/tui/src/chatwidget/slash_dispatch.rs`, `codex-rs/tui/src/chatwidget/interaction.rs`, `codex-rs/tui/src/app/thread_routing.rs`
- TUI notification update path: `codex-rs/tui/src/chatwidget/protocol.rs`, `codex-rs/tui/src/chatwidget/session_flow.rs`
- Resume picker display path: `codex-rs/tui/src/resume_picker.rs`

## Validation

M2 test worker validation was reused:

- `cd codex-rs && just fmt`: passed.
- `cd codex-rs && just test -p codex-app-server thread_name`: passed, 5/5.
- `cd codex-rs && just test -p codex-tui rename`: passed, 9/9.
- `cd codex-rs && just test -p codex-rollout thread_name`: passed, 3/3.
- `cd codex-rs && just test -p codex-tui live_app_server_thread_name_update_shows_resume_hint`: passed, 1/1.

No Rust production changes were made in this implement pass, so this worker did not rerun formatting, fix, or tests.
