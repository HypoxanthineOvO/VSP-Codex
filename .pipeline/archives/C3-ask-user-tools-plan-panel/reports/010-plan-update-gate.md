# M10 Plan 强制更新门禁报告

## 结果

已完成。M10 增加了 core 侧 Plan freshness 状态与模型可见提醒，覆盖 active Plan 在 resume、fork、compact 和连续工具调用后可能 stale 的场景。提醒只注入一次，新的 `update_plan` 会清除 stale 状态，避免重复打扰或死循环。

## 实现摘要

- 新增 `PlanFreshness` session 状态，记录 active Plan、stale 原因、是否已提醒、以及自上次 Plan 更新后的工具调用数。
- `PlanUpdate` 事件会刷新 active Plan 并清除 stale gate。
- resume/fork 从 rollout 中恢复最近的 Plan：
  - 优先使用持久化的 `PlanUpdate` 事件；
  - 同时支持从历史中的 `update_plan` function call 参数恢复，解决真实 resume 场景下事件不在模型历史里的问题。
- compact 后标记 active Plan stale，并在下一次模型上下文注入提醒。
- 连续 3 个非 `update_plan` 工具调用后标记 active Plan stale。
- 新增 `<plan_freshness_reminder>` developer context，明确要求继续实质性工作前 review active Plan 并调用 `update_plan`。

## 修改文件

- `codex-rs/core/src/context/plan_freshness_reminder.rs`
- `codex-rs/core/src/context/mod.rs`
- `codex-rs/core/src/state/plan_freshness.rs`
- `codex-rs/core/src/state/mod.rs`
- `codex-rs/core/src/state/session.rs`
- `codex-rs/core/src/session/mod.rs`
- `codex-rs/core/src/tools/router.rs`
- `codex-rs/core/src/session/tests.rs`
- `codex-rs/core/tests/suite/plan_update_gate.rs`
- `codex-rs/core/tests/suite/mod.rs`

## 验证

- `cd codex-rs && just fmt`：通过。
- `cd codex-rs && just test -p codex-core record_context_updates_injects_plan_freshness_reminder_once_when_stale fresh_plan_update_clears_plan_freshness_reminder tool_burst_marks_active_plan_stale_until_update_plan`：3 passed。
- `cd codex-rs && just test -p codex-core plan_update_gate`：3 passed。
  - `resume_with_active_plan_injects_update_plan_reminder`
  - `compaction_with_active_plan_injects_update_plan_reminder_on_next_turn`
  - `fresh_update_plan_followup_does_not_repeat_plan_reminder`
- `cd codex-rs && just fix -p codex-core`：通过。

按仓库规则，`just fix` 后未重复跑测试。

## 剩余风险

- M10 未实现“阶段切换”专用业务枚举，因为当前 core 没有稳定的通用阶段语义可安全挂钩；M10 通过 resume/compact/tool-burst 覆盖最容易导致 Plan stale 的运行时路径。
- 这不是硬性阻止 assistant 输出的拦截器，而是模型可见的强提醒 gate。这样避免普通问答被死锁，也符合“用户不能手动编辑 Plan，LLM 通过 `update_plan` 负责刷新”的要求。
- M7 的真实跨设备 pending Ask resume 验收仍未完成，不能隐藏到 M10 里。
