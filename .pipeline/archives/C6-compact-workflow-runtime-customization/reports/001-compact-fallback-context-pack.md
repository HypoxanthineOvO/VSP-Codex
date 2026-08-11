# M1 Compact Fallback Context Pack Report

## Result

Status: completed

M1 adds a bounded Hypo-Workflow fallback context pack for compact failure modes where the installed compact history would otherwise be empty or unusable.

## Files Changed

- `codex-rs/core/src/workflow_context.rs`
- `codex-rs/core/src/compact.rs`
- `codex-rs/core/src/compact_remote.rs`
- `codex-rs/core/src/compact_remote_v2.rs`
- `codex-rs/core/src/lib.rs`
- `codex-rs/core/tests/suite/compact.rs`
- `codex-rs/core/tests/suite/compact_remote.rs`
- `.pipeline/reviews/C6/M1/test.md`
- `.pipeline/reviews/C6/M1/implement.md`
- `.pipeline/reviews/C6/M1/audit.md`

## Behavior

- Local compact installs fallback context when the compact summary is empty or `(no summary available)`.
- Remote compact v1 installs fallback context after filtering when compacted history has no usable compaction context.
- Remote compact v2 installs fallback context for unusable retained/output history and recovers zero compaction output after `response.completed`.
- Fallback activates only for roots containing `.pipeline/`.
- Fallback sources are project-local, canonicalized, bounded, and rendered in priority order: project/architecture summary, current goal, recent dialogue, state, cycle, active prompt, compact files.

## Validation

- `cargo fmt --check` passed with existing stable-rust warnings for nightly-only `imports_granularity = Item`.
- `cargo test -p codex-core --test all suite::compact::local_empty_compact_summary_installs_workflow_fallback_context -- --nocapture` passed.
- `cargo test -p codex-core --test all suite::compact_remote::remote_v1_empty_filtered_compact_output_installs_workflow_fallback_context -- --nocapture` passed.
- `cargo test -p codex-core --test all suite::compact_remote::remote_v2_marker_only_compact_output_installs_workflow_fallback_context -- --nocapture` passed.
- `cargo test -p codex-core workflow_context::tests::builds_fallback_from_project_local_pipeline_sources -- --nocapture` passed.
- `git diff --check` passed.

## Review

Audit worker `019e727f-cb82-7e81-9d6f-f41dddfb4a4e` returned `pass` with no blocking findings and accepted M1 completion under the recorded user-confirmed implement downgrade.

## Residual Risks

- Full compact/remote compact suites were not run.
- Fallback source list and goal extraction remain static/heuristic until later structured Workflow API milestones.
- Remote v2 zero-output recovery is an intentional behavior change for Workflow continuity.
