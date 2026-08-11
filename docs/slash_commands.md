# Slash commands

For an overview of Codex CLI slash commands, see [this documentation](https://developers.openai.com/codex/cli/slash-commands).

## Appearance

- `/theme` opens the unified Appearance picker. From there, choose either the UI palette or the syntax highlighting theme.
- `/ui-theme` is a shortcut to the UI palette picker for message, Markdown, and composer colors.

The two settings stay independent: the syntax theme is stored in `[tui].theme`, while the UI palette is stored in `[tui].ui_theme`.

## Hypo-Workflow helpers

VSP Codex includes a small helper parser for Hypo-Workflow command forms:

- `/hw:<command> [arguments]`
- `$hypo-workflow:<command> [arguments]`

The parser and command palette consume the command set from a verified installed Hypo-Workflow release manifest. The current contract includes `/hw:experiment`. Descriptor identity, SemVer, checksums, command bindings, uniqueness, bounded command counts, and Skill paths are validated before any route is exposed.

Commands not declared by the installed manifest are rejected. The parser also ignores unrelated slash commands such as `/review`, `/compact`, `/model`, `/quality`, and `/optimize`, as well as ordinary text.

Semantic workspaces are identified by regular, non-symlink `.pipeline/INDEX.md` and `.pipeline/cycles/` entries. Their SessionStart guidance reads the semantic index and current Cycle without requiring a legacy Host Contract refresh. Legacy manifest-based workspaces continue to use the read-only Host Contract v1 projection and fail closed when that projection is genuinely invalid.

Hypo-Workflow owns Workflow semantics, lifecycle, and every Workflow file write. VSP Codex provides Host UX only. Generic `request_user_input` / Ask UI, Plan/SubPlan freshness, Workflow Markdown fallback, Codex Hooks runtime, and installed plugin discovery remain available.

Consultation-style inputs are not Workflow execution commands. For `discussion`, `background`, `idea`, `complaint`, `question`, and `solution-discussion` style prompts, the agent should first produce a Mini-contract in the order `我的理解` -> `问题原因` -> `推荐方案` and avoid file edits or mutating commands. Clear imperative requests with concrete targets may still execute directly. After a plan or Mini-contract is shown, confirmations such as `可以`, `确认`, `OK`, `go ahead`, or `apply it` authorize only the displayed scope.

Integration sync is a target-repository compatibility process, not a user slash command.
