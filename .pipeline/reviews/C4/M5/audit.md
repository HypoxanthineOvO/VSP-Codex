# C4/M5 Audit

## verdict

pass (retry)

## reviewed_refs

- `git diff -- codex-rs/tui/src/markdown_render.rs codex-rs/tui/src/history_cell/mod.rs codex-rs/tui/src/history_cell/messages.rs codex-rs/tui/src/history_cell/mcp.rs codex-rs/tui/src/history_cell/request_user_input.rs`
- `codex-rs/tui/src/markdown_render.rs`
- `codex-rs/tui/src/history_cell/mod.rs`
- `codex-rs/tui/src/history_cell/messages.rs`
- `codex-rs/tui/src/history_cell/mcp.rs`
- `codex-rs/tui/src/history_cell/request_user_input.rs`
- `codex-rs/tui/src/history_cell/tests.rs`
- `.pipeline/reviews/C4/M5/test.md`
- `.pipeline/reviews/C4/M5/implement.md`
- `.pipeline/reports/005-markdown-color-default-blocks.md`

## checks

- Markdown restrained palette: partially implemented. `MarkdownStyles::default()` now assigns visible foreground roles for emphasis, strong, unordered list markers, table borders, and table headers, while existing code/link cyan and blockquote green remain. However, the unconstrained path conditionally disables the new strong/emphasis/list marker colors for common inputs.
- Markdown wrapping/table semantics: no direct wrapping or table layout regression was found in the reviewed diff. Table borders are styled through existing border rendering, and header cell style is patched after width computation. The padding-style propagation in table rows appears color-only and does not change displayed text width.
- History block accents: implemented through existing prefixes/bullets in user, assistant, reasoning, tool, and Ask cells. I did not find newly added explicit title rows for default block distinction.
- Scope creep: no theme picker, theme config, persistence, config schema, or docs scope expansion was found in the M5 diff. There is unrelated behavior expansion in `request_user_input.rs` around structured answers/secret masking, but it is not the blocking issue for this audit.
- Test/report artifacts: test and implementation reports describe the intended red/green flow and list the filtered validation commands. Main-agent verification also reports `markdown_render` and `history_cell` filtered tests passing and no pending snapshots. I did not rerun tests per audit-worker instructions.

## findings

### Blocking: content-dependent legacy branch leaves common Markdown roles without visible colors

- File/lines: `codex-rs/tui/src/markdown_render.rs:107-117`, with test evidence at `codex-rs/tui/src/markdown_render.rs:2079-2144` and legacy expectations still present at `codex-rs/tui/src/markdown_render_tests.rs:629-642`.
- Severity: Critical for M5 acceptance.
- Finding: `MarkdownStyles::legacy_unconstrained()` conditionally resets `emphasis` to italic-only and `strong` to bold-only unless the raw input contains the literal substring `" ***"`. It also resets unordered list markers to the default style unless the input contains a backtick or `"]("`. This makes visible colors depend on incidental source text rather than Markdown semantics.
- Impact: common calls such as `render_markdown_text("**Strong**")`, `render_markdown_text("*Emphasis*")`, and `render_markdown_text("- item")` still render without an explicit foreground role in the unconstrained renderer. The new M5 tests pass because `inline_emphasis_styles_have_visible_distinct_colors` uses `plain **strong** *emph* ***both***`, which happens to include `" ***"`, and the list-marker test includes both backticks and a link, which keeps unordered marker coloring enabled. This does not fully satisfy the milestone requirement to fill visible colors for strong/emphasis/list markers in the Markdown renderer.
- Recommendation: remove the content-probing compatibility branch or replace it with a clearly scoped test-only compatibility strategy. The renderer's style defaults should be semantic and stable across constrained and unconstrained calls. Update the older exact-style tests to assert content/modifiers without preserving uncolored strong/emphasis/list markers.

## residual_risks

- I did not run tests, formatting, or snapshot commands by instruction. The validation status relies on the supplied main-agent results and the M5 reports.
- The worktree contains many unrelated dirty changes outside M5. This audit ignored them except where needed to understand current call paths.
- The new history accent helpers use a small set of cyan/green/magenta/yellow/red roles. That is not a blocker, but final visual review in a real terminal theme is still useful because terminal palettes can vary significantly.

## retry_verdict

pass

## retry_checks

- First blocker resolved. `codex-rs/tui/src/markdown_render.rs` no longer defines or calls `MarkdownStyles::legacy_unconstrained()`, and no content-probing fallback remains for `" ***"`, backticks, or `"]("`. `Writer::new(...)` now initializes `styles: MarkdownStyles::default()` for both unconstrained and width-aware render paths.
- Minimal Markdown paths now have stable visible foreground roles: `MarkdownStyles::default()` assigns `emphasis` green+italic, `strong` yellow+bold, and `unordered_list_marker` light blue. The minimal regression tests cover `render_markdown_text("**Strong**")`, `render_markdown_text("*Emphasis*")`, and `render_markdown_text("- item")`.
- M5 Markdown scope is coherent. The diff adds visible roles for strong/emphasis/list markers, table borders, and table headers while preserving existing code/link cyan and link underline. Existing wrapping and table layout tests remain content/width based, including narrow wrapping and pipe fallback cases; the reviewed table styling changes do not alter displayed text width.
- M5 history block accents are scoped to existing prefixes/bullets for user, assistant, reasoning, tool, Ask, and existing error blocks. I did not find new title rows added for default block distinction.
- No theme picker, theme config persistence, config schema, or product-doc expansion was found in the reviewed M5 diff.
- Test/report artifacts are sufficient for this retry. The test report records the minimal-path red retry, the implement report records removal of the legacy fallback, and the second test retry updates old exact-style expectations to the new visible-color semantics. Main-agent verification reports `just test -p codex-tui markdown_render` passing 125/125, `just test -p codex-tui history_cell` passing 121/121, and no pending snapshots.

## retry_evidence

- Read-only commands used: `git status --short`, `git diff`, `rg`, `sed`, and `nl`.
- `rg 'legacy_unconstrained|input\.contains\(" \*\*\*"\)|unordered_list_marker = Style::new\(\)|MarkdownStyles::legacy|contains\("`"\)|contains\("\]\("' ...` found matches only in the preserved original audit history and implement report, not in Markdown production/test code.
- I did not run tests, formatting, or snapshot commands per retry-worker instructions.
