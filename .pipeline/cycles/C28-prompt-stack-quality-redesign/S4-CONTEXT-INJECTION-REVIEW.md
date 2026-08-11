---
kind: stone-review
cycle: C28-prompt-stack-quality-redesign
stone: S4
status: accepted
updated: 2026-08-10T22:24:25+08:00
source_edits: locked
---

# S4 AGENTS / Environment / External Injection Review

## Current source map

| Surface | Observed size | Role | Owner | Injection condition |
|---|---:|---|---|---|
| Root `AGENTS.md` plus wrapper | about 26.1K chars | user contextual fragment | repository + stale Hypo/OpenCode adapter text | every session rooted in this repository |
| `<environment_context>` | about 0.4K chars | user contextual fragment | VSP core | enabled by config; initial state and changed world state |
| `<hypo_workflow_host>` | 343 chars | developer | VSP host boundary for external plugin | semantic `.pipeline` workspace |
| active-Cycle focus warning | 280 chars | developer | VSP host projection | multiple active Cycles without trusted session focus |
| triggered external Skill body | variable | user fragment | installed plugin/User/Repo Skill | selected Skill only |

The M1 user-context total is 26,472 characters. The root `AGENTS.md` body alone is 25,981 characters, so AGENTS is approximately 98% of this layer. Environment facts are not a meaningful baseline-cost problem.

## Root AGENTS section cost

| Section | Characters | Scope judgment |
|---|---:|---|
| Runtime contract | 7,189 | mixed universal Rust/repo rules and stale Workflow/OpenCode rules |
| Consultation-first boundary | 637 | duplicated by accepted BaseInstructions/Workflow behavior |
| Plan gate visibility | 626 | Workflow/plugin-owned |
| Completion/report surface | 1,046 | duplicated by accepted BaseInstructions and plugin Skills |
| Workflow state persistence | 894 | stale and conflicting |
| Protected files | 191 | Workflow/plugin-owned |
| Analysis boundary | 683 | Workflow/plugin-owned; references a nonexistent derived path |
| Active rules/change size | 1,462 | mixed Workflow and repository engineering rules |
| TUI guidance | 2,487 | TUI-only |
| Test guidance | 6,214 | code/test-only, principally `codex-rs` |
| App-server guidance | 3,688 | app-server/protocol-only |
| Python guidance | 347 | Python-only |
| Platform support | 303 | universal engineering rule |

## Correctness findings

1. The document labels itself an OpenCode adapter and names OpenCode-only `question`/`todowrite`, but VSP-Codex injects it into Codex sessions.
2. It tells agents to update `.pipeline/state.yaml`, while that protected file is still a completed C10 legacy state. The installed v15 plugin and VSP host use `.pipeline/INDEX.md` plus Cycle-local records and explicitly prohibit direct protected-file writes.
3. It requires `.opencode/hypo-workflow.json.analysis`, which does not exist. The actual `analysis.effective` data is nested in `.opencode/hypo-workflow.json` and currently says `hybrid`/confirm.
4. Workflow semantics are duplicated between a stale repository adapter, installed Hypo-Workflow Skills, and VSP's short host boundary.
5. The loader only discovers `AGENTS.md` files from project root to session cwd. A root-cwd session does not see `codex-rs/tui/src/bottom_pane/AGENTS.md` even when later editing that directory.
6. The 32 KiB aggregate project-doc budget is consumed root-first. The current 25,981-character root leaves little room for nested rules and can truncate later, more specific instructions.

## Recommended AGENTS ownership and layout

Target a root `AGENTS.md` of at most 4K characters containing only:

- repository-wide worktree/destructive-change discipline;
- platform support and universal source-quality constraints;
- instruction precedence and the rule that target-path AGENTS must be read before edits;
- a compact domain map, not the domain bodies.

Move unchanged or lightly normalized rules to scoped files:

| Target | Content |
|---|---|
| `codex-rs/AGENTS.md` | Rust API/style/build-data/dependency/schema rules and general Rust test contracts |
| `codex-rs/tui/AGENTS.md` | Ratatui style, wrapping, snapshot, high-touch module guidance |
| `codex-rs/tui/src/bottom_pane/AGENTS.md` | existing state-machine detail |
| `codex-rs/app-server/AGENTS.md` | app-server v2 API/docs/schema/integration guidance |
| `codex-rs/app-server-protocol/AGENTS.md` | protocol wire-shape subset, generated from the same source fragment |
| relevant Python roots | Python 3/minimum-runtime rule |

Do not copy Hypo-Workflow lifecycle semantics into repository AGENTS. Installed plugin Skills own semantic behavior; VSP owns only the short conditional host boundary. Remove OpenCode-specific tool names and stale state instructions from the Codex-visible repository rule surface.

Because most sessions start at repository root, path scoping needs an explicit enforcement path:

- the root rule requires reading AGENTS from repository root through every target path before editing;
- prompt debug reports which AGENTS sources were injected or subsequently loaded;
- M2 should prefer existing discovery helpers and must not infer paths from arbitrary shell strings;
- S5 must decide whether an apply-patch preflight is needed to reject an edit when applicable nested rules have not been loaded.

This layout intentionally creates some upstream-merge structure difference. S5 should land the smallest coherent stage: remove stale Workflow/OpenCode content and split the three large stable headings first; add runtime preflight only if the existing instruction route proves insufficient.

## Environment context

Keep the facts because each has an operational consumer:

- `cwd` and selected environment: relative paths and remote execution;
- `shell`: command syntax;
- date/timezone: time-sensitive answers and Workflow timestamps;
- workspace roots and filesystem profile: tool targeting and access explanation;
- subagent identities only while active.

The block is host-generated state, not user intent. Candidate role is developer, while repository/global AGENTS remain user-role instructions because users or repository owners author them. Keep world-state diffing: initial injection once, then replacement only when relevant facts change. Do not duplicate prose permission policy inside environment context; retain its compact structured permission snapshot for tooling/debugging.

## External injection boundary

- Keep `<hypo_workflow_host>` developer-role and conditional on the semantic workspace/plugin capability. It may state ownership, semantic index location, and recovery entrypoint; it must not re-embed plugin lifecycle procedures.
- Replace filesystem-inferred active-Cycle warnings with a trusted plugin/host projection. If focus is unknown, say the session is unbound and allow ordinary work; do not contradict a focused semantic index.
- External Skill bodies remain trigger-only user fragments with clear source markers. Their content cannot override system/developer safety or expand user authorization.
- Plugin/Apps common metadata remains developer-role and conditional as accepted in S3. External business prompts are not rewritten in C28.
- Compaction restores current world-state fragments once; stale AGENTS/environment/host fragments are replaced or removed through existing section IDs.

## Expected cost and behavior

- Ordinary root sessions: AGENTS body target <=4K instead of 25,981 characters, a reduction of at least 84% for this body.
- Rust/test/TUI/app-server rules load only for the relevant work scope; no rule is deleted solely for token savings.
- Ordinary questions stop receiving OpenCode tool instructions, stale C10 state duties, TUI rules, app-server wire rules, and test harness detail.
- Relevant code tasks still receive all applicable domain rules before mutation.

## Test design for M2/M3

- Root/non-code turn receives only universal AGENTS and no OpenCode/Hypo lifecycle body.
- Target paths resolve ordered root -> `codex-rs` -> domain -> leaf instructions.
- Root-cwd edit preflight cannot bypass applicable nested rules; multi-root/remote paths remain environment-specific.
- Aggregate budgets do not let root content starve the more specific file; truncation is visible in prompt debug.
- AGENTS updates replace prior world state once across ordinary turns, compaction, resume, fork, and selected-environment changes.
- Environment facts use the accepted role, escape XML, update only on change, and preserve remote/multi-environment behavior.
- Workflow host appears only with valid external capability and semantic workspace; stale or missing projection does not block ordinary prompts.
- No platform adapter mentions unavailable tools on Codex.

## Risks and open integration points

- Splitting the upstream root AGENTS increases merge conflict risk unless fragments and sync ownership are explicit.
- A model-only “read nested rules” contract can be missed; an edit preflight is safer but adds runtime complexity and must cover non-`apply_patch` mutations carefully.
- Moving environment context from user to developer changes cache/order behavior and needs real rollout comparison before acceptance.
- The installed plugin version and committed OpenCode adapter have drifted. C28 can remove stale Codex-visible content, but plugin packaging/sync fixes belong to the plugin owner unless explicitly added to VSP scope.
