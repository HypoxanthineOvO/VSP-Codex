# C6/M1 Test Worker Report

## Scope

Role: `test` worker for M1 Compact Fallback Context Pack.

I only edited test files and this report. I did not edit production/runtime code.

## Modified Test Files

- `codex-rs/core/tests/suite/compact.rs`
- `codex-rs/core/tests/suite/compact_remote.rs`
- `.pipeline/reviews/C6/M1/test.md`

## Coverage Added

- Local compact path:
  - Adds `local_empty_compact_summary_installs_workflow_fallback_context`.
  - Simulates local `/compact` returning `(no summary available)`.
  - Asserts the next model-visible request contains a labeled Workflow fallback context block.

- Remote compact v1:
  - Adds `remote_v1_empty_filtered_compact_output_installs_workflow_fallback_context`.
  - Simulates `/v1/responses/compact` returning only an assistant `(no summary available)` item, which should be unusable after filtering.
  - Asserts the follow-up request contains Workflow fallback context.

- Remote compact v2:
  - Adds `remote_v2_marker_only_compact_output_installs_workflow_fallback_context`.
  - Simulates a v2 compaction trigger request whose compact stream completes without a compaction payload.
  - Asserts the follow-up request contains Workflow fallback context.

- Shared fixture/assertions:
  - Creates temporary `.pipeline/` fixture content under the test workspace.
  - Covers source priority order:
    1. `PROJECT-SUMMARY.md`
    2. current goal from `.pipeline/state.yaml`
    3. recent dialogue from `.pipeline/chat/journal.compact.md`
    4. `.pipeline/state.yaml`
    5. `.pipeline/cycle.yaml`
    6. `.pipeline/prompts/001-compact-fallback-context-pack.md`
    7. compact files such as `.pipeline/PROGRESS.compact.md`
  - Adds a root-level `outside-secret.md` sentinel and asserts it does not appear in fallback context.
  - Adds a low-priority compact-file tail sentinel and asserts it is truncated away.

## Commands Run

```bash
cargo fmt --check
cargo test -p codex-core --test all suite::compact::local_empty_compact_summary_installs_workflow_fallback_context -- --nocapture
cargo test -p codex-core --test all suite::compact_remote::remote_v1_empty_filtered_compact_output_installs_workflow_fallback_context -- --nocapture
cargo test -p codex-core --test all suite::compact_remote::remote_v2_marker_only_compact_output_installs_workflow_fallback_context -- --nocapture
cargo fmt
git diff --check -- codex-rs/core/tests/suite/compact.rs codex-rs/core/tests/suite/compact_remote.rs
```

## Results

- `cargo fmt --check` failed only because the newly added tests needed formatting.
- `cargo fmt` completed successfully.
- `git diff --check -- codex-rs/core/tests/suite/compact.rs codex-rs/core/tests/suite/compact_remote.rs` passed.
- The three focused `cargo test` commands did not reach test assertion output within the time available. They spent several minutes waiting on Cargo package/artifact locks and compiling `codex-core`. I terminated the three focused test processes to comply with the request to stop expanding and converge.

## Expected Red Failure Signal

Once the focused tests complete compilation, the expected red failure is an assertion like:

```text
missing Workflow fallback marker: <workflow_context_fallback>
```

or one of the priority sentinel markers, because current production compact code does not yet install a Workflow fallback context pack after empty/unusable compact output.

## Implement Worker Contract

The implement worker must make the new tests pass by providing these behaviors:

- Detect unusable compact output for local compact when the summary is empty or exactly/semantically `(no summary available)`.
- Detect unusable compact output for remote v1 after filtering, including empty output or marker-only/assistant-only summary placeholders.
- Detect unusable compact output for remote v2 when the compaction trigger completes without a usable compaction payload.
- When `.pipeline/` exists in the project root, install a model-visible fallback context block containing `<workflow_context_fallback>`.
- Build fallback context only from project-root-scoped sources; do not follow or read paths outside the project root.
- Preserve deterministic priority order:
  `PROJECT-SUMMARY/architecture -> current goal -> recent dialogue -> state -> cycle -> prompt -> compact files`.
- Bound fallback size and truncate low-priority compact sources before they can dominate the next turn.
- Preserve normal non-Workflow compact behavior when no `.pipeline/` project is detected.
