# C8 P3 Generate

Timestamp: 2026-05-30T13:43:31+08:00
Status: prompt_audited_pending_p4_confirmation

## Generated Prompts

- `.pipeline/prompts/c8-001-remote-ssh-vscode-integration-probe.md`
- `.pipeline/prompts/c8-002-vscode-plugin-protocol-capability-map.md`
- `.pipeline/prompts/c8-003-provider-resume-compatibility-contract.md`
- `.pipeline/prompts/c8-004-session-discovery-strategy-official-codex.md`
- `.pipeline/prompts/c8-005-request-tool-request-input-plugin-ux.md`
- `.pipeline/prompts/c8-006-markdown-workflow-surface-remote-ssh-acceptance.md`

## Execution Policy

- Worker Separation: strict.
- Codex execution subworkers: authorized for `/hw:start` and `/hw:resume`.
- Each implementation prompt assigns exactly three worker roles: `test`, `implement`, and `audit`.
- `test` owns failing tests, fixtures, assertions, snapshots, black-box probes, and final verification evidence.
- `implement` owns production edits only and must not create, edit, rewrite, or satisfy `test` assets.
- `audit` checks final diff, evidence quality, protocol assumptions, privacy boundaries, Remote SSH fit, permission boundaries, and worker identity separation.
- Main agent orchestrates, integrates returned work, updates lifecycle/progress/log files, and makes final decisions; it must not satisfy both implementation and verification roles.
- Required Workflow decisions must use `request_user_input` when available.
- Destructive operations, system-level dependency installation, network/remote side effects, remote publication, tag creation, and release upload require explicit confirmation/authorization.

## Prompt Audit Default

Every generated Workflow execution prompt set must receive a Subagent prompt-audit pass before P4 confirmation. The prompt-audit Subagent checks:

- P2 technical fields are preserved, not compressed into vague goals.
- Remote SSH is treated as the primary VSCode environment.
- Official VSCode plugin remains black-box by default.
- Worker Separation is explicit and role overlap is forbidden.
- Real validation methods and pass/fail signals are present.
- Prompt order matches the accepted delivery priority.
- Required Workflow decisions use `request_user_input` when available.
- authorization-intent remains distinct from real permission grant.
- Destructive operations, external side effects, remote publication, and system-level dependency installation remain confirmation/authorization gates.
- M6 includes real Remote SSH VSCode plugin acceptance.

Expected audit artifact: `.pipeline/reviews/C8/prompt-audit.md`.

Prompt audit completed: 2026-05-30T13:50:47+08:00
Prompt audit verdict: pass

## Generated Milestone Order

1. Remote SSH VSCode Integration Probe
2. VSCode Plugin Protocol Capability Map
3. Provider And Resume Compatibility Contract
4. Session Discovery Strategy For Official Codex
5. Request Tool And Request Input UX For Plugin
6. Markdown Workflow Surface And Remote SSH Acceptance

## P4 Gate

P4 must wait for both:

- user confirmation of the generated prompt set
- completed prompt-audit Subagent evidence
