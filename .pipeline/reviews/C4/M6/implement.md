# C4/M6 Implement Worker Report

Updated: 2026-05-28T13:04:23+08:00

## Scope

Role: `implement`

Milestone: M6 `Performance Evidence + Low-Risk Startup Fixes`

Implemented only the production/script-side evidence path. No test files, snapshots, audit artifacts, cache systems, or broad startup rewrites were changed.

## Changes

- Added `scripts/collect_performance_evidence.py`
  - Collects structured preview-release startup evidence.
  - Writes JSON artifact with scenario, command, status, duration, reason, and details.
  - Uses isolated temporary `CODEX_HOME` for baseline app-server/TUI scenarios.
  - Records `skipped` with reasons when a scenario cannot be measured.
- Audit retry fix:
  - `cold_tui_start` now strips ANSI/control sequences and only passes after meaningful visible text is observed.
  - If PTY output contains only terminal control bytes before timeout, it records `skipped`, keeps `durationMs` as null, and writes raw/visible samples in details.
- Added `.pipeline/reports/006-performance-evidence-startup.md`
  - Documents runner behavior, smoke evidence, and M7 release handoff.

Generated smoke artifact:

- `.pipeline/evidence/performance-startup.json`

## Validation

Commands run from repo root:

```sh
python3 -m py_compile scripts/collect_performance_evidence.py
python3 scripts/collect_performance_evidence.py --timeout 3
python3 -m json.tool .pipeline/evidence/performance-startup.json
```

Results:

- Python compile check passed.
- Runner smoke passed and wrote `.pipeline/evidence/performance-startup.json`.
- JSON artifact validation passed.
- Smoke statuses:
  - `cold_tui_start`: skipped with `only terminal control output observed before timeout`.
  - `resume_list_first_page`: passed.
  - `open_historical_session`: skipped with a structured reason because no real historical thread was supplied.
  - `vscode_app_server_initialize`: passed.

## Explicit Non-Goals

- Did not implement resume metadata cache.
- Did not implement recent preview cache.
- Did not implement progressive stale-cache verification.
- Did not claim performance improvements beyond the recorded baseline evidence.
- Did not add a user-facing `/perf` command.
