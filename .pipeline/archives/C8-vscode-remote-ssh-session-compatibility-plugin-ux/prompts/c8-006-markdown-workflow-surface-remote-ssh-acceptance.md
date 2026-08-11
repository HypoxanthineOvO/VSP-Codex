# M6 Markdown Workflow Surface And Remote SSH Acceptance

## Objective

Make Workflow output readable in the VSCode plugin and complete the C8 Remote SSH acceptance path: resume a VSP session, view Workflow/Markdown surfaces, handle Request Input, and inspect Subagent prompt/report information.

## Source Of Truth

- User wants to interactively see translated Subagent prompts and returned reports, rendered as Markdown.
- C7 improved TUI-side Subagent prompt/report viewer and Workflow Plan/SubPlan live surface; C8 must make the plugin path coherent.
- M1-M5 should have established Remote SSH settings priority, protocol capability, provider resume compatibility, session discovery strategy, and request input routing.
- Final C8 acceptance must include real Remote SSH plugin resume/interaction.

## Subworker Assignment Plan

Status: authorized, strict worker separation.

- `test`
  - Scope: owns protocol/rendering tests, markdown fixture snapshots, Remote SSH acceptance checklist, and `.pipeline/reviews/C8/M6/test.md`.
  - Write boundary: tests/fixtures/snapshots/review artifact only.
- `implement`
  - Scope: owns production Markdown surface/fallback output, Workflow thread item formatting, report links, and acceptance documentation.
  - Must not create, edit, rewrite, or satisfy tests owned by `test`.
  - Expected artifact: `.pipeline/reviews/C8/M6/implement.md`.
- `audit`
  - Scope: read-only review of plugin readability, Remote SSH acceptance evidence, markdown safety, translation boundaries, and end-to-end continuity.
  - Expected artifact: `.pipeline/reviews/C8/M6/audit.md`.

Workers are not alone in the codebase. Do not revert user edits or another worker's edits; adapt around them and stop on scope conflicts.

Required user decisions must use `request_user_input` when available. Do not publish a release, push tags, install system dependencies, or perform remote side effects without explicit confirmation.

## Technical Solution

Provide a Markdown-first Workflow surface for plugin thread output whenever native structured UI is unavailable. The surface should cover Subagent prompt/report references, Plan/SubPlan progress, Workflow report links/summaries, tool output summaries, and request input fallback cards. Original evidence remains available; translated content must be labeled as derived and must preserve code fences, paths, YAML, diffs, and command output.

## Technical Route

1. Use M2 capability matrix and M5 routing results to decide native versus fallback display paths.
2. Identify the server-side item/output construction used by the VSCode plugin thread.
3. Add fixture/snapshot tests for:
   - Subagent prompt preview and full report Markdown
   - translated report label and original-preserving behavior
   - Plan/SubPlan progress with separate subplan/root counts
   - tool output summary without confusing duplicate translation
   - request input fallback card
4. Implement minimal Markdown formatting helpers or reuse existing render/report helpers.
5. Add a Remote SSH acceptance checklist/report:
   - run `vsp-codex vscode status`
   - resume a VSP session from plugin
   - trigger a Workflow request input
   - inspect Subagent prompt/report or the markdown report substitute
   - confirm Plan/SubPlan progress is readable
6. Run final focused regression for C8 touched crates.

Non-goals: release publication, VSIX publishing, or replacing the plugin UI.

## Research Required

- Depends on M1-M5 evidence.
- If plugin cannot render a specific native item, record fallback behavior and do not claim native support.

## Risks And Alternatives

- Risk: Markdown fallback becomes too verbose. Mitigation: use summaries with expandable/report references where possible.
- Risk: translation corrupts evidence. Mitigation: original-first display and derived translation labels.
- Risk: Remote SSH acceptance is environment-sensitive. Mitigation: record exact environment, commands, and observed plugin behavior.

## Validation Path

Required validation:

- Focused protocol/rendering tests for Markdown Workflow surface.
- C8 touched-crate regression as appropriate.
- `cargo fmt --check`
- `git diff --check`

Manual acceptance:

- Real Remote SSH VSCode plugin session can resume VSP session.
- Plugin can handle or display a Workflow request input.
- Plugin shows readable Markdown Workflow output and Subagent prompt/report information.

Pass signal: the user can use the Remote SSH plugin path without raw UUID/protocol/debug clutter for the target flows.

Fail signal: resume still fails, request input is invisible/unusable, or Workflow/Subagent output appears as unrendered confusing raw transcript.

## Audit Focus

- Verify final acceptance is from Remote SSH, not only local CLI tests.
- Verify original evidence is preserved when translation is shown.
- Verify plugin fallback is readable and does not claim unsupported native features.
- Verify no release/publish/tag side effect occurred.

