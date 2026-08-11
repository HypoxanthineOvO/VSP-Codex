# C9 M9 - Protocol Contract And App-Server Bridge Merge

## Objective

Reconcile alpha.25 app-server protocol/schema churn with VSP Question Tool, plan update, thread/workflow metadata, and app-server bridge behavior.

## Requirements

- Treat `codex-rs/app-server-protocol` as the contract root.
- Preserve VSP `request_user_input` / Question Tool semantics unless official alpha.25 provides direct equivalent behavior.
- Keep Rust protocol definitions, JSON schema, TypeScript outputs, app-server, app-server-client, and tests in lockstep.

## Boundaries

- In scope: `codex-rs/app-server-protocol/`, `codex-rs/app-server-client/`, `codex-rs/app-server/`, and related tests.
- Out of scope: broad TUI visual merge, release packaging, plugin marketplace changes not required by protocol.

## Technical Solution

Reconcile schemas first, regenerate derived artifacts, then adapt app-server request processors and clients. Do not let TUI merge pressure define protocol shape implicitly.

## Technical Route

1. Compare alpha.25 protocol additions/removals with VSP `ToolRequestUserInput` and `TurnPlanUpdated` surfaces.
2. Decide official replacement versus VSP preservation for Ask/Question and plan update events.
3. Update Rust protocol definitions and regenerate schema fixtures.
4. Adapt app-server request processors, app-server-client, and tests to the reconciled contract.
5. Record decisions in `.pipeline/reports/C9/M9-protocol-app-server.md`.

## Research Required

Resolved by local code and deferred implementation checks:

- Diff evidence shows alpha.25 adds, removes, and renames protocol schema and TypeScript files.
- Existing VSP tests cover request_user_input and workflow surfaces.
- Exact schema generation and app-server tests must run after conflict resolution.

## Risks And Alternatives

Overwriting VSP protocol with upstream can break Hypo-Workflow clients. Freezing VSP protocol can miss upstream client changes. The selected route reconciles from the protocol contract boundary.

## Validation Commands

```bash
cargo run --manifest-path codex-rs/Cargo.toml -p codex-app-server-protocol --bin write_schema_fixtures
cargo test --manifest-path codex-rs/Cargo.toml -p codex-app-server-protocol
cargo test --manifest-path codex-rs/Cargo.toml -p codex-app-server --test all request_user_input -- --nocapture
cargo test --manifest-path codex-rs/Cargo.toml -p codex-app-server --test all workflow_markdown_surface -- --nocapture
git diff --check
```

## Evidence

Pass signals: schema generation is clean, protocol tests pass, and VSP Question Tool behavior remains covered.

Fail signals: generated schema drift is unexplained, app-server clients lose VSP request_user_input fields, or tests cover only mocks while real protocol paths break.

## Subworker Assignment Plan

- `test`: owns protocol/app-server test selection, schema drift checks, and final validation evidence. Writes `.pipeline/reviews/C9/M9/test.md`.
- `implement`: owns protocol/app-server reconciliation after `test` fixes the expected validation path. It must not update TUI snapshots in this milestone unless strictly required for compile.
- `audit`: read-only review of generated artifacts, app-server-client sync, and Question Tool downgrade risk. Writes `.pipeline/reviews/C9/M9/audit.md`.
- Main agent: coordinates schema generation and integrates source changes after worker assignment.
- Non-overlap: `test`, `implement`, and `audit` must remain distinct.
- Spawned worker write scope: spawned workers may edit `.pipeline/` artifacts only unless explicitly granted named source/schema paths by the orchestrator. `audit` is read-only.

## Expected Artifacts

- Reconciled protocol and app-server code.
- Regenerated schema artifacts where required.
- `.pipeline/reports/C9/M9-protocol-app-server.md`
- `.pipeline/reviews/C9/M9/test.md`
- `.pipeline/reviews/C9/M9/audit.md`
