# C6/M2 Test Worker Report

## Scope

- Role: test worker for `Structured Workflow File API`
- Write boundary honored: only test suite files and this report were edited.
- Production code touched by this worker: no.

## Files Changed

- `codex-rs/core/tests/suite/workflow_files.rs`
  - Added real filesystem red tests for the proposed structured Workflow file API.
- `codex-rs/core/tests/suite/mod.rs`
  - Registered the new `workflow_files` suite module.
- `.pipeline/reviews/C6/M2/test.md`
  - Recorded test evidence and expected red failure.

## Test Coverage Added

- `malformed_yaml_rejected_with_no_partial_state_write`
  - Writes malformed `.pipeline/state.yaml`, calls the typed state update API, and asserts the operation fails without changing file bytes.
- `heartbeat_and_current_step_update_rewrites_state_through_typed_operation`
  - Writes a real state file and expects `current.step` and heartbeat to be updated while preserving unrelated current prompt fields.
- `log_event_append_preserves_existing_events_and_adds_structured_event`
  - Writes a real `.pipeline/log.yaml`, appends a structured event, and checks existing event order/content is preserved.
- `progress_timeline_append_preserves_existing_markdown_and_adds_entry`
  - Writes a real `.pipeline/PROGRESS.md`, appends a timeline entry, and checks markdown content is preserved.
- `subagent_lifecycle_events_append_in_required_sequence`
  - Appends `requested`, `started`, `completed`, and `closed` Subagent lifecycle events to real log storage and asserts sequence/order evidence.
- `protected_file_guard_denies_unsupported_raw_state_write_shape`
  - Expects raw/unprotected writes to protected `.pipeline/state.yaml` to be denied without mutating the file.
- `project_local_pipeline_path_guard_denies_outside_root_writes`
  - Expects relative traversal and absolute outside-root writes to be denied, with no outside file created.

## Expected Red Failure

The tests intentionally call the expected production API:

- `codex_core::workflow_files::WorkflowFiles`
- `CurrentStepUpdate`
- `LogEvent`
- `ProgressEntry`
- `SubagentLifecycleEvent`
- `SubagentLifecycleStatus`
- `SubagentRole`

Current expected failure is compile-time red because `codex_core::workflow_files` is not implemented or exported yet.

## Commands Run

- `cargo test -p codex-core workflow_files --no-run`
  - Latest result: failed as expected with exit code 101.
  - Failure reason: `E0432` unresolved imports because `codex_core::workflow_files` is not implemented or exported yet.
  - Representative missing imports: `WorkflowFiles`, `CurrentStepUpdate`, `LogEvent`, `ProgressEntry`, `SubagentLifecycleEvent`, `SubagentLifecycleStatus`, and `SubagentRole`.
- `cargo fmt --check`
  - Result: passed. Output includes existing stable-toolchain warnings: `can't set imports_granularity = Item`.

## Closeout

Test worker work is complete. Red tests and evidence are in place, and no production code was edited by this worker.

## Notes For Implement Worker

- The tests require real file I/O under a temp project root with `.pipeline/`.
- The path guard should canonicalize/normalize paths and restrict writes to project-local `.pipeline/`.
- Protected files should only be mutated through explicit typed operations, not generic raw YAML writes.
- Append helpers should preserve existing content order and avoid arbitrary filesystem writes.

## Needs Changes Fix Round - Symlink Parent Escape Regression

### Scope

- Role: test worker for C6/M2 needs_changes repair.
- Write boundary honored: only `codex-rs/core/tests/suite/workflow_files.rs` and this report were edited.
- Production code touched by this worker: no.

### Test Added

- `pipeline_child_symlink_parent_escape_is_denied`
  - Unix-only regression test.
  - Creates `.pipeline/link` as a directory symlink to an outside temp directory.
  - Calls `WorkflowFiles::write_unprotected_yaml("link/out.yaml", "owned: true\n")`.
  - Expects an error and asserts the outside `out.yaml` file is not created.

### Command Run

- `cargo test -p codex-core pipeline_child_symlink_parent_escape_is_denied`
  - Result: failed as expected with exit code 101.
  - Failure point: `core/tests/suite/workflow_files.rs:317`, assertion that writes through a symlinked `.pipeline` child directory should be denied.
  - This confirms the current production implementation still permits the audit-reported symlink-parent escape.

### Closeout

Needs_changes repair test worker work is complete. Red regression coverage is in place; production code was not edited.
