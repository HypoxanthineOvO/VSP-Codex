# M1 Test Baseline

## Summary

Current baseline is mixed.

- Build entrypoint passes: `cargo build --bin codex` and `./target/debug/codex --version`.
- Non-TUI targeted crates pass: `codex-cli`, `codex-config`, `codex-model-provider-info`, `codex-model-provider`, `codex-login`, and `codex-exec-server`.
- `just test -p codex-tui` fails in the current workspace: 2603 passed, 11 failed, 4 skipped. The failures are snapshot/status-surface related and include project/cwd preview differences such as `my-project`/`project` changing to `tmp`.
- `tmux` is available and the ignored resize/reflow smoke passes: 3 passed, 2615 skipped.
- `cargo insta pending-snapshots -p codex-tui` was attempted exactly as requested but fails with this installed `cargo-insta` CLI because `pending-snapshots` does not accept `-p`. Auxiliary `cargo insta pending-snapshots` reports `No pending snapshots.`
- `cargo insta` was initially missing, then installed with `cargo install --locked cargo-insta`; this changed user tooling only, not repository files.

Do not treat `codex-tui` as a passing regression gate until the existing snapshot/status-surface failures are resolved or explicitly waived.

## Environment

- Host cwd for repo commands: `/home/heyx/Codex-VSP`
- Rust command cwd: `/home/heyx/Codex-VSP/codex-rs`
- `rustc --version`: `rustc 1.93.0 (254b59607 2026-01-19)`
- `cargo --version`: `cargo 1.93.0 (083ac5135 2025-12-15)`
- `just --version`: `just 1.51.0`
- `cargo nextest --version`: `cargo-nextest 0.9.136`
- `cargo insta --version`: initially failed with `no such command: insta`; after install, `cargo-insta 1.47.2`
- `tmux -V`: `tmux 3.4`

Artifacts are under `.pipeline/reports/test-baseline-artifacts/`.

## Worktree State

Initial and final `git status --short` both showed:

```text
?? .pipeline/
?? .plan-state/
```

No business source, tests, snapshots, Cargo files, Bazel files, or justfiles were edited. The failed `codex-tui` snapshot run generated five `.snap.new` files under `codex-rs/tui/src/chatwidget/snapshots/`; those generated files were removed immediately and not accepted.

## Command Matrix

| Area | Cwd | Command | Status | Duration | Key evidence | Recommended baseline role |
|---|---|---:|---|---:|---|---|
| worktree | `/home/heyx/Codex-VSP` | `git status --short` | passed | <1s | `?? .pipeline/`, `?? .plan-state/` | Required preflight; records dirty/untracked state only. |
| toolchain | `/home/heyx/Codex-VSP/codex-rs` | `rustc --version` | passed | <1s | `rustc 1.93.0` | Required environment fingerprint. |
| toolchain | `/home/heyx/Codex-VSP/codex-rs` | `cargo --version` | passed | <1s | `cargo 1.93.0` | Required environment fingerprint. |
| toolchain | `/home/heyx/Codex-VSP/codex-rs` | `just --version` | passed | <1s | `just 1.51.0` | Required environment fingerprint. |
| toolchain | `/home/heyx/Codex-VSP/codex-rs` | `cargo nextest --version` | passed | <1s | `cargo-nextest 0.9.136` | Required test runner fingerprint. |
| toolchain | `/home/heyx/Codex-VSP/codex-rs` | `cargo insta --version` | failed then passed after install | <1s, then <1s | Initial status 101: `no such command: insta`; after install: `cargo-insta 1.47.2` | Required snapshot tooling check; install prerequisite before snapshot commands. |
| build | `/home/heyx/Codex-VSP/codex-rs` | `cargo build --bin codex` | passed | 1s | `Finished dev profile` | Build-only smoke, not a test gate. |
| build | `/home/heyx/Codex-VSP/codex-rs` | `./target/debug/codex --version` | passed | <1s | `codex-cli 0.0.0` | CLI binary presence/version smoke. |
| TUI | `/home/heyx/Codex-VSP/codex-rs` | `just test -p codex-tui` | failed | 340s | `2614 tests run: 2603 passed, 11 failed, 4 skipped`; exit 100 | Must be a red gate for M2 UI work until fixed or explicitly waived. |
| TUI snapshots | `/home/heyx/Codex-VSP/codex-rs` | `cargo insta pending-snapshots -p codex-tui` | failed | <1s | `unexpected argument '-p' found`; exit 2 | Tool-incompatible as written; do not use this exact command as a gate with `cargo-insta 1.47.2`. |
| TUI snapshots aux | `/home/heyx/Codex-VSP/codex-rs` | `cargo insta pending-snapshots` | passed | 1s | `No pending snapshots.` | Auxiliary pending snapshot hygiene check. |
| CLI | `/home/heyx/Codex-VSP/codex-rs` | `just test -p codex-cli` | passed | 277s | `217 tests run: 217 passed, 0 skipped` plus bench smoke | Recommended gate for CLI and command parsing changes. |
| config | `/home/heyx/Codex-VSP/codex-rs` | `just test -p codex-config` | passed | 81s | `144 tests run: 144 passed, 0 skipped` plus bench smoke | Recommended gate for config/home/profile/provider routing changes. |
| provider info | `/home/heyx/Codex-VSP/codex-rs` | `just test -p codex-model-provider-info` | passed | 144s | `20 tests run: 20 passed, 0 skipped` plus bench smoke | Recommended gate for provider catalog/config metadata changes. |
| provider runtime | `/home/heyx/Codex-VSP/codex-rs` | `just test -p codex-model-provider` | passed | 211s | `30 tests run: 30 passed, 0 skipped` plus bench smoke | Recommended gate for provider auth/runtime behavior changes. |
| login | `/home/heyx/Codex-VSP/codex-rs` | `just test -p codex-login` | passed | 350s | `115 tests run: 115 passed, 0 skipped` plus bench smoke | Recommended gate for login, official service disabling, auth storage, workspace restrictions. |
| exec server | `/home/heyx/Codex-VSP/codex-rs` | `just test -p codex-exec-server` | passed | 75s | `185 tests run: 185 passed, 0 skipped` plus bench smoke | Recommended gate for exec-server and remote/local process/filesystem behavior. |
| TUI smoke env | `/home/heyx/Codex-VSP` | `tmux -V` | passed | <1s | `tmux 3.4` | Required environment preflight for tmux smoke. |
| TUI smoke | `/home/heyx/Codex-VSP/codex-rs` | `just test -p codex-tui resize_reflow -- --ignored` | passed | 25s | `3 tests run: 3 passed, 2615 skipped` | Strong targeted user-simulation gate for resize/reflow, but narrow. |

## TUI User Simulation Coverage

Snapshot tests:
- Present and substantial in `codex-tui`.
- Current full TUI crate run is not clean because 11 tests fail.
- The observed failures are around `chatwidget::tests::status_surface_previews`, `terminal_title`, `ide_context::ipc`, and snapshot text that depends on cwd/project labels.
- Failed snapshot run produced `.snap.new`; generated files were removed to respect the no-snapshot-edit boundary.

VT100-style tests:
- `codex-tui` uses rendered terminal/snapshot coverage and many unit-style UI assertions, but current full crate failure prevents using it as a green baseline.
- Passing subset evidence exists inside the failed run, but it cannot substitute for the full crate gate.

tmux ignored smoke:
- `tmux 3.4` is available.
- `resize_reflow` ignored smoke passed all 3 tests and exercises real tmux resize/reflow behavior.
- This is a useful M2-M6 targeted smoke for layout regressions, but it is not broad enough to replace `just test -p codex-tui`.

Gaps before M2:
- Resolve or waive the 11 `codex-tui` failures.
- Decide whether to standardize pending snapshot command on supported `cargo-insta` syntax, because `cargo insta pending-snapshots -p codex-tui` is not valid for installed `cargo-insta 1.47.2`.
- For UI-visible M2 changes, add or update `insta` snapshots only after the current baseline issue is understood.

## Failure And Blocker Details

`cargo insta --version` initially failed:
- Exit status: 101.
- Evidence: `error: no such command: insta`.
- Action taken: installed `cargo-insta v1.47.2` via `cargo install --locked cargo-insta`.
- Result: `cargo insta --version` passed.

`just test -p codex-tui` failed:
- Exit status: 100.
- Summary: `2614 tests run: 2603 passed, 11 failed, 4 skipped`.
- Failed tests listed by nextest:
  - `chatwidget::tests::status_surface_previews::missing_project_root_uses_different_status_and_title_preview_sources`
  - `chatwidget::tests::terminal_title::terminal_title_activity_indicators_do_not_animate_when_animations_are_disabled`
  - `chatwidget::tests::terminal_title::terminal_title_shows_action_required_while_exec_approval_is_pending`
  - `ide_context::ipc::tests::fetch_ide_context_uses_unregistered_request_route`
  - `chatwidget::tests::terminal_title::terminal_title_action_required_blinks_when_animations_are_enabled`
  - `chatwidget::tests::terminal_title::terminal_title_action_required_respects_spinner_setting`
  - `chatwidget::tests::status_surface_previews::status_line_setup_popup_mixed_snapshot`
  - `chatwidget::tests::status_surface_previews::status_line_setup_popup_hardcoded_only_snapshot`
  - `chatwidget::tests::status_surface_previews::status_surface_preview_lines_hardcoded_only_snapshot`
  - `chatwidget::tests::status_surface_previews::terminal_title_setup_popup_mixed_snapshot`
  - `chatwidget::tests::status_surface_previews::status_surface_preview_lines_mixed_snapshot`
- Representative diffs:
  - `my-project · feature/mixed-preview · Mixed preview thread` changed to `tmp · feature/mixed-preview · Mixed preview thread`.
  - `project | Mixed preview thread | Tasks 0/0` changed to `tmp | Mixed preview thread | Tasks 0/0`.

`cargo insta pending-snapshots -p codex-tui` failed:
- Exit status: 2.
- Evidence: `error: unexpected argument '-p' found`.
- Auxiliary command `cargo insta pending-snapshots` passed and reported `No pending snapshots.`

No command was marked passed unless it completed with exit status 0.

## Recommended Regression Profile For M2-M6

Minimum preflight for every milestone:
- `git status --short`
- `cargo build --bin codex`
- `./target/debug/codex --version`

TUI and branding/UI work:
- Required once current baseline is fixed or waived: `just test -p codex-tui`
- Required for resize/layout work: `just test -p codex-tui resize_reflow -- --ignored`
- Snapshot hygiene: use a supported `cargo-insta` pending command for the installed version; current auxiliary command is `cargo insta pending-snapshots`.

CLI and command behavior:
- `just test -p codex-cli`

Config, provider, home directory, and provider-routing behavior:
- `just test -p codex-config`
- `just test -p codex-model-provider-info`
- `just test -p codex-model-provider`

Login, official service disabling, auth storage, workspace, and account behavior:
- `just test -p codex-login`

Exec-server and process/filesystem behavior:
- `just test -p codex-exec-server`

Execution guidance:
- Run these commands serially for stable timings. Some commands in this M1 run were parallelized and showed Cargo file-lock waits.
- Do not run complete workspace `just test` unless the user explicitly approves it.

## Upstream Merge Risk Notes

- The TUI baseline is already red, so future M2 UI changes will be hard to review unless the existing failures are fixed, pinned to an environment cause, or formally waived.
- The failing TUI snapshots include cwd/project label content, which is high-risk for VSP rename, status line, terminal title, and home/cwd presentation work.
- The exact requested `cargo insta pending-snapshots -p codex-tui` command is incompatible with installed `cargo-insta 1.47.2`; scripts or docs that assume package filtering there may fail locally.
- `just test -p <crate>` runs `just bench-smoke` after nextest in this repo. Treat command duration as test plus bench smoke cost, not nextest-only cost.
- Parallel test execution across crates causes Cargo lock contention and noisy elapsed times. CI-style profiles should run serially unless explicitly optimizing wall-clock time.

## Next Decision Gate

Recommended decision before M2:

1. Fix or explicitly waive the current `codex-tui` baseline failures.
2. Decide whether M2 may proceed with `codex-tui resize_reflow -- --ignored` plus non-TUI crate gates while full `codex-tui` remains red.
3. Standardize the snapshot pending command for this repo/tool version.
4. Keep full workspace `just test` out of M1/M2 unless the user confirms the cost and scope.

## Worker Lifecycle

- test worker requested: yes, Descartes (`019e59c9-f18a-7ec3-8eef-f7621d4ff9b7`).
- test worker started: 2026-05-24, Asia/Shanghai local time.
- test worker completed/closed: yes.
- test worker wrote allowed artifacts: `.pipeline/reports/test-baseline-artifacts/`, `.pipeline/reports/test-baseline.md`, `.pipeline/reviews/001-test-baseline-test.md`.
- implement worker requested: yes, Ampere (`019e59ca-0b51-7201-abf1-2aed05114835`).
- implement worker completed/closed: yes; wrote `.pipeline/reviews/001-test-baseline-implement.md` and confirmed no implementation work was needed for M1.
- audit worker requested: yes, Dirac (`019e59e0-481f-7730-badb-9491d2562407`).
- audit worker completed/closed: yes; wrote `.pipeline/reviews/001-test-baseline-audit.md` with verdict `pass`.
- lifecycle close status: M1 completed with known red `codex-tui` baseline and no retained source/snapshot changes.
