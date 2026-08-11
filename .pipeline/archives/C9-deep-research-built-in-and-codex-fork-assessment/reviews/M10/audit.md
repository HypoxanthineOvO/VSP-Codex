# C9 M10 Audit Worker Review

审计时间：2026-06-26T16:51:47+08:00

## Verdict

**SUPERSEDED: initial verdict was FAIL / BLOCK M10 acceptance.**

后续 persisted-goals follow-up 与完整验证已解除该阻塞。保留本节作为历史审计记录；最终复核见文末 “Follow-up Audit Recheck”。

M10 不能通过。当前变更修复了部分显性 blocker，但还没有满足 `.pipeline/prompts/c9-010-core-workflow-runtime-and-adapter-preservation.md` 的核心验收条件：Goals runtime 只做了 core-local 最小实现，未保留既有 state-db / ext goal runtime 的持久化、用量核算、budget/usage 状态、resume rehydrate、外部 mutation 协调和服务 API 语义；同时必跑验证 `cargo test --manifest-path codex-rs/Cargo.toml -p codex-core request_user_input -- --nocapture` 仍失败。M10 应阻断，不应以窄测通过声明完成。

## Scope Reviewed

- M10 prompt：`.pipeline/prompts/c9-010-core-workflow-runtime-and-adapter-preservation.md`
- Test worker evidence：`.pipeline/reviews/C9/M10/test.md`
- Implement worker report：`.pipeline/reports/C9/M10-core-adapters.md`
- Diff scope：`codex-rs/core/`、`codex-rs/core-plugins/`
- Untracked source：`codex-rs/core/src/tools/handlers/goal.rs`、`codex-rs/core/src/tools/handlers/goal_spec.rs`

## Findings

### Critical: core-local Goals handler is a semantic regression versus existing state-db / ext goal runtime

References:

- `codex-rs/core/src/tools/handlers/goal.rs:37`
- `codex-rs/core/src/tools/handlers/goal.rs:113`
- `codex-rs/core/src/tools/handlers/goal.rs:132`
- `codex-rs/core/src/tools/handlers/goal.rs:183`
- `codex-rs/core/src/session/session.rs:495`
- `codex-rs/core/src/goals.rs:1`
- `codex-rs/core/src/goals.rs:422`
- `codex-rs/core/src/goals.rs:582`
- `codex-rs/core/src/goals.rs:1000`
- `codex-rs/ext/goal/tests/goal_extension_backend.rs:404`
- `codex-rs/ext/goal/tests/goal_extension_backend.rs:1007`
- `codex-rs/ext/goal/tests/goal_extension_backend.rs:1053`

Evidence:

- 新增 `CoreGoalRuntime` 仅保存 `Arc<Mutex<Option<ThreadGoal>>>` 与 continuation flag，存储位置是 `thread_extension_data`，不是 state-db。
- `create_goal` 创建的 `ThreadGoal` 将 `tokens_used` 和 `time_used_seconds` 固定为 0，后续 `update_goal` 只改 status 和 updated_at，没有调用 state-db goal accounting。
- 新增 continuation loop 只等待 session idle 后注入 `<goal_context>` user message；它不接入既有 `goal_runtime_apply` 生命周期事件，也不处理 budget limit、usage limit、turn error、external mutation、thread stop/resume rehydrate、metrics 或 thread preview。
- 既有 `core/src/goals.rs` 明确是 “Core support for persisted thread goals”，并通过 state-db 读写、accounting、preview 和 runtime hooks 维护完整目标状态。
- `ext/goal` 测试证明既有语义包括 token budget 后 BudgetLimited、resume 后 wall-clock accounting、GoalService set/get/clear 与 metadata preview。当前 core-local handler 没有等价覆盖。

Impact:

- 目标在进程/线程生命周期外不可恢复，resume 后 active goal 会丢失。
- budgeted goal 的最终 usage report 不可信，因为 `tokens_used`/`time_used_seconds` 不会累计。
- 外部 goal 服务、state-db thread_goals、目标 preview、usage/budget limited 状态和 metrics 可能与 core tool 返回的 goal 分裂。
- M10 要求 “Preserve goals” 和 “core workflow runtime” 语义；这不是可接受的最小替代，只是让 `create_goal` 不再返回 unsupported。

Required next action:

- 不要以当前 core-local handler 作为 M10 完成实现。
- 恢复或重新接入 state-db/ext goal runtime 作为 `create_goal`/`update_goal` 的实际后端，或提供明确的 alpha.25 官方替代证据与等价测试。
- 至少补齐并通过覆盖持久化、usage/budget accounting、resume rehydrate、external mutation、GoalService/API、pending `request_user_input` 不抢跑 continuation 的测试。

### Critical: required `request_user_input` validation still fails

References:

- `.pipeline/prompts/c9-010-core-workflow-runtime-and-adapter-preservation.md`
- `.pipeline/reports/C9/M10-core-adapters.md`
- `.pipeline/reviews/C9/M10/test.md`

Evidence:

- Required validation 包含 `cargo test --manifest-path codex-rs/Cargo.toml -p codex-core request_user_input -- --nocapture`。
- Implement report 记录：该命令在 lib 内 18 个匹配测试通过后，进入 `tests/all.rs` 仍发生 `tokio-rt-worker` stack overflow abort。
- 这已经不是原先 `create_goal unsupported`，而是剩余 suite-level stack overflow。

Impact:

- M10 要求保留 `request_user_input` tool semantics、multi-agent/root-thread boundaries 和 workflow decision gates。全过滤器失败意味着该验收面没有闭合。
- stack overflow abort 会中断进程并可能掩盖后续失败，不能被视为可忽略噪声。

Required next action:

- 隔离 `tests/all.rs` 中触发 `request_user_input` stack overflow 的具体测试和调用路径。
- 修复或证明是纯测试 harness 限制，并让 required command 本身通过，或取得显式接受的非 M10 blocker 豁免。

### Warning: stack-size harness changes are acceptable only as scoped test harness fixes, not as product proof

References:

- `codex-rs/core/tests/suite/plan_update_gate.rs:27`
- `codex-rs/core/tests/suite/request_user_input.rs:39`
- `codex-rs/core/src/session/tests.rs:178`

Evidence:

- 相关测试被改为在 16 MiB OS thread stack 和 Tokio `thread_stack_size` 下执行。
- 原断言主体没有被删除或明显放宽；implement report 也记录 `RUST_MIN_STACK=16777216` 可让相关窄测通过。
- 这支持 “部分重型 async session tests 在默认 test runtime stack 下溢出” 的解释。

Impact:

- 对 `plan_update_gate` 的 3 个测试和若干 request_user_input round-trip 窄测而言，当前改法可以作为测试 harness 修复接受。
- 但由于完整 `request_user_input` 过滤器仍在 `tests/all.rs` stack overflow，不能据此证明不存在产品递归或全套测试 harness 问题。

Required next action:

- 将大栈 helper 收敛为共享测试工具以减少重复，或保留局部 helper 但记录理由。
- 继续处理 `tests/all.rs` stack overflow；在该命令变绿前，M10 仍阻断。

### Info: core-plugins disabled remote fix preserves intended disabled behavior

References:

- `codex-rs/core-plugins/src/manager.rs:763`
- `codex-rs/core-plugins/src/manager.rs:770`
- `codex-rs/core-plugins/src/manager_tests.rs:749`

Evidence:

- `build_and_cache_remote_installed_plugin_marketplaces` 现在在 `!config.remote_plugin_enabled` 时提前返回 `Ok(Vec::new())`。
- 同一模块已有多个路径在 remote disabled 时跳过 remote behavior；新增 guard 与现有 pattern 一致。
- Implement report 记录窄测 `remote_installed_marketplaces_skip_fetch_when_remote_plugin_disabled` 已通过。

Impact:

- 修复保留了 disabled remote plugin path 的 “不 auth、不 fetch” 行为。
- 未看到该变更破坏本地插件、visible marketplace 分组或 remote enabled path；remote enabled 分支仍会继续 fetch。

Required next action:

- 仍需重新运行完整 `cargo test --manifest-path codex-rs/Cargo.toml -p codex-core-plugins`，因为 implement report只记录了失败测试的窄测通过，不是完整插件包最终绿色。

## Validation Reviewed

Reviewed pass evidence:

- `workflow_routing`：PASS，14 matching tests passed。
- `plan_update_gate`：修复后 PASS，3 matching tests passed。
- `request_user_input_round_trip_resolves_pending`：PASS。
- `pending_request_user_input_does_not_spawn_extra_goal_continuation`：goal handler follow-up 后 PASS。
- `active_goal_continuation_runs_again_after_no_tool_turn`：PASS。
- `remote_installed_marketplaces_skip_fetch_when_remote_plugin_disabled`：PASS。
- `git diff --check`：PASS。

Reviewed fail / incomplete evidence:

- `cargo test --manifest-path codex-rs/Cargo.toml -p codex-core request_user_input -- --nocapture`：FAIL；18 lib tests pass 后 `tests/all.rs` stack overflow abort。
- `cargo test --manifest-path codex-rs/Cargo.toml -p codex-core-plugins`：implement report 未提供修复后的完整包级 PASS，只提供 disabled remote 窄测 PASS。

## Required Next Actions Before M10 Can Pass

1. Replace the core-local in-memory Goals implementation with a state-db/ext-backed implementation, or document and test a proven official equivalent that preserves persistence, accounting, budget/usage limit, resume, external mutation, preview, events, and metrics semantics.
2. Make the required `request_user_input` validation command pass end to end, including `tests/all.rs`, or obtain explicit acceptance of a documented non-M10 blocker.
3. Re-run the full M10 validation sequence:

```bash
cargo test --manifest-path codex-rs/Cargo.toml -p codex-core workflow_routing -- --nocapture
cargo test --manifest-path codex-rs/Cargo.toml -p codex-core plan_update_gate -- --nocapture
cargo test --manifest-path codex-rs/Cargo.toml -p codex-core request_user_input -- --nocapture
cargo test --manifest-path codex-rs/Cargo.toml -p codex-core-plugins
git diff --check
```

4. Update `.pipeline/reports/C9/M10-core-adapters.md` with final validation results and explicit goal-runtime preservation rationale.

## Completion Report

- 改动摘要：完成只读审计并写入本报告；未修改源码。
- 技术思路：按 M10 prompt 的 preserve-by-default contract，对 goal runtime、test harness、plugin disabled remote path 和 required validation evidence 做风险分级。
- 修改文件/模块：仅新增 `.pipeline/reviews/C9/M10/audit.md`。审阅范围包括 `codex-rs/core/`、`codex-rs/core-plugins/`、M10 prompt、test review 和 implement report。
- 测试设计：未运行新测试；审计复核既有 worker 记录、git diff、源码引用和测试覆盖语义。
- 验证结果：Critical 2、Warning 1、Info 1。结论为 M10 阻断。
- 预期结果：修复后 Goals runtime 应与 state-db/ext runtime 语义等价，required validation 全绿，plugin disabled remote path 保持不 auth、不 fetch。
- 遇到的问题：`/home/heyx/Hypo-Workflow/skills/audit/references/completion-report-contract.md` 不存在；按 audit SKILL.md 内列出的完成报告字段执行。
- 风险/后续：若继续保留当前 core-local Goals handler，后续用户可见目标状态、预算用量、resume 和外部 goal API 会出现分裂；若忽略 `request_user_input` full-filter stack overflow，M10 可能在 Ask/root-thread workflow 上留下未定位 abort 风险。

## Follow-up Audit Recheck

复核时间：2026-06-26T17:14:35+08:00

### Verdict

**PASS with conditions.**

此前阻断 M10 的核心问题已经解除：`create_goal` / `update_goal` 不再使用 in-memory `CoreGoalRuntime`，而是分别调用 `Session::create_thread_goal` 和 `Session::set_thread_goal`，后端走 `core/src/goals.rs` 的 state-db persisted goals runtime。

### 审计依据

- `codex-rs/core/src/tools/handlers/goal.rs`：`create_goal` 调用 `session.create_thread_goal(...)`，`update_goal` 调用 `session.set_thread_goal(...)`。
- `codex-rs/core/src/lib.rs`：已重新接入 `mod goals;`。
- `codex-rs/core/src/session/session.rs`：`Session` 持有 `GoalRuntimeState`。
- `codex-rs/core/src/tasks/lifecycle.rs`、`codex-rs/core/src/tasks/mod.rs`、`codex-rs/core/src/tools/lifecycle.rs`：`TurnStarted`、`ToolCompleted`、`TurnFinished`、`MaybeContinueIfIdle` 已接入 `goal_runtime_apply`。
- `codex-rs/core/src/goals.rs`：create/set/get/accounting 均使用 `self.thread_id` + state-db `thread_goals()`；idle continuation 会先检查 active turn、pending input、trigger mailbox 和 state-db current active goal。
- `codex-rs/core/src/session/tests.rs`：覆盖普通 idle goal continuation 会继续运行，以及 pending `request_user_input` 时不会启动额外 continuation。
- `codex-rs/core-plugins/src/manager.rs`：`remote_plugin_enabled=false` 时提前 `Ok(Vec::new())`，保留 no fetch / no auth。

### 验证复核

主编排已运行完整 M10 validation sequence，并全部通过：

```bash
cargo test --manifest-path codex-rs/Cargo.toml -p codex-core workflow_routing -- --nocapture
cargo test --manifest-path codex-rs/Cargo.toml -p codex-core plan_update_gate -- --nocapture
cargo test --manifest-path codex-rs/Cargo.toml -p codex-core request_user_input -- --nocapture
cargo test --manifest-path codex-rs/Cargo.toml -p codex-core-plugins
git diff --check
```

另补跑并通过：

```bash
cargo test --manifest-path codex-rs/Cargo.toml -p codex-core pending_request_user_input_does_not_spawn_extra_goal_continuation -- --nocapture
cargo test --manifest-path codex-rs/Cargo.toml -p codex-core active_goal_continuation_runs_again_after_no_tool_turn -- --nocapture
```

### 剩余风险

- `GoalRuntimeEvent::{UsageLimitReached, ExternalMutationStarting, ExternalSet, ExternalClear, ThreadResumed}` 当前仍有 dead-code warning；external mutation / resume / usage-limit lifecycle 接入可能仍非完整 alpha.25 等价。
- 工作区中 `codex-rs/app-server/src/lib.rs`、`codex-rs/tui/src/lib.rs`、`codex-rs/state/*` 改动归属 M8 SQLite logging hotfix，不应计入 M10 goal follow-up。
- 测试源中的大栈 harness 改动源于 M10 baseline stack overflow 修复；报告中已说明其为测试运行环境修复，未放宽产品断言。

### 建议

M10 可以接受为 **core workflow runtime/adapters preservation pass with follow-up risks**。不要把该结论表述为 full alpha.25 core/runtime merge；后续 M11/M12 继续处理 TUI/CLI/VSCode/release surfaces，external goal lifecycle 的完整等价性应作为后续 technical debt 跟踪。
