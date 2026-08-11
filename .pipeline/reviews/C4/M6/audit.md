# C4/M6 Audit Report

verdict: needs_changes

## reviewed_refs

- `.pipeline/prompts/006-performance-evidence-startup.md`
- `.pipeline/reviews/C4/M6/test.md`
- `.pipeline/reviews/C4/M6/implement.md`
- `.pipeline/reports/006-performance-evidence-startup.md`
- `.pipeline/evidence/performance-startup.json`
- `scripts/collect_performance_evidence.py`
- `codex-rs/tui/src/resume_picker.rs`
- `codex-rs/app-server/tests/suite/v2/initialize.rs`

## checks

- Evidence runner is structured and M7-callable: mostly yes. It emits schemaVersion, generatedAt, runId, codexBin, timeoutSeconds, and per-scenario records with status/duration/reason/details.
- Four scenario honesty:
  - `resume_list_first_page`: honest `passed` for app-server `thread/list` first page.
  - `open_historical_session`: honest `skipped` when no real thread/home is supplied.
  - `vscode_app_server_initialize`: honest `passed` for initialize response timing.
  - `cold_tui_start`: not honest enough as currently labeled `passed`; see blocker.
- No hidden cache system or user-facing `/perf`: passed. I found no M6 implementation of resume metadata cache, recent preview cache, progressive stale-cache verification, or `/perf` command.
- Test coverage:
  - Resume picker tests protect initial first-page-only load and no eager preview/transcript load.
  - App-server initialize test protects VSCode-style initialize from a delayed remote plugin warmup path.

## findings

- Critical: `cold_tui_start` passes on terminal control bytes, not meaningful visible first screen.
  - `scripts/collect_performance_evidence.py` stops as soon as `output.strip()` is truthy in `run_pty_until_output`.
  - The generated artifact shows `sample: "\u001b[?2004h"` and `observedBytes: 8`, which is only bracketed-paste terminal escape output.
  - This overstates the evidence as cold TUI start / first usable screen. It must either wait for meaningful visible text after stripping ANSI/control sequences, or record this scenario as `skipped`/`partial` with a reason such as “only terminal control output observed before timeout.”
  - Relevant refs: `scripts/collect_performance_evidence.py:156-158`, `scripts/collect_performance_evidence.py:219-231`, `.pipeline/evidence/performance-startup.json:14-22`.

## residual_risks

- The runner measures baseline smoke timings, not before/after improvements. That is acceptable for M6 if M7 treats it as reproducible release evidence rather than an optimization claim.
- `resume_list_first_page` uses an isolated empty `CODEX_HOME`, so it proves API path timing but not large-history behavior.
- `open_historical_session` depends on a real `CODEX_PERF_THREAD_ID` and `CODEX_PERF_EXISTING_CODEX_HOME`; M7 should supply these for release evidence if historical-open timing is required.

## retry_verdict

pass

## retry_checks

- The original blocker is resolved. `run_pty_until_output` no longer stops on `output.strip()` alone; it waits until `visible_terminal_text(decoded)` is non-empty.
- `visible_terminal_text` strips ANSI/control sequences and non-visible control characters before deciding whether output is meaningful.
- `measure_tui_first_output` now marks `cold_tui_start` as `passed` and writes `durationMs` only when meaningful visible text is observed.
- Control-only output is now recorded as `skipped`, with `durationMs: null` and reason `only terminal control output observed before timeout`.
- `.pipeline/evidence/performance-startup.json` is consistent with the fix: `cold_tui_start.status` is `skipped`, `durationMs` is `null`, `details.visibleSample` is empty, and the raw control-byte sample is retained for diagnostics.
- The M6 report and implement report both describe the corrected cold-start semantics and validation.
- I found no hidden cache system, user-facing `/perf` command, or broad startup optimization added in M6.

## retry_findings

- No blockers remain.

## retry_residual_risks

- `cold_tui_start` is honest but incomplete in this local smoke: it reports skipped because no meaningful visible first-screen text was observed before timeout.
- `resume_list_first_page` still uses an isolated empty `CODEX_HOME`; large-history release evidence should be collected in M7 if needed.
- `open_historical_session` still requires `CODEX_PERF_THREAD_ID` and `CODEX_PERF_EXISTING_CODEX_HOME` for real historical-open timing.
