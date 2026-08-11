# C6/M2 Implement Review - Structured Workflow File API

Timestamp: 2026-05-29T15:51:17+08:00
Role: implement worker
Scope: production/runtime code only

## Summary

Implemented `codex_core::workflow_files` as a small internal production API for structured Hypo-Workflow file operations.

Changed files:

- `codex-rs/core/src/workflow_files.rs`
- `codex-rs/core/src/lib.rs`
- `codex-rs/core/Cargo.toml`

No test files, fixtures, snapshots, or assertions were edited by this worker.

## API Implemented

Public module:

- `WorkflowFiles`
- `CurrentStepUpdate`
- `LogEvent`
- `ProgressEntry`
- `SubagentLifecycleEvent`
- `SubagentLifecycleStatus`
- `SubagentRole`

Implemented operations:

- `WorkflowFiles::open(project_root)`
  - Canonicalizes project root and `.pipeline`.
  - Requires `.pipeline/` to exist directly under the project root.

- `update_current_step(CurrentStepUpdate)`
  - Reads and parses `.pipeline/state.yaml` before mutation.
  - Rejects malformed YAML without writing.
  - Updates `current.phase` when provided, `current.step`, `current.heartbeat`, and `current.last_heartbeat`.
  - Preserves unrelated YAML fields through `serde_yaml::Value`.
  - Re-serializes and re-parses before replacing the file.

- `append_log_event(LogEvent)`
  - Appends structured events to `events` when present, otherwise `entries` when present, otherwise creates `events`.
  - Preserves existing event order.

- `append_progress_entry(ProgressEntry)`
  - Appends markdown timeline entries to `.pipeline/PROGRESS.md`.
  - Preserves existing content.

- `append_subagent_lifecycle_event(SubagentLifecycleEvent)`
  - Appends lifecycle records as structured log events with `type: subagent_lifecycle`.
  - Supports requested, started, completed, failed, blocked, closed, and close_failed states.

- `write_unprotected_yaml(path, contents)`
  - Validates YAML before writing.
  - Allows only YAML targets under project-local `.pipeline/`.
  - Rejects raw writes to `state.yaml`, `cycle.yaml`, and `rules.yaml`.
  - Rejects relative traversal and absolute paths outside `.pipeline/`.

## Safety Notes

- File replacement is staged through a same-directory temporary file followed by `rename`.
- Malformed input is parsed before any write attempt, so protected state files remain byte-for-byte unchanged on parse failure.
- No model-visible tool registration was added.
- Added `serde_yaml = { workspace = true }` to `codex-rs/core/Cargo.toml`; the workspace already declares the dependency.

## Verification

Commands run from `codex-rs/`:

```bash
cargo test -p codex-core workflow_files --no-run
```

Result:

- Passed.
- Completed in 7m 11s.
- Built `codex-core`, `codex-write-config-schema`, `tests/all.rs`, and `tests/responses_headers.rs`.

```bash
cargo test -p codex-core workflow_files
```

Result:

- Passed.
- `7 passed; 0 failed; 0 ignored`.

```bash
cargo fmt --check
```

Result:

- Passed with exit code 0.
- Emitted existing stable-toolchain warnings: `can't set imports_granularity = Item`, because that rustfmt option is nightly-only.

## Remaining Issues

None known within M2 implement scope.

---

# C6/M2 Implement Fix Round - Symlink Parent Escape

Timestamp: 2026-05-29T16:02:22+08:00
Role: implement worker
Scope: production/runtime code only, needs_changes fix

## Summary

Fixed the audit finding in `write_unprotected_yaml` where an existing symlink ancestor under `.pipeline/` could redirect a raw YAML write outside the project-local workflow directory.

Changed files:

- `codex-rs/core/src/workflow_files.rs`
- `.pipeline/reviews/C6/M2/implement.md`

No tests, Cargo files, or unrelated production files were edited in this fix round.

## Implementation

`write_unprotected_yaml` now prepares the target parent directory before the atomic write:

- Scans existing ancestors between canonical `.pipeline/` and the target parent with `symlink_metadata`.
- Rejects any existing symlink ancestor before creating missing directories.
- Creates missing directories normally for legitimate new nested paths.
- Rechecks ancestors after creation.
- Canonicalizes the final parent and requires it to remain under canonical `.pipeline/`.

This keeps the public API and existing write behavior unchanged for normal project-local YAML files while denying `.pipeline/link -> outside` parent escapes.

## Verification

Commands run from `codex-rs/`:

```bash
cargo test -p codex-core pipeline_child_symlink_parent_escape_is_denied
```

Result:

- Passed.
- `1 passed; 0 failed`.

```bash
cargo test -p codex-core workflow_files
```

Result:

- Passed.
- `8 passed; 0 failed`.

```bash
cargo fmt --check
```

Result:

- Passed with exit code 0.
- Emitted existing stable-toolchain warnings: `can't set imports_granularity = Item`, because that rustfmt option is nightly-only.

## Remaining Issues

None known within this fix scope.
