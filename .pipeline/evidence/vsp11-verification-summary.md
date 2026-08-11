# VSP 11.0-alpha.1 Verification Summary

## Candidate identity

- Baseline: `v0.145.0-vsp.9.5` at `aabadcf78cba320b79ca323a4fd3d6ffb5b87f8d`
- Version: `0.145.0-vsp.11.0-alpha.1`
- Release binary SHA-256: `febc1290ce7c73b5bd3572b4083f3ba601ce844d99caa2216fea34e3e4a77c6d`
- Archive SHA-256: `f5ddd9265b3275bbb9c26cbbef419a94fd7b1431785a92a6819d18c4ec740dad`
- Stable manifest: unchanged from the 9.5 baseline
- Remote publication: not performed

The Linux `bwrap` source is unchanged from 9.5. Building it locally was blocked by
the missing system `libcap.pc`, so the candidate reuses the installed 9.5 binary
with SHA-256 `52231e1caf55bcbc667b269f49c63599a6f7db4767ae6a039580d0ff853db712`.

## Acceptance evidence

| Criterion | Result | Evidence |
| --- | --- | --- |
| A1: clean 9.5 baseline and Responses-only | Passed | 9.5 tag is an ancestor; production search found no `WireApi::Chat` or `/chat/completions`; source and package versions agree. |
| A2: bounded persistence | Passed | `vsp11-persistence-stress.json`; `codex-state` 156 tests passed; default persistent filter excludes TRACE and INFO while retaining WARN. |
| A3: memory and stall regression | Passed | `vsp11-vs-vsp95-comparison.json`; metadata-write P95 reduced 96.6%, compatibility-read P95 reduced 86.0%, RSS delta reduced from 110852 KiB to 0 KiB. |
| A4: multi-terminal request delivery | Passed | 24 request-user-input tests passed under default parallel execution, including first-answer-wins, disconnect, late resume, unsubscribe, and resolution fan-out. |
| A5: reasoning translation and rendering | Passed | 14 translation tests and 2 rendering tests passed, including byte-level UTF-8 chunking, revision replacement, duplicate suppression, and 20/40/80-column reflow. |
| A6: package, upgrade, and rollback | Passed | `vsp11-install-upgrade-rollback.json`; isolated 9.5 -> 11.0-alpha.1 -> 9.5 sequence passed and preserved the user marker. |
| A7: preserve existing work and databases | Passed | Product work stayed in `/home/heyx/VSP-Codex-v11`; the 10.1 worktree, installed release, and user databases were not reset, removed, or migrated. |
| A8: local acceptance gate only | Passed | Candidate artifacts are local under `/tmp`; no push, tag, release, upload, or remote manifest update was performed. |

## Verification runs

- `codex-thread-store`: 97 passed
- `codex-state`: 156 passed
- `codex-rollout`: 107 passed (99 unit and 8 integration)
- TUI translation: 14 passed
- TUI reasoning Markdown: 2 passed
- App-server request-user-input: 24 passed
- Release-builder and installer Python tests: 25 passed
- Rust formatting, diff whitespace checks, and Python byte-compilation: passed
- Six-process shared-home stress: 240 starts, 240 metadata writes, 2400 state reads, and 2400 compatibility reads; no lock or worker errors; both SQLite quick checks returned `ok`; no idle database or WAL growth
- Startup evidence: cold TUI 401 ms, thread list 2 ms, materialized historical resume 76 ms, app-server initialize 185 ms

The full upstream monorepo test matrix is not a release gate for this local fork.
All changed modules and persistence modules were tested directly; remote/provider
integration tests that require external services were not used as evidence.
