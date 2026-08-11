# C9 M11 - TUI CLI VSCode And User Surface Convergence

## Objective

Merge user-facing alpha.25 changes while preserving VSP TUI workflow UX, Question Tool cards, plan panel, provider setup, `vsp-codex` CLI identity, VSCode/Remote integration, and update behavior.

## Requirements

- Merge TUI only after protocol, core, and app-server contracts are stable.
- Preserve VSP provider setup and `vsp-codex` command identity.
- Preserve Question Tool cards, plan panel, workflow slash routing, status/footer surfaces, VSCode install/status/rollback, and updater behavior.
- Treat snapshot updates as semantic UI evidence, not mechanical churn.

## Boundaries

- In scope: `codex-rs/tui/`, `codex-rs/cli/`, selected docs snippets for confirmed behavior, and focused tests.
- Out of scope: protocol contract redesign already owned by M9, core runtime policy owned by M10, release packaging owned by M12.

## Technical Solution

Reconcile the large TUI surface after lower-level contracts stabilize. Preserve VSP user-visible behavior unless official alpha.25 has a proven equivalent.

## Technical Route

1. Reconcile TUI by feature area: request_user_input cards, workflow cards, plan panel, status/footer, provider setup, markdown/workflow rendering, and slash routing.
2. Reconcile CLI entrypoint, VSCode install/status/rollback, and update command behavior.
3. Run focused TUI and CLI tests before broad TUI regression.
4. Review snapshot changes for user-visible meaning.
5. Update docs snippets only for confirmed behavior.

## Research Required

Resolved by Architecture ordering and existing C9 preservation policy:

- Architecture places TUI after protocol/core/app-server to reduce repeated conflicts.
- Existing C9 policy identifies VSP provider, VSCode Remote SSH, Workflow, Subagent UX, and plugin surfaces as must-preserve.

## Risks And Alternatives

Updating snapshots too early can hide behavior regressions. Deferring TUI entirely was rejected because release readiness requires user-facing validation.

## Validation Commands

```bash
cargo test --manifest-path codex-rs/Cargo.toml -p codex-tui workflow_slash_routing -- --nocapture
cargo test --manifest-path codex-rs/Cargo.toml -p codex-tui request_user_input -- --nocapture
cargo test --manifest-path codex-rs/Cargo.toml -p codex-cli --test vscode -- --nocapture
cargo nextest run --manifest-path codex-rs/Cargo.toml --no-fail-fast -p codex-tui
git diff --check
```

## Evidence

Pass signals: focused TUI/CLI tests pass, broad TUI regression is run or limitations are documented, and snapshot changes map to intentional UI semantics.

Fail signals: Question cards, plan panel, provider setup, `vsp-codex` identity, or VSCode Remote behavior regresses.

## Subworker Assignment Plan

- `test`: owns TUI/CLI test selection, snapshot review checklist, and final test evidence. Writes `.pipeline/reviews/C9/M11/test.md`.
- `implement`: owns scoped TUI/CLI/docs reconciliation after tests are defined.
- `audit`: read-only review of VSP UX preservation, snapshot semantics, and official-equivalence claims. Writes `.pipeline/reviews/C9/M11/audit.md`.
- Main agent: coordinates source integration and lifecycle evidence after role assignment.
- Non-overlap: `test`, `implement`, and `audit` must remain distinct.
- Spawned worker write scope: spawned workers may edit `.pipeline/` artifacts only unless explicitly granted named TUI/CLI/docs paths by the orchestrator. `audit` is read-only.

## Expected Artifacts

- Reconciled TUI/CLI user-surface changes.
- Updated snapshots only when semantically justified.
- `.pipeline/reports/C9/M11-user-surfaces.md`
- `.pipeline/reviews/C9/M11/test.md`
- `.pipeline/reviews/C9/M11/audit.md`
