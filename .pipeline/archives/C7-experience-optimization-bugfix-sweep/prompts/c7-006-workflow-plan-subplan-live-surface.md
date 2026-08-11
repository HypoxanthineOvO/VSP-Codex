# M6 Workflow Plan/SubPlan Live Surface

## Objective

Make `update_plan` a trustworthy live Workflow surface with Cycle/Milestone/SubPlan structure, proactive refresh triggers, separate progress counts, and full `/hw:*` command palette wiring.

## Source Of Truth

- P1 issue: Issue 7 `update_plan` is not proactive enough and must publish Cycle/Milestone/SubPlan structure with separate SubPlan/root Plan counts.
- P2 technical solution: use `PlanState { title, summary, items, subplan }` as the canonical live UI shape; root items are Milestones; active Milestone expands its SubPlan; future/completed Milestones collapse.
- Dependency: M5 provides structured confirmation gates for plan/phase decisions.
- Carry-over from C6: full TUI command palette wiring for `/hw:*`; clean deprecated `turn_context.cwd` warning.
- Relevant modules: `codex-rs/protocol/src/plan_tool.rs`, `codex-rs/tui/src/chatwidget/plan_panel.rs`, `codex-rs/core/tests/suite/plan_update_gate.rs`, `codex-rs/core/src/workflow_routing.rs`, `codex-rs/tui/src/workflow_slash_routing.rs`, and `codex-rs/core/src/session/mod.rs`.

## Subworker Assignment Plan

Status: authorized, strict worker separation.

- `test`
  - Scope: owns failing tests, fixtures, assertions, and snapshots for Workflow PlanState shape, current SubPlan expansion, future milestone collapse, separate progress counts, freshness reminders, command palette routing, and deprecated cwd warning cleanup.
  - Write boundary: test/snapshot files and `.pipeline/reviews/C7/M6/test.md` only.
  - Expected artifact: `.pipeline/reviews/C7/M6/test.md`.
- `implement`
  - Scope: owns production PlanState builder/conventions, Plan panel/status progress rendering, freshness gates, Workflow command palette entries, routing integration, and cwd warning cleanup.
  - Must not create, edit, rewrite, or satisfy tests owned by `test`.
  - Expected artifact: `.pipeline/reviews/C7/M6/implement.md`.
- `audit`
  - Scope: read-only review of Plan/SubPlan count semantics, update frequency, ordinary-chat non-spam behavior, slash routing compatibility, structured confirmation use, and stale-plan final-response behavior.
  - Must reject mixed counts such as a combined `2/9` when the intended surface is current SubPlan plus root Plan.
  - Expected artifact: `.pipeline/reviews/C7/M6/audit.md`.

Workers must not revert user or other-worker edits. Required user decisions must use `request_user_input` when available. Destructive operations, system-level dependency installation, network/remote side effects, and release publication require explicit confirmation/authorization.

## Technical Solution

Use structured `PlanState` as the Workflow live status surface. Root items represent Cycle Milestones (`M1`, `M2`, ...). The current Milestone carries one expanded SubPlan with actionable steps such as discover/test/implement/audit/update-state. Completed/future Milestones are collapsed with compact summaries. Progress displays two separate values: current SubPlan completed/total and root Plan completed/total, e.g. semantic form `(2/5 | 0/8)`. Add Workflow-specific freshness triggers at phase, issue, edit, validation, worker, resume, compact, and final-answer boundaries. Required Plan confirmations and phase transitions use M5 structured cards.

## Technical Route

1. Define a Hypo-Workflow PlanState builder/convention from Cycle metadata, P2/P3 prompt metadata, and current milestone/step.
2. Update `plan_panel.rs`, turn runtime, and status surfaces to compute/render separate current-SubPlan and root-Plan counts.
3. Extend plan freshness reminders beyond resume/compact/tool bursts to Workflow state advances and stale final response cases.
4. Add runtime guidance that `update_plan` is mandatory for Workflow phase/issue/worker boundaries.
5. Route required Plan confirmations and phase decisions through M5 card templates.
6. Wire `/hw:status`, `/hw:plan`, `/hw:resume`, and related Workflow entries into the command palette without misrouting non-Workflow slash commands.
7. Clean the non-blocking deprecated `turn_context.cwd` warning by using the selected turn environment cwd where appropriate.

Non-goals: automatic replacement of durable `.pipeline/state.yaml` with the transient Plan panel. `.pipeline` remains source of truth; `update_plan` is the live UI surface.

## Research Required

- Resolved by local evidence: `PlanState` supports one-level SubPlan and TUI has a persistent plan panel.
- Resolved by local evidence: M5 provides the structured interaction contract that Plan gates consume.
- No external research required.

## Risks And Alternatives

- Risk: too many plan updates become noise. Mitigation: stricter triggers apply only in Workflow context; ordinary small chats remain lightweight.
- Risk: `.pipeline` state and model-authored Plan diverge. Mitigation: use `.pipeline` as recovery/freshness evidence and explicit `update_plan` as the live UI source.
- Rejected alternative: flat checklist for C7. It cannot represent Milestone/SubPlan structure or separate counts.

## Validation Path

Required validation:

- `cargo test -p codex-core plan_update_gate workflow_routing -- --nocapture`.
- `cargo test -p codex-tui plan_panel workflow_slash_routing -- --nocapture`.
- Snapshot: current Milestone expanded, future Milestones collapsed, and counts shown as current SubPlan plus root Plan rather than combined.
- Test or fixture: stale Workflow state after tools triggers a plan freshness reminder before final response.
- `cargo fmt --check`.
- `git diff --check`.

Manual acceptance:

- Run a multi-issue planning discussion and short execution pass.
- Verify Plan panel updates at each phase/issue boundary without user reminders.
- Verify `Cycle 7 Plan` shows current Milestone SubPlan and collapsed future Milestones.

Pass signal: Workflow plan state shows Cycle/Milestone/SubPlan structure, progress counts are split as current SubPlan plus root Plan, freshness reminders trigger at Workflow boundaries, and non-Workflow slash commands are not misrouted.

Fail signal: counts are mixed into one total, future Milestones expand noisily, Plan updates spam ordinary chats, or `/review`/`/compact`/`/model` are routed as Workflow commands.

## Audit Focus

- Verify Plan updates do not spam ordinary Default chats.
- Verify root Plan and current SubPlan counts are never mixed.
- Verify Plan confirmations use Request Input cards when required.
- Verify `/hw:*` command palette wiring does not catch `/review`, `/compact`, `/model`, or ordinary text.
