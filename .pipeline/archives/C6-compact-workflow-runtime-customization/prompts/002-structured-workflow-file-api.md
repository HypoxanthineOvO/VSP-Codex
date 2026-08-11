# M2 Structured Workflow File API

## Objective

提供系统化 Workflow 文件读写能力，减少手写 YAML、状态更新、日志追加、Question/Plan 桥接和 Subagent 生命周期记录中的格式错误和遗漏。

## Source Of Truth

- P1 用户目标：各种读写不要出现 YAML 格式化错误；交互确认尽量用 Request Questions；计划纪律用 Plan；Subagent 生命周期需要系统化记录。
- P2 技术路线：优先做结构化 YAML 写入器、State/Progress/Log API、Question/Plan bridge、Subagent lifecycle API；Lock/并发暂不优先。
- 代码证据：
  - `serde_yaml` 已在 workspace 依赖中存在。
  - 工具 handler 模式：`codex-rs/core/src/tools/handlers/plan.rs`、`request_user_input.rs`
  - 工具注册：`codex-rs/core/src/tools/spec_plan.rs`

## Subworker Assignment Plan

Status: authorized, strict worker separation.

- `test`
  - Scope: owns red/failing tests, fixtures, assertions, snapshots if needed, YAML parse/write validation, protected file guards, no partial writes, state/log/progress updates, and lifecycle event append.
  - Expected artifact: `.pipeline/reviews/C6/M2/test.md`
- `implement`
  - Scope: owns production API/tool implementation and integration with tool registry where needed.
  - Must not create, edit, or rewrite `test` assets.
  - Expected artifact: `.pipeline/reviews/C6/M2/implement.md`
- `audit`
  - Scope: checks file safety, schema tolerance, atomicity, prompt surface, and evidence quality.
  - Must reject fake tests that only serialize in-memory YAML without touching temp files.
  - Expected artifact: `.pipeline/reviews/C6/M2/audit.md`

Lifecycle evidence required before completion: requested, started, completed/failed/blocked, and closed/close_failed for all roles.

Destructive operations and system-level dependency installation require user confirmation.

## Technical Solution

Create a small Workflow runtime/file module that exposes safe operations for project-local `.pipeline/` files:

- parse and validate YAML before write
- protected-file-aware updates for `.pipeline/state.yaml`, `.pipeline/cycle.yaml`, `.pipeline/rules.yaml`
- append helpers for `.pipeline/log.yaml` and `.pipeline/PROGRESS.md`
- typed helper for Subagent lifecycle events
- policy/helper mapping Workflow hard gates to `request_user_input`
- policy/helper mapping visible planning state to `update_plan`

Expose model-visible tools only if they are compact and safer than shell/apply_patch edits; otherwise keep APIs internal for Workflow routing first.

## Technical Route

1. Inspect current `.pipeline` files and archive variants to define tolerant typed structs.
2. Add a module such as `codex-rs/core/src/workflow_files.rs` or scoped equivalent.
3. Implement project root detection and `.pipeline/` path guard.
4. Implement YAML parse-modify-validate-write with temp file swap where files are rewritten.
5. Implement append-only helpers for logs/progress where practical.
6. Implement lifecycle event API for `requested`, `started`, `completed`, `failed`, `blocked`, `closed`, `close_failed`.
7. Decide whether to register a model-visible tool in `spec_plan.rs`; if exposed, keep schema narrow and deny arbitrary path writes.
8. Write `.pipeline/reports/002-structured-workflow-file-api.md`.

## Research Required

Implementation must inspect historical `.pipeline/state.yaml`, `cycle.yaml`, and `log.yaml` shapes in archives before choosing strict struct fields. Older files must not break read/update paths unnecessarily.

## Risks And Alternatives

- Risk: YAML rewrite churns formatting. Prefer narrow updates and append-only events.
- Risk: too many tools increase model confusion. Start internal or expose one grouped tool.
- Risk: protected files become too hard to update. Allow explicit safe operations with validation.
- Rejected alternative: continue shell `sed`/manual YAML edits, because it does not satisfy the user’s reliability goal.

## Validation Path

Real tests must include:

- malformed YAML rejected with no partial write
- heartbeat/current step update
- log event append
- PROGRESS timeline append
- Subagent lifecycle event sequence
- protected file guard denies unsupported write shape

Suggested commands:

- `just test -p codex-core workflow`
- `just test -p codex-core tools`
- `git diff --check`

Pass signal: Workflow file operations succeed through structured APIs and invalid YAML cannot corrupt files.

## Audit Focus

- No arbitrary filesystem writes.
- Protected files have explicit allowed operations.
- Atomicity or no-partial-write behavior is demonstrated.
- Tool schema, if exposed, is small and hard to misuse.
