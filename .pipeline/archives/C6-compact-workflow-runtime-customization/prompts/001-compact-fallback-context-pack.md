# M1 Compact Fallback Context Pack

## Objective

实现 Compact 返回空、只返回不可用历史、或过滤后几乎无上下文时的 Workflow 兜底恢复包，确保下一轮像没有压缩坏掉一样继续工作。

## Source Of Truth

- P1 用户目标：优先解决 Compact 空返回；恢复材料优先级为 Architecture/项目摘要、全局/当前目标、最近 N 条对话、state/cycle、Plan Prompt、compact logs。
- P2 技术路线：在 core/session 和 compact 路径中引入 `WorkflowContextPack`，先解决实际恢复，不依赖 hook 输出注入。
- 代码证据：
  - 本地 Compact：`codex-rs/core/src/compact.rs`
  - Remote Compact v1：`codex-rs/core/src/compact_remote.rs`
  - Remote Compact v2：`codex-rs/core/src/compact_remote_v2.rs`
  - 历史安装：`Session::replace_compacted_history`
  - 初始上下文：`Session::build_initial_context`

## Subworker Assignment Plan

Status: authorized, strict worker separation.

- `test`
  - Scope: owns failing tests and fixtures for empty/near-empty compact output, Workflow context pack ordering, path safety, and budget truncation.
  - Must create or update test assets before implementation changes.
  - Expected artifact: `.pipeline/reviews/C6/M1/test.md`
- `implement`
  - Scope: owns production code for Workflow context pack generation and local/remote compact fallback integration.
  - Must not create, edit, or rewrite tests owned by `test`.
  - Expected artifact: `.pipeline/reviews/C6/M1/implement.md`
- `audit`
  - Scope: reviews final diff, compact fallback behavior, test evidence, path safety, and Worker Separation evidence.
  - Must reject fake tests that only assert helper strings without exercising compact history installation or fallback conditions.
  - Expected artifact: `.pipeline/reviews/C6/M1/audit.md`

Lifecycle evidence required before completion: each role must be recorded as requested, started, completed/failed/blocked, and closed/close_failed.

Destructive operations and system-level dependency installation require user confirmation.

## Technical Solution

Add a core-owned `WorkflowContextPack` builder that detects a `.pipeline/` project and emits bounded model-visible fallback context when compact output is empty or unusable. The pack is recovery context, not a new source of truth. It should read only from project-local files and render a concise, labeled context block in the priority order requested by the user.

Initial source order:

1. `.pipeline/architecture/*.md` and `PROJECT-SUMMARY.md`
2. active goal/current user goal when available
3. recent N dialogue turns or rollout-derived recent summaries
4. `.pipeline/state.yaml` and `.pipeline/cycle.yaml`
5. active plan prompt and current milestone prompt
6. `.pipeline/PROGRESS.compact.md`, `.pipeline/log.compact.yaml`, `.pipeline/knowledge/knowledge.compact.md`

## Technical Route

1. Inspect current compact tests to reuse existing session/turn harnesses.
2. Add a small module near context/session code, for example `codex-rs/core/src/workflow_context.rs`.
3. Implement project-root-scoped file selection with missing-file tolerance and no traversal outside cwd/project root.
4. Implement source budgets and deterministic truncation; keep pack small enough to preserve compaction benefits.
5. Detect unusable compact output in local compact:
   - empty assistant summary suffix
   - summary-only fallback that would become `(no summary available)`
   - replacement history with no useful non-context content
6. Detect unusable output in remote v1/v2 after `should_keep_compacted_history_item` filtering.
7. Inject fallback context before `replace_compacted_history` installs replacement history.
8. Preserve existing `SessionStartSource::Compact` behavior so post-compact SessionStart hooks still run.
9. Write `.pipeline/reports/001-compact-fallback-context-pack.md` with files changed, tests, and remaining risks.

## Research Required

Implementation must determine the exact unusable-output predicate from current compact data shapes. Do not cross into P3/P4 assumptions by treating all short summaries as failures.

## Risks And Alternatives

- Risk: fallback pack is too large. Use strict budgets.
- Risk: stale Workflow files override live context. Label content as recovery context and keep current turn context authoritative.
- Risk: remote compact returns only markers. Treat marker-only replacement as fallback-worthy.
- Rejected alternative: only use PostCompact hook, because current PostCompact schema does not inject additional model context.

## Validation Path

Real tests must include:

- local compact empty summary fallback
- remote v1/v2 empty or marker-only filtered history fallback
- temporary `.pipeline/` fixture containing architecture, state, cycle, prompt, and compact files
- path safety test proving files outside project root are ignored
- budget/truncation test

Suggested commands:

- `just test -p codex-core compact`
- `just test -p codex-core compact_remote`
- `git diff --check`

Pass signal: compact replacement history is non-empty, contains labeled Workflow fallback context in priority order, and the next turn can build model-visible input.

Fail signal: empty Compact can still leave the next turn without current Workflow continuity, or fallback reads outside project root.

## Audit Focus

- No out-of-root reads.
- Bounded output.
- Empty Compact failure mode is covered by tests, not just helper unit tests.
- Existing non-Workflow compaction behavior is unchanged unless `.pipeline/` is detected.
