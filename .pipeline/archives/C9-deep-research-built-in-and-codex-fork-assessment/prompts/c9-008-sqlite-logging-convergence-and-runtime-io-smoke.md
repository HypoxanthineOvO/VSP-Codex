# C9 M8 - SQLite Logging Convergence And Runtime I/O Smoke

## Objective

Converge VSP SQLite persistent logging behavior with official alpha.25 fixes and add a real runtime I/O smoke so disk-write amplification cannot regress silently.

## Requirements

- Compare the current VSP hotfix with official alpha.25 logging behavior.
- Retain the stronger filtering behavior for `log`, `codex_otel.log_only`, and `codex_otel.trace_safe`.
- Validate at both unit-test and runtime levels.
- Produce real SQLite/WAL growth evidence from an isolated home.

## Boundaries

- In scope: `codex-rs/state/src/log_db.rs`, `codex-rs/state/src/log_db_filter_tests.rs`, logging layer setup in app-server/TUI if needed, and a repeatable smoke command/script.
- Out of scope: deleting historical user SQLite databases, vacuum/rotation of existing `~/.vsp-codex`, broad protocol/TUI merge work, remote release actions.

## Technical Solution

Make persistent logging filtering explicit and testable at two layers: target filtering in unit tests and process-level SQLite database/WAL observation in a bounded runtime scenario.

## Technical Route

1. Compare current VSP `log_db` behavior with alpha.25 upstream fixes.
2. Keep upstream implementation when it is equivalent or stronger; keep VSP guard if upstream is missing a local protection.
3. Add or refine a smoke path that launches a real VSP Codex binary/app-server/TUI path with isolated `CODEX_HOME` and `CODEX_SQLITE_HOME`.
4. Measure `logs_2.sqlite` and `logs_2.sqlite-wal` before/after, and query persisted targets.
5. Record evidence in `.pipeline/evidence/C9/M8-sqlite-io-smoke.json` and conclusions in `.pipeline/reports/C9/M8-sqlite-logging.md`.

## Research Required

Resolved by user confirmation with a remaining implementation detail: Decompose accepted qualitative bounded-growth plus strict noisy-target rejection. Implementation may tighten this into a numeric byte/sample threshold once the smoke scenario is concrete.

## Risks And Alternatives

Rejected alternative: static/unit tests only. They do not prove that the actual runtime path stops sustained SQLite/WAL writes.

Risk: a network-dependent smoke can be flaky. Prefer a launch/minimal app-server scenario that avoids live provider traffic where possible.

## Validation Commands

```bash
cargo test --manifest-path codex-rs/Cargo.toml -p codex-state filter -- --nocapture
cargo check --manifest-path codex-rs/Cargo.toml -p codex-state -p codex-app-server -p codex-tui
CODEX_HOME="$(mktemp -d)" CODEX_SQLITE_HOME="$(mktemp -d)" <sqlite-io-smoke-command>
git diff --check
```

## Evidence

Pass signals: filter tests pass; smoke evidence shows no persisted noisy targets and no sustained WAL/database growth for the bounded scenario.

Fail signals: noisy targets persist, database/WAL grows continuously, or the smoke fabricates data instead of running the real binary path.

## Subworker Assignment Plan

- `test`: owns the runtime smoke design, final command run, evidence review, and pseudo-test rejection. Writes `.pipeline/reviews/C9/M8/test.md`.
- `implement`: owns scoped logging/smoke implementation after `test` defines the real scenario. It must summarize touched files and evidence path.
- `audit`: read-only review of final diff, smoke evidence, and claims about historical database cleanup. Writes `.pipeline/reviews/C9/M8/audit.md`.
- Main agent: orchestrates worker handoff and integrates source edits only after worker roles are assigned.
- Non-overlap: `test`, `implement`, and `audit` must be distinct identities under strict worker separation.
- Spawned worker write scope: spawned workers may edit `.pipeline/` artifacts only unless the orchestrator grants explicit local implementation scope for named source files. `audit` remains read-only.

## Expected Artifacts

- Updated logging code/tests if needed.
- Repeatable SQLite I/O smoke command or script.
- `.pipeline/evidence/C9/M8-sqlite-io-smoke.json`
- `.pipeline/reports/C9/M8-sqlite-logging.md`
- `.pipeline/reviews/C9/M8/test.md`
- `.pipeline/reviews/C9/M8/audit.md`
