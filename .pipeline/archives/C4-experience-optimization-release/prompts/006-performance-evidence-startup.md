# M6 Performance Evidence + Low-Risk Startup Fixes

## Objective

为预览版提供冷启动、resume/history 和 VSCode initialize 的性能证据，并只做低风险启动优化。

## Source Of Truth

- `.plan-state/decompose.yaml` M6
- P1 final scope: measurement plus low-risk startup optimization only

## Subworker Assignment Plan

Status: authorized, strict worker separation.

- `test`
  - Owns timing/probe fixtures and no-network/slow-provider startup tests.
  - Expected artifact: `.pipeline/reviews/C4/M6/test.md`
- `implement`
  - Owns low-risk startup deferral and evidence script/report hooks.
  - Expected artifact: `.pipeline/reviews/C4/M6/implement.md`
- `audit`
  - Checks no half-built cache system, evidence quality, and correctness of deferred work.
  - Expected artifact: `.pipeline/reviews/C4/M6/audit.md`

## Technical Solution

Measure first. Add reproducible timing evidence for cold start, resume list, historical session open, and VSCode app-server initialize. Only defer non-first-screen work such as full history scan, provider/account probe, or MCP/tool discovery when correctness allows. Do not implement metadata caches.

## Technical Route

1. Identify startup, resume picker, thread open, and app-server initialize timing points.
2. Add testable timing/probe helpers or release evidence scripts; do not add a user-facing `/perf` command.
3. Build no-network/slow-provider startup probe.
4. Defer non-essential provider/account and MCP/tool discovery from first input when safe.
5. Avoid resume metadata cache, recent preview cache, and progressive stale-cache verification.
6. Record before/after or baseline/final evidence in release artifacts.

## Research Required

Pending implementation evidence: exact blocking points require timing. Do not guess and build caches without data.

## Risks And Alternatives

- Risk: background deferral changes visible state order. Only defer non-essential work and test failure visibility.
- Risk: performance claims are unprovable. Release evidence must be reproducible.
- Deferred: cache/progressive loading systems.

## Validation Path

Commands/scenarios:

- `just test -p codex-tui startup`
- `just test -p codex-app-server initialize`
- release evidence script for cold start/resume/open historical/VSCode initialize

Pass signal: timing evidence exists; no-network/slow-provider does not block first usable TUI screen; no cache consistency risk is introduced.

Fail signal: performance report has no data, startup still blocks on provider/MCP/history, or stale cache behavior appears.

## Audit Focus

- No hidden cache system.
- Evidence is reproducible.
- Correctness and diagnostics are not sacrificed for speed.
