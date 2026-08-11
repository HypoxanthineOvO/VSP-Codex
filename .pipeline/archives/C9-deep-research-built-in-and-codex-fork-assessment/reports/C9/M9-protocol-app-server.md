# C9 M9 Protocol/App-Server Contract Reconciliation

## 结论

M9 采用最小安全实现：不做破坏性 protocol/app-server 重写，不手工覆盖 generated schema，不降级 VSP Question Tool / Ask UI / workflow markdown / `TurnPlanUpdated` 语义。

当前 `codex-rs/app-server-protocol/` 已经以 Rust protocol 为源头，同时包含 VSP `request_user_input` 合同和可安全共存的 alpha.25 app-server additions。运行 schema fixture generator 后没有产生 schema diff，说明 Rust、JSON Schema、TypeScript 生成物处于 lockstep。M9 因此只新增本报告，产品源码保持现有安全状态。

## Changed Files

- `.pipeline/reports/C9/M9-protocol-app-server.md`：新增本 M9 实现报告。

未新增产品源码改动。工作树中已有的 `codex-rs/app-server/src/lib.rs` 差异属于 M8 SQLite logging convergence/hotfix 现有改动，本次未回退、未重写、未依赖其内部实现。

## Accepted Upstream-Compatible Additions

以下 alpha.25-compatible surfaces 已在当前 Rust protocol/app-server 中存在，并经 M9 验证确认可与 VSP contract 共存：

- Account rate-limit surfaces：`account/rateLimits/read`、`account/rateLimits/updated`、`GetAccountRateLimitsResponse`、`AccountRateLimitsUpdatedNotification`。
- External agent config migration surfaces：`externalAgentConfig/detect`、`externalAgentConfig/import`、`externalAgentConfig/import/completed` 及对应 v2 schema/TypeScript。
- Dynamic tool bridge：`thread/start.dynamicTools`、`item/tool/call`、`dynamicToolCall` thread item replay/serialization。
- Model reroute / verification notifications：`model/rerouted`、`model/verification`。
- Process/command execution protocol surfaces already present under Rust protocol and app-server processors.

这些内容均从 `codex-rs/app-server-protocol/src/` Rust 类型和 request/notification 宏导出；schema generator 本轮无 diff。

## Preserved VSP Semantics

- `ServerRequest::ToolRequestUserInput` 保留为 native Ask UI 请求，不被普通文本消息替代。
- `InitializeCapabilities.nativeAskUi` 与 `experimentalApi` 保持分离；官方 popup-capable clients 仍可通过兼容名单获得 native Ask UI。
- `ToolRequestUserInputQuestion` 保留 `workflowDecisionType`、`intentOnly`、`grantsApproval`。
- `ToolRequestUserInputParams` 保留 `autoResolutionMs`。
- 非 native Ask UI clients 保留 workflow markdown fallback，且 fallback 使用显式 workflow metadata，不把 reserved-looking question id 当成授权/Workflow 决策。
- authorization-intent 仍是 intent-only，不授予 command/file/permission approval；实际权限仍走原审批机制。
- 普通非 Workflow clarification 即使在 native Ask-capable client 下也保持 agent message，不升级为 Ask card。
- `TurnPlanUpdated` 保留 structured plan state，并继续区分 root plan 与 subplan progress。

## Deferred Alpha.25 Areas

以下内容不在 M9 做半截迁移，延后到后续 milestone：

- M10：core workflow/runtime、world-state/current-time/reminder/runtime replay、provider-aware fallback、MCP/cache/runtime lifecycle 等核心行为收敛。
- M10/M11：plugin、marketplace、connector、skill-loader 和 executor plugin policy 的深度融合。
- M11：TUI native card/snapshot/plan panel/visible workflow surfaces 的大范围 UI 合并。
- M11：CLI/VSCode/code-mode host IPC、exec-server lifecycle、VSCode Remote 行为收敛。
- M12：docs、CI、install、release tooling、package/version/release artifacts。

这些区域会影响 Workflow runtime、用户可见 UI 或 release surface；在 M9 内强行合并会超过 protocol/app-server bridge 的最小安全边界。

## Validation

已运行：

```bash
cargo run --manifest-path codex-rs/Cargo.toml -p codex-app-server-protocol --bin write_schema_fixtures
cargo test --manifest-path codex-rs/Cargo.toml -p codex-app-server-protocol
cargo test --manifest-path codex-rs/Cargo.toml -p codex-app-server --test all request_user_input -- --nocapture
cargo test --manifest-path codex-rs/Cargo.toml -p codex-app-server --test all workflow_markdown_surface -- --nocapture
git diff --check
```

结果：

- schema fixture generator：PASS，无 generated JSON/TypeScript diff。
- `codex-app-server-protocol`：PASS，240 unit tests + 2 schema fixture tests passed。
- `request_user_input` app-server tests：PASS，18 passed，616 filtered。
- `workflow_markdown_surface` app-server tests：PASS，2 passed，632 filtered。
- `git diff --check`：PASS。

观察到的 warning 为既有测试环境噪声：`codex-core` dead-code warnings、临时目录 trust warning、`mock-model` metadata warning、file watcher teardown warning。未发现 M9 contract failure。

## Risks And Follow-Up

- `ToolRequestUserInput` 当前仍标记为 experimental server request；VSP 依赖 native Ask UI capability 与 fallback 兼容逻辑维持安全行为，后续如果 upstream 提供正式等价 Ask API，需要用同等测试覆盖迁移。
- alpha.25 processor/runtime additions 已有一部分安全共存，但更深的 current-time/world-state/runtime replay 需要 M10 在 core runtime 层验证，不能只从 app-server schema 判断安全。
- TUI 和 app-server client 的可见交互仍需 M11 在 UI 层验证；M9 只证明 protocol/app-server bridge 没有降级 Ask/Plan 合同。
- 工作树存在 M8 SQLite hotfix 和 `.pipeline` 状态/证据改动；M9 未回退这些改动，后续合并时仍需保持它们与 app-server tracing setup 兼容。
