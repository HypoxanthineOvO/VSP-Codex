# C6/M1 Implement Report

## Scope

Role: `implement` for M1 Compact Fallback Context Pack.

Worker Separation note: the original strict implement worker and replacement implement worker failed to produce lifecycle evidence. The user explicitly confirmed downgrading M1 implementation to single-agent/off. Test and audit separation remain required where possible.

## Production Files Changed

- `codex-rs/core/src/workflow_context.rs`
- `codex-rs/core/src/compact.rs`
- `codex-rs/core/src/compact_remote.rs`
- `codex-rs/core/src/compact_remote_v2.rs`
- `codex-rs/core/src/lib.rs`

## Implementation Summary

- Added a bounded Workflow fallback context builder activated only when a project root contains `.pipeline/`.
- The fallback renders a `<workflow_context_fallback>` user-context block from project-local files in this priority order:
  1. project summary / architecture
  2. current goal-like state context
  3. recent dialogue / inbox
  4. state
  5. cycle
  6. active prompt / plan-state
  7. compact files
- Reads are restricted to safe relative paths under the canonical project root and use `symlink_metadata`/canonical path checks to avoid out-of-root reads.
- Fallback source selection accepts candidate roots, so compact code can try the active turn cwd and the durable session/config cwd while still requiring a real `.pipeline/` project root.
- Source rendering uses prefix truncation for low-priority compact files so the recovery context preserves priority order instead of retaining tail content from oversized compact logs.
- Local compact now inserts fallback context when the summary suffix is empty or `(no summary available)`.
- Remote v1 compact processing now inserts fallback context when filtered compact output is unusable.
- Remote v2 compact processing now also checks for unusable retained/output history before handing off to the shared remote processor, and treats zero compaction output after `response.completed` as an unusable compact payload that can be recovered by fallback context.
- Existing post-compact SessionStart behavior is preserved because compaction still installs history through existing `replace_compacted_history` flow.

## Commands Run

- `cargo fmt --check` — passed; emitted existing stable-rust warnings that `imports_granularity = Item` is nightly-only.
- `cargo test -p codex-core --test all suite::compact::local_empty_compact_summary_installs_workflow_fallback_context -- --nocapture` — passed.
- `cargo test -p codex-core --test all suite::compact_remote::remote_v1_empty_filtered_compact_output_installs_workflow_fallback_context -- --nocapture` — passed.
- `cargo test -p codex-core --test all suite::compact_remote::remote_v2_marker_only_compact_output_installs_workflow_fallback_context -- --nocapture` — passed.
- `cargo test -p codex-core workflow_context::tests::builds_fallback_from_project_local_pipeline_sources -- --nocapture` — passed.
- `git diff --check` — passed.

## Risks

- The fallback source list is intentionally conservative and static for M1; later milestones can route this through structured Workflow APIs.
- Current goal extraction is heuristic and reads goal-like lines from state plus optional goal files. This is enough for recovery context but not a schema contract.
- M1 implementation was completed under user-confirmed single-agent downgrade, so audit should explicitly review worker-separation risk.
- The focused tests cover the new fallback paths but do not run the full compact suite.

## Green Validation Result

- The test-worker red tests now pass for local compact, remote v1 compact, and remote v2 marker-only/empty compact output.
- The fixture asserts priority ordering, bounded compact-source truncation, and absence of the out-of-root sentinel.
- A builder unit test confirms fallback generation from project-local `.pipeline` sources.
