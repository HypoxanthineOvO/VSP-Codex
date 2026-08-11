# M4 Subagent Prompt And Report Viewer

## Objective

Let users inspect the full Prompt sent to a Subagent and the Subagent report in a readable Markdown-rendered viewer, with original and translated forms available.

## Source Of Truth

- P1 issue: Issue 5 interactive Subagent prompt and report viewer with Markdown translation.
- P2 technical solution: retain or locate full Subagent prompt/report text; add an interactive viewer reachable from active Subagents and/or lifecycle history rows; render Markdown; provide original/translated panes or tabs with lazy translation.
- Dependencies: M2 supplies stable Subagent identity metadata; M3 establishes literal-by-default tool/status translation boundaries so this viewer can offer explicit translated content without polluting lifecycle rows.
- Relevant modules: `codex-rs/tui/src/multi_agents.rs`, `codex-rs/tui/src/chatwidget/active_subagents.rs`, collaboration state/history cells, Subagent transcript handling, and existing Markdown renderer.

## Subworker Assignment Plan

Status: authorized, strict worker separation.

- `test`
  - Scope: owns failing tests, fixtures, assertions, and snapshots for prompt preview vs full viewer, Markdown rendering, original/translated modes, narrow terminal layout, and lifecycle-row concision.
  - Write boundary: test/snapshot files and `.pipeline/reviews/C7/M4/test.md` only.
  - Expected artifact: `.pipeline/reviews/C7/M4/test.md`.
- `implement`
  - Scope: owns production state/history metadata, viewer entry points, Markdown-rendered prompt/report UI, lazy translation, and integration with Subagent lifecycle surfaces.
  - Must not create, edit, rewrite, or satisfy tests owned by `test`.
  - Expected artifact: `.pipeline/reviews/C7/M4/implement.md`.
- `audit`
  - Scope: read-only review of prompt/report exposure boundaries, Markdown layout, translation labeling, sensitive-content handling, and lifecycle UI clarity.
  - Must reject fake tests that only verify stored strings without opening/rendering the viewer surface.
  - Expected artifact: `.pipeline/reviews/C7/M4/audit.md`.

Workers must not revert user or other-worker edits. Required user decisions must use `request_user_input` when available. Destructive operations, system-level dependency installation, network/remote side effects, and release publication require explicit confirmation/authorization.

## Technical Solution

Extend Subagent state/history metadata to retain or locate the full prompt and final response/report. Add an interactive viewer reachable from the active Subagents panel and/or lifecycle history rows. The viewer renders Markdown through the existing renderer and exposes original and translated modes. Translation is lazy, explicit, and labeled; original evidence remains available. Code fences, paths, YAML, and command snippets must remain literal or clearly protected.

## Technical Route

1. Determine whether prompt/report should be sourced from `CollabAgentState`, the subagent thread transcript, final `AgentMessage`, or a dedicated lightweight reference.
2. Add bounded prompt/report references to lifecycle state where needed without dumping full content into unrelated rows.
3. Implement viewer UI using existing selection, overlay, popup, or pager patterns.
4. Render prompt/report Markdown with headings, lists, code blocks, paths, and links intact.
5. Add original/translated modes; generate translation lazily when opened.
6. Ensure lifecycle rows remain concise and are not polluted by automatic translation.
7. Add snapshots for prompt preview, full prompt, report, translated prompt/report, and narrow terminal behavior.

Non-goals: role/effort scheduling changes, handled in M2.

## Research Required

- Resolved by local evidence for candidate state/transcript locations.
- Implementation-time choice remains for the least invasive data source. If final report retrieval from transcript is incomplete, use a dedicated stored report reference.

## Risks And Alternatives

- Risk: prompt/report text can contain sensitive content. Mitigation: do not add broad new persistent logs; reuse transcript/artifact references and require explicit viewer access.
- Risk: translation corrupts operational evidence. Mitigation: always keep original available and label translation as derived.
- Rejected alternative: only show truncated prompt previews. The user explicitly requested full Markdown-rendered inspection.

## Validation Path

Required validation:

- `cargo test -p codex-tui multi_agents markdown_render -- --nocapture` or focused equivalent.
- TUI snapshots for viewer entry, prompt Markdown, report Markdown, translated modes, and narrow terminal layout.
- Tests proving lifecycle rows remain concise while viewer exposes full content.
- `cargo fmt --check`.
- `git diff --check`.

Manual acceptance:

- Spawn a Subagent with a Markdown prompt and Markdown report.
- Open the viewer, switch original/translation, and verify headings, lists, code blocks, paths, and links render correctly.

Pass signal: the full original prompt/report and explicitly labeled translated views render as Markdown, lifecycle rows stay concise, and sensitive content is not exposed outside intended viewer surfaces.

Fail signal: only truncated previews are available, translated content overwrites original evidence, Markdown layout overlaps in narrow terminals, or prompt/report text leaks into unrelated rows.

## Audit Focus

- Verify no prompt/report content appears in unintended surfaces.
- Verify Markdown rendering handles narrow terminals without overlap.
- Verify translation is opt-in/lazy and clearly labeled.
- Verify original evidence remains available for review.
