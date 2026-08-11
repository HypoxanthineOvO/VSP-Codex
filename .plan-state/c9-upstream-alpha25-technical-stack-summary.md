# C9 Upstream Alpha.25 Technical Stack Checkpoint

## Discover Boundary

The plan targets a broad Codex-VSP upstream synchronization toward `rust-v0.143.0-alpha.25`. The merge direction is incremental by conflict group from the current VSP branch. VSP behavior should be replaced by official behavior when alpha.25 is equivalent, and preserved only when official Codex lacks the VSP capability or the VSP difference remains intentional.

The public version remains in the `0.140.0-vsp.8` series. A patch version such as `0.140.0-vsp.8.1` is allowed, but the final public tag/package version must be confirmed again before release execution.

## Stack Facts

| Area | Decision / Fact |
| --- | --- |
| Primary stack | Rust workspace under `codex-rs/Cargo.toml` |
| Secondary stack | Node/pnpm for repo formatting and JS/plugin maintenance; Python scripts for release/evidence tooling |
| CI baseline | GitLab CI uses Rust `1.93.0`, `cargo-nextest`, `uv`, and runner tag `latex-runner` |
| Package builder | `scripts/build_vsp_preview_release.py` reads `[workspace.package].version` and writes `.pipeline/release/v<version>/` |
| Current release line | Existing release artifacts are under `.pipeline/release/v0.140.0-vsp.8/` |
| Remote writes | GitLab release creation/update remains out of scope until a later explicit confirmation |

## Validation Contract

The minimum release-preparation floor is full local release dry run plus broad regression plus SQLite runtime I/O smoke.

Required command families:

- Formatting and hygiene: `cargo fmt --manifest-path codex-rs/Cargo.toml --all -- --check`, `pnpm format`, `python3 scripts/check_ci_docs.py`, `git diff --check`.
- Targeted Rust checks: `cargo test --manifest-path codex-rs/Cargo.toml -p codex-state filter -- --nocapture` and focused `cargo check` across `codex-state`, `codex-app-server`, `codex-app-server-protocol`, `codex-core`, `codex-cli`, and `codex-tui`.
- Broad regression candidates: `just test`, focused `cargo nextest` packages, full TUI regression, and bench smoke where practical.
- Release dry run: `cargo build --manifest-path codex-rs/Cargo.toml --release --bin vsp-codex`, then `python3 scripts/build_vsp_preview_release.py --entrypoint-bin codex-rs/target/release/vsp-codex --force`.
- SQLite I/O smoke: run a bounded real VSP Codex TUI/app-server scenario with isolated `CODEX_HOME` and `CODEX_SQLITE_HOME`, measure `logs_2.sqlite` plus `logs_2.sqlite-wal`, and inspect persisted targets for `log`, `codex_otel.log_only`, and `codex_otel.trace_safe`.

## Architecture Inputs

Architecture must classify the large VSP diff into conflict domains before Decompose:

- persistent SQLite logging and `state/log_db`;
- app-server protocol, request_user_input, and Question Tool surfaces;
- core workflow context, plan freshness, goals, and multi-agent routing;
- TUI workflow cards, plan panels, status surfaces, snapshots, and VSP provider UX;
- CLI entrypoint, VSCode/Remote integration, updater, installer, and release scripts;
- plugin/marketplace behavior and OpenCode/Hypo-Workflow adapters;
- docs, GitLab CI, and release metadata.

The existing `.pipeline/architecture.md` was not available as a readable baseline in this checkpoint, so Architecture should regenerate or repair the baseline rather than assuming one exists.

## Open Items

- Define the exact SQLite runtime I/O threshold: scenario duration, permitted byte growth, and target-row rejection query.
- Decide the final public release version/tag before execution: strict `v0.140.0-vsp.8` versus `v0.140.0-vsp.8.1`.
- For each VSP feature domain, determine whether alpha.25 has an equivalent official implementation.
