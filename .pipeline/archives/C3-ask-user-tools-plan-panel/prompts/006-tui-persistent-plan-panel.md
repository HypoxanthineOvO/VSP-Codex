# M6 TUI 常驻 Plan/SubPlan 面板

## Objective

在 TUI 输入框上方右侧常驻显示 structured Plan/SubPlan state，Default idle 折叠为一行摘要并支持快捷键展开。

## Source Of Truth

- P2: `.pipeline/plan-state/decompose.yaml` -> `M6`
- Depends on M5 structured PlanState.
- Key files:
  - New TUI Plan panel module near bottom pane/chat state.
  - `codex-rs/tui/src/chatwidget/protocol.rs`
  - `codex-rs/tui/src/bottom_pane/mod.rs`
  - TUI keymap/help hint modules.

## Subworker Assignment Plan

Status: authorized, strict worker separation.

- `test`
  - Owns snapshots for no plan, root-only, one-level SubPlan, Default idle collapsed, shortcut-expanded, narrow layout, and Ask overlay coexistence.
  - Must verify panel state sync after structured plan update.
- `implement`
  - Owns TUI state/render/layout/key handling.
  - Must keep panel read-only and avoid central `chatwidget.rs` growth where possible.
- `audit`
  - Checks layout overlap, composer height, source-of-truth, and no manual edit.
  - Must reject if panel parses markdown/history cell for canonical state.

## Technical Solution

Add a dedicated Plan panel module consuming M5 structured PlanState. The panel is visible during ordinary work; in Default idle it collapses to a one-line summary with current root item, active SubPlan title if present, and completed/total progress. A shortcut expands details.

The panel is visual only; editing remains model/tool-driven through `update_plan`.

## Technical Route

1. Create Plan panel state/render module.
2. Wire structured plan notification/state into TUI.
3. Insert panel above composer/right side when width allows.
4. Define responsive breakpoints and max heights.
5. Add shortcut to expand collapsed Default idle state.
6. Ensure Ask overlay and other bottom pane surfaces do not overlap.
7. Add snapshots and state-sync tests.

## Research Status

Resolved. Current plan UI is history-cell based in `history_cell/plans.rs`; C2 added similar TUI panel work for Subagent cards, so avoid growing `chatwidget.rs`.

## Risks And Alternatives

- Risk: panel crowds composer. Mitigation: responsive constraints and idle collapsed mode.
- Risk: Ask overlay overlap. Mitigation: overlay priority or panel fallback/hide.
- Risk: shortcut conflicts. Mitigation: use keymap/help patterns and snapshot hints.

## Validation Path

- `just test -p codex-tui`
- `cargo insta pending-snapshots -p codex-tui`

Pass signal:

- Panel renders correctly across wide/narrow/idle/expanded.
- Shortcut expands details.
- Ask overlay and composer remain usable.

## Completion Criteria

- Persistent read-only Plan/SubPlan panel works.
- Default idle collapsed summary and shortcut expansion work.
- M6 report is written to `.pipeline/reports/006-tui-persistent-plan-panel.md`.
