# C6 P3 Generate

Timestamp: 2026-05-29T00:03:47+08:00
Status: generated_pending_prompt_audit

## Generated Prompts

- `.pipeline/prompts/001-compact-fallback-context-pack.md`
- `.pipeline/prompts/002-structured-workflow-file-api.md`
- `.pipeline/prompts/003-native-workflow-runtime-routing.md`
- `.pipeline/prompts/004-ordinary-conversation-capture.md`
- `.pipeline/prompts/005-workflow-command-ux-polish.md`

## Execution Policy

- Worker Separation: strict.
- Codex execution subworkers: authorized for `/hw:start` and `/hw:resume`.
- Each implementation prompt assigns exactly three worker roles: `test`, `implement`, and `audit`.
- `test` owns red tests, fixtures, assertions, snapshots, and final verification evidence.
- `implement` owns production edits only and must not create, edit, rewrite, or satisfy `test` assets.
- `audit` checks final diff, evidence quality, worker identity separation, and acceptance risks.
- Main agent orchestrates, integrates, writes lifecycle evidence, and makes final decisions; it must not satisfy both implementation and verification roles.

## Prompt Audit Default

Starting with C6 P3, every generated Workflow execution prompt set must receive a prompt-audit pass by Subagent before P4 confirmation. The prompt-audit Subagent checks:

- P2 technical fields are preserved, not compressed into vague goals.
- Worker Separation is explicit and role overlap is forbidden.
- Real validation methods and pass/fail signals are present.
- Prompt order matches the accepted delivery priority.
- Destructive operations and system-level dependency installation remain confirmation gates.

Expected audit artifact: `.pipeline/reviews/C6/prompt-audit.md`.

## Generated Milestone Order

1. Compact Fallback Context Pack
2. Structured Workflow File API
3. Native Workflow Runtime Routing
4. Ordinary Conversation Capture
5. Workflow Command And UX Polish

## P4 Gate

P4 must wait for both:

- user confirmation of the generated prompt set
- completed prompt-audit Subagent evidence
