# C8 M3 Test Worker Report

## Files Changed

- `codex-rs/app-server/tests/suite/v2/thread_resume.rs`
- `.pipeline/reviews/C8/M3/test.md`

No production code, docs, prompts, state, cycle, progress, log, lock, reports, or audit files were edited.

## Tests Added

- `c8_m3_thread_resume_empty_model_provider_falls_back_to_configured_provider`
  - Sends `thread/resume` with explicit `modelProvider: ""`.
  - Asserts resume succeeds through configured `mock_provider`.
- `c8_m3_thread_resume_omitted_vsp_provider_falls_back_without_persisting_private_provider`
  - Uses a fake rollout whose `session_meta` omits `model_provider`.
  - Asserts resume succeeds through configured `mock_provider`.
  - Asserts the source rollout does not gain `model_provider` during resume.
- `c8_m3_thread_resume_unknown_non_empty_model_provider_remains_clear_error`
  - Uses a fake rollout with persisted `model_provider: "missing_provider"`.
  - Expects a clear `Model provider \`missing_provider\` not found` error.

Existing rollout recorder coverage `vsp_provider_is_omitted_from_persisted_session_meta` was left unchanged.

## Commands Run

- `cargo fmt --package codex-app-server --package codex-rollout`
  - Completed; emitted existing nightly-only `imports_granularity` warnings.
- `cargo test -p codex-app-server c8_m3_thread_resume -- --nocapture`
  - Final focused run: 2 passed, 1 failed.
- `cargo test -p codex-app-server c8_m3_thread_resume_empty_model_provider_falls_back_to_configured_provider -- --nocapture`
  - Passed.
- `cargo test -p codex-app-server c8_m3_thread_resume_unknown_non_empty_model_provider_remains_clear_error -- --nocapture`
  - Failed as intended red coverage.
- `cargo fmt --check --package codex-app-server`
  - Passed; emitted existing nightly-only `imports_granularity` warnings.
- `git diff --check -- codex-rs/app-server/tests/suite/v2/thread_resume.rs .pipeline/reviews/C8/M3/test.md`
  - Passed.

## Observed Status

Green on current tree:

- Explicit empty `modelProvider: ""` on `thread/resume` falls back to configured `mock_provider`.
- Omitted provider metadata resumes through configured `mock_provider`.
- Omitted-provider source rollout remains unchanged and does not persist private provider metadata.
- Unknown non-empty request override `modelProvider: "missing_provider"` already errors clearly.

Red on current tree:

- Persisted non-empty unknown provider does not error.
- Exact final focused failure:
  - Test: `suite::v2::thread_resume::c8_m3_thread_resume_unknown_non_empty_model_provider_remains_clear_error`
  - Command: `cargo test -p codex-app-server c8_m3_thread_resume -- --nocapture`
  - Expected: JSON-RPC error containing `failed to load configuration: Model provider \`missing_provider\` not found`
  - Actual: JSON-RPC response succeeds with top-level `modelProvider: "mock_provider"` while `thread.modelProvider` is `"missing_provider"`.
  - Panic: `expected JSONRPCMessage::Error, got Response(...)`

## Assumptions

- The relevant unknown-provider contract applies to non-empty persisted session metadata as well as explicit request overrides.
- Existing recorder tests are sufficient for VSP private provider omission; no rollout test edits were needed.
