# C8 M3 Audit Worker Report

## Verdict

pass

## Blocking Findings

None.

## Non-Blocking Observations

- The focused unknown-provider test covers non-empty provider metadata from rollout `session_meta`. State DB persisted `ThreadMetadata.model_provider` follows the same `merge_persisted_resume_metadata` path and is passed to config loading, but there is not a dedicated M3 test fixture that seeds state DB with an unknown provider. Code evidence is strong enough for this milestone because `merge_persisted_resume_metadata` calls the shared merge helper with `Some(persisted_metadata.model_provider.as_str())`.
- `thread_fork_inner` still builds config directly from request overrides and stored thread cwd; M3 tests and implementation focus on `thread/resume`. The M3 prompt mentions resume/list/read/fork inspection, but the acceptance criteria supplied for this audit are specifically provider resume compatibility and VSP omission preservation.

## Evidence Checked

- Prompt and worker separation:
  - Read `.pipeline/prompts/c8-003-provider-resume-compatibility-contract.md`.
  - Read `.pipeline/reviews/C8/M3/test.md` and `.pipeline/reviews/C8/M3/implement.md`.
  - Worker reports show distinct ownership: test worker changed only `codex-rs/app-server/tests/suite/v2/thread_resume.rs` plus its report; implement worker changed only `codex-rs/app-server/src/request_processors/thread_processor.rs` plus its report. Audit worker performed read-only review and wrote only this file.

- Implementation boundary/helper:
  - `codex-rs/app-server/src/request_processors/thread_processor.rs` centralizes persisted resume provider handling in `merge_persisted_resume_metadata`, `merge_persisted_resume_values`, `normalize_persisted_resume_model_provider`, and `persisted_resume_model_provider_from_history`.
  - `thread_resume_inner` applies persisted metadata before `config_manager.load_for_cwd(...)`, so non-empty persisted provider IDs are validated by normal config loading.
  - Unknown provider diagnostics come from core config loading as `Model provider `<id>` not found`, wrapped by app-server as `failed to load configuration: ...`.

- Contract checks:
  - Non-empty persisted provider from rollout history: `persisted_resume_model_provider_from_history` extracts non-empty `session_meta.model_provider`; `load_and_apply_persisted_resume_metadata` merges it into `ConfigOverrides.model_provider`; unknown provider reaches config loading and errors clearly.
  - Non-empty persisted provider from state DB: `merge_persisted_resume_metadata` passes `persisted_metadata.model_provider` through the shared helper into `ConfigOverrides.model_provider`; empty/whitespace is normalized away.
  - Empty explicit `modelProvider: ""`: config loading filters empty override values and falls back to configured `mock_provider`; focused test asserts both response and thread provider are `mock_provider`.
  - Omitted provider: absent rollout provider remains unset and falls back to configured provider.
  - VSP provider omission: `codex-rs/rollout/src/recorder_tests.rs` keeps `vsp_provider_is_omitted_from_persisted_session_meta`; M3 resume test removes provider metadata from the source rollout, resumes, then asserts the source file is byte-for-byte unchanged and still lacks `"model_provider"`.
  - No provider metadata is written back into the source rollout by resume.

- Test quality:
  - `c8_m3_thread_resume_empty_model_provider_falls_back_to_configured_provider` exercises a real MCP `thread/resume` request with `model_provider: Some(String::new())` and asserts successful configured-provider fallback.
  - `c8_m3_thread_resume_omitted_vsp_provider_falls_back_without_persisting_private_provider` uses an external rollout fixture with provider metadata removed and asserts successful fallback plus no source rollout mutation.
  - `c8_m3_thread_resume_unknown_non_empty_model_provider_remains_clear_error` uses a rollout fixture with `model_provider: "missing_provider"` and asserts a JSON-RPC error containing the config-load diagnostic.
  - Assertions are behavioral, not fake: they inspect JSON-RPC response/error payloads and persisted rollout contents.

- Validation evidence reviewed from worker reports:
  - `cargo test -p codex-app-server c8_m3_thread_resume -- --nocapture` passed: 3 passed, 0 failed.
  - `cargo test -p codex-rollout vsp_provider_is_omitted_from_persisted_session_meta -- --nocapture` passed: 1 passed, 0 failed.
  - `cargo fmt --check --package codex-app-server --package codex-rollout` passed with existing nightly-only `imports_granularity` warnings.
  - `git diff --check -- codex-rs/app-server/src/request_processors/thread_processor.rs .pipeline/reviews/C8/M3/implement.md` was reported passed by implement worker; audit also ran repository-level `git diff --check` successfully.

## Audit Result

The M3 provider and resume compatibility contract is satisfied for the audited scope. Empty and omitted providers fall back to the configured provider, unknown non-empty persisted provider metadata is validated through config loading with a clear diagnostic, and VSP private provider omission remains preserved without writing provider metadata back into source rollouts.
