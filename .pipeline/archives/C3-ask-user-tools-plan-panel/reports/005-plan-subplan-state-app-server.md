# M5 Report: Plan/SubPlan structured state 与 app-server v2 暴露

完成时间：2026-05-26T21:32:13+08:00

## 结果

已完成。现有 `update_plan` 已演进为兼容 legacy flat checklist 的结构化 Plan/SubPlan 工具；app-server v2 `turn/plan/updated` 可暴露 canonical structured state。

## 主要变更

- `codex-rs/protocol/src/plan_tool.rs`
  - 新增 `PlanState`、`PlanNode`、`SubPlan`、`PlanSubItem`。
  - `UpdatePlanArgs` 支持 legacy `plan` 与 structured `state`。
  - `normalize()` 在 legacy 与 structured 之间补齐兼容表示。
- `codex-rs/core/src/tools/handlers/plan.rs`
  - 解析后统一 normalize，拒绝空计划。
- `codex-rs/core/src/tools/handlers/plan_spec.rs`
  - tool schema 支持一层 SubPlan。
- `codex-rs/app-server-protocol/src/protocol/v2/turn.rs`
  - `TurnPlanUpdatedNotification.state` 作为 experimental 字段暴露结构化状态。
- `codex-rs/app-server/src/bespoke_event_handling.rs`
  - plan update notification 同步输出 structured state。

## 验证

- `cd codex-rs && just test -p codex-core update_plan`
- `cd codex-rs && just test -p codex-app-server-protocol turn_plan_updated_round_trip_structured_state`
- `cd codex-rs && just write-app-server-schema --experimental`
- `cd codex-rs && just test -p codex-app-server-protocol`

结果：通过。

## 风险

SubPlan 首版只支持 root item 下一层；更深层级后续再扩展。
