# C9 M10 Test Worker Review

## Verdict

**BLOCKED / baseline not green.**

M10 core workflow/runtime and adapter preservation 的 baseline 不能进入实现合并的绿色状态：

- `workflow_routing` 通过，说明 Workflow runtime guidance 的文本路由合同目前仍被测试覆盖并通过。
- `plan_update_gate` filter 命中 3 个测试，但测试进程 stack overflow abort；这是 M10 直接相关 blocker，因为它覆盖 plan freshness / update_plan gate。
- `request_user_input` filter 命中 core lib 与 suite 相关测试，已有多项通过后同一进程 stack overflow abort；这是 M10 直接相关 blocker，因为它覆盖 Ask/request_user_input tool semantics。
- `codex-core-plugins` 编译和大多数测试通过，但 `remote_installed_marketplaces_skip_fetch_when_remote_plugin_disabled` 因 `AuthRequired` 失败；这是 M10 plugin/marketplace preservation blocker。
- `git diff --check` 通过。

因此 implement worker 开始前必须把上述 blocker 当作回归调查入口，不能仅以“部分测试已通过”或“filter 有命中”为通过依据。

## Required Context Read

- `.pipeline/prompts/c9-010-core-workflow-runtime-and-adapter-preservation.md`
  - M10 目标是合并 core runtime changes，同时保留 Hypo-Workflow routing、plan freshness gates、goals、worker separation、multi-agent boundaries、plugin behavior、OpenCode adapters。
  - 明确要求保留 strict worker separation、Codex `/hw:start` / `/hw:resume` subworker authorization、Workflow command routing、plan freshness、plugin/marketplace behavior 与 adapter alignment。
- `.pipeline/reports/C9/M7-vsp-equivalence-matrix.yaml`
  - M7 将 Hypo-Workflow routing/state-file contract、goals、plan freshness runtime、OpenCode/Hypo adapters 标为 preserve-by-default。
  - M7 将 plugin/marketplace/connector/skill-loader convergence 延后到 M10/M11，风险是丢失 plugin permissions 或破坏 remote MCP/plugin startup。
- `.pipeline/reports/C9/M9-protocol-app-server.md`
  - M9 已保留 VSP `ToolRequestUserInput`、native Ask UI、workflow markdown fallback、authorization-intent 与 `TurnPlanUpdated` structured plan state。
  - M10 不应重新打开 M9 protocol/app-server redesign，也不应手工覆盖 generated schema。
- `.pipeline/config.yaml`
  - `execution.worker_separation.mode: strict`
  - `execution.worker_separation.codex_execution_subworkers_authorized: true`
  - `authorized_scopes: [/hw:start, /hw:resume]`
  - `required_roles: [test, implement, audit]`

## Commands

```bash
cargo test --manifest-path codex-rs/Cargo.toml -p codex-core workflow_routing -- --nocapture
cargo test --manifest-path codex-rs/Cargo.toml -p codex-core plan_update_gate -- --nocapture
cargo test --manifest-path codex-rs/Cargo.toml -p codex-core request_user_input -- --nocapture
cargo test --manifest-path codex-rs/Cargo.toml -p codex-core-plugins
git diff --check
```

## Baseline Results

### workflow_routing

Result: **PASS**

Observed:

- `tests/all.rs` ran 14 matching tests.
- 14 passed, 0 failed.
- Matching tests covered native Hypo-Workflow runtime guidance, `/hw:plan` routing, quality/optimize guidance, required `request_user_input` wording, pre-card context, authorization-intent vs permission-grant separation, report-content final response requirements, and non-Workflow chat not being forced into Workflow Ask cards.

Warning:

- `codex-core` emitted existing dead-code warnings in `core/src/agent/control.rs` for unused helper functions. This is not a M10 routing failure by itself.

### plan_update_gate

Result: **BLOCKER**

Observed:

- Filter matched tests in `tests/all.rs`: `running 3 tests`.
- The process aborted with stack overflow:
  - `thread 'tokio-rt-worker' ... has overflowed its stack`
  - `fatal runtime error: stack overflow, aborting`
  - process exited with `signal: 6, SIGABRT`

M10 relevance:

- Directly related. M10 must preserve plan freshness and `update_plan` gate behavior.
- This is not a missing-filter warning; tests were selected and then the runtime aborted.
- Implement worker must identify whether the stack overflow is caused by current core session/tool/plan freshness interactions, test fixture recursion, or upstream merge skew before claiming M10 baseline recovery.

### request_user_input

Result: **BLOCKER**

Observed:

- Core lib test binary started 18 matching tests.
- The following displayed tests passed before abort, including request_user_input config, normalization, schema, tool description, MCP elicitation conversion, subagent rejection, and tool exposure tests.
- The process then aborted with stack overflow:
  - `thread 'tokio-rt-worker' ... has overflowed its stack`
  - `fatal runtime error: stack overflow, aborting`
  - process exited with `signal: 6, SIGABRT`

M10 relevance:

- Directly related. M10 must preserve request_user_input tool semantics and multi-agent/root-thread boundaries.
- M9 proved protocol/app-server Ask contracts; M10 still must prove core runtime request/response flow does not recurse or dead-end.
- Passing unit-level schema/normalization tests before abort is insufficient because the selected command failed overall.

### codex-core-plugins

Result: **BLOCKER**

Observed:

- Test binary ran 232 tests.
- 231 passed, 1 failed.
- Failure:
  - `manager::tests::remote_installed_marketplaces_skip_fetch_when_remote_plugin_disabled`
  - panic at `core-plugins/src/manager_tests.rs:770`
  - `called Result::unwrap() on an Err value: AuthRequired`

M10 relevance:

- Directly related. M10 owns core plugin/marketplace preservation and must not regress disabled-remote behavior.
- The test name indicates remote installed marketplaces should skip fetch when remote plugin is disabled; receiving `AuthRequired` suggests a fetch/auth path is still being touched.
- Implement worker should treat this as a plugin baseline failure, not environment noise, unless a repeatable pre-existing-auth explanation is proven with narrower evidence.

### git diff --check

Result: **PASS**

Observed:

- No whitespace or conflict-marker errors reported.

## Configuration And Adapter Preservation Checks

### Worker Separation

Status: **PASS by static config**

Evidence from `.pipeline/config.yaml`:

- Strict worker separation remains configured.
- Codex execution subworkers remain authorized.
- Authorization scopes are limited to `/hw:start` and `/hw:resume`.
- Required worker roles are `test`, `implement`, and `audit`.

Guard:

- Do not relax strict separation to make implementation easier.
- Do not broaden Codex execution subworker authorization beyond `/hw:start` and `/hw:resume` in M10.

### Core Workflow Routing / Plan Freshness / Request User Input

Status: **MIXED**

- Routing guidance tests pass.
- Plan freshness/update gate tests abort.
- Request user input runtime tests abort.

Guard:

- Preserve `workflow_context`, `workflow_files`, `workflow_routing`, `state/plan_freshness`, goals, and request_user_input root-thread semantics.
- Do not route Workflow commands as ordinary chat.
- Do not downgrade required Workflow decisions from `request_user_input`/Ask to prose-only prompts.
- Do not collapse authorization-intent into real command/file/permission grants.
- Do not remove visible pre-gate artifact/context requirements from plan phases.

### Core Plugins Baseline

Status: **BLOCKER**

- `codex-core-plugins` has one failing marketplace/remote plugin test.

Guard:

- Preserve VSP plugin permission, marketplace, remote sync, and disabled-remote behavior unless an official alpha.25 equivalent is proven with tests.
- Do not adopt upstream connector/plugin changes by replacing VSP marketplace behavior wholesale.
- Do not introduce auth-required remote calls when remote plugin paths are disabled.

### OpenCode / Hypo Adapter Stale Risk

Status: **WARNING / no obvious version skew, but adapter guard weakness observed**

Read-only observations:

- `.opencode/hypo-workflow.json`, `.opencode/plugins/hypo-workflow.ts`, and `.opencode/package.json` all report adapter version `12.8.0`.
- `.opencode/hypo-workflow.json` maps `/hw:start` and `/hw:resume` to `hw-build` with `route: pipeline`.
- `AGENTS.md` remains Hypo-Workflow managed and says `.pipeline/` is source of truth.
- `AGENTS.md` and `.opencode/commands/hw:plan:*` preserve the rule that `question`/Ask gates require visible artifact/context first.
- `.opencode/opencode.json` includes `question: allow`.
- `.opencode/runtime/hypo-workflow-hooks.js` currently returns informational allow decisions for file guard and says "OpenCode YOLO permissions are enabled; file guard is informational only."

Risk interpretation:

- No obvious stale version mismatch was found in the adapter metadata inspected.
- The informational-only file guard is a preservation risk if M10 changes adapter/runtime write paths or relies on adapter hooks to enforce protected-file safety. It is not a baseline test failure in this worker scope, but implement worker must not assume the OpenCode adapter itself prevents protected file mutation.

## Guardrails For Implement Worker

1. Treat the current baseline as red. Do not report M10 complete until the same required commands pass or each remaining failure has a documented, reproducible, explicitly accepted non-M10 cause.
2. Keep edits scoped to M10-owned surfaces: `codex-rs/core/`, `codex-rs/core-plugins/`, selected `.opencode/` adapter files only if runtime behavior requires it, and related tests.
3. Do not edit protocol/app-server generated schema for M10. M9 owns and already stabilized that contract.
4. Preserve strict worker separation and Codex `/hw:start` / `/hw:resume` subworker authorization exactly as configured.
5. Preserve Workflow native command routing and guidance:
   - `/hw:*` commands stay native Workflow intents.
   - Required decisions use `request_user_input`/Ask when available.
   - Ask cards remain gates after visible context, not substitutes for the context.
   - `update_plan`/plan freshness remains an execution discipline, not optional text.
6. Preserve request_user_input semantics:
   - Root-thread only for user questions.
   - Multi-agent/subagent boundaries remain enforced.
   - `autoResolutionMs`, `workflowDecisionType`, `intentOnly`, and `grantsApproval` semantics remain intact.
   - Authorization-intent must not grant real approval.
7. Preserve plugin/marketplace behavior:
   - Disabled remote plugin paths must not require auth or fetch.
   - VSP marketplace/cache/permission behavior should be adapted, not overwritten.
   - Official alpha.25 replacement claims require direct test evidence.
8. Preserve OpenCode/Hypo adapter alignment:
   - If core runtime behavior changes affect `/hw:start`, `/hw:resume`, Ask/question gates, plan state, or protected-file assumptions, adapter metadata/docs/hooks must be checked and updated in the same M10 scope.
   - Do not treat adapter files as passive docs; M7 classifies them as active integration surfaces.
9. Do not touch `.pipeline/state.yaml`, `.pipeline/cycle.yaml`, `.pipeline/log.yaml`, `.pipeline/PROGRESS.md`, prompts, manifests, reports, release artifacts, or unrelated source files from this test worker result.

## Risks

- Stack overflow in `plan_update_gate` may hide a recursion bug in plan freshness, session event handling, or test support. Because it aborts the process, it can mask additional failing tests.
- Stack overflow in `request_user_input` may indicate a runtime loop around pending Ask/tool resolution, token-count deferral, continuation handling, or test harness event polling. This directly threatens M10 request tool preservation.
- The plugin failure indicates disabled-remote marketplace behavior may still touch an auth-gated path. That would be user-visible in offline or unauthenticated startup flows.
- Adapter metadata looks aligned, but the OpenCode hook file guard is informational-only. Runtime safety must come from core Workflow discipline and explicit implementation boundaries, not assumed adapter enforcement.

## Suggested Revalidation After Implement Fixes

Run the original baseline again:

```bash
cargo test --manifest-path codex-rs/Cargo.toml -p codex-core workflow_routing -- --nocapture
cargo test --manifest-path codex-rs/Cargo.toml -p codex-core plan_update_gate -- --nocapture
cargo test --manifest-path codex-rs/Cargo.toml -p codex-core request_user_input -- --nocapture
cargo test --manifest-path codex-rs/Cargo.toml -p codex-core-plugins
git diff --check
```

If stack overflow persists, rerun the affected filters with `RUST_BACKTRACE=1` and then isolate individual tests under:

- `codex-rs/core/tests/suite/plan_update_gate.rs`
- `codex-rs/core/tests/suite/request_user_input.rs`
- `codex-rs/core/src/session/tests.rs` request_user_input-related cases
- `codex-rs/core-plugins/src/manager_tests.rs::remote_installed_marketplaces_skip_fetch_when_remote_plugin_disabled`
