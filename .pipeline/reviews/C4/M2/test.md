# C4/M2 Test Worker Report

## Scope

- Added app-server coverage for canonical rename persistence across an app-server restart.
- Added app-server failure-path coverage for blank rename requests: rejects the request, leaves `thread.name` unset, and does not add transcript turns.
- Added rollout compatibility coverage for batched thread-name lookup ignoring blank latest index entries.
- Reused existing TUI rename/resume-hint coverage and verified the focused snapshot-bearing test.

## Changed Files

- `codex-rs/app-server/tests/suite/v2/thread_read.rs`
- `codex-rs/rollout/src/session_index_tests.rs`
- `.pipeline/reviews/C4/M2/test.md`

## Validation

- `cd codex-rs && just fmt`
  - Passed.
- `cd codex-rs && just test -p codex-app-server thread_name`
  - Passed: 5 tests run, 5 passed.
  - Covered loaded and stored websocket rename notification tests, read/list/resume visibility, restart persistence, and empty-name failure path.
- `cd codex-rs && just test -p codex-tui rename`
  - Passed: 9 tests run, 9 passed.
  - Covered TUI slash rename flows and renamed footer snapshot.
- `cd codex-rs && just test -p codex-rollout thread_name`
  - Passed: 3 tests run, 3 passed.
  - Covered rollout thread-name compatibility lookups including blank latest-entry handling.
- `cd codex-rs && just test -p codex-tui thread_name_update_resume_hint`
  - Failed with 0 tests matched; filter was snapshot-name based, not test-name based.
- `cd codex-rs && just test -p codex-tui live_app_server_thread_name_update_shows_resume_hint`
  - Passed: 1 test run, 1 passed.
  - Verified existing resume-hint snapshot coverage for app-server `thread/name/updated`.

## Signal

Green on the focused coverage present in this workspace. The app-server restart test confirms rename is durable canonical thread metadata visible through `thread/read` and `thread/list` after process restart, and the transcript length remains unchanged. The empty-name test confirms invalid rename does not save a blank name or append transcript content.
