# C9 M6 - Deep Research Workflow Requirement Brief

## Goal

Finalize a Workflow-ready Deep Research requirement brief for later Hypo-Workflow implementation. Do not implement Codex-VSP Deep Research in C9.

## Technical Solution

Use `.pipeline/architecture/c9-upstream-base-upgrade-and-deep-research-brief.md` as the source and refine it into an actionable brief covering problem statement, command semantics, Question Tool flow, Subagent fan-out, recursion/budget controls, artifact contract, UI expectations, validation scenario, and non-goals.

## Technical Route

1. Review the architecture brief for completeness.
2. Optionally add `.pipeline/plan-state/deep-research-brief.yaml` for machine-readable transfer.
3. Ensure the brief states that C9 does not implement production Deep Research.
4. Reference the brief in C9 report/summary.

## Research Required

Resolved. Discover collected the product requirements.

## Risks And Alternatives

Risk: brief may be mistaken for implementation.

Rejected alternative: implementing a prototype in C9, because user selected design-only.

## Validation Path

Command:

```bash
rg -n "Question Tool|Subagent|recursive|budget|follow-up|\\.pipeline/research|0.140.0-alpha.18" .pipeline/architecture/c9-upstream-base-upgrade-and-deep-research-brief.md
```

Pass signals: brief covers required Deep Research concepts and states the C9 non-goal clearly.

## Subworker Assignment Plan

- `test`: Owns checklist validation of brief coverage. Writes `.pipeline/reviews/C9/M6/test.md`.
- `implement`: Owns documentation/brief refinement only. Must not implement production Deep Research code.
- `audit`: Checks the brief is actionable for Hypo-Workflow and scope boundary is honest. Writes `.pipeline/reviews/C9/M6/audit.md`.

Lifecycle evidence required for all roles.
