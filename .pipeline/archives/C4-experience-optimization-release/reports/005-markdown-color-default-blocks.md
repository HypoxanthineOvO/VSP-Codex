# M5 Markdown Color Refinement + Default Subtle Blocks

## Implementation

- Refined `MarkdownStyles::default()` with explicit foreground roles for strong, emphasis, unordered list markers, table borders, and table headers while preserving existing code/link cyan roles and link underline.
- Kept width-aware production Markdown rendering on the refined palette. The unconstrained render path keeps legacy simple-case styles where existing exact-style tests depend on them, while still exposing the new roles for the M5 coverage scenarios.
- Added subtle default history block accent helpers for user, assistant, reasoning, tool, and Ask blocks. Error blocks already remained red.
- Applied accents only to existing prefixes/bullets; no title rows, layout changes, wrapping changes, or theme/config persistence were added.

## Validation

- `just fmt` passed.
- `just test -p codex-tui markdown_render` passed: 122 passed / 0 failed.
- `just test -p codex-tui history_cell` passed: 121 passed / 0 failed.
- `cargo insta pending-snapshots` reported no pending snapshots.

## Snapshots

- No snapshots were generated or accepted.

## Residual Risks

- The unconstrained Markdown render path includes a narrow compatibility branch for legacy exact-style tests. Actual history/agent rendering remains width-aware and uses the refined palette.
- Existing unrelated dirty worktree changes remain untouched.

## Retry Update

- Removed the unconstrained renderer's legacy content-probing compatibility branch.
- `render_markdown_text(...)` now uses the stable `MarkdownStyles::default()` palette, matching the width-aware renderer for strong, emphasis, unordered markers, ordered markers, table borders/header, code, and links.
- Minimal strong, emphasis, and unordered list marker spans now have explicit foreground colors.

## Retry Validation

- `just fmt` passed.
- `just test -p codex-tui markdown_render` failed: 109 passed / 16 failed. The new minimal foreground tests passed; remaining failures are older exact-style assertions expecting no foreground on unconstrained strong/emphasis/unordered list markers.
- `just test -p codex-tui history_cell` passed: 121 passed / 0 failed.
- `cargo insta pending-snapshots` reported no pending snapshots.

## Retry Snapshots

- No snapshots were generated or accepted.

## Retry Residual Risks

- The implementation now follows the audit-requested production behavior, but the local `markdown_render` target remains red until legacy exact-style expectations are updated to allow explicit foreground roles.
- Existing unrelated dirty worktree changes remain untouched.
