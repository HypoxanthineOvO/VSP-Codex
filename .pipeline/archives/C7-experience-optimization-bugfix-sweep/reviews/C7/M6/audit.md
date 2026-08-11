---
verdict: pass
cycle: C7
milestone: M6
title: Workflow Plan/SubPlan Live Surface
audit_type: read-only
---

# C7/M6 Audit Report

## 检查范围

已审计 M6 指定生产代码、测试/快照和证据文件。工作树中其它 Milestone 改动未纳入结论。审计期间未编辑文件、未更新 snapshot、未运行会修改文件的命令。

## 证据

- Plan panel：`plan_panel.rs` 中 Workflow split progress 仅在 `PlanState.id=C*`、标题 `Cycle N...`、root item id `M*` 全部满足时启用；当前 SubPlan 进度和 root Plan 进度分开渲染为 `1/5 | 5/8`。
- 普通 Plan 仍走 aggregate progress；现有 `plan_panel` snapshot 保留 `2/4`。
- 只展开 active root item 的 SubPlan；M7/M8 仅显示 root item 和 summary，不渲染未来 SubPlan 标题/内容。
- Workflow runtime guidance 明确要求 `update_plan` 在 phase changes、worker handoffs、validation boundaries、final response boundaries 刷新。
- Workflow slash routing 只接受 `/hw:` 和 `$hypo-workflow:`，负向测试覆盖 `/review`、`/compact`、`/model`、普通文本和空输入。
- `session/mod.rs` 的 model-visible permissions、Workflow guidance、user instructions directory 均使用 selected turn environment cwd，并 fallback 到 config cwd；未依赖 deprecated `turn_context.cwd` 构造模型可见提示。
- M6 状态记录完整：`.pipeline/state.yaml` 处于 C7/M6 `review_code`；`.pipeline/PROGRESS.md` 和 `.pipeline/log.yaml` 记录了 test、implement、test-fix、green validation、audit start。

## 验证结果

- PASS: `cargo test -p codex-tui workflow_cycle_plan_surface_shows_current_subplan_and_root_progress_separately -- --nocapture`
- PASS: `cargo test -p codex-core workflow_routing -- --nocapture`，9 passed
- PASS: `cargo test -p codex-tui workflow_slash_routing -- --nocapture`，8 passed
- PASS: `cargo test -p codex-tui plan_panel -- --nocapture`，5 passed
- PASS: `cargo fmt --check`，仅 stable rustfmt 对 unstable `imports_granularity` 的既有 warning
- PASS: `git diff --check`
- PASS: 未发现残留 `.snap.new`

## 发现

阻塞发现：无。

非阻塞观察：

- 测试不是单纯依赖 snapshot：`workflow_cycle_plan_surface...` 直接断言 split progress、当前 SubPlan 展开、未来 SubPlan 不泄露，并裁剪 snapshot 聚焦 Plan surface。
- `turn_context.cwd` deprecated warning 仍出现在 conversation-capture 路径，与用户标注一致，不影响 M6 的 model-visible prompt construction。
- `session_workflow_guidance_uses_selected_turn_environment_cwd_without_deprecated_allowance` 包含 source-text 断言，覆盖方式不如端到端多环境 prompt 捕获强；代码审计确认实际构造路径已正确使用 `model_visible_cwd`。

## 剩余风险

- Workflow split progress 依赖 PlanState 约定：`id=C*`、标题 `Cycle N...`、root id `M*`。若未来 builder 输出偏离该约定，会 fallback 到 aggregate progress；建议后续 builder 侧保持该契约或增加集中 helper。
- 完成态、无 SubPlan、无 items 未见 panic 风险；无 items 在 ChatWidget 层隐藏，完成态显示完成摘要。未来若要求 completed Workflow 仍只显示 root count，可补一条专门回归测试。
