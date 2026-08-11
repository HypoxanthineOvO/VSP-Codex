# C9 M3 Test Worker Report

Generated: 2026-06-17T11:31:29+08:00

## Scope

Role: test worker for C9 M3 `Preserve VSP Runtime Surfaces`.

Constraints honored:

- No commit.
- No push, tag, or release.
- Did not delete `.pipeline/.lock`.
- Did not intentionally change production behavior.
- Wrote this report and made focused test-asset/build-fixture fixes only where test compilation blocked the requested validation.

Important decision applied:

- `voice` / `realtime` is not a VSP retained feature.
- `remote-control` is not a VSP retained feature.
- Their unavailability must not fail M3, but they remain recorded as non-retained upstream surfaces.

## Commands And Results

### 1. TUI Workflow Slash Routing

Command:

```bash
cargo test --manifest-path codex-rs/Cargo.toml -p codex-tui workflow_slash_routing -- --nocapture
```

Initial result:

- Failed during `codex-tui` lib-test compilation before the filtered routing tests could execute.
- Root cause was stale test-only code and test constructors after upstream API/module changes.

Focused test-asset fixes applied:

- Removed stale `crate::chatwidget::realtime::RealtimeConversationPhase` test import because realtime/voice is non-retained.
- Added missing `ToolRequestUserInputParams.auto_resolution_ms` in request-user-input overlay tests.
- Added missing `FooterProps.queue_submissions` in footer tests.
- Updated command-palette test flags to the current `BuiltinCommandFlags` fields.
- Updated resume-picker tests for `crate::thread_transcript` and `Thread.parent_thread_id`.
- Removed invalid helper methods that had been accidentally placed as free functions with `&self` in `history_cell/notices.rs`.

Final result:

- Pass.
- 9 tests passed, 0 failed.
- Warnings only:
  - `codex-core` dead-code warnings in `core/src/agent/control.rs`.
  - `codex-tui` warnings for realtime audio event field and `TableCell::patch_style`.

Evidence:

- `suite::workflow_slash_routing::*`: 9 passed.

### 2. Core Workflow Routing

Command:

```bash
cargo test --manifest-path codex-rs/Cargo.toml -p codex-core workflow_routing -- --nocapture
```

Result:

- Failed before executing `workflow_routing` tests.
- The `codex-core` `test "all"` target does not compile.

Representative failures:

- `core/tests/suite/workflow_routing.rs` still uses old `Op::UserInput.environments` and `ThreadSettingsOverrides.cwd`.
- `core/tests/suite/workflow_routing.rs` still passes `AbsolutePathBuf` where file-system helpers now expect `PathUri`.
- Broader target compile blockers also exist in unrelated core test modules:
  - duplicate imports and `PathUri` drift in `compact.rs` / `compact_remote.rs`;
  - stale `SetGoalRequest`, `RequestUserInputAnswer`, `mount_sse_sequence`, and `wait_for_event_match` references in session/request-user-input tests;
  - stale `conversation_id` direct field writes in multi-agent tests;
  - stale `assert_cmd::Command` vs `std::process::Command` usage in `cli_stream.rs`.

Assessment:

- No pass evidence for core Workflow routing yet.
- This is a test-target compile blocker, not an observed Workflow routing assertion failure.

### 3. App Server request_user_input

Command:

```bash
cargo test --manifest-path codex-rs/Cargo.toml -p codex-app-server --test all request_user_input -- --nocapture
```

Result:

- Failed before executing filtered `request_user_input` tests.
- The `codex-app-server` `test "all"` target does not compile.

Failures:

- `STARTUP_REMOTE_PLUGIN_SYNC_MARKER_FILE` not in scope in `app-server/tests/suite/v2/plugin_list.rs`.
- `McpProcess` not in scope in:
  - `app-server/tests/suite/v2/initialize.rs`
  - `app-server/tests/suite/v2/thread_read.rs`
  - `app-server/tests/suite/v2/thread_resume.rs`
  - `app-server/tests/suite/v2/thread_settings_update.rs`

Assessment:

- No pass evidence for app-server `request_user_input` integration tests yet.
- Source scan shows focused tests exist for workflow decision metadata, native Ask UI fallback, pending Ask replay, and `autoResolutionMs`, but they are currently unreachable until the test target compiles.

### 4. App Server Workflow Markdown Surface

Command:

```bash
cargo test --manifest-path codex-rs/Cargo.toml -p codex-app-server --test all workflow_markdown_surface -- --nocapture
```

Result:

- Failed before executing filtered `workflow_markdown_surface` tests.
- Same `codex-app-server` `test "all"` compilation blockers as command 3.

Assessment:

- No pass evidence for app-server Workflow Markdown surface yet.
- Source scan confirms `app-server/tests/suite/v2/workflow_markdown_surface.rs` covers plan progress, subagent markdown, tool-output summaries, and request-input fallback markdown, but it is currently unreachable.

### 5. CLI VSCode Remote SSH

Command:

```bash
cargo test --manifest-path codex-rs/Cargo.toml -p codex-cli --test vscode -- --nocapture
```

Result:

- Pass.
- 14 tests passed, 0 failed.
- Warnings only:
  - `codex-core` dead-code warnings.
  - `codex-tui` realtime audio / table-cell warnings.
  - `codex-cli` unused `resume_hint` import and `remote_control_disabled` variable.

Evidence:

- Remote-machine settings warning.
- Remote `.vscode-server` settings preference.
- Status JSON remote scope reporting.
- Wrapper pins default VSP Codex home.
- Install/rollback/dry-run behavior.
- App-server probe initialize contract.

## Surface Coverage Assessment

### Provider / Config

Observed coverage:

- Existing tests touch model provider configuration in config/core/app-server/exec.
- VSP provider helpers exist in `tui/src/vsp_provider.rs`, including tests around provider allow-list behavior.
- TUI provider selection paths call `vsp_provider::is_provider_allowed` and show VSP official-service disabled errors.

Gap:

- No successful M3 command directly validated VSP provider onboarding/config after this merge.
- Recommended targeted validation: run or add focused tests for VSP provider setup edits, provider allow-list rejection of official OpenAI when disabled, and config persistence of `model_provider = "vsp"`.

### VSCode Remote SSH

Observed coverage:

- Strong coverage from passing `codex-cli --test vscode`.

Assessment:

- Pass evidence is sufficient for M3 test-worker scope.

### Workflow Routing / Question Tool Metadata

Observed coverage:

- TUI `/hw:*` and `$hypo-workflow:*` routing passed.
- Source tests exist for core Workflow runtime guidance, request_user_input mandatory wording, card categories, and non-Workflow chat behavior.
- Source tests exist for app-server request_user_input metadata, workflow decision fields, fallback/replay, and `autoResolutionMs`.

Gap:

- Core and app-server integration test targets do not compile, so there is no executable pass evidence for the core/app-server halves yet.

### Workflow Markdown Surface

Observed coverage:

- Source tests exist for markdown rendering, subagent sections, tool summary, and request input fallback markdown.

Gap:

- App-server `test "all"` target compile blockers prevent execution.

### Subagent UX

Observed coverage:

- Source tests cover core agent nickname pools, Chinese/internal nickname candidates, subagent notifications, model-visible `<subagents>` environment context, subagent lifecycle hooks, and app-server subagent lineage headers.

Gap:

- No requested command directly executed subagent UX tests.
- Broader core test target compile blockers currently reduce confidence in executable subagent regression coverage.

### VSP Updater Surfaces

Observed coverage:

- TUI source has VSP release-update card handling and tests around local update decision behavior.
- `history_cell/notices.rs` has VSP update notice tests ensuring official OpenAI links are not shown when no VSP manifest action exists.
- CLI source preserves `VSP_CODEX_RELEASE_MANIFEST_URL` messaging.

Gap:

- No requested command directly executed updater tests.
- Recommended targeted validation: run focused TUI update/history-cell tests once the relevant test filters are identified, and add CLI tests for manifest URL fallback if missing.

## Non-Retained Surfaces

- `voice` / `realtime`: non-retained. Current warnings and unavailable/degraded runtime should not fail M3. Tests should not require realtime phase types or working realtime voice.
- `remote-control`: non-retained. CLI/app-server disabled behavior should be preserved where VSP intentionally disables it, but lack of usable remote-control is not an M3 failure.

## Recommended Fixes

Priority 1:

- Repair `codex-core` `test "all"` compilation enough to run `workflow_routing`.
  - Update `Op::UserInput` initializers from `environments` to `additional_context`.
  - Update `ThreadSettingsOverrides` usage to current fields.
  - Convert test file-system paths to `PathUri`.
  - Refresh stale request-user-input/session helper names and imports.
  - Update multi-agent tests for the current session identity storage.

Priority 2:

- Repair `codex-app-server` `test "all"` compilation.
  - Restore/import `McpProcess` test helper from the renamed `test_app_server` support module.
  - Import or replace `STARTUP_REMOTE_PLUGIN_SYNC_MARKER_FILE`.

Priority 3:

- Add or run focused tests for provider/config and VSP updater surfaces after compile blockers are removed.

## Verdict

`blocked_partial_pass`

Passing evidence:

- TUI Workflow slash routing passed.
- CLI VSCode Remote SSH passed.

Blocking risks:

- Core Workflow routing tests did not execute because the core integration test target does not compile.
- App-server request_user_input and Workflow Markdown surface tests did not execute because the app-server integration test target does not compile.

M3 should not be accepted as fully validated until the core and app-server test-target compile blockers are fixed and the requested filtered tests are rerun.
