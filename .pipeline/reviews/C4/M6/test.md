# C4/M6 Test Worker Report

Updated: 2026-05-28T12:47:05+08:00

## Scope

Role: `test`

Milestone: M6 `Performance Evidence + Low-Risk Startup Fixes`

Allowed surface used:

- `codex-rs/tui/src/resume_picker.rs` test module only.
- `codex-rs/app-server/tests/suite/v2/initialize.rs` integration tests only.
- `.pipeline/reviews/C4/M6/test.md`.

No production runtime behavior, cache system, resume metadata cache, recent preview cache, or progressive stale-cache verification was implemented.

## Test Coverage Added

### TUI resume/history first screen

Added focused regression tests in `codex-rs/tui/src/resume_picker.rs`:

- `startup_initial_load_requests_only_first_page`
  - Verifies startup resume picker initial load emits only one first-page request with no cursor.
  - Fails if preview or transcript loading is accidentally pulled into the first-screen path.

- `ingesting_startup_page_does_not_eagerly_load_previews_or_transcripts`
  - Verifies a first page can become visible without eager preview/transcript requests.
  - Fails if first-page ingest starts historical transcript preview or full transcript work.

### VSCode/app-server initialize

Added integration coverage in `codex-rs/app-server/tests/suite/v2/initialize.rs`:

- `initialize_is_not_blocked_by_slow_plugin_startup_warmup`
  - Starts app-server with plugin startup tasks enabled.
  - Configures remote plugin warmup and a `plugins/featured` response delayed by 5 seconds.
  - Expects VSCode-style `initialize` to complete within 2 seconds.
  - This is a reproducible guard against non-first-screen plugin warmup blocking initialize.

## Validation

Commands run from `codex-rs`:

- `just fmt`
  - Passed.

- `just test -p codex-tui startup_initial_load_requests_only_first_page`
  - Passed: 1 passed / 2693 skipped.

- `just test -p codex-tui ingesting_startup_page_does_not_eagerly_load_previews_or_transcripts`
  - Passed: 1 passed / 2693 skipped.

- `just test -p codex-app-server initialize_is_not_blocked_by_slow_plugin_startup_warmup`
  - Passed: 1 passed / 790 skipped.

- `just test -p codex-tui startup`
  - Passed: 45 passed / 2649 skipped.

- `just test -p codex-app-server initialize`
  - Passed: 14 passed / 777 skipped.

## Red/Green Result

No expected-failing red test remains from this test-worker pass.

The slow plugin startup warmup guard passed, so current app-server initialize behavior is already non-blocking for that specific non-first-screen dependency.

## Remaining Evidence Gap For Implement Worker

The test coverage now protects two startup invariants, but M6 still needs a reproducible release evidence path for:

- cold TUI start,
- resume list first visible page,
- opening a historical session/transcript,
- VSCode app-server initialize.

Sanitized failure/implementation summary:

- Do not build a cache system to satisfy M6.
- Preserve the new lazy-load tests: first usable resume picker screen must not wait for transcript preview/full transcript work.
- Preserve the new initialize guard: slow plugin startup warmup must not block VSCode-style initialize.
- Implement worker should add or wire a release evidence runner/report hook if an existing script location is available, and only apply low-risk deferrals backed by that evidence.

## Notes

`codex-rs/tui/src/resume_picker.rs` and `codex-rs/app-server/tests/suite/v2/initialize.rs` already had unrelated dirty changes before this worker pass. They were preserved and not reverted.
