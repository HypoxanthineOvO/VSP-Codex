# M5 Workflow Command And UX Polish

## Objective

在核心运行时稳定后，补齐 Workflow 命令、状态展示、文档和发布体验，让用户能直观看到 VSP-Codex 的 Workflow-native 行为。

## Source Of Truth

- P1 用户目标：Hypo-Workflow 成为系统内部能力，Slash 命令体验也应像内建命令。
- P2 技术路线：最后做命令/UX polish，避免 UI 先承诺尚未稳定的运行时能力。
- 代码证据：
  - Slash command registry: `codex-rs/tui/src/slash_command.rs`
  - Status/debug surfaces exist in TUI and command palette patterns.
  - Docs touched in previous releases: `README.md`, `docs/vsp-codex.md`, `docs/install.md`, `CHANGELOG.md`

## Subworker Assignment Plan

Status: authorized, strict worker separation.

- `test`
  - Scope: owns red/failing tests, fixtures, assertions, snapshots if needed, TUI slash/command snapshot tests, status rendering tests, and docs/release smoke checks.
  - Expected artifact: `.pipeline/reviews/C6/M5/test.md`
- `implement`
  - Scope: owns command descriptions/routing polish, status/debug display, docs, changelog, and release-note updates.
  - Must not create, edit, or rewrite test assets.
  - Expected artifact: `.pipeline/reviews/C6/M5/implement.md`
- `audit`
  - Scope: checks that UX/docs match actual runtime behavior, non-Workflow projects are not misleading, and release notes are honest.
  - Must reject fake tests that only update snapshots without proving behavior.
  - Expected artifact: `.pipeline/reviews/C6/M5/audit.md`

Lifecycle evidence required before completion: requested, started, completed/failed/blocked, and closed/close_failed for all roles.

Destructive operations and system-level dependency installation require user confirmation.

## Technical Solution

Add polish once M1-M4 are functional:

- first-class Workflow command descriptions and command palette routing
- status/debug output for Workflow mode, context pack source selection, Journal/Inbox/Mini-Cycle capture, and prompt-audit policy
- documentation explaining Workflow-native behavior and safety gates

## Technical Route

1. Inspect current slash command and command palette patterns.
2. Add or refine `/hw:*` discoverability and status/debug presentation.
3. Add status output indicating:
   - Workflow project detected
   - compact fallback pack enabled
   - ordinary capture enabled
   - Subagent prompt audit default enabled
4. Update docs and changelog.
5. Run focused UI/docs validation.
6. Write `.pipeline/reports/005-workflow-command-ux-polish.md`.

## Research Required

Implementation must inspect existing command palette and status rendering before editing to avoid inconsistent UI behavior.

## Risks And Alternatives

- Risk: docs claim behavior from incomplete earlier milestones. Only document verified behavior.
- Risk: non-Workflow users see irrelevant Workflow controls. Guard UI visibility or label inactive state clearly.

## Validation Path

Suggested commands:

- `just test -p codex-tui slash`
- `just test -p codex-tui command_palette`
- `just test -p codex-tui status`
- docs/release checks used by the current release flow
- `git diff --check`

Pass signal: Workflow-native behavior is discoverable, status is accurate, and docs match runtime implementation.

## Audit Focus

- UI/docs do not overpromise.
- Non-Workflow projects are not cluttered.
- Release notes include limitations and safety gates.
