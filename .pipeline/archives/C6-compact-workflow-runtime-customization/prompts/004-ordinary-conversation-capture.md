# M4 Ordinary Conversation Capture

## Objective

让普通 Codex 对话也轻量记录进 Workflow 文件：默认 Journal + Inbox，长主题自动升级 Mini-Cycle，并让这些记录参与 Compact 空返回恢复。

## Source Of Truth

- P1 用户目标：随便发个对话，后面也能整理归档。
- 用户确认：默认 Journal + Inbox；长主题自动升级 Mini-Cycle。
- P2 技术路线：普通对话捕获是被动记录层，不是新的执行 runner。

## Subworker Assignment Plan

Status: authorized, strict worker separation.

- `test`
  - Scope: owns red/failing tests, fixtures, assertions, snapshots if needed, Journal append, Inbox extraction, long-topic Mini-Cycle upgrade, duplicate prevention on resume/replay, and Compact fallback inclusion.
  - Expected artifact: `.pipeline/reviews/C6/M4/test.md`
- `implement`
  - Scope: owns conversation capture schema, turn completion integration, extraction policy, Mini-Cycle threshold, and fallback pack integration.
  - Must not create, edit, or rewrite test assets.
  - Expected artifact: `.pipeline/reviews/C6/M4/implement.md`
- `audit`
  - Scope: checks privacy/redaction, bounded summaries, duplicate prevention, and separation from formal active Cycle state.
  - Must reject fake tests that only call pure helper functions without exercising turn completion or file writes.
  - Expected artifact: `.pipeline/reviews/C6/M4/audit.md`

Lifecycle evidence required before completion: requested, started, completed/failed/blocked, and closed/close_failed for all roles.

Destructive operations and system-level dependency installation require user confirmation.

## Technical Solution

Add ordinary conversation capture under `.pipeline/`:

- `.pipeline/chat/journal.yaml`: append-only concise turn summaries.
- `.pipeline/inbox/items.yaml`: actionable or reusable items extracted from conversation.
- `.pipeline/chats/<id>/state.yaml`: Mini-Cycle state for long ordinary topics.

This layer should not turn every ordinary chat into a formal Cycle. Journal is the baseline, Inbox is the organizing queue, and Mini-Cycle is automatic only when topic continuity crosses a threshold.

## Technical Route

1. Define tolerant schemas for Journal, Inbox, and Mini-Cycle state.
2. Choose a turn completion integration point that records completed assistant output, not partial/interrupted output.
3. Add bounded summarization/extraction policy; avoid raw full transcript persistence by default.
4. Implement duplicate prevention for resume/replay.
5. Implement long-topic upgrade threshold, such as repeated topic continuity across N turns or runtime topic marker.
6. Connect recent Journal/Inbox/Mini-Cycle data into M1 fallback context pack.
7. Write `.pipeline/reports/004-ordinary-conversation-capture.md`.

## Research Required

Implementation must inspect session turn completion and persistence flow to pick the right hook point. Avoid recording subagent/internal turns as ordinary root conversation unless explicitly intended.

## Risks And Alternatives

- Risk: noisy Inbox. Keep extraction conservative and include status fields.
- Risk: secrets copied into Workflow files. Apply redaction and bounded summaries.
- Risk: Mini-Cycle mutates formal Cycle unexpectedly. Store Mini-Cycle separately under `.pipeline/chats/`.
- Rejected alternative: Mini-Cycle for every chat, because it is too heavy for casual usage.

## Validation Path

Real tests must include:

- root ordinary chat appends Journal.
- actionable content creates Inbox item.
- repeated long topic auto-upgrades to Mini-Cycle.
- resume/replay does not duplicate entries.
- Compact fallback includes recent ordinary conversation capture.

Suggested commands:

- `just test -p codex-core conversation`
- `just test -p codex-core compact`
- `git diff --check`

Pass signal: ordinary chat can be recovered, organized, and compact-fallback-visible without requiring formal Workflow commands.

## Audit Focus

- Redaction and bounded summaries.
- No duplicate writes.
- Formal Cycle state remains protected.
- Subagent/internal turns are classified correctly.
