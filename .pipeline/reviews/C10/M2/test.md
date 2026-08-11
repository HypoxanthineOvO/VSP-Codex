# C10 M2 Test Worker Evidence

## Verdict

`baseline_red_contract_drift_with_sqlite_smoke_pass`: M2 is not green. The SQLite runtime IO smoke passed with isolated homes and zero noisy-target rows, but protocol/schema validation is red and app-server/state focused suites are blocked by compile-time contract drift. No production source, generated schema output, snapshots, Cargo manifests, or protected lifecycle files were edited by this test worker.

## Scope And Role

- Role: independent `test` worker for C10 M2.
- Allowed write targets used:
  - `.pipeline/reviews/C10/M2/test.md`
  - `.pipeline/evidence/C10/M2-sqlite-io-smoke.json`
- Milestone prompt: `.pipeline/prompts/c10-002-stage1-contract-validation-and-schema-regeneration.md`.
- Branch observed: `c10-upstream-0.143.0-alpha.33`.
- Existing untracked file observed at start: `.pipeline/.lock`; not edited.
- Protected files not edited: `.pipeline/state.yaml`, `.pipeline/log.yaml`, `.pipeline/PROGRESS.md`.

## Schema Generator Discovery

The schema regeneration command is mutating and belongs to the implement worker:

```bash
just write-app-server-schema
```

The root `justfile` sets `working-directory := "codex-rs"` and defines this recipe as:

```bash
cargo run -p codex-app-server-protocol --bin write_schema_fixtures -- {args}
```

The binary is `codex-rs/app-server-protocol/src/bin/write_schema_fixtures.rs`. By default it writes under `codex-rs/app-server-protocol/schema`; `--schema-root DIR` can redirect output, and `--experimental` includes experimental API methods/fields. The implementation calls `write_schema_fixtures_with_options`, which deletes and recreates `schema/typescript/` and `schema/json/`, so this worker did not run it against vendored schema outputs.

Pure validation exists in `codex-rs/app-server-protocol/tests/schema_fixtures.rs`: it generates fresh fixtures in memory/temp dirs and compares them to vendored schema fixtures. This is covered by:

```bash
CODEX_SKIP_BWRAP_BUILD=1 cargo test --manifest-path codex-rs/Cargo.toml -p codex-app-server-protocol
```

## Validation Plan

1. Confirm no current schema/protocol/app-server/state diff is already present before testing.
2. Run protocol/schema fixture validation to prove Rust protocol source and vendored JSON/TS fixtures are in lockstep.
3. Check VSP extension survival in generated/vendored schema surfaces:
   - `nativeAskUi`
   - `item/tool/requestUserInput`
   - `ToolRequestUserInputParams`
   - `autoResolutionMs`
   - question kinds: `singleChoice`, `multiChoice`, `freeText`, `ranking`
   - `turn/plan/updated`
   - `TurnPlanUpdatedNotification`
   - workflow markdown surface tests
4. Run M2 focused app-server suites:
   - `request_user_input`
   - `workflow_markdown_surface`
5. Run state filter unit coverage for SQLite/noisy-target behavior.
6. Run `git diff --check`.
7. Run isolated SQLite runtime IO smoke with an existing debug `vsp-codex` binary and record evidence JSON.

## Red / Baseline Expectations

- Expected before implement: schema fixture validation may be red because M1 explicitly deferred schema regeneration after Rust protocol reconciliation.
- Required M2 green: protocol tests pass, schema fixture comparison passes, app-server focused suites compile and pass, state filter tests compile and pass, `git diff --check` passes, and SQLite smoke shows `noisy_target_total = 0`.
- A pseudo-green based only on grepping stale schema files is rejected; field grep is only a survival clue. Lockstep must come from generator-backed schema fixture tests or implement worker regeneration plus reviewed diff.

## Baseline Commands And Results

All commands were run from `/home/heyx/Codex-VSP`.

| Command | Status | Evidence |
| --- | --- | --- |
| `git status --short --branch` | observation | Branch `c10-upstream-0.143.0-alpha.33`; untracked `.pipeline/.lock` existed before this worker wrote M2 artifacts. |
| `sed -n '1,260p' .pipeline/prompts/c10-002-stage1-contract-validation-and-schema-regeneration.md` | pass | M2 objective, validation commands, schema regeneration requirement, and SQLite smoke contract read. |
| `rg -n "write-app-server-schema|generate-ts|generate-json-schema|..." ...` | pass with one harmless missing path | Located `just write-app-server-schema`, app-server README generator docs, schema fixture tests, and `write_schema_fixtures` binary. The command also included nonexistent `codex-rs/justfile`, producing a read-only rg warning. |
| `git diff --name-status -- codex-rs/app-server-protocol/schema codex-rs/app-server-protocol/src codex-rs/app-server-protocol/tests codex-rs/app-server/src codex-rs/app-server/tests codex-rs/state` | pass | No output before this worker's allowed evidence/review writes; no current schema/protocol/app-server/state diff was present. |
| `rg -n 'nativeAskUi|item/tool/requestUserInput|ToolRequestUserInput|autoResolutionMs|ToolRequestUserInputQuestionKind|singleChoice|multiChoice|freeText|ranking|turn/plan/updated|TurnPlanUpdatedNotification|workflow_markdown|workflowMarkdown|workflow markdown' ...` | survival clue pass | Found VSP protocol extension surfaces in source, tests, README, and vendored JSON/TS schema. This does not prove lockstep by itself. |
| `ls -l codex-rs/target/debug/vsp-codex codex-rs/target/debug/codex 2>/dev/null || true` | pass | Existing debug binary found at `codex-rs/target/debug/vsp-codex`; no debug `codex` binary was needed. |
| `CODEX_SKIP_BWRAP_BUILD=1 cargo test --manifest-path codex-rs/Cargo.toml -p codex-app-server-protocol` | fail | 254 passed, 2 failed. Failures: `protocol::common::tests::serialize_initialize_with_native_ask_capability` and `export::tests::generated_ts_optional_nullable_fields_only_in_params`. |
| `CODEX_SKIP_BWRAP_BUILD=1 cargo test --manifest-path codex-rs/Cargo.toml -p codex-app-server --test all request_user_input -- --nocapture` | blocked red | Test binary did not compile: `app-server/tests/suite/v2/thread_read.rs:1104:35` missing `ancestor_thread_id` and `parent_thread_id` in `ThreadListParams` initializer. |
| `CODEX_SKIP_BWRAP_BUILD=1 cargo test --manifest-path codex-rs/Cargo.toml -p codex-app-server --test all workflow_markdown_surface -- --nocapture` | blocked red | Same compile blocker in `thread_read.rs:1104:35`, because `--test all` compiles the whole app-server integration test binary before applying the name filter. |
| `CODEX_SKIP_BWRAP_BUILD=1 cargo test --manifest-path codex-rs/Cargo.toml -p codex-state filter -- --nocapture` | blocked red | `codex-state` test build failed at `state/src/runtime/threads.rs:2264:19`: `SessionMeta` initializer missing `context_window`, `selected_capability_roots`, and `session_id`. |
| `git diff --check` | pass | No whitespace errors reported. |
| `HOME=/tmp/c10-m2-sqlite-smoke.E52PaM/home CODEX_HOME=/tmp/c10-m2-sqlite-smoke.E52PaM/codex-home CODEX_SQLITE_HOME=/tmp/c10-m2-sqlite-smoke.E52PaM/sqlite-home RUST_LOG=trace timeout -s INT 5s /home/heyx/Codex-VSP/codex-rs/target/debug/vsp-codex app-server --listen off` | pass smoke | Process exited `1` in 212 ms with expected `no transport configured` after initialization. SQLite DB created, 49152 bytes, WAL 0 bytes, total rows 0, noisy rows 0. Evidence written to `.pipeline/evidence/C10/M2-sqlite-io-smoke.json`. |

## Protocol Failure Details

### `serialize_initialize_with_native_ask_capability`

The expected serialized initialize payload no longer matched the actual payload. Actual output includes:

```json
"mcpServerOpenaiFormElicitation": false
```

The expected value in the test did not include that field. This is a protocol contract drift after upstream merge, not a schema-output-only issue.

### `generated_ts_optional_nullable_fields_only_in_params`

The test expected `ServerRequest.ts` not to contain `item/tool/requestUserInput`, but actual generated TypeScript does contain it. This is a direct VSP/native Ask survival and export policy conflict that needs implement-worker reconciliation rather than snapshot/schema hand editing.

## Schema Field Survival Checks

Field grep confirmed these surfaces still exist in current source or vendored schema:

- `nativeAskUi`: present in `InitializeCapabilities.ts`, JSON schema, and initialize tests.
- `item/tool/requestUserInput`: present in `ServerRequest.ts`, JSON schema, protocol common mapping, export tests, and app-server request tests.
- `ToolRequestUserInputParams`: present in TS schema import/export, JSON schema checks, protocol v2 structs/tests, and app-server tests.
- `autoResolutionMs`: present in request-user-input app-server tests.
- Question kinds `singleChoice`, `multiChoice`, `freeText`, `ranking`: present in protocol v2 tests and app-server tests.
- `turn/plan/updated` and `TurnPlanUpdatedNotification`: present in README, protocol common mapping, JSON schema, TS schema, and protocol v2 tests.
- `workflow_markdown_surface`: present in app-server test module, but the focused command is currently blocked by unrelated app-server integration test compile drift.

These checks only show survival in the current checked-in files. They do not prove regenerated lockstep until the protocol test suite passes or schema fixtures are regenerated and reviewed.

## SQLite Smoke Evidence

Result: `PASS`.

Evidence file: `.pipeline/evidence/C10/M2-sqlite-io-smoke.json`.

Key facts:

- Isolated homes were used under `/tmp/c10-m2-sqlite-smoke.E52PaM`.
- Existing debug binary used: `/home/heyx/Codex-VSP/codex-rs/target/debug/vsp-codex`.
- `logs_2.sqlite` was created under isolated `CODEX_SQLITE_HOME`.
- `logs_2.sqlite` size: 49152 bytes.
- `logs_2.sqlite-wal` size: 0 bytes.
- SQLite tables: `_sqlx_migrations`, `logs`, `sqlite_sequence`.
- `total_rows`: 0.
- `noisy_target_total`: 0 for `log`, `codex_otel.log_only`, and `codex_otel.trace_safe`.
- Exit code 1 is acceptable for this smoke because `--listen off` reaches the expected no-transport error after initialization.

## Required Implement Worker Next Actions

1. Fix protocol contract drift in Rust source/tests, especially:
   - initialize capability serialization expectation around `mcpServerOpenaiFormElicitation`;
   - generated TS/export policy around native `item/tool/requestUserInput` survival.
2. Fix compile drift in focused validation suites:
   - add the new `ThreadListParams` fields at `app-server/tests/suite/v2/thread_read.rs:1104`;
   - add the new `SessionMeta` fields at `state/src/runtime/threads.rs:2264`.
3. Regenerate schema fixtures with the mutating command after Rust protocol convergence:
   - `just write-app-server-schema`
   - evaluate whether `just write-app-server-schema --experimental` is also required for the affected experimental surfaces.
4. Review the regenerated JSON/TS diff for the VSP surfaces listed above.
5. Re-run the full M2 validation command set plus `git diff --check`.

## Current Green Gate State

- Protocol/schema lockstep: red.
- VSP extension survival by grep: present but not sufficient.
- App-server request-user-input focused suite: blocked red at compile.
- App-server workflow markdown focused suite: blocked red at compile.
- SQLite state filter unit suite: blocked red at compile.
- SQLite runtime IO smoke: green.
- Whitespace diff check: green.

## Final GREEN Follow-Up

Timestamp: `2026-07-02T19:15:01+08:00`

Verdict: `pass`

This test evidence began as the M2 RED baseline. After implement repair and schema regeneration, the main agent reran the full M2 validation set and the RED baseline was resolved.

Final pass evidence:

- `CODEX_SKIP_BWRAP_BUILD=1 cargo test --manifest-path codex-rs/Cargo.toml -p codex-app-server-protocol`: 256 unit tests passed plus 2 schema fixture tests passed.
- `CODEX_SKIP_BWRAP_BUILD=1 cargo test --manifest-path codex-rs/Cargo.toml -p codex-app-server --test all request_user_input -- --nocapture`: 18 tests passed.
- `CODEX_SKIP_BWRAP_BUILD=1 cargo test --manifest-path codex-rs/Cargo.toml -p codex-app-server --test all workflow_markdown_surface -- --nocapture`: 2 tests passed.
- `CODEX_SKIP_BWRAP_BUILD=1 cargo test --manifest-path codex-rs/Cargo.toml -p codex-state filter -- --nocapture`: 9 tests passed.
- `git diff --check`: passed.
- Conflict marker scan: passed.
- VSP field survival checks found the required native Ask, request-user-input, question-kind, auto-resolution, and turn-plan-update surfaces in regenerated schema/source.

Final acceptance evidence is summarized in `.pipeline/reports/C10/M2-contract-validation.md`; this file preserves both the original RED baseline and the later GREEN closure.
