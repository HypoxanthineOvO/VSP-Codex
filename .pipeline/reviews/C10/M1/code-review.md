# C10 M1 Code Review

Timestamp: `2026-07-02T18:30:36+08:00`

Verdict: `needs_changes`

The review found two issues that must be fixed before M1 can be accepted.

## Findings

1. P1: VSP curated marketplace branding was overwritten in production TUI code.

   `codex-rs/tui/src/chatwidget/plugin_catalog.rs` returns and displays `OpenAI Curated` for the curated marketplace, while C10/M1 preservation requirements and existing tests expect `VSP Curated`. This is a user-facing preserve regression and was not covered by the final focused GREEN suite.

2. P2: `codex-app-server-test-client` answers `multiChoice` and `ranking` questions as single-choice answers.

   `codex-rs/app-server-test-client/src/request_user_input.rs` fills only `selected_option_ids` with one option whenever options exist. The app-server protocol differentiates `singleChoice`, `multiChoice`, `freeText`, and `ranking`; the fallback answerer maps multi-choice questions to all selected IDs and ranking questions to ordered IDs.

## Reviewed Refs

- `.pipeline/reports/C10/M1-merge-conflicts.md`
- `.pipeline/reviews/C10/M1/test.md`
- `codex-rs/tui/src/chatwidget/plugin_catalog.rs`
- `codex-rs/tui/src/chatwidget/plugins.rs`
- `codex-rs/tui/src/bottom_pane/request_user_input/render.rs`
- `codex-rs/app-server-test-client/src/request_user_input.rs`
- `codex-rs/app-server-test-client/src/request_user_input_tests.rs`
- `codex-rs/app-server-protocol/src/protocol/v2/item.rs`
- `codex-rs/app-server/src/request_user_input_fallback.rs`

## Validation Context

- No unresolved merge conflict markers were found in reviewed target files.
- `git diff --check` on the reviewed target set passed.
- The audit worker did not run cargo tests to avoid mutating snapshots in read-only review.

## Required Follow-Up

- Add or repair test coverage for the VSP curated label and app-server-test-client multi-choice/ranking answer shapes.
- Repair production code.
- Rerun focused tests and M1 GREEN validation after fixes.

## Remediation

Timestamp: `2026-07-02T18:39:02+08:00`

- Test worker added `codex-app-server-test-client` coverage for multi-choice `selected_option_ids` and ranking `ordered_option_ids`, plus a TUI assertion that the VSP curated tab does not render `OpenAI Curated`.
- Implement worker restored `VSP Curated` production labels in `plugin_catalog.rs`.
- Implement worker updated `codex-app-server-test-client` to branch on `ToolRequestUserInputQuestionKind`: single choice fills `selected_option_ids`, multi-choice fills selected IDs from comma-separated selections, ranking fills ordered IDs from comma-separated order, and free text keeps `free_text`.
- Worker validation: `CODEX_SKIP_BWRAP_BUILD=1 cargo test --manifest-path codex-rs/Cargo.toml -p codex-app-server-test-client request_user_input` passed 4 tests after implementation.
- TUI targeted test still cannot be used as a focused gate in M1 because the wider `codex-tui` test build fails in unrelated test compile drift outside this review fix scope; production `codex-tui` check remains the available M1 gate.

## Remediation Validation

Timestamp: `2026-07-02T18:40:58+08:00`

Verdict: `pass`

Commands run:

| Command | Status | Evidence |
| --- | --- | --- |
| `CODEX_SKIP_BWRAP_BUILD=1 cargo test --manifest-path codex-rs/Cargo.toml -p codex-app-server-test-client` | pass | 7 tests passed, 0 failed. |
| `CODEX_SKIP_BWRAP_BUILD=1 cargo check --manifest-path codex-rs/Cargo.toml -p codex-app-server-test-client -p codex-tui` | pass | Finished with existing warnings only. |
| `CODEX_SKIP_BWRAP_BUILD=1 cargo check --manifest-path codex-rs/Cargo.toml --workspace` | pass | Finished with existing warnings only. |
| `CODEX_SKIP_BWRAP_BUILD=1 cargo test --manifest-path codex-rs/Cargo.toml -p codex-core workflow_routing -- --nocapture` | pass | 14 tests passed, 0 failed. |
| `CODEX_SKIP_BWRAP_BUILD=1 cargo test --manifest-path codex-rs/Cargo.toml -p codex-cli --test vscode -- --nocapture` | flaky | Initial parallel run failed one test with `Text file busy (os error 26)`. |
| `CODEX_SKIP_BWRAP_BUILD=1 cargo test --manifest-path codex-rs/Cargo.toml -p codex-cli --test vscode -- --nocapture --test-threads=1` | pass | 19 tests passed, 0 failed. |
| `git diff --check` | pass | No output. |
| `rg -n '^(<<<<<<<|>>>>>>>)' codex-rs AGENTS.md docs/install.md .pipeline/reports/C10/M1-merge-conflicts.md .pipeline/reviews/C10/M1/test.md .pipeline/reviews/C10/M1/code-review.md` | pass | No matches; `rg` exited 1 as expected. |

## Re-Audit

Timestamp: `2026-07-02T18:43:57+08:00`

Verdict: `pass`

No blocking findings remain.

- `VSP Curated` branding is restored in production plugin catalog labels, loading/empty/header text, and the test now asserts the curated tab does not render `OpenAI Curated`.
- `codex-app-server-test-client` now branches by `ToolRequestUserInputQuestionKind`: multi-choice writes `selected_option_ids`; ranking writes `ordered_option_ids`; single choice and free text retain their existing semantics.
- The new app-server-test-client tests cover multi-choice and ranking answer shapes.

Residual risks / test gaps:

- The targeted TUI test was not used as a final gate because unrelated `codex-tui` test compile drift still blocks the wider test binary; production `codex-tui` check passed.
- `multiChoice` / `ranking` with `is_other=true` is still not specially supported by the test client. The fallback implementation also does not cover that combination, so this is tracked as a follow-up gap rather than a blocking M1 finding.
