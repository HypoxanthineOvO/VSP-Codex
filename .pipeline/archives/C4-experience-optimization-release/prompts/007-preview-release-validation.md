# M7 Preview Release Validation

## Objective

把 VSCode/TUI preview 的核心路径打包、验证并产出可发布证据。

## Source Of Truth

- `.plan-state/decompose.yaml` M7
- Outputs from M1-M6

## Subworker Assignment Plan

Status: authorized, strict worker separation.

- `test`
  - Owns final focused test matrix, smoke scripts, snapshot/schema checks, and release evidence verification.
  - Expected artifact: `.pipeline/reviews/C4/M7/test.md`
- `implement`
  - Owns release artifact assembly, quickstart, rollback docs, known limitations, and report collation.
  - Expected artifact: `.pipeline/reviews/C4/M7/implement.md`
- `audit`
  - Checks blocker evidence, worker separation evidence, release usability, and deferred scope honesty.
  - Expected artifact: `.pipeline/reviews/C4/M7/audit.md`

## Technical Solution

This milestone does not add product features. It closes the preview release by running focused validation, building at least Linux x64 artifact, validating VSCode hijack install/rollback, and producing release evidence. Should-have gaps become known limitations, not silent blockers.

## Technical Route

1. Run `just fmt`.
2. Run scoped `just fix -p <changed-crate>` for changed crates.
3. Run focused tests for CLI, TUI, app-server, app-server-protocol, and core as applicable.
4. Run schema flow if app-server protocol changed.
5. Run TUI snapshot flow and accept intended snapshots.
6. Build Linux x64 preview artifact; attempt macOS arm64 only if environment/build chain is ready.
7. Run VSCode install/status/rollback smoke in isolated settings.
8. Run app-server initialize, thread start/list/read/name set, and reasoning verdict probes.
9. Write `.pipeline/reports/007-preview-release-validation.md` with artifact paths, checksums, logs, known limitations, and follow-up Cycle candidates.

## Research Required

Pending implementation evidence: final release target and build chain details are environment-dependent. Linux x64 is the hard gate.

## Risks And Alternatives

- Risk: release validation discovers earlier feature gaps. Do not hide them; either fix blocker or list should-have limitation.
- Risk: full workspace test is slow. Follow repository rule: ask user before full `just test`.
- Rejected: source-only release without usable binary.

## Validation Path

Commands:

- `just fmt`
- `just fix -p codex-cli`
- `just fix -p codex-tui`
- `just fix -p codex-app-server`
- `just test -p codex-cli`
- `just test -p codex-tui`
- `just test -p codex-app-server`
- `just test -p codex-app-server-protocol`

Pass signal: release report contains artifact, smoke logs, rollback proof, known limitations, and all blocker evidence.

Fail signal: no usable binary, VSCode install/rollback cannot be verified, or focused tests/snapshots fail.

## Audit Focus

- Blocker vs should-have vs deferred is clear.
- A new user can install and rollback.
- Worker Separation evidence is complete for test/implement/audit.
