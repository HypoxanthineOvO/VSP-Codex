# C7 P2 Decompose Checkpoint

Timestamp: 2026-05-29T21:00:31+08:00
Status: proposed

## P1 Summary

Cycle 7 is a focused experience optimization and bugfix sweep. P1 collected ten user-visible issues:

1. Long-session resume flickers 5-10 times before settling.
2. Scrolled-up terminal view can jump to the conversation start after refresh/reflow.
3. Internal/local/GitLab builds and external GitHub builds need different Subagent display-name pools.
4. Tool/collaboration output is blindly translated and rendered as confusing duplicate raw text.
5. Subagent prompts and reports need an interactive Markdown-rendered original/translated viewer.
6. Subagent lifecycle rows expose UUIDs, role and reasoning effort are treated as conflicting, and names are not leased.
7. `update_plan` is not proactive enough and should publish Cycle/Milestone/SubPlan structure with separate SubPlan and root Plan counts.
8. VSP releases need a startup update prompt and one-key standalone update path.
9. `/theme` and `/ui-theme` should be merged at the user-facing entry point while preserving separate config semantics.
10. Required Workflow confirmations and authorization-like interactions should use structured `request_user_input` cards instead of plain prose.

Carry-over items from C6:

- Full TUI command palette wiring for `/hw:*`.
- Clean the deprecated `turn_context.cwd` warning accepted as non-blocking in C6 M4.

## Milestone 1: Resume And Scrollback Stability

### Goal

Stabilize long-session resume and terminal scrollback behavior so opening or reading a long conversation does not flicker repeatedly or jump to the conversation start.

### Implementation Scope

- TUI initial history replay and resume paths.
- Resize/reflow scheduling and terminal scrollback rebuild logic.
- Tests/smoke scenarios for long replay, resize reflow, and user-scrolled history behavior.

Non-goals: redesign transcript storage, replace terminal backends, or remove existing scrollback insertion entirely.

### technical_solution

Treat resume/reflow as a bounded replay transaction. During initial resume, buffer history and coalesce redraw/reflow work until the transcript tail is stable. For post-resume refreshes, avoid unconditional full scrollback clear/rebuild when the user may be reading terminal scrollback. Add guardrails around `clear_scrollback_and_visible_screen_ansi()` and reflow triggers so terminal history is rebuilt only when necessary and never repeatedly in a tight startup window.

### technical_route

1. Inspect `chatwidget/replay.rs`, `app/thread_routing.rs`, `app/resize_reflow.rs`, `insert_history.rs`, `custom_terminal.rs`, and `tui.rs`.
2. Add instrumentation/test hooks for replay buffer boundaries, resize-reflow scheduling, and full scrollback clears.
3. Write failing tests that simulate long initial replay and repeated resize/refresh after replay.
4. Make initial replay flush once, then debounce or suppress redundant resize/reflow work that is caused only by replay settling.
5. Add a policy for user-scrolled history: skip or delay destructive scrollback rebuilds when live viewport anchoring cannot be preserved.
6. Preserve existing resize smoke coverage and add manual acceptance notes for VS Code Terminal and macOS Terminal over SSH.

### research_required

- Resolved by local evidence: relevant TUI replay/reflow modules and existing resize tests are in the repo.
- Implementation-time investigation remains needed for exact terminal-scroll detection limits; if terminal scroll position cannot be observed directly, use deterministic proxy tests for reflow/clear scheduling.

### risks_and_alternatives

- Risk: suppressing reflow can leave stale wrapping after real terminal resize. Mitigation: only suppress duplicate startup/replay reflows, keep real resize handling.
- Risk: terminal scroll state is not reliably observable. Mitigation: test the internal destructive-clear scheduling and use manual terminal scenarios for final acceptance.
- Alternative rejected: always keep clearing and rebuilding scrollback because it is simpler. The user-visible jump/flicker is the target bug.

### validation_path

- Add/run focused TUI tests around replay buffering and resize reflow, for example `cargo test -p codex-tui resize_reflow -- --nocapture` plus new replay/reflow tests.
- `cargo fmt --check`.
- `git diff --check`.
- Manual: resume a long session in VS Code Terminal and macOS Terminal over SSH; observe at most one initial settling pass and no jump to the session start while reading scrolled-up history.

### audit_focus

- Verify no real resize regressions.
- Verify no repeated full-screen clear/rebuild loop after resume.
- Verify manual acceptance covers both flicker and scrolled-up jump.

## Milestone 2: Subagent Identity, Role Policy, And Name Leases

### Goal

Make Subagents feel like named collaborators rather than UUID-backed internal threads, and allow role plus explicit reasoning effort to compose predictably.

### Implementation Scope

- Shared Subagent display-name resolver and active-name lease table.
- Lifecycle rows: spawned, sent input, waiting, completed, closed.
- Core spawn role/model/reasoning precedence.
- Role lock/default semantics and tool descriptions.
- Configurable/channel-specific fallback nickname pools.

Non-goals: full prompt/report viewer, which is Milestone 4.

### technical_solution

Introduce one authoritative Subagent identity resolver used by both lifecycle history rows and the active Subagents panel. At first sight of a thread id, assign a stable display name from explicit metadata, role candidates, configured candidates, channel defaults, or built-ins. Lease names while active and release them on terminal lifecycle state according to a documented policy. In core spawn handling, change role model/reasoning from "always locked override" to "role supplies defaults; explicit request overrides unless locked".

### technical_route

1. Extract or centralize fallback nickname logic currently split between `multi_agents.rs` and `active_subagents.rs`.
2. Add a lease table keyed by `ThreadId`, with release on close/shutdown/not_found and a conservative policy for completed-but-inspectable agents.
3. Replace lifecycle-row UUID fallback with leased display names; keep raw IDs only in detail/search/debug contexts.
4. Add role/default/lock fields or equivalent config semantics in core role handling.
5. Change spawn config application order to role defaults first, then explicit `model`/`reasoning_effort` overrides when allowed.
6. Update tests currently asserting role always overrides requested reasoning.
7. Add channel/default nickname-pool selection hooks without hardcoding final private internal name lists into core behavior.

### research_required

- Resolved by local evidence: display-name and role override code paths have been identified.
- Internal VSP Lab full name pool is not required for implementation because the resolver will support configurable/build-channel candidates; fixture tests can use the names already provided in P1.
- Full-history fork override relaxation is deferred unless explicitly requested; default plan keeps full-history forks inherited and rejects role/model/effort overrides there.

### risks_and_alternatives

- Risk: name leases create duplicate or stale names after abnormal shutdown. Mitigation: release on terminal states and use deterministic fallback if pool is exhausted.
- Risk: role lock semantics break existing role configs. Mitigation: default existing roles to default-overridable unless explicit lock fields are present, and update descriptions accordingly.
- Alternative rejected: only hide UUIDs in one UI row. That would not fix active panel/history consistency.

### validation_path

- `cargo test -p codex-tui multi_agents active_subagents -- --nocapture` with new snapshots for spawn/wait/close labels.
- `cargo test -p codex-core subagent_notifications -- --nocapture` with updated role/effort tests.
- Manual: spawn several Subagents with roles and explicit reasoning efforts; verify names stay stable, no UUID primary labels appear, and names can be reused after release.

### audit_focus

- Verify display name never becomes the thread id in primary UI when fallback candidates exist.
- Verify explicit reasoning effort is not silently discarded.
- Verify internal IDs remain available in details for troubleshooting.

## Milestone 3: Tool And Collaboration Output Rendering Boundaries

### Goal

Stop confusing duplicate translations and make tool/collaboration cards clearly distinguish status, command output, translated content, and transcript expansion.

### Implementation Scope

- Exec/tool output translation source selection.
- MCP tool result translation behavior.
- Collaboration lifecycle translation behavior.
- Tool card labels and truncation/expansion affordances.

Non-goals: full Subagent prompt/report viewer, which depends on this boundary and is Milestone 4.

### technical_solution

Change default translation policy from "append translated tool output" to "do not translate technical/tool/status rows by default." Preserve reasoning/prose translation where intended, and add an explicit/labeled translation view only for content that benefits from translation. Lifecycle rows should be localized or rendered directly through UI labels, not machine-translated as raw transcript text.

### technical_route

1. Update `history_cell/base.rs` so `ExecCell` stdout/stderr is not wrapped as append-translated `ToolOutput` by default.
2. Update `history_cell/mcp.rs` similarly for MCP results.
3. Update collaboration cell translation wrapping so lifecycle/status rows are not duplicate-translated.
4. Keep code, paths, YAML, Markdown source, diffs, and command output literal.
5. Improve labels for command run, waiting, output summary, truncation, and transcript expansion.
6. Add regression snapshots for `sed`/Markdown output and Subagent wait rows.

### research_required

- Resolved by local evidence: translation source functions and append behavior have been identified.

### risks_and_alternatives

- Risk: removing automatic tool-output translation disappoints users who wanted translated logs. Mitigation: provide explicit translation in the prompt/report viewer and keep prose translation separate.
- Risk: localized lifecycle labels conflict with existing English snapshots. Mitigation: update snapshots intentionally and keep raw IDs/details in debug contexts.
- Alternative rejected: keep append translation but improve prompt. Technical output is too brittle for blind translation.

### validation_path

- Add/run TUI history cell and translation tests, for example `cargo test -p codex-tui history_cell hypo_translate -- --nocapture`.
- Snapshot: command output is not duplicated with translated raw text.
- Snapshot: `Waiting for <agent>` row renders as a clear user-facing lifecycle row without appended translation.
- Manual: run `sed`, `rg`, `cat` over Markdown/YAML and verify no confusing translated duplicate block.

### audit_focus

- Verify no hidden reasoning or tool output is mislabeled.
- Verify technical literals remain exact.
- Verify the change does not disable intended assistant prose translation.

## Milestone 4: Subagent Prompt And Report Viewer

### Goal

Let users inspect the full Prompt sent to a Subagent and the Subagent report in a readable Markdown-rendered viewer, with original and translated forms available.

### Implementation Scope

- Capture/link full Subagent prompt bodies and final report text.
- TUI detail/history entry point for Prompt/report inspection.
- Markdown rendering for original and translated content.
- Lazy translation to avoid unnecessary background work.

Non-goals: changing Subagent scheduling or role semantics, handled in Milestone 2.

### technical_solution

Extend Subagent state/history metadata to retain or locate the full prompt and final response. Add an interactive viewer reachable from active Subagents and/or lifecycle history rows. The viewer renders Markdown using the existing renderer and provides original/translated panes or tabs. Translation should be explicit, labeled, and code-fence/path aware; it should not mutate the original evidence text.

### technical_route

1. Identify whether prompt/report should be sourced from `CollabAgentState`, subagent thread transcript, or a new lightweight main-thread artifact.
2. Add bounded prompt/report references to lifecycle state where needed.
3. Implement viewer UI using existing selection/overlay/pager patterns and `markdown_render`.
4. Add original/translated modes; generate translation lazily when opened.
5. Ensure code fences, YAML, paths, and command snippets remain literal or clearly protected.
6. Add snapshots for prompt preview, full prompt, report, and translated report.

### research_required

- Resolved by local evidence for candidate state/transcript locations.
- Implementation-time choice remains for the least invasive data source; if final report retrieval from subagent transcript is incomplete, use a dedicated stored report reference.

### risks_and_alternatives

- Risk: prompt/report text can contain sensitive content. Mitigation: do not add new broad persistent logs; reuse transcript/artifact references and keep original access explicit.
- Risk: translation corrupts operational evidence. Mitigation: always keep original visible and label translation as derived.
- Alternative rejected: only show truncated prompt previews. User specifically requested full Markdown-rendered inspection.

### validation_path

- TUI snapshots for viewer entry and Markdown rendering.
- Tests that lifecycle rows remain concise while viewer exposes full content.
- Manual: spawn a Subagent with a Markdown prompt and report, open viewer, switch original/translation, verify headings/lists/code blocks render correctly.

### audit_focus

- Verify no prompt/report content is exposed in unintended surfaces.
- Verify Markdown rendering handles narrow terminals without overlap.
- Verify translation is opt-in/lazy and clearly labeled.

## Milestone 5: Structured Interaction Gates And Authorization Cards

### Goal

Make required Workflow decisions, confirmations, and authorization-like prompts use structured `request_user_input` cards with clear TUI rendering instead of ordinary chat text.

### Implementation Scope

- Workflow interaction-gate policy for required decisions.
- Confirmation card templates for continue/stop, approve/reject, choose scope, and phase transitions.
- Authorization card guidance and boundaries between `request_user_input`, `request_permissions`, and approval overlays.
- TUI Request User Input card polish, localization, and snapshots.
- Prompt/runtime guidance that Workflow gates must use structured questions when the tool is available.

Non-goals: weakening sandbox, network, destructive-action, remote-resource, or external-side-effect approval enforcement; replacing existing exec/patch approval overlays.

### technical_solution

Define a small Workflow interaction-gate contract: informational status needs no answer, optional clarification can remain conversational, required user decisions use `request_user_input`, and true permission expansion uses `request_permissions` or the existing approval overlay. Add reusable confirmation and authorization-like question templates with concise summaries, explicit choices, cancellation/rejection paths, and zh-CN wording. Update Workflow runtime guidance so required confirmations are not asked only in assistant prose when `request_user_input` is available.

### technical_route

1. Inspect `protocol/src/request_user_input.rs`, `protocol/src/request_permissions.rs`, `tui/src/bottom_pane/request_user_input/`, and `tui/src/bottom_pane/approval_overlay.rs` to map current capabilities.
2. Add a Workflow interaction-gate classification in prompt/runtime guidance: status, optional clarification, required decision, and permission/authorization.
3. Add or document structured templates for plan confirmation, phase transition confirmation, scope selection, release/update confirmation, reject/revise, and cancel/abort.
4. Polish Request User Input card rendering for confirmation/authorization use cases: clear title, description, option labels, selected/default states, cancellation, and narrow-terminal behavior.
5. Keep `request_permissions` and approval overlays authoritative for real permission grants; use `request_user_input` only for user decisions and intent confirmation.
6. Add tests/snapshots proving Workflow gates are represented as structured cards and localized correctly.

### research_required

- Resolved by local evidence: `request_user_input` protocol, TUI overlay, zh-CN snapshots, `request_permissions`, and approval overlay all exist in the repo.
- No external service research required.
- Implementation-time design remains for exact card copy and visual affordances, but the protocol surface is already available.

### risks_and_alternatives

- Risk: overusing cards makes ordinary chat feel bureaucratic. Mitigation: apply strict card requirements only to required Workflow/high-impact decisions, not low-stakes clarifications.
- Risk: users confuse confirmation with permission grant. Mitigation: card copy and routing must distinguish "confirm intent" from "grant permission", and true permissions remain in approval/permissions flows.
- Risk: model can still ask plain-text confirmations. Mitigation: update Workflow guidance, add tests around routing/guidance text, and make P3 prompts explicitly require `request_user_input` for gates.
- Alternative rejected: rely only on prompt wording. That would not give consistent TUI cards or testable gate behavior.

### validation_path

- `cargo test -p codex-core request_user_input workflow_routing -- --nocapture` with new guidance/fixture cases.
- `cargo test -p codex-tui request_user_input approval_overlay -- --nocapture` with confirmation/authorization zh-CN snapshots.
- Snapshot: plan confirmation, release-update confirmation, and scope-choice gates render as cards with explicit choices rather than assistant-only prose.
- Manual: run a P2 confirmation and a release/update prompt; verify the user answers through a card and can reject/cancel clearly.

### audit_focus

- Verify no approval or permission enforcement is bypassed.
- Verify required Workflow decisions use structured cards when available.
- Verify card language clearly separates confirmation, authorization, and ordinary clarification.
- Verify ordinary non-Workflow chats are not forced into unnecessary cards.

## Milestone 6: Workflow Plan/SubPlan Live Surface

### Goal

Make `update_plan` a trustworthy live Workflow surface with Cycle/Milestone/SubPlan structure, proactive refresh triggers, separate progress counts, and full `/hw:*` command palette wiring.

### Implementation Scope

- Structured `PlanState` conventions for Hypo-Workflow Cycles.
- TUI Plan panel rendering and status/terminal progress counts.
- Plan freshness reminders beyond resume/compact.
- Workflow bridge from `.pipeline` state to visible Plan where practical.
- Full TUI command palette entries for `/hw:*`.
- Integration with Milestone 5's structured confirmation gates for plan/phase decisions.
- C6 carry-over cleanup for deprecated `turn_context.cwd` warning.

### technical_solution

Use existing `PlanState { title, summary, items, subplan }` as the canonical live UI shape for Workflow plans. Root items represent Milestones; the active Milestone carries an expanded SubPlan; future/completed Milestones are collapsed. Progress displays two separate values: current SubPlan progress and root Plan progress. Add Workflow-specific freshness triggers so the model refreshes the plan at phase, issue, edit, validation, worker, resume, compact, and final-answer boundaries. Plan confirmation and phase-transition decisions use Milestone 5's structured interaction gates instead of plain prose.

### technical_route

1. Define a Hypo-Workflow PlanState builder/convention from Cycle metadata and current milestone/step.
2. Update `plan_panel.rs`, `turn_runtime.rs`, and status surfaces to compute/display separate SubPlan and root Plan counts.
3. Extend plan freshness gate beyond resume/compact/tool-burst to Workflow state advances and stale final response cases.
4. Add prompt/runtime guidance that `update_plan` is mandatory for Workflow phase/issue/worker boundaries.
5. Route required Plan confirmations and phase decisions through the structured card templates from Milestone 5.
6. Wire `/hw:status`, `/hw:plan`, `/hw:resume`, and related entries into command palette without misrouting non-Workflow slash commands.
7. Clean the non-blocking deprecated `turn_context.cwd` warning by using the selected turn environment cwd where appropriate.

### research_required

- Resolved by local evidence: `PlanState` already supports one-level SubPlan and TUI has a persistent plan panel.
- Resolved by local evidence: Milestone 5 will provide the structured interaction contract that Plan gates consume.
- No external research required.

### risks_and_alternatives

- Risk: too many plan updates become noise. Mitigation: stricter behavior only in Workflow context; ordinary small chats keep current lightweight rules.
- Risk: `.pipeline` bridge and model-authored plan diverge. Mitigation: use `.pipeline` state as recovery/freshness evidence, but keep explicit `update_plan` as UI source.
- Alternative rejected: flat checklist for C7. It cannot represent Milestone/SubPlan structure or separate counts.

### validation_path

- `cargo test -p codex-core plan_update_gate workflow_routing -- --nocapture` with new freshness cases.
- `cargo test -p codex-tui plan_panel workflow_slash_routing -- --nocapture` with current expanded/future collapsed snapshots.
- Snapshot: `2/5 | 0/4`-style semantic split, not combined `2/9`.
- Manual: run a multi-issue planning discussion and short execution; Plan panel updates at each phase/issue boundary without user reminders.

### audit_focus

- Verify Plan updates do not spam ordinary Default chats.
- Verify root Plan and SubPlan counts are never mixed.
- Verify Plan confirmations use Request Input cards when required.
- Verify `/hw:*` command palette wiring does not catch `/review`, `/compact`, `/model`, or ordinary text.

## Milestone 7: VSP Release Channels, Naming Metadata, And Updater

### Goal

Provide a VSP-specific release/update path and release-channel metadata, including internal/external Subagent name defaults and a startup update prompt with one-key standalone update.

### Implementation Scope

- VSP release manifest schema and parser.
- Channel/target/version selection.
- VSP update prompt data source and CLI update action.
- Standalone archive download, checksum validation, staged install, and `current` switch.
- Release-channel metadata used by Subagent default name pools.

Non-goals: silent background auto-update by default; private GitLab token management in C7 unless separately approved.

### technical_solution

Build a VSP updater parallel to the official OpenAI updater. The TUI checks a VSP manifest cache/source, shows a VSP-branded update prompt using the structured interaction-card contract from Milestone 5, and exits to a CLI update action when the user selects update now. The CLI downloads the selected archive, verifies SHA-256, unpacks into the standalone releases directory, atomically updates `current`, and asks the user to restart. The same channel metadata identifies internal/local/GitLab builds versus external GitHub builds for default Subagent name pools.

### technical_route

1. Add `vsp_release_manifest` structs and version/target/channel selection tests.
2. Add config/build metadata for release channel and manifest URL, with safe defaults for local/debug.
3. Add a VSP update source that does not call `openai/codex` release APIs.
4. Add VSP standalone update action and CLI path guarded to standalone release installs.
5. Reuse the existing update prompt shape through Milestone 5's structured confirmation card semantics, replacing release notes URL, command text, dismissal cache, and update action.
6. Add installer/release metadata so installed builds know channel, target, version, and manifest URL.
7. Integrate channel default Subagent name pools with Milestone 2's resolver, preserving user-configured nicknames above channel defaults.

### research_required

- External release hosting is a hard integration point. C7 scope is proposed as manifest-driven and testable against local/mock HTTP or file URLs.
- GitLab private-token auth is deferred from C7 unless the user explicitly adds it; C7 assumes release assets or manifest URLs are reachable in the target environment.
- GitHub external release naming remains supported through channel metadata, not through semver/tag changes.

### risks_and_alternatives

- Risk: auto-update security. Mitigation: one-key prompted update by default, checksum verification required, silent update not default.
- Risk: private GitLab assets are not reachable. Mitigation: document accessible Release/Package Registry requirement and fail with clear message.
- Risk: update breaks VSCode integration. Mitigation: standalone `current` switch preserves `~/.local/bin/vsp-codex` and VSCode `cliExecutable`.
- Alternative rejected: re-enable official updater. It targets OpenAI release channels, not VSP.

### validation_path

- Unit tests for manifest parse, target selection, channel selection, version comparison, and checksum mismatch.
- CLI integration test using a local mock archive/manifest.
- TUI snapshot/test for VSP update prompt and dismissal behavior.
- Manual: install older standalone build, point to newer mock/published manifest, update, restart, verify `vsp-codex --version` and `vsp-codex vscode status --json`.

### audit_focus

- Verify updater never calls official OpenAI release endpoints in VSP mode.
- Verify the update prompt is a clear confirmation card, not a silent update or plain chat question.
- Verify checksum failure aborts without switching `current`.
- Verify release-channel name defaults do not override explicit user nicknames.

## Milestone 8: Appearance Entry Point And Final UX Polish

### Goal

Unify theme/appearance entry points, polish command descriptions and docs, and run a final experience-focused regression pass across C7.

### Implementation Scope

- `/theme` becomes or opens a unified Appearance/Theme panel.
- The panel has separate UI palette and syntax theme sections.
- `/ui-theme` remains as a direct shortcut to UI palette.
- Docs and command palette labels explain the distinction.
- Final release notes and targeted regression checklist.

### technical_solution

Keep `[tui].theme` and `[tui].ui_theme` separate but merge the user-facing appearance entry. Reuse existing syntax theme picker and UI theme picker functionality under a parent Appearance view, or implement a lightweight two-section selector that delegates to existing pickers. Ensure Markdown, Subagent viewer, Plan panel, and tool cards all use the active UI palette consistently.

### technical_route

1. Update slash command and command palette descriptions: "Appearance", "UI palette", "Syntax theme".
2. Add unified `/theme` behavior or an `/appearance` alias, with `/theme` as the natural primary entry.
3. Preserve `/ui-theme` as a direct route to UI palette.
4. Add tests proving UI palette changes Markdown/message colors but does not mutate syntax theme, and syntax theme changes code highlighting but does not mutate UI palette.
5. Update `README.md`, `docs/vsp-codex.md`, `docs/install.md`, and release notes for C7 user-facing changes.
6. Run final focused regression commands from M1-M7 plus formatting and diff checks.

### research_required

- Resolved by local evidence: existing `/theme`, `/ui-theme`, config keys, and persistence paths have been inspected.

### risks_and_alternatives

- Risk: changing `/theme` surprises users who expect direct syntax-theme picker. Mitigation: the unified panel should make Syntax Theme one action away and keep `/ui-theme` shortcut.
- Risk: appearance panel becomes too broad. Mitigation: only include UI palette and syntax theme in C7.
- Alternative rejected: merge the two config keys. Syntax theme and curated UI palette have different cardinality and semantics.

### validation_path

- `cargo test -p codex-tui theme ui_theme command_palette -- --nocapture`.
- Snapshots for Appearance panel and command palette labels.
- Manual: open `/theme`, understand both sections, change UI palette, change syntax theme, verify both persist independently.

### audit_focus

- Verify config compatibility for existing `[tui].theme` and `[tui].ui_theme`.
- Verify no layout overlap in narrow terminals.
- Verify docs do not overpromise silent auto-update or incomplete Workflow command behavior.

## Proposed Execution Order

1. M1 Resume And Scrollback Stability
2. M2 Subagent Identity, Role Policy, And Name Leases
3. M3 Tool And Collaboration Output Rendering Boundaries
4. M4 Subagent Prompt And Report Viewer
5. M5 Structured Interaction Gates And Authorization Cards
6. M6 Workflow Plan/SubPlan Live Surface
7. M7 VSP Release Channels, Naming Metadata, And Updater
8. M8 Appearance Entry Point And Final UX Polish

## Worker Separation And P3 Notes

- Keep strict worker separation for each implementation milestone: `test`, `implement`, `audit`.
- Generated prompts in P3 must include the standard Subworker Assignment Plan.
- Per the user's C6 instruction, after writing prompts in P3, use a Subagent to audit the generated prompts before execution starts.
- P2 does not start implementation. P3 should begin only after the user approves this decomposition and route.
