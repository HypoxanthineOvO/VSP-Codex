# C9 M3 - Preserve VSP Runtime Surfaces

## Goal

Reapply or adapt VSP-specific runtime behavior after the upstream merge: provider/config, VSCode Remote SSH, Workflow surface, Subagent UX, and installer/updater compatibility.

## Technical Solution

Treat VSP customizations as protected behavioral surfaces. Add or repair focused tests for affected surfaces, adapting VSP code to upstream APIs instead of freezing old internals.

## Technical Route

1. Audit VSP provider/config paths and session provider cache behavior.
2. Audit `vsp-codex vscode install/status/rollback` and Remote SSH settings priority.
3. Audit Workflow runtime guidance, `/hw:*` routing, request_user_input metadata, Workflow Markdown surface, and plan freshness.
4. Audit Subagent naming, active panel, and Prompt/Report viewer behavior.
5. Repair compile/test failures and update fixtures/snapshots where justified.

## Research Required

Pending execution. Actual repair scope depends on upstream merge changes.

## Risks And Alternatives

Risks: upstream renamed modules or changed protocol types; tests may need significant fixture updates.

Rejected alternative: blindly preserving old VSP code against new upstream architecture.

## Validation Path

Commands:

```bash
cargo test --manifest-path codex-rs/Cargo.toml -p codex-tui workflow_slash_routing -- --nocapture
cargo test --manifest-path codex-rs/Cargo.toml -p codex-core workflow_routing -- --nocapture
cargo test --manifest-path codex-rs/Cargo.toml -p codex-app-server --test all request_user_input -- --nocapture
cargo test --manifest-path codex-rs/Cargo.toml -p codex-app-server --test all workflow_markdown_surface -- --nocapture
cargo test --manifest-path codex-rs/Cargo.toml -p codex-cli --test vscode -- --nocapture
```

Pass signals: focused protected-surface tests pass; no fake or unrelated-only validation.

## Subworker Assignment Plan

- `test`: Owns red/focused tests, fixtures, assertions, snapshots, and final focused validation. Writes `.pipeline/reviews/C9/M3/test.md`.
- `implement`: Owns production repairs only. Must not create, edit, rewrite, or satisfy test-owned assets.
- `audit`: Checks final diff, protected VSP surfaces, evidence quality, and worker separation. Writes `.pipeline/reviews/C9/M3/audit.md`.

Lifecycle evidence required for all roles.
