# M5 Plan/SubPlan structured state 与 app-server v2 暴露

## Objective

把现有 `update_plan` 演进为兼容 flat checklist 的结构化 Plan/SubPlan 工具，并通过 app-server v2 暴露 canonical state 给 TUI 和未来客户端。

## Source Of Truth

- P2: `.pipeline/plan-state/decompose.yaml` -> `M5`
- Key files:
  - `codex-rs/protocol/src/plan_tool.rs`
  - `codex-rs/core/src/tools/handlers/plan_spec.rs`
  - `codex-rs/core/src/tools/handlers/plan.rs`
  - `codex-rs/app-server-protocol/src/protocol/v2/turn.rs`
  - `codex-rs/app-server/src/bespoke_event_handling.rs`

## Subworker Assignment Plan

Status: authorized, strict worker separation.

- `test`
  - Owns legacy `update_plan` compatibility tests and structured Plan/SubPlan protocol/app-server tests.
  - Must verify one-level SubPlan and app-server exposure.
- `implement`
  - Owns protocol/core/app-server state changes.
  - Must evolve `update_plan`; must not add a disconnected parallel plan tool.
- `audit`
  - Checks source-of-truth boundaries, legacy compatibility, one-level SubPlan limit, and future-client app-server exposure.
  - Must reject if panel state is derived by parsing proposed-plan markdown.

## Technical Solution

Use `update_plan` as the single model-facing plan update tool. Add structured fields for plan id/title, root items, stable ids, summaries, status, and one-level SubPlan. Keep legacy flat checklist input and normalize it into structured state.

Expose structured Plan/SubPlan state through app-server v2 as an experimental notification or read surface. This state is canonical for the persistent panel.

## Technical Route

1. Extend `codex-rs/protocol/src/plan_tool.rs`.
   - Add `PlanState`, `PlanNode`, `SubPlan`.
   - Keep legacy `UpdatePlanArgs` compatibility.
2. Extend `plan_spec.rs`.
   - Support structured fields.
   - Preserve old `plan: [{ step, status }]`.
3. Update `plan.rs`.
   - Normalize legacy and structured args.
   - Emit structured plan event/state.
   - Preserve existing history update where useful.
4. Add app-server v2 structured Plan/SubPlan payload.
   - Follow app-server v2 rules.
   - Mark new surface experimental if required.
5. Store thread-level current Plan snapshot for resume and M6.

## Research Status

Resolved. Current facts:

- `plan_tool.rs` only has `StepStatus`, `PlanItemArg`, `UpdatePlanArgs`.
- `plan.rs` sends `EventMsg::PlanUpdate`.
- app-server `TurnPlanUpdatedNotification` is a flat list of steps.

## Risks And Alternatives

- Risk: proposed-plan markdown and structured state diverge. Mitigation: structured state is canonical; markdown is proposal/history only.
- Risk: old callers break. Mitigation: preserve legacy shape.
- Rejected alternative: new plan_state tool. User asked to transform old `update_plan`.

## Validation Path

- `just write-app-server-schema --experimental`
- `just test -p codex-app-server-protocol`
- `just test -p codex-core update_plan`

Pass signal:

- Legacy flat `update_plan` still works.
- Structured root plan + one-level SubPlan round-trips.
- app-server exposes structured state.

## Completion Criteria

- Canonical Plan/SubPlan state exists.
- Future clients can consume state via app-server v2.
- M5 report is written to `.pipeline/reports/005-plan-subplan-state-app-server.md`.
