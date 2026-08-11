# C10 M2 Contract Validation Report

Timestamp: `2026-07-02T19:10:34+08:00`

Verdict: `pass_pending_audit`

## Summary

M2 restored app-server protocol/schema lockstep after the Stage-1 merge. The schema generator was run with the stable default recipe, generated app-server protocol schema files were updated, VSP native Ask / Question Tool / workflow plan surfaces were confirmed present, and the SQLite runtime IO smoke passed with zero noisy-target rows.

## Technical Approach

- Used Rust protocol sources as the source of truth.
- Fixed protocol/export drift before regenerating schema fixtures.
- Ran `just write-app-server-schema`, which invokes `cargo run -p codex-app-server-protocol --bin write_schema_fixtures --`.
- Did not run `--experimental`; `item/tool/requestUserInput` is now native Ask capability gated and present in stable schema output.
- Fixed focused test compile drift in app-server thread list tests and state runtime thread metadata tests.

## Modified Areas

- `codex-rs/app-server-protocol/src/protocol/common.rs`
- `codex-rs/app-server-protocol/src/protocol/v2/item.rs`
- `codex-rs/app-server-protocol/src/export.rs`
- `codex-rs/app-server/tests/suite/v2/thread_read.rs`
- `codex-rs/state/src/runtime/threads.rs`
- `codex-rs/app-server-protocol/schema/**`

## Schema / VSP Surface Checks

Confirmed present in regenerated schema/source:

- `nativeAskUi`
- `item/tool/requestUserInput`
- `ToolRequestUserInputParams`
- `autoResolutionMs`
- `singleChoice`, `multiChoice`, `freeText`, `ranking`
- `turn/plan/updated`
- `TurnPlanUpdatedNotification`

## Validation Results

| Command | Result |
| --- | --- |
| `CODEX_SKIP_BWRAP_BUILD=1 cargo test --manifest-path codex-rs/Cargo.toml -p codex-app-server-protocol` | pass: 256 unit tests plus 2 schema fixture tests |
| `CODEX_SKIP_BWRAP_BUILD=1 cargo test --manifest-path codex-rs/Cargo.toml -p codex-app-server --test all request_user_input -- --nocapture` | pass: 18 tests |
| `CODEX_SKIP_BWRAP_BUILD=1 cargo test --manifest-path codex-rs/Cargo.toml -p codex-app-server --test all workflow_markdown_surface -- --nocapture` | pass: 2 tests |
| `CODEX_SKIP_BWRAP_BUILD=1 cargo test --manifest-path codex-rs/Cargo.toml -p codex-state filter -- --nocapture` | pass: 9 tests |
| `git diff --check` | pass |
| conflict marker scan over M2 protocol/state/report files | pass |

## SQLite Smoke

Evidence: `.pipeline/evidence/C10/M2-sqlite-io-smoke.json`

- Isolated `HOME`, `CODEX_HOME`, and `CODEX_SQLITE_HOME`.
- Existing debug binary: `codex-rs/target/debug/vsp-codex`.
- `logs_2.sqlite` created.
- `total_rows`: 0.
- `noisy_target_total`: 0.
- Exit code 1 was expected for `app-server --listen off` after initialization because no transport was configured.

## Residual Risks

- Default full workspace cargo commands without `CODEX_SKIP_BWRAP_BUILD=1` remain blocked on this host by the known missing `libcap.pc` / `codex-bwrap` environment issue.
- M2 did not run Stage-2 merge work or release/version changes; those remain out of scope for later milestones.
