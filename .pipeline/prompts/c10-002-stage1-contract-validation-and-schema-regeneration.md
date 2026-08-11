# C10 M2 - Stage-1 Contract Validation And Schema Regeneration

## Objective

After the Stage-1 merge, re-establish protocol contract integrity: regenerate schemas from the Rust protocol source of truth, verify VSP protocol extensions survived, and prove SQLite logging remains converged via runtime IO smoke.

## Requirements

- Regenerate JSON/TypeScript schemas from the Rust protocol crates; review the diff.
- Verify VSP extension semantics: `request_user_input` metadata, Question Tool, workflow markdown surface, TurnPlanUpdated.
- Run SQLite runtime IO smoke with isolated `CODEX_HOME`/`CODEX_SQLITE_HOME` expecting zero noisy-target rows.

## Boundaries

- In scope: schema regeneration, protocol/app-server focused fixes strictly needed for contract lockstep, smoke evidence files.
- Out of scope: Stage-2 merge content, extension capability gap work (M4), version/release edits.

## Technical Solution

Rust protocol is the source of truth (C9 M9 guardrail). Schema regeneration after merge is mandatory, not optional. VSP protocol extensions are contract surfaces; any upstream restructuring must keep their wire semantics or be escalated.

## Technical Route

1. Run the schema generator; commit regenerated outputs if diff exists; review diff for VSP field survival.
2. Run protocol and app-server focused suites.
3. Run SQLite IO smoke against the debug binary; record byte deltas and per-target row counts to `.pipeline/evidence/C10/M2-sqlite-io-smoke.json`.
4. Write `.pipeline/reports/C10/M2-contract-validation.md`.

## Research Required

None; C9 M8/M9 established the procedures and baselines.

## Risks And Alternatives

- Risk: upstream schema restructuring silently drops VSP extension fields; mitigation is field-level diff review plus focused tests, and TUI snapshots must not substitute for protocol tests.
- Rejected shortcut: skipping schema regeneration when "no protocol files conflicted" — regeneration runs unconditionally.

## Validation Commands

```bash
cargo test --manifest-path codex-rs/Cargo.toml -p codex-app-server-protocol
cargo test --manifest-path codex-rs/Cargo.toml -p codex-app-server --test all request_user_input -- --nocapture
cargo test --manifest-path codex-rs/Cargo.toml -p codex-app-server --test all workflow_markdown_surface -- --nocapture
cargo test --manifest-path codex-rs/Cargo.toml -p codex-state filter -- --nocapture
git diff --check
```

SQLite smoke: isolated homes, `RUST_LOG=trace timeout -s INT 5s <debug vsp-codex> app-server --listen off`, then assert noisy-target row total is 0.

## Evidence

Pass signals: protocol suites green, schema outputs in lockstep (regenerated with no unexplained diff), smoke shows 0 noisy rows, VSP extension fields present in generated schemas.

Fail signals: schema diff drops VSP fields; smoke writes noisy targets; any suite red without a recorded, user-visible blocker.

## Subworker Assignment Plan

- `test`: owns validation runs, smoke execution, and evidence JSON; writes `.pipeline/reviews/C10/M2/test.md`. Rejects pseudo-tests (e.g., asserting on stale pre-merge schema files).
- `implement`: owns only contract-lockstep fixes and regenerated schema commits. No test asset edits.
- `audit`: read-only; verifies schema/source lockstep, VSP field survival, evidence quality; writes `.pipeline/reviews/C10/M2/audit.md`.
- Main agent: orchestration, lifecycle records, integration only.
- Non-overlap: distinct identities for test/implement/audit.

## Expected Artifacts

- Regenerated schema files (if diff)
- `.pipeline/reports/C10/M2-contract-validation.md`
- `.pipeline/evidence/C10/M2-sqlite-io-smoke.json`
- `.pipeline/reviews/C10/M2/test.md`
- `.pipeline/reviews/C10/M2/audit.md`
