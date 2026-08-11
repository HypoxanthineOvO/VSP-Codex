# C9 M9 Test Worker Report - Protocol/App-Server Contract Reconciliation

## Verdict

当前 baseline 为 **PASS**。M9 的验证边界应以 `codex-rs/app-server-protocol/` 为 contract root，并把 `codex-rs/app-server/`、`codex-rs/app-server-client/` 以及 `codex-rs/app-server/tests/suite/v2/` 作为桥接行为验证面。现有 protocol unit tests、schema fixture drift checks、`request_user_input` app-server 契约测试、workflow markdown surface 测试均通过。

当前未发现与 M9 直接相关的 baseline fail。测试输出中的 `codex-core` dead-code warnings、临时目录 trust warning、mock model warning、file watcher unwatch warning 是既有测试环境噪声，不构成 protocol/app-server contract 失败。

## Risk Surface

- `codex-rs/app-server-protocol/`：M9 的 contract root。需要保留 VSP `ToolRequestUserInput`、`ToolRequestUserInputQuestion` 的 `workflowDecisionType` / `intentOnly` / `grantsApproval` 字段、`autoResolutionMs`、`TurnPlanUpdated`/structured plan state，以及 `nativeAskUi` capability 与 `experimentalApi` 的分离语义。
- `codex-rs/app-server/`：桥接风险集中在 request processors、`bespoke_event_handling.rs`、`thread_state.rs`、transport delivery filtering、`request_user_input.rs`、`request_user_input_fallback.rs`、`workflow_markdown_surface.rs`。实现必须保持 native Ask UI、有 fallback 的非 native clients、pending ask resume replay、`serverRequest/resolved` 顺序。
- `codex-rs/app-server-client/`：初始化 capability 必须继续传递 `native_ask_ui`；client event handling 不能丢失 `ServerRequest::ToolRequestUserInput` 或把它降级为普通 notification。
- 相关 tests：`codex-rs/app-server-protocol/tests/schema_fixtures.rs`、`codex-rs/app-server/tests/suite/v2/request_user_input.rs`、`codex-rs/app-server/tests/suite/v2/workflow_markdown_surface.rs` 是 M9 最小回归边界。

## Commands

```bash
cargo test --manifest-path codex-rs/Cargo.toml -p codex-app-server-protocol
cargo test --manifest-path codex-rs/Cargo.toml -p codex-app-server --test all request_user_input -- --nocapture
cargo test --manifest-path codex-rs/Cargo.toml -p codex-app-server --test all workflow_markdown_surface -- --nocapture
```

Schema generation assessment:

- `codex-rs/app-server-protocol/tests/schema_fixtures.rs` is safe for this worker: TypeScript fixtures are generated in memory, JSON fixtures are generated under a temp dir, then compared with vendored fixtures.
- `cargo run --manifest-path codex-rs/Cargo.toml -p codex-app-server-protocol --bin write_schema_fixtures` was **not executed** because it writes to `codex-rs/app-server-protocol/schema` by default, outside this worker's allowed write scope.
- Implement stage must run schema regeneration after Rust protocol reconciliation, then verify generated JSON/TypeScript diffs are intentional.

## Baseline Results

| Check | Result | Notes |
| --- | --- | --- |
| `codex-app-server-protocol` tests | PASS | 240 unit tests passed; 2 schema fixture drift tests passed. Covers protocol serialization, experimental filtering, schema generation consistency, `ToolRequestUserInput`, and `TurnPlanUpdated`. |
| `codex-app-server --test all request_user_input` | PASS | 18 focused tests passed; 616 filtered. Covers native Ask UI round trip, readable fallback, official popup-capable client compatibility, pending ask resume replay, visible context before Ask, workflow metadata, reserved id rejection, and non-workflow clarification behavior. |
| `codex-app-server --test all workflow_markdown_surface` | PASS | 2 focused tests passed; 632 filtered. Covers plugin-readable fallback markdown and separation of subplan/root progress counts. |

Observed warnings are not M9 blockers:

- `codex-core` unused/dead-code warnings in `core/src/agent/control.rs`.
- temporary `Project-local config... disabled until the project is trusted` warnings from isolated test homes.
- `Unknown model mock-model` warnings from mock-model test fixtures.
- `codex_file_watcher` unwatch warnings during test teardown.

## Required Implementation Guardrails

1. Reconcile Rust protocol first, then regenerate derived schema artifacts. Do not manually edit generated JSON/TypeScript as the source of truth.
2. Preserve VSP Question Tool semantics unless alpha.25 has exact equivalent behavior with matching tests. Required preserved behavior:
   - `ServerRequest::ToolRequestUserInput` native Ask UI path remains available for clients with `nativeAskUi` or official popup-capable names.
   - non-native clients receive readable workflow markdown fallback and can answer through ordinary turn input.
   - `autoResolutionMs` survives protocol/app-server round trip.
   - `workflowDecisionType`, `intentOnly`, and `grantsApproval` survive native Ask UI and fallback surfaces.
   - authorization-intent cards remain intent-only and must not grant command, file, or permission approval.
   - reserved-looking question ids do not become workflow decisions without explicit metadata.
   - ordinary non-workflow clarification remains an agent message, not a native Ask card.
3. Preserve plan update behavior:
   - `TurnPlanUpdated` notification and structured plan state must remain serializable.
   - workflow markdown must keep subplan and root progress counts separate.
   - app-server bridge must not collapse plan update into TUI-only snapshot behavior.
4. App-server/client sync is required:
   - `app-server-protocol`, `app-server`, and `app-server-client` must compile and agree on request/notification shapes.
   - transport/backpressure filtering must not silently drop native Ask requests for supported clients without resolving/rejecting them.
5. Implement worker minimum modification boundary:
   - in scope: `codex-rs/app-server-protocol/`, `codex-rs/app-server-client/`, `codex-rs/app-server/`, and related tests.
   - avoid broad TUI snapshot churn in M9 unless strictly required for compile.
   - avoid unrelated core workflow/runtime, release, packaging, plugin marketplace, manifest, and report/state changes.

## Pseudo-Test Rejection Standards

- Reject any M9 implementation that only edits schema files or generated TypeScript/JSON without running protocol and app-server contract tests.
- Reject any implementation that treats TUI snapshots as a substitute for `codex-app-server-protocol` tests plus `request_user_input`/`workflow_markdown_surface` app-server tests.
- Reject any implementation that passes compile by deleting VSP Ask/Workflow metadata fields, hiding them behind `experimentalApi` only, or converting required workflow decisions into plain assistant text for native Ask-capable clients.
- Reject any implementation that regenerates schema artifacts before the Rust protocol contract is reconciled, or leaves generated artifacts out of lockstep after protocol changes.

## Risks

- Alpha.25 protocol additions may introduce legitimate schema churn, but blind overwrite can remove VSP `ToolRequestUserInput` and workflow metadata semantics.
- Request processor refactors can accidentally change delivery order: `serverRequest/resolved` must still arrive before `turn/completed` for resolved Ask requests.
- Client compatibility logic is subtle: official VS Code/ChatGPT clients can receive native Ask UI even without declared capability, while unknown clients must opt in explicitly.
- Fallback markdown is user-visible for plugin/non-native clients; leaking raw protocol names such as `ToolRequestUserInputParams` or `workflowDecisionType` into that surface is a regression.
- Current worktree contains unrelated existing `.pipeline` and SQLite/M8 source changes; M9 implementation should not revert or depend on those unrelated modifications.
