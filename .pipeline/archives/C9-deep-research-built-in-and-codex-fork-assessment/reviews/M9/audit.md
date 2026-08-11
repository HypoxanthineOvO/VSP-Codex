# C9 M9 Audit Worker Report - Protocol/App-Server Contract Reconciliation

## Verdict

**PASS with deferred follow-up.**

M9 当前结果足以通过本 milestone：它没有尝试宣称完成 full alpha.25 protocol/app-server merge，而是把本 milestone 收敛为 protocol/app-server contract 安全核对、schema lockstep 验证、VSP Ask/Workflow surface 保留确认，并把大范围 runtime/UI/release/plugin 差异显式延期到 M10-M12。

审计未发现 M9 范围内的 Critical blocker。当前证据支持以下判断：

- `codex-rs/app-server-protocol/` 仍是 contract root；Rust protocol 类型是 source of truth。
- vendored JSON/TypeScript schema fixture 与从 Rust 生成的输出一致；未发现手工改 generated schema 的迹象。
- VSP `request_user_input` / Question Tool / workflow markdown / `TurnPlanUpdated` / native Ask UI / fallback / Workflow metadata 语义仍保留并有 focused tests 覆盖。
- M9 报告没有虚假声称 full alpha.25 merge 完成；报告明确列出 Deferred Alpha.25 Areas。
- 未发现 M9 实现越界改 TUI/core/CLI/release/plugin 或 schema generated artifacts。本审计可见的审计范围内唯一源码 diff 是 `codex-rs/app-server/src/lib.rs` tracing log filter 调整，和 M9 报告描述的 M8 SQLite logging hotfix 一致，不是 protocol contract/schema 迁移。

## Reviewed Refs

- `.pipeline/prompts/c9-009-protocol-contract-and-app-server-bridge-merge.md`
- `.pipeline/reviews/C9/M9/test.md`
- `.pipeline/reports/C9/M9-protocol-app-server.md`
- `codex-rs/app-server-protocol/`
- `codex-rs/app-server-client/`
- `codex-rs/app-server/`
- `codex-rs/app-server-test-client/`

## Commands Run

```bash
git status --short
git diff --name-only
git diff --check
rg -n "ToolRequestUserInput|request_user_input|ToolRequestUserInputQuestion|workflowDecisionType|intentOnly|grantsApproval|autoResolutionMs|nativeAskUi|TurnPlanUpdated|workflow_markdown_surface|Question Tool|request_user_input_fallback" codex-rs/app-server-protocol codex-rs/app-server-client codex-rs/app-server codex-rs/app-server-test-client .pipeline/prompts/c9-009-protocol-contract-and-app-server-bridge-merge.md .pipeline/reviews/C9/M9/test.md .pipeline/reports/C9/M9-protocol-app-server.md
git diff -- codex-rs/app-server-protocol codex-rs/app-server-client codex-rs/app-server codex-rs/app-server-test-client
cargo test --manifest-path codex-rs/Cargo.toml -p codex-app-server-protocol schema_fixtures -- --nocapture
rg -n "full alpha\\.25|alpha\\.25.*complete|complete.*alpha\\.25|defer|Deferred|延期|M10|M11|M12|generated schema|write_schema_fixtures|schema fixture|schema diff|手工" .pipeline/reports/C9/M9-protocol-app-server.md .pipeline/reviews/C9/M9/test.md .pipeline/prompts/c9-009-protocol-contract-and-app-server-bridge-merge.md codex-rs/app-server-protocol codex-rs/app-server-client codex-rs/app-server codex-rs/app-server-test-client
git diff --name-only -- codex-rs/app-server-protocol codex-rs/app-server-client codex-rs/app-server codex-rs/app-server-test-client codex-rs/tui codex-rs/core codex-rs/cli codex-rs/execpolicy codex-rs/vscode .pipeline/reports .pipeline/reviews/C9/M9
```

Validation results:

- `git diff --check`: PASS.
- protocol schema fixture tests: PASS, including `typescript_schema_fixtures_match_generated` and `json_schema_fixtures_match_generated`.
- focused `rg` checks found the required VSP Ask/Workflow symbols and tests in protocol/app-server/client surfaces.

## Findings

### Info - Rust protocol remains the source of truth

Evidence:

- `codex-rs/app-server-protocol/src/protocol/common.rs` still declares `ToolRequestUserInput => "item/tool/requestUserInput"` and `TurnPlanUpdated => "turn/plan/updated"`.
- `codex-rs/app-server-protocol/src/protocol/v2/item.rs` still defines `ToolRequestUserInputQuestion`, `ToolRequestUserInputParams`, `ToolRequestUserInputResponse`, and the VSP metadata fields.
- `cargo test --manifest-path codex-rs/Cargo.toml -p codex-app-server-protocol schema_fixtures -- --nocapture` passed the vendored JSON/TypeScript fixture comparison against generated output.

Assessment: PASS. The schema state is consistent with Rust source, and there is no observed generated-schema drift.

### Info - VSP Question Tool and Workflow metadata are preserved

Evidence:

- Protocol fields remain present: `workflowDecisionType`, `intentOnly`, `grantsApproval`, and `autoResolutionMs`.
- Client capability remains present: `InitializeCapabilities.nativeAskUi`.
- App-server bridge code still contains native Ask UI delivery and fallback handling through `bespoke_event_handling.rs`, `transport.rs`, `request_user_input.rs`, `request_user_input_fallback.rs`, `thread_state.rs`, and request processors.
- Focused tests remain in `codex-rs/app-server/tests/suite/v2/request_user_input.rs` and `codex-rs/app-server/tests/suite/v2/workflow_markdown_surface.rs`.
- `.pipeline/reviews/C9/M9/test.md` reports PASS for protocol tests, `request_user_input`, and `workflow_markdown_surface`.

Assessment: PASS. The M9 final state preserves the VSP surfaces required by the prompt.

### Info - M9 does not overclaim full alpha.25 merge

Evidence:

- `.pipeline/reports/C9/M9-protocol-app-server.md` explicitly states M9 is a minimal safe implementation.
- The report lists Deferred Alpha.25 Areas for M10, M11, and M12, including core workflow/runtime, plugin/marketplace/connector, TUI native card/snapshot/plan panel, CLI/VSCode/host IPC, docs/CI/install/release tooling.

Assessment: PASS. The report frames M9 as contract reconciliation and preservation, not as full alpha.25 completion.

### Warning - M9 success depends on accepting a narrowed milestone interpretation

Evidence:

- The prompt's Technical Route originally included updating Rust protocol definitions, regenerating fixtures, and adapting processors/clients.
- The actual M9 report says no product source was newly modified in M9 because the current baseline already contains compatible protocol/app-server surfaces and generated schema is clean.

Assessment: Acceptable for this milestone only because the report is explicit about deferral and the validation evidence supports a no-op product change. This should be tracked as deferred/needs_followup, not treated as complete alpha.25 reconciliation.

### Info - No M9 scope breach found

Evidence:

- `git diff --name-only -- codex-rs/app-server-protocol codex-rs/app-server-client codex-rs/app-server codex-rs/app-server-test-client codex-rs/tui codex-rs/core codex-rs/cli codex-rs/execpolicy codex-rs/vscode .pipeline/reports .pipeline/reviews/C9/M9` showed `codex-rs/app-server/src/lib.rs` and `codex-rs/tui/src/lib.rs` as existing diffs in the broader workspace, but `git diff -- codex-rs/app-server-protocol codex-rs/app-server-client codex-rs/app-server codex-rs/app-server-test-client` showed only the app-server logging filter change.
- No diff was observed under `codex-rs/app-server-protocol/`, `codex-rs/app-server-client/`, or `codex-rs/app-server-test-client/`.
- No M9 generated schema file diff was observed.

Assessment: PASS for M9. The visible `codex-rs/tui/src/lib.rs` workspace diff remains outside M9's reported implementation and should not be attributed to M9 without separate review.

## Risks

- `ToolRequestUserInput` remains experimental in protocol filtering. That is acceptable for VSP compatibility only while `nativeAskUi` and fallback behavior remain tested.
- The schema fixture test verifies generated schema consistency, but it does not prove every alpha.25 runtime behavior has been merged. M9 correctly limits that claim.
- app-server/client compatibility depends on continuing to preserve official popup-capable client compatibility and native Ask UI delivery filtering.
- Existing unrelated workspace diffs in `.pipeline`, `codex-rs/state`, `codex-rs/tui`, and `codex-rs/app-server/src/lib.rs` need owner-aware handling during later milestones; this audit did not review or validate those changes beyond M9 scope.

## Required Follow-Up

1. Mark M9 as **passed with deferred alpha.25 follow-up**, not as full protocol/app-server alpha.25 merge completion.
2. Carry the Deferred Alpha.25 Areas from `.pipeline/reports/C9/M9-protocol-app-server.md` into M10-M12 acceptance criteria.
3. In M10, validate core workflow/runtime additions such as world-state/current-time/reminder/runtime replay/provider fallback at the core boundary instead of inferring safety from app-server schema.
4. In M11, validate TUI/native card/snapshot/plan panel and VSCode/CLI host behavior with focused user-surface tests.
5. Before final C9 release preparation, rerun the full M9 validation set:

```bash
cargo run --manifest-path codex-rs/Cargo.toml -p codex-app-server-protocol --bin write_schema_fixtures
cargo test --manifest-path codex-rs/Cargo.toml -p codex-app-server-protocol
cargo test --manifest-path codex-rs/Cargo.toml -p codex-app-server --test all request_user_input -- --nocapture
cargo test --manifest-path codex-rs/Cargo.toml -p codex-app-server --test all workflow_markdown_surface -- --nocapture
git diff --check
```
