# C9 M10 Core Workflow Runtime And Adapter Preservation

## 结论

M10 已完成，结论为 **PASS with follow-up risks**。

本里程碑修复了 baseline 阻塞项，并保留了 VSP 的 Workflow/runtime 关键语义：

- `plan_update_gate` 和 `request_user_input` 的 stack overflow 归因为重型 async session tests 的默认 worker stack 不足，已用局部大栈测试 harness 修复，未放宽断言。
- `request_user_input` + goal continuation 中暴露的 `create_goal` unsupported 已修复，且最终实现使用 state-db backed persisted goals runtime，不再使用早期被审计否决的 in-memory `CoreGoalRuntime`。
- `remote_plugin_enabled=false` 时的 remote installed marketplace path 已恢复为 no fetch / no auth，`codex-core-plugins` 全包测试通过。
- Workflow routing、plan freshness、request_user_input root-thread/Ask 语义、goal idle continuation、plugin disabled-remote 行为均通过聚焦验证。

M10 不是 full alpha.25 core/runtime merge。alpha.25 仍有若干 core、external goal mutation/resume/usage-limit lifecycle、UI/CLI/release 差异继续由 M11/M12 或后续 Cycle 处理。

## 技术处理

### Test Harness

`codex-rs/core/tests/suite/plan_update_gate.rs`、`codex-rs/core/tests/suite/request_user_input.rs` 和相关重型 `session` tests 改为在 16 MiB OS thread stack + Tokio runtime 下运行。原测试主体和断言保留，用于避免默认 `tokio-rt-worker` stack overflow 掩盖真实 runtime 结果。

### Persisted Goals Runtime

`codex-rs/core/src/goals.rs` 重新作为 core persisted goals runtime 接入：

- `Session` 持有 `GoalRuntimeState`。
- `create_goal` / `update_goal` handler 调用 `Session::create_thread_goal` / `Session::set_thread_goal`。
- goals runtime 使用 `state_db.thread_goals()` 持久化 objective/status/budget/accounting，并通过 `ThreadGoalUpdated` 事件报告状态。
- goal continuation 使用 hidden internal context 注入，并在启动前检查 active turn、pending input、trigger mailbox 和 state-db 当前 active goal。

### Lifecycle Wiring

新增/恢复以下生命周期接线：

- `TurnStarted`：在 turn start lifecycle 中初始化 goal accounting baseline。
- `ToolCompleted`：在 tool finish success 后做 goal progress accounting；`update_goal` 走 suppressed accounting。
- `TurnFinished`：在 turn complete 处做 turn-end accounting 并清理 continuation turn state。
- `TaskAborted`：在 abort lifecycle 中清理/结算 goal turn state。
- `MaybeContinueIfIdle`：active turn 清空后触发 idle goal continuation 检查；pending `request_user_input` 保持 active turn，因此不会抢跑额外 continuation。

### Core Plugins

`codex-rs/core-plugins/src/manager.rs` 在 `!config.remote_plugin_enabled` 时提前返回空 remote installed marketplaces，避免 disabled remote path 触发 auth/fetch。

## 修改范围

M10 相关源码/测试变更集中在：

- `codex-rs/core/src/goals.rs`
- `codex-rs/core/src/lib.rs`
- `codex-rs/core/src/session/session.rs`
- `codex-rs/core/src/session/tests.rs`
- `codex-rs/core/src/tasks/lifecycle.rs`
- `codex-rs/core/src/tasks/mod.rs`
- `codex-rs/core/src/tools/lifecycle.rs`
- `codex-rs/core/src/tools/handlers/goal.rs`
- `codex-rs/core/src/tools/handlers/goal_spec.rs`
- `codex-rs/core/src/tools/handlers/mod.rs`
- `codex-rs/core/src/tools/spec_plan.rs`
- `codex-rs/core/tests/suite/mcp_turn_metadata.rs`
- `codex-rs/core/tests/suite/plan_update_gate.rs`
- `codex-rs/core/tests/suite/request_user_input.rs`
- `codex-rs/core-plugins/src/manager.rs`

当前工作区中 `codex-rs/state/src/*`、`codex-rs/app-server/src/lib.rs` 和 `codex-rs/tui/src/lib.rs` 的改动归属 M8 SQLite logging hotfix，不是 M10 goal follow-up；M10 没有重新打开 M9 protocol/app-server redesign、M11 TUI/CLI surface 或 M12 release packaging。

## 验证结果

已运行并通过：

```bash
cargo test --manifest-path codex-rs/Cargo.toml -p codex-core pending_request_user_input_does_not_spawn_extra_goal_continuation -- --nocapture
cargo test --manifest-path codex-rs/Cargo.toml -p codex-core active_goal_continuation_runs_again_after_no_tool_turn -- --nocapture
cargo test --manifest-path codex-rs/Cargo.toml -p codex-core workflow_routing -- --nocapture
cargo test --manifest-path codex-rs/Cargo.toml -p codex-core plan_update_gate -- --nocapture
cargo test --manifest-path codex-rs/Cargo.toml -p codex-core request_user_input -- --nocapture
cargo test --manifest-path codex-rs/Cargo.toml -p codex-core-plugins
git diff --check
```

结果摘要：

- goal/request_user_input guard：2 个关键 session tests 均通过。
- `workflow_routing`：14 passed。
- `plan_update_gate`：3 passed。
- `request_user_input`：lib 18 passed，`tests/all.rs` 10 passed，总体命令通过。
- `codex-core-plugins`：232 passed。
- `git diff --check`：通过。

## 审计复核

初始 audit 阻塞点有效：早期 core-local/in-memory goal runtime 不满足 M10 preserve goals 要求。后续已替换为 state-db persisted runtime 并接入生命周期。

只读复核结论为 **PASS with conditions**：

- 已确认 `create_goal` / `update_goal` 使用 persisted backend。
- 已确认 `CoreGoalRuntime`、`continuation_running`、`core_goal_wait_until_idle` 等临时回退实现不再存在。
- 已确认 `TurnStarted`、`ToolCompleted`、`TurnFinished`、`MaybeContinueIfIdle` 接入 core lifecycle。
- 已确认 plugin disabled-remote no fetch/no auth 行为恢复。

剩余条件/风险已在本报告记录：external mutation / resume / usage-limit lifecycle variants 仍显示未构造 warning，属于后续完整 alpha.25 convergence 风险，不阻塞本 M10 聚焦验收。

## 风险与后续

- `GoalRuntimeEvent::{UsageLimitReached, ExternalMutationStarting, ExternalSet, ExternalClear, ThreadResumed}` 当前仍有 dead-code warning；这提示 external goal service、resume rehydrate、usage-limit 等更深集成仍需后续审计/合并，不应宣称 alpha.25 goal runtime 全量等价。
- 大栈测试 harness 是针对重型 async test 的窄修复；如果后续继续出现 stack overflow，应抽成共享测试工具并定位是否有新的产品递归。
- `.opencode` adapter 本轮只做只读检查，未发现明显 version skew；其 file guard 仍是 informational-only，protected-file 安全仍依赖 Workflow 纪律和主代理状态写入。
- 未执行 push、tag、release、release artifact refresh 或远端副作用。
