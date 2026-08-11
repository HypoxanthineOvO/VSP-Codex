# C7 P3 Generate

Timestamp: 2026-05-29T21:00:31+08:00
Status: generated_pending_prompt_audit

## Generated Prompts

- `.pipeline/prompts/c7-001-resume-and-scrollback-stability.md`
- `.pipeline/prompts/c7-002-subagent-identity-role-policy-name-leases.md`
- `.pipeline/prompts/c7-003-tool-collaboration-rendering-boundaries.md`
- `.pipeline/prompts/c7-004-subagent-prompt-report-viewer.md`
- `.pipeline/prompts/c7-005-structured-interaction-gates-authorization-cards.md`
- `.pipeline/prompts/c7-006-workflow-plan-subplan-live-surface.md`
- `.pipeline/prompts/c7-007-vsp-release-channels-naming-updater.md`
- `.pipeline/prompts/c7-008-appearance-entry-final-ux-polish.md`

## Execution Policy

- Worker Separation: strict.
- Codex execution subworkers: authorized for `/hw:start` and `/hw:resume`.
- Each implementation prompt assigns exactly three worker roles: `test`, `implement`, and `audit`.
- `test` owns red tests, fixtures, assertions, snapshots, and final verification evidence.
- `implement` owns production edits only and must not create, edit, rewrite, or satisfy `test` assets.
- `audit` checks final diff, evidence quality, assumptions, risks, permission boundaries, and worker identity separation.
- Main agent orchestrates, integrates returned work, updates lifecycle/progress/log files, and makes final decisions; it must not satisfy both implementation and verification roles.
- Required user decisions must use `request_user_input` when available.
- Destructive operations, system-level dependency installation, network/remote side effects, and release publication require explicit confirmation/authorization.

## Structured Interaction Card Types

C7 M5 codifies at least these card templates:

1. Plan confirmation card.
2. Phase transition card.
3. Scope selection card.
4. Revision/rejection card.
5. Authorization-intent card.
6. Release/update confirmation card.

True permission grants remain controlled by `request_permissions` or existing approval overlays. A confirmation card must not bypass sandbox, network, destructive-action, or external-side-effect approval policy.

## Prompt Audit Default

Starting with C6 P3 and continuing in C7, every generated Workflow execution prompt set must receive a prompt-audit pass by Subagent before P4 confirmation. The prompt-audit Subagent checks:

- P2 technical fields are preserved, not compressed into vague goals.
- Worker Separation is explicit and role overlap is forbidden.
- Real validation methods and pass/fail signals are present.
- Prompt order matches the accepted delivery priority.
- Required Workflow decisions use `request_user_input` when available.
- M5 card semantics distinguish confirmation, authorization intent, real permission grant, and ordinary clarification.
- Destructive operations, external side effects, and system-level dependency installation remain confirmation/authorization gates.

Expected audit artifact: `.pipeline/reviews/C7/prompt-audit.md`.

## Generated Milestone Order

1. Resume And Scrollback Stability
2. Subagent Identity, Role Policy, And Name Leases
3. Tool And Collaboration Output Rendering Boundaries
4. Subagent Prompt And Report Viewer
5. Structured Interaction Gates And Authorization Cards
6. Workflow Plan/SubPlan Live Surface
7. VSP Release Channels, Naming Metadata, And Updater
8. Appearance Entry Point And Final UX Polish

## P4 Gate

P4 must wait for both:

- user confirmation of the generated prompt set
- completed prompt-audit Subagent evidence

