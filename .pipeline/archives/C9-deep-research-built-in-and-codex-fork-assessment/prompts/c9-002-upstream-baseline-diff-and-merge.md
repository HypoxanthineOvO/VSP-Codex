# C9 M2 - Upstream Baseline Diff And Merge

## Goal

Fetch and merge upstream `rust-v0.140.0-alpha.18`, producing a conflict-resolved tree that preserves VSP surfaces and structurally compiles.

## Technical Solution

Fetch upstream tags, compare current VSP divergence with the target tag, merge the selected tag into the isolated upgrade branch, and resolve conflicts by adapting VSP customizations to the new upstream shape.

## Technical Route

1. Confirm execution gate for network fetch and merge.
2. Run `git fetch upstream --tags`.
3. Confirm `rust-v0.140.0-alpha.18` resolves to the expected upstream tag.
4. Inspect high-risk diffs and merge the target tag.
5. Resolve conflicts in Rust workspace, protocol, app-server, TUI, installer/docs, release artifacts, and `.pipeline` files.
6. Run structural checks.

## Research Required

Pending execution. The conflict set is unknowable until the actual fetch/merge.

## Risks And Alternatives

Risks: upstream alpha may introduce app-server/protocol/TUI changes; VSP provider, VSCode, Workflow and release customizations may conflict.

Rejected alternatives: cherry-picking selected commits, or targeting `0.139.0`, because user selected latest release `0.140.0-alpha.18`.

## Validation Path

Commands:

```bash
git status --short
cargo fmt --manifest-path codex-rs/Cargo.toml --check
cargo check --manifest-path codex-rs/Cargo.toml -p codex-cli --bin vsp-codex
```

Pass signals: no unresolved conflicts, formatting passes or blocker is recorded, `vsp-codex` target type-checks.

Audit must reject merge completion without conflict status and compile/fmt evidence.

## Subworker Assignment Plan

- `test`: Owns pre/post merge evidence collection, target tag verification, and structural validation commands. Writes `.pipeline/reviews/C9/M2/test.md`.
- `implement`: Owns fetch/merge/conflict resolution after gates. Must not rewrite test-owned evidence.
- `audit`: Read-only review of conflict resolutions and target tag correctness. Writes `.pipeline/reviews/C9/M2/audit.md`.

Lifecycle evidence required for all roles.
