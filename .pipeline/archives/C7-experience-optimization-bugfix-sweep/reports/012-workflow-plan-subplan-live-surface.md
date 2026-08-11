# C7/M6 Workflow Plan/SubPlan Live Surface Report

## 结果

M6 已完成并通过只读审计。Workflow Cycle Plan surface 现在能把当前 SubPlan 进度与总 Plan 进度分开显示，例如 `1/5 | 5/8`；普通非 Workflow Plan 仍保持原有 aggregate progress 行为，例如 `2/4`。

## 已完成内容

- `codex-rs/tui/src/chatwidget/plan_panel.rs`
  - Workflow Cycle Plan 启用 split progress：当前 SubPlan 进度在前，root Milestone 进度在后。
  - 仅展开 active root item 的 SubPlan，未来 Milestone 只显示 root item 和 summary。
  - 普通 Plan 不进入 Workflow 特化路径，继续显示聚合计数。
- `codex-rs/core/src/workflow_routing.rs`
  - Workflow runtime guidance 要求在 phase changes、worker handoffs、validation boundaries、final response boundaries 刷新 `update_plan`。
- `codex-rs/core/src/session/mod.rs`
  - 模型可见的权限说明、Workflow guidance 和用户指令目录使用 selected turn environment cwd，并 fallback 到 config cwd。

## 验证

- PASS: `cargo test -p codex-tui workflow_cycle_plan_surface_shows_current_subplan_and_root_progress_separately -- --nocapture`
- PASS: `cargo test -p codex-core workflow_routing -- --nocapture`
- PASS: `cargo test -p codex-tui workflow_slash_routing -- --nocapture`
- PASS: `cargo test -p codex-tui plan_panel -- --nocapture`
- PASS: `cargo fmt --check`
- PASS: `git diff --check`

## 审计结论

`.pipeline/reviews/C7/M6/audit.md` verdict 为 `pass`，无阻塞发现。

剩余非阻塞风险：Workflow split progress 依赖 PlanState builder 输出 `id=C*`、标题 `Cycle N...`、root item id `M*` 的约定；如果后续 builder 改变格式，应补集中 helper 或 builder 侧测试。
