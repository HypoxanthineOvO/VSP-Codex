# M8 Appearance Entry Point And Final UX Polish

## Objective

Unify theme/appearance entry points, polish command descriptions and docs, and run a final experience-focused regression pass across C7.

## Source Of Truth

- P1 issue: Issue 9 clarify and possibly unify `/theme` and `/ui-theme`.
- P2 technical solution: keep `[tui].theme` and `[tui].ui_theme` separate but merge the user-facing appearance entry. `/theme` should open a unified Appearance/Theme panel; `/ui-theme` remains a direct UI palette shortcut.
- Dependencies: M3 tool card rendering, M4 Subagent viewer, M6 Plan panel should all respect UI palette and layout constraints.
- Relevant modules: TUI slash command routing, command palette entries, existing syntax theme picker, VSP UI theme picker, config persistence, Markdown render styling, docs.

## Subworker Assignment Plan

Status: authorized, strict worker separation.

- `test`
  - Scope: owns failing tests, fixtures, assertions, snapshots, and final regression checklist for `/theme`, `/ui-theme`, command palette labels, config compatibility, UI palette vs syntax theme independence, and C7 focused regression commands.
  - Write boundary: test/snapshot/checklist files and `.pipeline/reviews/C7/M8/test.md` only.
  - Expected artifact: `.pipeline/reviews/C7/M8/test.md`.
- `implement`
  - Scope: owns production UI/command changes, docs/changelog/release notes for implemented behavior, and final integration polish across M1-M7 surfaces.
  - Must not create, edit, rewrite, or satisfy tests owned by `test`.
  - Expected artifact: `.pipeline/reviews/C7/M8/implement.md`.
- `audit`
  - Scope: read-only final UX audit covering appearance config compatibility, narrow terminal layout, docs honesty, C7 regression evidence, and no overpromised updater/Workflow behavior.
  - Must reject docs that claim silent auto-update or incomplete Workflow command behavior as shipped.
  - Expected artifact: `.pipeline/reviews/C7/M8/audit.md`.

Workers must not revert user or other-worker edits. Required user decisions must use `request_user_input` when available. Destructive operations, system-level dependency installation, network/remote side effects, and release publication require explicit confirmation/authorization.

## Technical Solution

Keep syntax theme and UI palette as separate config concepts:

- `[tui].theme`: syntax highlighting theme for code/diff rendering.
- `[tui].ui_theme`: VSP UI palette for Markdown, messages, cards, composer, Plan panel, and Subagent viewer.

Merge the user-facing appearance entry point. `/theme` should become the natural primary Appearance/Theme panel with separate sections for UI palette and syntax theme. `/ui-theme` remains a direct route to the UI palette section for compatibility and discoverability. Docs and command palette labels should make the split explicit.

## Technical Route

1. Update slash command and command palette descriptions to distinguish Appearance, UI palette, and Syntax theme.
2. Add unified `/theme` behavior or `/appearance` alias, with `/theme` as the primary entry.
3. Preserve `/ui-theme` as a direct UI palette shortcut.
4. Add tests proving UI palette changes Markdown/message/card colors but does not mutate syntax theme.
5. Add tests proving syntax theme changes code highlighting but does not mutate UI palette.
6. Ensure Markdown viewer, Subagent viewer, Plan panel, tool cards, and request-input cards use active UI palette consistently.
7. Update `README.md`, `docs/vsp-codex.md`, `docs/install.md`, slash command docs, and release notes for implemented C7 behavior.
8. Run final focused regression commands from M1-M7 plus formatting and diff checks.

Non-goals: merging `[tui].theme` and `[tui].ui_theme` into one config key; adding more than UI palette and syntax theme to Appearance in C7.

## Research Required

- Resolved by local evidence: existing `/theme`, `/ui-theme`, config keys, and persistence paths have been inspected.
- No external research required.

## Risks And Alternatives

- Risk: changing `/theme` surprises users who expect direct syntax-theme picker. Mitigation: make Syntax Theme one action away and keep `/ui-theme` shortcut.
- Risk: appearance panel becomes too broad. Mitigation: only include UI palette and syntax theme in C7.
- Rejected alternative: merge the config keys. Syntax themes and curated UI palettes have different cardinality and semantics.

## Validation Path

Required validation:

- `cargo test -p codex-tui theme ui_theme command_palette -- --nocapture`.
- Snapshots for Appearance panel and command palette labels.
- Focused regression commands from M1-M7 where practical.
- `cargo fmt --check`.
- `git diff --check`.

Manual acceptance:

- Open `/theme`, understand both sections, change UI palette, change syntax theme, and verify both persist independently.
- Verify C7 cards/panels do not overlap in narrow terminals.

Pass signal: `/theme` exposes a unified Appearance/Theme entry, `/ui-theme` remains a UI palette shortcut, `[tui].theme` and `[tui].ui_theme` persist independently, and final docs match implemented behavior.

Fail signal: syntax theme and UI palette configs are conflated, existing config compatibility breaks, narrow terminal layout overlaps, or docs overpromise silent update/Workflow behavior.

## Audit Focus

- Verify config compatibility for existing `[tui].theme` and `[tui].ui_theme`.
- Verify no layout overlap in narrow terminals.
- Verify docs match implemented behavior.
- Verify docs do not overpromise silent auto-update or incomplete Workflow command behavior.
