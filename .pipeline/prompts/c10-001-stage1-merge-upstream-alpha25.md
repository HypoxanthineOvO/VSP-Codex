# C10 M1 - Stage-1 Merge To Upstream rust-v0.143.0-alpha.25

## Objective

Merge upstream `rust-v0.143.0-alpha.25` into the VSP fork on a dedicated C10 upgrade branch, resolving conflicts with the C9 equivalence matrix as the preserve-by-default boundary, so the fork's real merge base advances from `rust-v0.140.0-alpha.18`.

## Requirements

- Create a checkpoint commit and branch `c10-upstream-0.143.0-alpha.33` from current `main` (v0.140.0-vsp.8.2).
- Run `git merge rust-v0.143.0-alpha.25` and resolve all conflicts by domain batches: state -> protocol/app-server -> core -> tui/cli -> scripts/docs.
- Apply `.pipeline/archives/C9-deep-research-built-in-and-codex-fork-assessment/reports/C9/M7-vsp-equivalence-matrix.yaml` (replace 2 / preserve 8 / defer 7) and the conflict-domain direction table in `.pipeline/architecture.md`.
- SQLite logging domain: adopt upstream fixes and keep the VSP in-layer `target == "log"` guard (C9 M8 decision).
- Check `codex-rs/rust-toolchain.toml` changes from upstream and reconcile the local toolchain.

## Boundaries

- In scope: merge commit, conflict resolution edits, toolchain reconciliation, `.pipeline/` reports.
- Out of scope: version number edits, release artifacts, push/tag/release, Stage-2 (alpha.25..alpha.33) content, extension protocol capability work (M4).
- Fallback contract: on uncontrollable conflicts (e.g., upstream refactor breaking Hypo-Workflow core runtime), STOP and report options to the user via Question Tool. Do not silently downgrade to equivalence convergence.

## Technical Solution

Two-stage full merge selected by user over equivalence convergence. Stage 1 reuses C9 M7 conflict inventory so domain-level decisions are already classified. VSP surfaces (provider/official-service disablement, `vsp-codex` CLI naming, VSCode Remote settings behavior, Hypo-Workflow routing/state contract, Subagent UX, release tooling) are preserve-by-default; upstream is preferred only where the matrix marks replace or where equivalence is proven.

## Technical Route

1. Checkpoint commit on `main`; create branch `c10-upstream-0.143.0-alpha.33`.
2. `git merge rust-v0.143.0-alpha.25`; record conflict count and file list to `.pipeline/reports/C10/M1-merge-conflicts.md`.
3. Resolve conflicts in domain batches per the direction table; commit per batch where practical.
4. Reconcile `rust-toolchain.toml` and workspace manifests (keep version `0.140.0-vsp.8.2` unchanged in M1).
5. `cargo fmt --all --check`, full-workspace `cargo check`, `git diff --check`; fix fallout.

## Research Required

None new; C9 M7 inventory is the evidence base. If a conflict domain appears that the inventory does not cover, treat it as a blocking research item and report before resolving.

## Risks And Alternatives

- Rejected alternative: continued equivalence convergence (user explicitly chose full merge).
- Rejected alternative: single-shot merge to alpha.33 (user chose two-stage for rollback granularity).
- Primary risk: C9 aborted this merge for broad conflicts; mitigation is domain-batched resolution plus the stop-and-ask fallback.

## Validation Commands

```bash
git merge HEAD --is-ancestor rust-v0.143.0-alpha.25 && echo merged
cargo fmt --manifest-path codex-rs/Cargo.toml --all -- --check
cargo check --manifest-path codex-rs/Cargo.toml --workspace
cargo test --manifest-path codex-rs/Cargo.toml -p codex-core workflow_routing -- --nocapture
cargo test --manifest-path codex-rs/Cargo.toml -p codex-cli --test vscode -- --nocapture
git diff --check
grep -rn '<<<<<<<' codex-rs && exit 1 || echo no-markers
```

## Evidence

Pass signals: merge ancestor check passes, no conflict markers, workspace `cargo check` green, focused workflow_routing and vscode suites green, conflict report lists every resolved domain with direction rationale.

Fail signals: any preserve-boundary surface replaced without matrix/evidence backing; unresolved conflict markers; silent scope reduction.

## Subworker Assignment Plan

- `test`: owns baseline red/green evidence and final validation runs; writes `.pipeline/reviews/C10/M1/test.md`. Binds to the real-test contract: actual cargo commands above, not mocked substitutes. Rejects pseudo-tests that skip full-workspace check.
- `implement`: owns merge conflict resolution edits in product source. Must not create/edit/rewrite test assets owned by `test`.
- `audit`: read-only review of final diff, preserve-boundary compliance, worker identity separation; writes `.pipeline/reviews/C10/M1/audit.md`.
- Main agent: orchestrates, integrates, records lifecycle (requested/started/completed|failed|blocked/closed per role); never satisfies implement+validation roles itself.
- Non-overlap: one worker identity must not fill two of test/implement/audit.

## Expected Artifacts

- Merge commit(s) on `c10-upstream-0.143.0-alpha.33`
- `.pipeline/reports/C10/M1-merge-conflicts.md`
- `.pipeline/reviews/C10/M1/test.md`
- `.pipeline/reviews/C10/M1/audit.md`
