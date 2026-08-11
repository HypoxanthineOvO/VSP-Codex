# C4/M7 Test Worker Report

worker: test
milestone: M7 Preview Release Validation
scope: focused preview-release validation matrix only; no production runtime changes and no final release artifact assembly.

## Commands Run

All commands were run from `/home/heyx/Codex-VSP` unless noted.

- `cd codex-rs && just fmt`: passed.
- `cd codex-rs && just test -p codex-cli vscode`: passed, 8 passed / 225 skipped.
  - Covers VSCode install, dry-run/status, rollback, server-settings fallback, missing-scope diagnostics, and synthetic app-server initialize probe for the official extension contract.
- `cd codex-rs && just test -p codex-app-server initialize`: passed, 14 passed / 777 skipped.
  - Includes `initialize_is_not_blocked_by_slow_plugin_startup_warmup`.
- `cd codex-rs && just test -p codex-app-server thread_read`: passed, 18 passed / 773 skipped.
  - Covers `thread/read`, `thread/list`, `thread/name/set`, rename notification, restart persistence, archived-thread read, fork metadata, and turns pagination.
- `cd codex-rs && just test -p codex-tui startup`: passed, 45 passed / 2649 skipped.
  - Covers startup gate behavior, resume picker first-page lazy load, no eager preview/transcript load, terminal probe, and MCP startup settling.
- `cd codex-rs && just test -p codex-tui keymap`: passed, 108 passed / 2586 skipped.
  - Covers VSCode-terminal-safe default workflow bindings and keymap setup snapshots.
- `cd codex-rs && just test -p codex-tui markdown_render`: passed, 125 passed / 2569 skipped.
  - Covers M5 Markdown role colors, minimal strong/emphasis/list marker color paths, links, code, tables, and snapshots.
- `python3 scripts/collect_performance_evidence.py --timeout 3`: passed and wrote `.pipeline/evidence/performance-startup.json`.
  - `resume_list_first_page`: passed, 5 ms after initialize.
  - `vscode_app_server_initialize`: passed, 304 ms.
  - `cold_tui_start`: skipped honestly, only terminal control output observed before timeout.
  - `open_historical_session`: skipped honestly, no real historical thread id/home supplied.
- Manual isolated VSCode CLI smoke with temp `HOME`, `XDG_CONFIG_HOME`, `CODEX_HOME`, and fake `code` binary: passed.
  - `vsp-codex vscode install --yes` wrote `chatgpt.cliExecutable` and `chatgpt.localeOverride`.
  - `vsp-codex vscode status` reported configured user scope and executable path.
  - `vsp-codex vscode rollback --yes` restored original settings.
  - Final settings were `{"editor.fontSize": 15, "workbench.colorTheme": "Default Dark Modern"}` with one backup retained.
- `cd codex-rs && cargo insta pending-snapshots -p codex-tui`: failed because this local `cargo-insta` does not support `-p` on `pending-snapshots`.
- `cargo insta pending-snapshots --manifest-path codex-rs/tui/Cargo.toml`: passed, no pending snapshots.
- `git diff --check`: failed on an existing modified snapshot line:
  - `codex-rs/tui/src/bottom_pane/request_user_input/snapshots/codex_tui__bottom_pane__request_user_input__tests__request_user_input_multi_question_last.snap:15`: trailing whitespace.

Full workspace `just test` was not run because repository rules require asking the user before running the complete suite. This worker only ran focused tests/checks.

## Release Evidence Matrix

| Evidence Area | Status | Evidence |
| --- | --- | --- |
| VSCode install/status/rollback, isolated settings | pass | `just test -p codex-cli vscode` and manual temp-home smoke both passed. |
| Official-extension app-server initialize contract | pass | CLI vscode tests cover synthetic probe; app-server `initialize` tests passed including slow warmup non-blocking case. |
| App-server `thread/list` / `thread/read` / `thread/name/set` | pass | `just test -p codex-app-server thread_read` passed 18/18. |
| TUI startup and resume lazy-load behavior | pass | `just test -p codex-tui startup` passed 45/45. |
| VSCode-terminal-safe keymap evidence | pass | `just test -p codex-tui keymap` passed 108/108, including `default_workflow_bindings_do_not_require_vscode_terminal_conflicts`. |
| Markdown/color rendering evidence | pass | `just test -p codex-tui markdown_render` passed 125/125. |
| Performance evidence runner | partial pass | Runner executes and produces structured JSON. Two scenarios passed; two are honest skips due missing meaningful TUI visible text and missing real historical session input. |
| TUI snapshots | pass | No pending snapshots via `cargo insta pending-snapshots --manifest-path codex-rs/tui/Cargo.toml`. |
| Diff whitespace hygiene | fail | `git diff --check` reports trailing whitespace in an existing modified request-user-input snapshot. |
| Linux x64 preview package artifact | missing for implement | Test worker inspected package commands but did not assemble final artifact. |
| Release checksums / artifact path / installable archive | missing for implement | Must be produced by implement worker/report. |
| M1-M6 blocker evidence | pass with known residuals | M1-M6 audit reports pass after retries. Known release residuals remain: strict JSON VSCode settings, synthetic extension presence probe, M6 historical-open data requires real thread/home. |

## Linux x64 Artifact Gate For Implement Worker

The release workflow treats Linux x64 MUSL as the preview hard gate. Relevant source:

- `.github/workflows/rust-release.yml` defines `x86_64-unknown-linux-musl` primary and app-server release matrix entries.
- `.github/scripts/build-codex-package-archive.sh` wraps the package builder and writes both `.tar.gz` and `.tar.zst`.
- `scripts/build_codex_package.py` accepts `--target x86_64-unknown-linux-musl`, `--variant codex`, `--package-dir`, and repeated `--archive-output`.

Expected implement command shape for the primary preview artifact, after release binaries exist in `target/x86_64-unknown-linux-musl/release`:

```sh
bash .github/scripts/build-codex-package-archive.sh \
  --target x86_64-unknown-linux-musl \
  --bundle primary \
  --entrypoint-dir target/x86_64-unknown-linux-musl/release \
  --archive-dir dist/x86_64-unknown-linux-musl
```

The package builder requires a Linux `bwrap` resource for Linux targets. The wrapper auto-passes `--bwrap-bin` if `target/x86_64-unknown-linux-musl/release/bwrap` exists. Implement should verify the final archive contains:

- `codex-package.json`
- `bin/vsp-codex`
- `codex-path/rg`
- `codex-resources/bwrap`

## Sanitized Failure Summary For Implement Worker

- Release blocker: final Linux x64 preview artifact and checksum are not yet produced by this worker. Implement must build/package and record artifact paths/checksums in `.pipeline/reports/007-preview-release-validation.md`.
- Release hygiene blocker or explicit known limitation: `git diff --check` currently fails on trailing whitespace in `codex-rs/tui/src/bottom_pane/request_user_input/snapshots/codex_tui__bottom_pane__request_user_input__tests__request_user_input_multi_question_last.snap:15`.
- Evidence limitation: `open_historical_session` performance remains skipped unless implement/M7 release validation supplies `CODEX_PERF_THREAD_ID` and `CODEX_PERF_EXISTING_CODEX_HOME`.
- Evidence limitation: `cold_tui_start` remains skipped in this environment because no meaningful visible first-screen text was observed before timeout; the runner is honest and does not fabricate a duration.
- Known M1 limitation to carry into preview notes: VSCode settings parsing is strict JSON, so JSONC comments/trailing commas/original formatting are not preserved.
- Known M1 limitation to carry into preview notes: extension presence/install is only simulated in local tests; real end-user validation still needs a machine with the official extension path or explicit install instructions.

## Files Changed By This Worker

- `.pipeline/reviews/C4/M7/test.md`

The performance command also refreshed/generated `.pipeline/evidence/performance-startup.json` as its normal output.

## Test Retry: Snapshot Whitespace Hygiene

Scope: only snapshot/test-report hygiene. No production files were changed and no release artifact was built.

Changed files:

- `codex-rs/tui/src/bottom_pane/request_user_input/snapshots/codex_tui__bottom_pane__request_user_input__tests__request_user_input_multi_question_last.snap`
- `.pipeline/reviews/C4/M7/test.md`

Commands:

- `git diff --check`: passed.
- `cargo insta pending-snapshots --manifest-path codex-rs/tui/Cargo.toml`: passed, no pending snapshots.
