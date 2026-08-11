# C4/M2 Audit Worker Report

verdict: pass

## reviewed_refs

- `.pipeline/prompts/002-canonical-rename.md`
- `.pipeline/reviews/C4/M2/test.md`
- `.pipeline/reviews/C4/M2/implement.md`
- `.pipeline/reports/002-canonical-rename.md`
- `codex-rs/app-server/tests/suite/v2/thread_read.rs`
- `codex-rs/rollout/src/session_index_tests.rs`
- `codex-rs/app-server/src/request_processors/thread_processor.rs`
- `codex-rs/thread-store/src/local/update_thread_metadata.rs`
- `codex-rs/rollout/src/session_index.rs`
- `codex-rs/tui/src/chatwidget/slash_dispatch.rs`
- `codex-rs/tui/src/chatwidget/interaction.rs`
- `codex-rs/tui/src/app/thread_routing.rs`
- `codex-rs/tui/src/chatwidget/protocol.rs`
- `codex-rs/tui/src/chatwidget/session_flow.rs`
- `codex-rs/tui/src/resume_picker.rs`

## findings

- No blocking findings.
- Canonical metadata is the source of truth for app-server rename. `thread_set_name_response_inner` normalizes the requested name, rejects empty names before writes, and calls `update_thread_metadata` with `StoreThreadMetadataPatch { name: Some(Some(name.clone())) }`. This is not a TUI-only alias path.
- `thread/name/updated` is emitted only after the metadata write returns success. `thread_set_name` sends the response and notification only after `thread_set_name_response_inner` returns `Ok`; write errors return JSON-RPC errors and do not emit the notification.
- The metadata write is visibility-oriented and appropriately transactional for this milestone: `apply_thread_name` updates the state DB title and then appends the rollout compatibility index. A failure in either path propagates as a thread-store error, so app-server does not report success or notify clients when rollout compatibility indexing fails.
- Loaded, stored, restart, list, read, and resume visibility have coverage. Existing `thread_name_set_is_reflected_in_read_list_and_resume` covers read/list/resume after rename; the M2 test worker added restart persistence and transcript-length checks.
- Empty names are rejected without saving or transcript pollution. App-server and TUI both normalize names before submission/write. The added app-server test verifies whitespace-only names return an error, leave `thread.name` unset, and keep transcript turns unchanged.
- Rollout compatibility is coherent for batch visible-name lookup. `find_thread_names_by_ids` keys by `ThreadId`, overwrites only with non-empty names while scanning append order, and the new test verifies a blank latest entry does not hide the prior visible name.
- Duplicate names are not being used as identity on the audited app-server/TUI display paths. App-server `thread/read`, `thread/list`, `thread/resume`, TUI notification handling, and resume picker rows carry `thread_id`; names are display/search labels. Existing rollout name lookup by name has guardrails that skip stale historical names per id and keep scanning when the newest matching id is not loadable.
- The implement worker's no-op production decision is reasonable. The runtime behavior requested by `.pipeline/prompts/002-canonical-rename.md` is already present; the M2 changes add targeted regression coverage rather than duplicating production logic.

## validation

- I did not run fmt/fix/tests in this audit worker because the assignment is read-only and explicitly prohibits fmt/fix. Validation is by source/test inspection plus the test worker's recorded results.
- Test worker recorded:
  - `cd codex-rs && just fmt`: passed.
  - `cd codex-rs && just test -p codex-app-server thread_name`: passed, 5/5.
  - `cd codex-rs && just test -p codex-tui rename`: passed, 9/9.
  - `cd codex-rs && just test -p codex-rollout thread_name`: passed, 3/3.
  - `cd codex-rs && just test -p codex-tui live_app_server_thread_name_update_shows_resume_hint`: passed, 1/1.
- Source inspection confirmed the notification is post-write, canonical metadata is read back by app-server attach paths, rollout indexing failure propagates before success reporting, and TUI visible updates depend on `ThreadNameUpdated` for the active thread id.

## residual risks

- `find_thread_name_by_id` returns the latest entry even if it is blank, while the audited batch list path deliberately ignores blanks. I did not find this used on the M2 app-server/TUI list/read/resume display path, so this is not a blocking M2 issue, but future callers should prefer the batch behavior or normalize blank results.
- The restart test covers stored rollout-backed threads. I did not see a new dedicated duplicate-name end-to-end test for app-server rename, but audited identity-sensitive paths use `thread_id`, and existing rollout tests cover same-name/stale-name behavior.
- TUI failure UX for a write failure relies on app-server error propagation and absence of `ThreadNameUpdated`; the audited code does not optimistically change `thread_name`, so stale visible state should remain unchanged.

## scope boundary

- This audit only wrote `.pipeline/reviews/C4/M2/audit.md`.
- No production code, tests, snapshots, reports other than this audit artifact, M1 artifacts, or non-scope files were modified by this worker.
- Current worktree contains many unrelated C4 and repository changes outside M2. I treated them as pre-existing/user/other-worker changes and did not modify or revert them.
