# C4/M7 Audit Report

verdict: pass

## reviewed_refs

- `.pipeline/prompts/007-preview-release-validation.md`
- `.pipeline/reviews/C4/M7/test.md`
- `.pipeline/reviews/C4/M7/implement.md`
- `.pipeline/reports/007-preview-release-validation.md`
- `.pipeline/release/C4-preview/vsp-codex-preview-linux-x64-gnu-debug.tar.gz`
- `.pipeline/release/C4-preview/vsp-codex-preview-linux-x64-gnu-debug.tar.gz.sha256`
- `.pipeline/evidence/performance-startup.json`
- `.pipeline/reviews/C4/M1/audit.md`
- `.pipeline/reviews/C4/M5/audit.md`
- `.pipeline/reviews/C4/M6/audit.md`

## checks

- Release artifact exists and checksum verifies.
  - `sha256sum -c .pipeline/release/C4-preview/vsp-codex-preview-linux-x64-gnu-debug.tar.gz.sha256`: passed.
  - SHA-256 matches `0cb38d6fc23de3aa57e125af5862f67b5400dc91009c5168e977e150b73be557`.
- Archive contents are sufficient for an internal Linux x64 preview smoke.
  - Verified package contains `codex-package.json`, `bin/vsp-codex`, `codex-path/rg`, `codex-resources/bwrap`, and `codex-resources/zsh/bin/zsh`.
  - Verified metadata reports `version: 0.134.0-vsp.2`, `target: x86_64-unknown-linux-gnu`, `variant: codex`, and `entrypoint: bin/vsp-codex`.
  - Verified executable bits are present for `bin/vsp-codex`, `codex-path/rg`, `codex-resources/bwrap`, and bundled `zsh`.
- GNU debug vs MUSL release status is not silently overclaimed.
  - M7 prompt makes Linux x64 the hard gate and says should-have gaps become known limitations.
  - Test worker noted MUSL as the release-workflow primary path.
  - Implement/report explicitly label the produced artifact as GNU/glibc debug/dev preview, not official MUSL production release.
  - This is acceptable for internal preview validation, but not sufficient for official MUSL release labeling.
- Focused validation matrix is sufficient for preview signoff.
  - `just test -p codex-cli vscode`: passed, 8/8.
  - `just test -p codex-app-server initialize`: passed, 14/14.
  - `just test -p codex-app-server thread_read`: passed, 18/18.
  - `just test -p codex-tui startup`: passed, 45/45.
  - `just test -p codex-tui keymap`: passed, 108/108.
  - `just test -p codex-tui markdown_render`: passed, 125/125.
  - TUI pending snapshots: none.
  - `git diff --check`: passed after test retry hygiene cleanup.
- Full workspace test omission is honest.
  - Report states full `just test` was not run because repository rules require user approval.
  - This is a residual risk, not an M7 blocker for scoped preview validation.
- VSCode hijack install/status/rollback evidence is sufficient for internal preview.
  - CLI focused tests cover install, dry-run/status, rollback, settings fallback, missing-scope diagnostics, and synthetic app-server initialize probe.
  - Manual isolated smoke confirms install writes `chatgpt.cliExecutable` and `chatgpt.localeOverride`, status reports configured user scope/path, and rollback restores the original settings.
  - M1 limitations are carried forward: strict JSON settings parsing and simulated extension presence.
- Performance evidence is honest.
  - `.pipeline/evidence/performance-startup.json` is valid JSON.
  - `resume_list_first_page`: passed.
  - `vscode_app_server_initialize`: passed.
  - `cold_tui_start`: skipped with `durationMs: null` because only terminal control bytes were observed.
  - `open_historical_session`: skipped with `durationMs: null` because no real thread id/home was supplied.
- Worker separation evidence is complete.
  - Prompt records authorized strict worker separation.
  - Test worker owned focused tests/smokes and did not assemble the release artifact.
  - Implement worker owned artifact assembly/checksum/report collation and did not edit production runtime code.
  - This audit was read-only.

## findings

- No blocker found.
- Info: The package is a usable internal Linux x64 GNU/glibc debug/dev preview artifact, not a MUSL release artifact. The report states this clearly, so this is not a needs_changes finding.
- Info: Preview install/rollback evidence is based on isolated settings and fake `code`/simulated extension presence. That is sufficient for local preview confidence, but real end-user extension-path validation remains deferred.
- Info: Performance evidence is structurally useful but incomplete for release-grade claims because `cold_tui_start` and `open_historical_session` are honest skips.

## residual_risks

- The artifact is very large because it contains an unstripped debug `vsp-codex` binary.
- It only targets compatible Linux x64/glibc environments; MUSL production packaging remains a follow-up.
- Strict JSON settings handling can reject or rewrite JSONC-style VSCode settings with comments/trailing commas.
- Full workspace `just test` was not run.
- Release-grade performance evidence should be rerun with a real historical session via `CODEX_PERF_THREAD_ID` and `CODEX_PERF_EXISTING_CODEX_HOME`.
