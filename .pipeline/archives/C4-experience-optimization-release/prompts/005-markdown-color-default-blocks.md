# M5 Markdown Color Refinement + Default Subtle Blocks

## Objective

在已有 Markdown renderer 基础上补齐可见颜色，让 bold/italic/list/table 等元素在暗色终端中清楚可扫读，并加入默认淡色块区分。

## Source Of Truth

- `.plan-state/decompose.yaml` M5
- Existing renderer: `codex-rs/tui/src/markdown_render.rs`
- User clarified bold/italic/list need visible color because default white/bold is hard to see

## Subworker Assignment Plan

Status: authorized, strict worker separation.

- `test`
  - Owns style assertions and snapshots for Markdown tokens, mixed message blocks, and narrow wrapping.
  - Expected artifact: `.pipeline/reviews/C4/M5/test.md`
- `implement`
  - Owns Markdown style palette and default block accent styling.
  - Expected artifact: `.pipeline/reviews/C4/M5/implement.md`
- `audit`
  - Checks no full theme picker scope creep, no heading title-row regression, and wrapping/table stability.
  - Expected artifact: `.pipeline/reviews/C4/M5/audit.md`

## Technical Solution

Do not rewrite Markdown parsing. Refine `MarkdownStyles` and nearby TUI style helpers. Use a compact three-role palette: emphasis text, code/link, structural elements. Apply default teal/amber-like subtle accents to output block boundaries without adding explicit title rows.

## Technical Route

1. Add tests proving strong/emphasis/bold+italic are non-default and visible.
2. Update `MarkdownStyles::default()` for strong, emphasis, headings, list markers, blockquote, code/link, table headers/borders.
3. Replace hardcoded cyan inline-code/link role with theme-derived code/link role.
4. Keep table/body prose calm; color markers, headers, and borders.
5. Add subtle history-cell accents for user/assistant/thinking/tool/Ask/subagent/error where locally owned and low risk.
6. Avoid a full `Color Theme` picker/config in this Cycle.
7. Update and review snapshots.

## Research Required

Resolved: Markdown renderer, agent wrapper, history-cell integration, and tests exist. Current inline code/link are cyan; strong/emphasis mainly rely on terminal attributes.

## Risks And Alternatives

- Risk: too many colors make long messages noisy. Use three-role palette.
- Risk: full truecolor/fallback matrix expands scope. Dark terminal first only.
- Deferred: full Color Theme picker and three persisted theme groups.

## Validation Path

Commands:

- `just test -p codex-tui markdown_render`
- `just test -p codex-tui history_cell`
- `cargo insta pending-snapshots -p codex-tui`

Pass signal: bold/italic/bold+italic/list/table/code/link/blockquote are visibly colored; mixed message blocks are readable; wrapping/table behavior is stable.

Fail signal: bold/italic remain default white, headings become large title rows, or accents cause narrow-terminal overlap.

## Audit Focus

- No explicit card title rows.
- No complete theme picker/config persistence.
- Markdown renderer behavior and table wrapping do not regress.
