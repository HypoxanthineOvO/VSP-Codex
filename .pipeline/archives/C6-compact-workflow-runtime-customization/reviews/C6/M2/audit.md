# C6/M2 Audit Recheck - Structured Workflow File API

verdict: pass

timestamp: 2026-05-29T16:06:43+08:00
role: read-only audit recheck worker
scope: M2 final state after needs_changes repair

## Reviewed Refs

- `.pipeline/prompts/002-structured-workflow-file-api.md`
- `.pipeline/reviews/C6/M2/test.md`
- `.pipeline/reviews/C6/M2/implement.md`
- `codex-rs/core/src/workflow_files.rs`
- `codex-rs/core/src/lib.rs`
- `codex-rs/core/Cargo.toml`
- `codex-rs/Cargo.lock`
- `codex-rs/core/tests/suite/workflow_files.rs`
- `codex-rs/core/tests/suite/mod.rs`

## Commands Run And Results

- `git diff -- codex-rs/core/src/workflow_files.rs codex-rs/core/tests/suite/workflow_files.rs`
  - No output because both files are currently untracked in the working tree; inspected file contents directly.
- `git diff --cached -- codex-rs/core/src/workflow_files.rs codex-rs/core/tests/suite/workflow_files.rs`
  - No cached diff.
- `sed -n '1,260p' codex-rs/core/src/workflow_files.rs`
  - Reviewed API surface, typed state/log/progress helpers, and raw YAML write path.
- `sed -n '260,620p' codex-rs/core/src/workflow_files.rs`
  - Reviewed path normalization, protected file guard, symlink ancestor guard, parent creation, canonical parent recheck, and atomic write helper.
- `sed -n '1,260p' codex-rs/core/tests/suite/workflow_files.rs`
  - Reviewed real filesystem tests for malformed YAML, state update, log/progress append, lifecycle events, protected file guard, and path guard.
- `sed -n '260,620p' codex-rs/core/tests/suite/workflow_files.rs`
  - Reviewed Unix-only symlink parent escape regression test.
- `rg -n "workflow_files|serde_yaml|pub mod workflow_files" codex-rs/core/src/lib.rs codex-rs/core/Cargo.toml codex-rs/Cargo.lock codex-rs/core/tests/suite/mod.rs`
  - Confirmed module export, test registration, and explicit `serde_yaml` dependency.
- `cargo test -p codex-core workflow_files`
  - Passed: 8 tests passed, 0 failed.
- `git diff --check`
  - Passed.
- `cargo fmt --check`
  - Passed with existing stable-toolchain warnings that `imports_granularity = Item` is nightly-only.

## Findings

### Critical

- None.

### Warning

- None blocking.

### Info

- The original needs_changes finding is fixed. `write_unprotected_yaml` now calls `prepare_unprotected_yaml_parent` before `atomic_write` (`codex-rs/core/src/workflow_files.rs:197`, `codex-rs/core/src/workflow_files.rs:206`).
- The repair rejects symlink ancestors below canonical `.pipeline/` both before and after missing directory creation (`codex-rs/core/src/workflow_files.rs:382`, `codex-rs/core/src/workflow_files.rs:387`, `codex-rs/core/src/workflow_files.rs:389`, `codex-rs/core/src/workflow_files.rs:400`, `codex-rs/core/src/workflow_files.rs:418`).
- The final parent is canonicalized and required to remain under canonical `.pipeline/`, which covers the intended project-local write boundary after directory creation (`codex-rs/core/src/workflow_files.rs:391`, `codex-rs/core/src/workflow_files.rs:393`).
- The helper is only wired into `write_unprotected_yaml`; typed state/log/progress update behavior is otherwise unchanged (`codex-rs/core/src/workflow_files.rs:120`, `codex-rs/core/src/workflow_files.rs:147`, `codex-rs/core/src/workflow_files.rs:160`, `codex-rs/core/src/workflow_files.rs:181`, `codex-rs/core/src/workflow_files.rs:197`).
- Normal missing directory creation remains supported because `prepare_unprotected_yaml_parent` still calls `fs::create_dir_all(parent)` after rejecting existing symlink ancestors (`codex-rs/core/src/workflow_files.rs:388`).
- The symlink regression test is real and targeted: it creates `.pipeline/link` as a directory symlink to an outside temp directory, writes `link/out.yaml`, expects an error, and asserts the outside file is not created (`codex-rs/core/tests/suite/workflow_files.rs:303`, `codex-rs/core/tests/suite/workflow_files.rs:312`, `codex-rs/core/tests/suite/workflow_files.rs:315`, `codex-rs/core/tests/suite/workflow_files.rs:317`, `codex-rs/core/tests/suite/workflow_files.rs:321`).
- M2 requested coverage remains present: malformed YAML no partial state write (`codex-rs/core/tests/suite/workflow_files.rs:104`), current step/heartbeat update (`codex-rs/core/tests/suite/workflow_files.rs:122`), log append (`codex-rs/core/tests/suite/workflow_files.rs:151`), PROGRESS append (`codex-rs/core/tests/suite/workflow_files.rs:192`), subagent lifecycle sequence (`codex-rs/core/tests/suite/workflow_files.rs:220`), protected file guard (`codex-rs/core/tests/suite/workflow_files.rs:253`), and path guard (`codex-rs/core/tests/suite/workflow_files.rs:273`).
- No model-visible workflow file tool was added; the change exposes an internal Rust API via `pub mod workflow_files` (`codex-rs/core/src/lib.rs:103`).
- The explicit `serde_yaml` dependency is reasonable and localized to `codex-core` (`codex-rs/core/Cargo.toml:99`).

## Evidence Quality Assessment

- Test evidence is strong. The tests use `tempfile::TempDir` and real filesystem reads/writes, including the symlink regression; they are not only in-memory serialization checks.
- The needs_changes repair has an actual red-to-green trail: the test worker reported the symlink regression failing before the production fix, and the implement worker reported it passing after the fix.
- Worker separation evidence remains acceptable: the test worker reports test-only edits, the implement worker reports production/runtime edits for the fix, and this recheck was read-only except for this audit report.
- Validation evidence is current and independently re-run in this recheck: `cargo test -p codex-core workflow_files`, `git diff --check`, and `cargo fmt --check` all passed.
- Residual risk is limited to general same-process temp filename collision behavior in `atomic_write`; that is not part of the repaired escape path and is not blocking for M2's internal API milestone.

## Acceptance

M2 completion is accepted.

The original symlink parent escape finding has been fixed, the regression test covers `.pipeline/link -> outside`, normal nested YAML parent creation remains supported, and the rest of the M2 completion conditions are acceptable: malformed YAML has no partial state write, typed state/log/progress/subagent lifecycle operations are covered, protected file and path guards are covered, no model-visible tool surface was introduced, the dependency change is reasonable, and worker evidence is sufficient.
