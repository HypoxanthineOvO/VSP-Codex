# M10 Plan 强制更新门禁

## Objective

解决长对话、上下文压缩、resume 和阶段切换后 LLM 忘记更新 Plan 的问题。引入指令层提醒和运行时 stale detection；当 Plan stale 时，继续执行前必须先调用 `update_plan`。

## Source Of Truth

- C3 P2 extension discussion on 2026-05-27.
- Depends on M5 structured Plan/SubPlan state and M6 persistent TUI Plan panel.
- M8/M9 Ask 修复已通过本机重启复验；M7 跨设备验收仍 pending。

## Subworker Assignment Plan

Status: authorized, strict worker separation.

- `test`
  - Owns failing/coverage tests for resume or compaction reminders, tool-burst stale detection, and phase-transition stale detection.
  - Must verify stale Plan gates require `update_plan`, while fresh Plan state does not repeatedly interrupt.
  - Must include focused TUI/app/core tests as appropriate and define manual TUI validation steps.
- `implement`
  - Owns instruction/context injection and runtime stale detection implementation.
  - Must not add user-editable Plan behavior; Plan remains read-only and LLM/tool-driven.
  - Must keep the gate narrowly scoped to avoid repeated nuisance triggers.
- `audit`
  - Checks that stale detection is deterministic, does not deadlock normal turns, preserves Plan/SubPlan source of truth, and does not rely on markdown/history parsing.
  - Must reject if the gate can loop forever, fire immediately after a fresh `update_plan`, or force users to edit Plan manually.

## Technical Solution

Add a two-layer Plan freshness mechanism:

1. Instruction/recovery layer:
   - After resume or context compaction, remind the model that any active Plan must be reviewed and refreshed before continuing substantial work.
   - The reminder must explicitly prefer calling `update_plan` when the visible/structured Plan may be stale.

2. Runtime stale gate:
   - Track enough lightweight Plan freshness metadata to detect stale state after:
     - resume or compaction;
     - consecutive tool execution bursts;
     - phase transitions such as analysis -> implementation, implementation -> tests, tests -> acceptance.
   - When stale and an active Plan exists, gate continuation on a new `update_plan` call.
   - When the Plan is fresh, do not show a reminder and do not force another update.

The TUI Plan panel remains read-only. The LLM owns the structured Plan and one-level SubPlan content through `update_plan`.

## Technical Route

1. Locate current Plan state ownership and update timestamps/versioning in core/protocol/TUI/app-server paths.
2. Identify resume/compaction context injection points and add a concise Plan freshness reminder.
3. Add a lightweight stale marker or version comparison for active Plan state.
4. Detect tool-burst and phase-transition cases without broad global blocking.
5. When stale, surface a model-facing gate that requires an `update_plan` call before meaningful continuation.
6. Clear or advance the stale marker after a valid `update_plan`.
7. Ensure Default idle collapsed panel and expanded Plan panel continue to render normally.
8. Add focused tests and manual validation notes.

## Constraints

- Do not add manual Plan editing.
- Do not reset user-visible Plan state during resume.
- Do not create a gate that blocks answering direct user questions unrelated to ongoing planned work.
- Do not grow large TUI orchestration files unnecessarily; prefer local modules/helpers.

## Validation Path

Automated:

- Focused tests for resume/compaction Plan reminder.
- Focused tests for stale gate after tool burst.
- Focused tests for stale gate on phase transition.
- Focused tests showing fresh Plan state does not trigger repeated gates.
- Relevant `codex-tui` tests if Plan panel state/render behavior changes.
- `just fmt`
- Scoped `just fix -p <changed-crates>`

Manual:

- In TUI, run a long multi-step task with Plan visible.
- Simulate resume or long-context continuation.
- Confirm the model refreshes Plan before continuing and the panel does not remain stale.

Pass signal:

- Stale Plan triggers one required `update_plan`.
- Fresh Plan does not repeatedly interrupt.
- TUI Plan panel remains read-only, visible, and coherent.

Fail signal:

- Gate loops forever.
- Gate triggers immediately after a fresh Plan update.
- User must manually edit Plan.
- Plan panel displays stale data after the model refreshes state.

## Completion Criteria

- M10 report is written to `.pipeline/reports/010-plan-update-gate.md`.
- Automated evidence and manual TUI validation steps are recorded.
- Any remaining cross-device acceptance dependency remains assigned to M7, not hidden in M10.
