# M3 Tool And Collaboration Output Rendering Boundaries

## Objective

Stop confusing duplicate translations of tool/status output and make tool/collaboration cards clearly distinguish commands, status rows, output summaries, translated content, and transcript expansion.

## Source Of Truth

- P1 issue: Issue 4 weird tool output translation and unclear tool cards.
- P2 technical solution: do not blindly append-translated technical/tool/status rows; preserve assistant prose translation where intended; localize lifecycle/status labels directly instead of machine-translating raw transcript text.
- Dependency: M2 should provide stable Subagent display names so lifecycle rows can render readable names rather than UUIDs.
- Relevant modules: `codex-rs/tui/src/history_cell/base.rs`, `codex-rs/tui/src/hypo_translate.rs`, `codex-rs/tui/src/exec_cell/render.rs`, `codex-rs/tui/src/history_cell/mcp.rs`, and collaboration/Subagent history cells.

## Subworker Assignment Plan

Status: authorized, strict worker separation.

- `test`
  - Scope: owns failing tests, fixtures, assertions, and snapshots proving exec/MCP/collaboration status rows are not blindly append-translated, while intended assistant prose translation still works.
  - Write boundary: test/snapshot files and `.pipeline/reviews/C7/M3/test.md` only.
  - Expected artifact: `.pipeline/reviews/C7/M3/test.md`.
- `implement`
  - Scope: owns production rendering and translation-source changes for exec cells, MCP cells, collaboration lifecycle rows, labels, and truncation/expansion affordances.
  - Must not create, edit, rewrite, or satisfy tests owned by `test`.
  - Expected artifact: `.pipeline/reviews/C7/M3/implement.md`.
- `audit`
  - Scope: read-only review of translation boundaries, technical literal preservation, hidden reasoning boundaries, and UI label clarity.
  - Must reject fake tests that only disable translation globally without proving intended prose translation still works.
  - Expected artifact: `.pipeline/reviews/C7/M3/audit.md`.

Workers must not revert user or other-worker edits. Required user decisions must use `request_user_input` when available. Destructive operations, system-level dependency installation, network/remote side effects, and release publication require explicit confirmation/authorization.

## Technical Solution

Change default translation policy for technical tool/status surfaces from append-translation to literal rendering. Exec stdout/stderr, MCP results, shell commands, paths, IDs, YAML, Markdown source, diffs, and code should not be auto-translated by default. Lifecycle rows should use first-class localized UI labels such as waiting/spawned/closed instead of appending machine-translated raw rows. Any translation retained for inspectable content must be explicit, labeled, and separated from the original evidence.

## Technical Route

1. Update `history_cell/base.rs` so `ExecCell` stdout/stderr is not wrapped as append-translated `ToolOutput` by default.
2. Update `history_cell/mcp.rs` so MCP results follow the same literal-by-default behavior.
3. Update collaboration cell translation wrapping so lifecycle/status rows are not duplicate-translated.
4. Preserve exact literals for commands, paths, IDs, code, Markdown source, YAML, diffs, and mixed technical text.
5. Improve labels for command run, waiting, output summary, truncation, and transcript expansion.
6. Add regression snapshots for `sed` over Markdown/YAML, Subagent waiting rows, and normal assistant prose translation.

Non-goals: full Subagent prompt/report viewer; M4 adds explicit original/translated Markdown inspection.

## Research Required

- Resolved by local evidence: translation source functions and append behavior have been identified.
- No external research required.

## Risks And Alternatives

- Risk: users who wanted translated logs lose automatic translated output. Mitigation: M4 provides explicit translated prompt/report inspection and prose translation remains separate.
- Risk: localized lifecycle labels conflict with existing English snapshots. Mitigation: update snapshots intentionally and keep debug details available.
- Rejected alternative: keep append translation and only improve prompts. Technical output is too brittle for blind translation.

## Validation Path

Required validation:

- `cargo test -p codex-tui history_cell hypo_translate -- --nocapture` or focused equivalent.
- Snapshot: command output is not duplicated with translated raw text.
- Snapshot: `Waiting for <agent>` renders as a clear lifecycle row with a display name and no appended translation.
- Snapshot/test: assistant prose translation still works when it is the intended surface.
- `cargo fmt --check`.
- `git diff --check`.

Manual acceptance:

- Run `sed`, `rg`, and `cat` over Markdown/YAML files and verify no confusing translated duplicate block appears.

Pass signal: technical/tool/status output remains literal by default, lifecycle rows are localized/rendered as UI labels without duplicate translation, and intended assistant prose translation still works.

Fail signal: command output is duplicated with appended machine translation, IDs/paths/code/YAML are mistranslated, or the fix disables all intended prose translation.

## Audit Focus

- Verify no hidden reasoning or tool output is mislabeled.
- Verify technical literals remain exact.
- Verify the change does not globally disable intended assistant prose translation.
- Verify lifecycle rows are localized/rendered through UI labels rather than blind append translation.
