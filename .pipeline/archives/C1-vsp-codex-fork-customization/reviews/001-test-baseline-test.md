# M1 Test Worker Review

## Role

Test worker for Milestone 1. Scope was limited to current baseline investigation and writing:

- `.pipeline/reports/test-baseline.md`
- `.pipeline/reviews/001-test-baseline-test.md`
- `.pipeline/reports/test-baseline-artifacts/`

No source, test, snapshot, manifest, lockfile, BUILD, or justfile edits were made or retained.

## Commands Covered

All required commands were attempted or run:

- `git status --short`
- `rustc --version`
- `cargo --version`
- `just --version`
- `cargo nextest --version`
- `cargo insta --version`
- `cargo build --bin codex`
- `./target/debug/codex --version`
- `just test -p codex-tui`
- `cargo insta pending-snapshots -p codex-tui`
- `just test -p codex-cli`
- `just test -p codex-config`
- `just test -p codex-model-provider-info`
- `just test -p codex-model-provider`
- `just test -p codex-login`
- `just test -p codex-exec-server`
- `tmux -V`
- `just test -p codex-tui resize_reflow -- --ignored`

Rust-related commands were run from `/home/heyx/Codex-VSP/codex-rs`. Complete command excerpts are in `.pipeline/reports/test-baseline-artifacts/`.

## Findings

- `cargo build --bin codex` passed.
- `./target/debug/codex --version` passed and printed `codex-cli 0.0.0`.
- `just test -p codex-tui` failed: 2603 passed, 11 failed, 4 skipped.
- `cargo insta pending-snapshots -p codex-tui` failed because installed `cargo-insta 1.47.2` does not accept `-p` for `pending-snapshots`.
- Auxiliary `cargo insta pending-snapshots` passed with `No pending snapshots.`
- `just test -p codex-cli`, `codex-config`, `codex-model-provider-info`, `codex-model-provider`, `codex-login`, and `codex-exec-server` all passed.
- `tmux -V` passed with `tmux 3.4`.
- `just test -p codex-tui resize_reflow -- --ignored` passed: 3 tests passed, 2615 skipped.

## Side Effects

- `cargo insta` was initially unavailable. It was installed with `cargo install --locked cargo-insta`, producing user-level toolchain state outside the repo.
- The failed TUI snapshot run generated five `.snap.new` files under `codex-rs/tui/src/chatwidget/snapshots/`. They were removed immediately and not accepted.
- Final `git status --short` still showed only untracked `.pipeline/` and `.plan-state/`.

## Baseline Judgment

Use the passing targeted crates as M2-M6 gates for their respective areas. Do not represent the TUI crate as green: full `codex-tui` is currently a red baseline and should be fixed or explicitly waived before UI-visible work depends on it.

The ignored tmux resize/reflow smoke is valuable and currently passes, but it is narrow and cannot replace full `codex-tui`.
