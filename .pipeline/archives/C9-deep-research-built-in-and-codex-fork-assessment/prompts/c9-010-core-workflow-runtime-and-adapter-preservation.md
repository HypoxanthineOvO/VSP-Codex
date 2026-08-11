# C9 M10 - Core Workflow Runtime And Adapter Preservation

## Objective

Merge core runtime changes while preserving Hypo-Workflow routing, plan freshness gates, goals, worker separation, multi-agent boundaries, plugin behavior, and OpenCode adapters.

## Requirements

- Preserve strict worker separation and Codex `/hw:start` / `/hw:resume` subworker authorization.
- Preserve Workflow command routing and plan freshness behavior.
- Preserve plugin/marketplace behavior unless official alpha.25 equivalent is proven.
- Keep OpenCode/Hypo adapters aligned with runtime behavior.

## Boundaries

- In scope: `codex-rs/core/`, `codex-rs/core-plugins/`, selected `.opencode/` adapter files when behavior requires it, and related tests.
- Out of scope: app-server protocol redesign already owned by M9, TUI surface merge owned by M11, release packaging owned by M12.

## Technical Solution

Use core workflow runtime as the semantic boundary after protocol is stable. Accept upstream internal refactors when they do not change VSP routing, freshness, worker separation, goal, multi-agent, or plugin contracts.

## Technical Route

1. Merge core session/tool/compaction changes by package and conflict domain.
2. Preserve `workflow_context`, `workflow_files`, `workflow_routing`, plan freshness, goals, and request_user_input tool semantics.
3. Reconcile multi-agent and plugin manager changes with alpha.25.
4. Update OpenCode/Hypo adapter metadata only when runtime behavior changes.
5. Record official replacements or preserve decisions in `.pipeline/reports/C9/M10-core-adapters.md`.

## Research Required

Resolved by existing project configuration and Architecture:

- `.pipeline/config.yaml` already sets strict worker separation and authorizes Codex execution subworkers for `/hw:start` and `/hw:resume`.
- Architecture classifies workflow routing and adapters as preserve-by-default.

## Risks And Alternatives

Core changes can compile while subtly breaking Workflow command routing. Treating adapters as docs-only was rejected because they are active integration surfaces.

## Validation Commands

```bash
cargo test --manifest-path codex-rs/Cargo.toml -p codex-core workflow_routing -- --nocapture
cargo test --manifest-path codex-rs/Cargo.toml -p codex-core plan_update_gate -- --nocapture
cargo test --manifest-path codex-rs/Cargo.toml -p codex-core request_user_input -- --nocapture
cargo test --manifest-path codex-rs/Cargo.toml -p codex-core-plugins
git diff --check
```

## Evidence

Pass signals: core workflow and plugin tests pass with strict worker separation semantics intact.

Fail signals: Workflow commands route as ordinary chat, plan freshness gates disappear, worker separation downgrades, or plugin behavior regresses.

## Subworker Assignment Plan

- `test`: owns core/plugin validation selection and final test evidence. Writes `.pipeline/reviews/C9/M10/test.md`.
- `implement`: owns scoped core/plugin/adapter reconciliation after test scope is defined.
- `audit`: read-only review of worker separation, adapter freshness, and upstream replacement claims. Writes `.pipeline/reviews/C9/M10/audit.md`.
- Main agent: coordinates lifecycle state and integrates implementation after role assignment.
- Non-overlap: strict separation requires distinct `test`, `implement`, and `audit` identities.
- Spawned worker write scope: spawned workers may edit `.pipeline/` artifacts only unless explicitly granted named runtime/adapter paths by the orchestrator. `audit` is read-only.

## Expected Artifacts

- Reconciled core/plugin/adapter changes as needed.
- `.pipeline/reports/C9/M10-core-adapters.md`
- `.pipeline/reviews/C9/M10/test.md`
- `.pipeline/reviews/C9/M10/audit.md`
