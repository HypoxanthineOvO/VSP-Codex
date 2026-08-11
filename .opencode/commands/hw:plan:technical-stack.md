---
agent: hw-plan
description: Hypo-Workflow mapping for /hw:plan:technical-stack
---

# /hw:plan:technical-stack

Canonical command: `/hw:plan:technical-stack`
Route: `plan`
Skill: `skills/plan-technical-stack/SKILL.md`

Load the corresponding Hypo-Workflow skill instructions from `skills/plan-technical-stack/SKILL.md`, then execute the canonical command semantics with any user-provided arguments.
Plan discipline: use `question` / Ask for every hard interactive gate unless automation is explicitly configured, and keep `todowrite` synchronized for Discover / Technical Stack / Architecture / Decompose / Generate checkpoint state. Progressive Discover starts with task category, desired effect, and verification method, then moves through assumptions, ambiguities, tradeoffs, and validation criteria as needed without discussing implementation stack or architecture prematurely. Technical Stack owns implementation substrate and existing stack choices. Architecture owns current architecture reading, diagrams, and integration points. Decompose owns milestone splitting after those phase artifacts exist. Confirmation is an in-phase Ask gate, not a standalone user command. For `/hw:guide`, use the intent router contract and recommend one next path before confirmation. For `/hw:plan --deep`, treat it as an alias and route to `/hw:plan:deep` before ordinary planning; ordinary `/hw:plan` keeps the named phase gates and must not skip them after Deep Plan conversion. For `/hw:plan --batch`, collect multiple Features in one Discover pass, then generate Feature Queue tables and Mermaid diagrams according to `batch.decompose_mode`. For `/hw:plan --insert`, convert the natural-language request into a structured queue operation, summarize the queue diff, and wait for explicit confirmation before writing `.pipeline/feature-queue.yaml`.

## Four-Rule Discipline

Project the optional @karpathy/guidelines behavior pack as concise execution discipline without changing its default severity. Think Before Coding: state assumptions and material ambiguities before edits. Simplicity First: choose the smallest sufficient solution. Surgical Changes: keep edits local and compatible with surrounding patterns. Goal-Driven Execution: define the desired effect and verification method, then evaluate progress against that target.

## Ask Questions Discipline

Use Ask Questions proactively when a decision materially changes scope, safety, architecture, release behavior, remote side effects, protected files, or acceptance criteria. Use the `question` tool when it is available; otherwise stop and ask the user in the normal response channel. Do not bury required user decisions in unrelated prose or proceed on a guess when the answer changes what should be edited, tested, pushed, released, installed, or delegated.

For major Plan gates, show the actual phase artifacts before Question Tool / Ask confirmation. This includes Discover, Technical Stack, Architecture, Decompose, and Generate outputs such as the stage summary, decision table, open questions, diagrams, milestone table, decision matrix, and dependency map.

Before invoking `question` / Ask, explain why the decision is needed now, what each answer changes downstream, and which visible phase artifact or evidence the user is confirming. The card is the gate after that explanation, not the explanation itself.

Prefer one concise question with the smallest actionable decision. Continue without asking only when the repo evidence and active configuration make the decision unambiguous.

## Consultation-first Action Boundary

Treat `discussion`, `background`, `idea`, `complaint`, `question`, and `solution-discussion` user input as non-editing signals. First respond with a Mini-contract in this order: `我的理解` -> `问题原因` -> `推荐方案`. Do not edit files, run mutating commands, or start execution from these signals unless the user then gives a clear post-plan affirmative reply.

Clear imperative requests with a concrete target may proceed directly. After a plan, Mini-contract, or recommendation has been shown, replies such as `可以`, `确认`, `OK`, `go ahead`, or `apply it` authorize only the scope already shown to the user.

When this command writes or references a report/plan artifact, the final response must show the core content in chat: conclusion/change summary, technical approach, modified files/modules or generated artifacts, validation result, expected result, and risks/follow-up. A path-only response is insufficient.

Before acting, inspect the relevant context when present:

- `.pipeline/config.yaml`
- `.pipeline/cycle.yaml`
- `.pipeline/state.yaml`
- `.pipeline/rules.yaml`
- current prompt/report files for pipeline commands
- open patches for Patch commands

Keep this command as an OpenCode-native slash mapping, not a separate runner. The OpenCode Agent performs the work and Hypo-Workflow files remain the source of truth.
