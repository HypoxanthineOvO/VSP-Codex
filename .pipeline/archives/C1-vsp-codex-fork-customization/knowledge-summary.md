# Codex-VSP Knowledge Compact

## Project Shape

Codex-VSP is a large Rust-first monorepo for OpenAI Codex CLI. The main workspace is `codex-rs/`; outer directories provide npm packaging, SDKs, docs, release scripts, CI, and Bazel integration.

## Build Defaults

Use the root `justfile`; it changes working directory to `codex-rs`. Rust tests should use `just test`, not direct `cargo test`. `just` and `cargo-nextest` are installed locally.

## Runtime Entrypoint

The primary binary is `codex`, built by crate `codex-cli`. Local debug binary `codex-rs/target/debug/codex` was rebuilt and verified with `--version`, `--help`, and `doctor --summary`.

## Runtime Pitfalls

Codex local state DB migration conflict was repaired by backing up `~/.codex/state_5.sqlite*` with a `codex-repair` suffix and allowing Codex to recreate the DB. The current Codex shell still resolves plain `node` to v20.20.0; use `volta run --node 22 ...` for repository scripts requiring Node >=22.

## Key Constraints

Avoid expanding `codex-core` unnecessarily. app-server API work belongs to v2. Rust dependency changes require Bazel lock refresh. Config type changes require config schema regeneration.
