# C7 Planning Seed: Experience Optimization And Bugfix Sweep

Timestamp: 2026-05-29T20:06:25+08:00
Status: seed

## User Intent

Plan the next Cycle as a complete experience optimization pass that fixes known bugs and current usability problems in one focused round.

## P1 Discover Interaction Protocol

The user wants C7 P1 to run as an iterative issue-discussion queue:

1. Discuss the current known pitfalls and candidate problem areas first.
2. The user sends one issue at a time.
3. Discuss that issue for several rounds until the desired behavior, suspected cause, priority, and validation signal are clear.
4. Record the issue into the C7 planning notes.
5. Repeat for the next issue.
6. Do not enter P2 Decompose until the user explicitly says OK, enough, or enter P2.

Priority axis selected in P1: correctness and stability first.

Acceptance scenario: after release, users can use VSP-Codex smoothly and the interface has no obvious problems.

## Carry-Over From C6

These items were explicitly deferred as non-blocking follow-ups in C6 and should be considered for C7 scope:

1. Full TUI command palette wiring for `/hw:*`
   - C6 only exposed helper-level Workflow command discoverability.
   - C7 should decide whether to wire actual palette entries, routing, labels, and non-Workflow fallback behavior.

2. Clean deprecated `turn_context.cwd` warning in `session/mod.rs`
   - M4 accepted the behavior as guarded and tested.
   - C7 can remove the warning by switching ordinary conversation capture to the selected turn environment cwd.

## Candidate Theme

"Experience optimization and bugfix sweep" should likely combine:

- User-reported current bugs and sharp edges.
- Small correctness fixes that were accepted as non-blocking in prior Cycles.
- Workflow-native UX polish exposed by C6.
- Focused regression coverage for fixed issues.
- Documentation or help text only where it directly improves usability.

## Planning Notes

- Keep this as a single Cycle only if the bug list is coherent and testable.
- If the list becomes broad, decompose into small milestones by user-facing workflow area rather than by code layer.
- Preserve C6's prompt-audit default: generated Workflow prompts should be audited by a Subagent before execution.
- Current project config already authorizes Codex execution subworkers for `/hw:start` and `/hw:resume` with strict worker separation.

## Open Questions For P1 Discover

1. Which concrete bugs or annoyances has the user already observed?
2. Is the priority "visible TUI experience first" or "correctness/stability first"?
3. What manual acceptance scenario should prove the Cycle improved the experience?

## Issue Queue

### Issue 1: Long Session Resume Flicker

Status: discussing
Priority: high
Category: TUI stability / resume UX

User observation:

- Running `vsp-codex resume` into a long session makes the TUI flicker aggressively about 5-10 times before stabilizing.
- It does not crash.
- The UI remains interactive during the flicker.
- The user suspects a cache or redraw mechanism.
- Observed paths include `vsp-codex resume` picker selection and in-TUI `/resume`; the user is not sure whether all resume modes reproduce.
- Recent correct content appears quickly, but the UI keeps flickering afterward. User intuition: the app may still be loading or rewriting earlier scrollback/history above the visible tail.
- Seen in VS Code Terminal and macOS Terminal SSHing into Ubuntu.
- Recent affected sessions have at least dozens of turns.

Initial technical notes:

- Relevant paths likely include `codex-rs/tui/src/app/event_dispatch.rs`, `codex-rs/tui/src/app/thread_routing.rs`, `codex-rs/tui/src/session_resume.rs`, `codex-rs/tui/src/chatwidget/replay.rs`, `codex-rs/tui/src/chatwidget/protocol.rs`, and `codex-rs/tui/src/app/resize_reflow.rs`.
- The TUI already has `BeginInitialHistoryReplayBuffer` / `EndInitialHistoryReplayBuffer` to batch initial resume replay, so flicker may mean some resume-time events bypass the buffer or trigger repeated resize/reflow/frame scheduling after the buffer flush.
- A plausible investigation route is to instrument or test frame scheduling, terminal clear/reinsert calls, resize reflow debounce, and replay event handling during long-session resume.
- Because the visible tail is already correct while flicker continues, a likely focus is post-tail terminal scrollback insertion or repeated `clear_scrollback_and_visible_screen_ansi` / resize reflow after initial replay.
- VS Code Terminal over SSH may make repeated scroll-region writes, clear-screen sequences, or delayed terminal size settling visibly flicker.

Desired outcome:

- Resuming a long session should visibly settle in one coherent load/replay phase, without repeated full-screen flicker.
- The fix should preserve resume interactivity and not hide real in-progress replay state.

Validation ideas:

- Add a focused regression test around long replay buffering or resize reflow scheduling if feasible.
- Add a manual TUI scenario: resume a long session and observe at most one initial repaint/settling pass before stable interaction.
- If automated visual testing is practical, capture frame counts or terminal clear/reflow calls during resume.

### Issue 2: Scrolled-Up Terminal Jumps To Conversation Start After Refresh

Status: discussing
Priority: high
Category: TUI stability / scrollback anchoring
Related: Issue 1

User observation:

- In the terminal, scroll upward with the mouse to inspect earlier conversation history.
- After a short time, when a refresh happens, the whole TUI suddenly jumps to the beginning of the session.
- This is disruptive because reading older context is not stable.
- The jump destination is the beginning of the conversation, not the latest bottom/input area.
- It may happen while the model/tool output is still streaming, after terminal/VS Code panel size changes, or under other refresh conditions.
- The user usually scrolls only partway upward, not all the way to the top.

Initial technical notes:

- This likely shares root causes with Issue 1: terminal scrollback insertion, resize reflow, and replay/rebuild behavior.
- Relevant code paths include `codex-rs/tui/src/app/resize_reflow.rs`, `codex-rs/tui/src/insert_history.rs`, `codex-rs/tui/src/custom_terminal.rs`, `codex-rs/tui/src/tui.rs`, and existing resize smoke tests under `codex-rs/tui/tests/suite/resize_reflow.rs`.
- The resize reflow path currently clears scrollback and visible screen through `clear_scrollback_and_visible_screen_ansi()` before replaying rendered transcript lines. That is inherently risky when the user has manually scrolled into terminal scrollback.
- Existing tests focus on composer row and visible sentinel anchoring after resize, but they do not appear to cover the case where the user is intentionally scrolled above the live viewport and a refresh/reflow occurs.
- Because the destination is the conversation start, one strong hypothesis is that a full scrollback rebuild causes the terminal's historical scroll position to point at the rebuilt buffer top instead of preserving the user's previous relative reading anchor.
- Streaming consolidation and terminal resize are both candidate triggers because they can force source-backed reflow after transient rows or dimensions change.

Desired outcome:

- If the user has scrolled upward to read history, normal TUI refreshes should not unexpectedly jump to the start of the session.
- If a full scrollback rebuild is unavoidable, the UI should avoid surprising jumps or should limit rebuilds to cases where terminal position can be restored.

Validation ideas:

- Add a regression test or smoke scenario for "user scrolled into history + refresh/reflow occurs".
- Prefer a deterministic unit/integration test around reflow scheduling and scrollback clearing if terminal scroll position cannot be observed reliably in ordinary tests.
- Manual acceptance: scroll upward in a long session, wait through status/refresh/reflow activity, and verify the visible history position remains stable enough to keep reading.

### Issue 3: Split Internal And External Release Naming Schemes

Status: discussing
Priority: medium
Category: release experience / version branding

User request:

- GitLab release versions and local versions should use the previous VSP Lab person-name combination.
- GitHub Release external versions should use the current ancient-person-name combination.
- Clarification: "version" here means the Subagent displayed person-name pool, not semver/tag/release-title versioning.
- Internal VSP Lab name examples: `娄鑫`, `李超凡`, `陈泉宇`.
- External GitHub Release should keep the current ancient-person-name pool.

Initial technical notes:

- Current workspace version is centralized in `codex-rs/Cargo.toml` as `0.134.0-vsp.5`.
- Current README/docs install examples reference GitLab raw preview artifacts under `.pipeline/release/C5-preview/` and version label `0.134.0-vsp.5-preview`.
- Current ancient-person-name scheme is visible in Subagent fallback name pools, for example `codex-rs/tui/src/chatwidget/active_subagents.rs`.
- No `.gitlab` release automation file was visible in the quick scan, so C7 should discover whether GitLab publishing is manual, external, or stored elsewhere.

Desired outcome:

- Internal/local distribution surfaces clearly use the VSP Lab naming scheme.
- External GitHub Release surfaces clearly use the ancient-person naming scheme.
- The two schemes do not corrupt semantic version comparisons, update checks, installer paths, or package names.
- Subagent display names should be channel-specific without forcing users to edit config manually after install.

Open design points:

- Decide how the runtime distinguishes internal GitLab/local builds from external GitHub Release builds: compile-time feature, build env, release-channel config, or install-time generated default config.
- Clarify the full internal VSP Lab name pool beyond `娄鑫`, `李超凡`, and `陈泉宇`.
- Preserve user-configured Subagent nicknames as higher priority than channel defaults.
- Decide whether docs should call this "channel default Subagent names" to avoid confusion with semver release versions.

Validation ideas:

- Add tests for internal/local channel default Subagent name pool vs external GitHub Release pool.
- Verify user-configured Subagent nicknames still override channel defaults.
- Confirm semver, tags, update checks, installer paths, and artifact filenames remain unaffected unless a later P2 decision explicitly changes them.

### Issue 4: Weird Tool Output Translation And Unclear Tool Cards

Status: discussing
Priority: high
Category: TUI readability / translation boundary / tool output rendering

User observation:

- Tool-related rows are sometimes translated in an awkward duplicated form, for example:
  - `• Waiting for 019e7313-80f9-7283-88bd-cbd2ee9ec89f`
  - `└ • 等待 019e7313-80f9-7283-88bd-cbd2ee9ec89f`
- Command output cards can show original command output plus a translated duplicate block, for example `sed` output over a Markdown planning file.
- The translated block may not render Markdown well and may mistranslate technical text, paths, English phrases, or mixed language content:
  - `No 和 .gitlab ...`
  - `do not 会破坏 ...`
  - `ancient-person-name` rendered as odd Chinese wording.
- The user suspects these may be thought-process output, but the UI does not make the card type or purpose clear.

Initial technical notes:

- This is likely not model hidden reasoning. It appears to be TUI history-cell rendering for tool execution / background wait / command output.
- `codex-rs/tui/src/history_cell/base.rs` has `exec_translation_source(...)`, which collects ExecCell stdout/stderr and wraps it as `TranslationScope::ToolOutput` with `TranslationDisplay::Append`.
- `TranslatedHistoryCell::Append` appends translated raw lines under the original cell with a `└` prefix.
- This explains why a `sed` output block can appear once as original truncated command output and again as translated duplicate content.
- `McpToolCallCell::translation_source(...)` also wraps MCP tool results as `ToolOutput + Append`.
- `wrap_collab_translated(...)` wraps collaboration/Subagent transcript rows as `CollabTranscript + Append`, which can translate ID-only worker lifecycle rows such as `Waiting for <agent-id>`.
- Relevant files include `codex-rs/tui/src/history_cell/base.rs`, `codex-rs/tui/src/hypo_translate.rs`, `codex-rs/tui/src/exec_cell/render.rs`, `codex-rs/tui/src/history_cell/mcp.rs`, and tool/background terminal history cells.

Desired outcome:

- Tool output should not be blindly translated by default.
- Commands, paths, IDs, code, Markdown source, YAML, diffs, and mixed technical text should remain literal unless the user explicitly opens a translation view or asks for translation.
- If translation is kept for selected tool outputs, the UI should clearly label it as a translation and avoid duplicating trivial or already-understandable content.
- Tool cards should make their purpose clear: command run, waiting for background worker, output summary, truncated output, and transcript expansion should be visually understandable.

Validation ideas:

- Add tests proving ExecCell command output is not auto-translated by default.
- Add tests that reasoning summaries can still be translated/replaced when intended.
- Add tests or snapshots for background wait/worker rows so they do not produce duplicated translated lines for IDs-only text.
- Manual acceptance: running `sed`, `rg`, `cat`, YAML/Markdown reads, and worker wait messages should not create confusing translated duplicate blocks.

### Issue 5: Interactive Subagent Prompt And Report Viewer With Markdown Translation

Status: discussing
Priority: high
Category: Subagent UX / reviewability / translation and Markdown rendering
Related: Issue 4

User request:

- For Subagent calls, the user wants to interactively see the translated Prompt sent to the Subagent and the Subagent's returned report.
- These Prompt/report bodies are Markdown, so they should be rendered well instead of shown as raw unformatted text.

Initial technical notes:

- Current collab/Subagent history rows mostly show lifecycle summaries such as Spawned/Sent input/Waiting/Finished waiting.
- `multi_agents.rs` currently truncates Prompt previews with `COLLAB_PROMPT_PREVIEW_GRAPHEMES`; full Prompt body is not presented as a first-class rendered Markdown card.
- Active Subagent detail popup shows metadata and a truncated `recent input`, but not a full rendered Prompt/report view.
- Subagent returned reports are likely available through subagent thread transcript/history rather than a dedicated main-thread Prompt/Report card.
- Issue 4 means lifecycle/status rows should not be blindly translated, but actual Subagent Prompt/report content should support translated, Markdown-rendered inspection.

Desired outcome:

- Subagent lifecycle rows stay concise and are not polluted by automatic duplicate translations.
- Users can open a Subagent detail view from the active panel or history row and inspect:
  - original Prompt sent to the Subagent
  - translated Prompt
  - Subagent report/response
  - translated report/response
- Prompt/report views render Markdown properly, including headings, bullets, code blocks, paths, and links.
- Original content remains available because Prompt/report text can be operational evidence.

Open design points:

- Decide the interaction entry point: active Subagents detail popup, history row action, `/agent` picker, transcript overlay, or a dedicated Subagent report panel.
- Decide whether translated Prompt/report is generated eagerly when the row is created or lazily when the user opens the detail view.
- Decide how to preserve raw Markdown and render translated Markdown without corrupting code fences, paths, YAML, or command snippets.
- Decide how reports are located: from `CollabAgentState.message`, subagent thread transcript, final AgentMessage item, or a Workflow report artifact path.

Validation ideas:

- Snapshot tests for Subagent Prompt preview vs full detail view.
- Snapshot tests that Markdown Prompt/report renders as Markdown in the detail view.
- Tests that lifecycle rows like `Waiting for <agent-id>` are not translated, while full Prompt/report detail can show translation.
- Manual acceptance: spawn a Subagent with a Markdown Prompt, wait for a Markdown report, open the viewer, and verify both translated and original forms are readable.

### Issue 6: Subagent Display Names, Role/Effort Overrides, And Name Leases

Status: discussing
Priority: high
Category: Subagent UX / tool protocol / role configuration
Related: Issue 3, Issue 4, Issue 5

User observation:

- Subagent lifecycle rows still expose raw worker/thread IDs in `Spawned` and `Waiting` states, for example `Waiting for 019e7313-80f9-7283-88bd-cbd2ee9ec89f`.
- The expected display is human-readable and localized, such as `等待李清照...` or equivalent concise lifecycle text using the assigned Subagent name.
- The current behavior makes the Subagent UI feel like an internal debug log rather than a user-facing collaboration interface.
- The user also observed or inferred that reasoning effort and role cannot be specified together, and considers that unreasonable.
- Expected behavior: role and reasoning effort should be able to coexist. A role may provide defaults, but the user/tool request should be able to specify effort where policy allows it.
- Each role/name should be fixed while that Subagent is active, then released when the Subagent is closed/completed so future spawns can reuse names cleanly.

Initial technical notes:

- `codex-rs/tui/src/multi_agents.rs` renders `Spawned`, `Sent input to`, `Waiting for`, `Finished waiting`, and `Closed` lifecycle rows.
- In `agent_label_spans(...)`, if metadata lacks `agent_nickname`, the code currently falls back to rendering `thread_id.to_string()`, which directly explains UUID-like IDs in lifecycle rows.
- `codex-rs/tui/src/chatwidget/active_subagents.rs` already has richer fallback-name logic through `configured_fallback_subagent_nickname(...)`, role-specific candidate pools, configured nickname candidates, and built-in fallback names.
- This suggests lifecycle rows and active panel are not sharing one authoritative Subagent display-name resolver.
- Spawn/Waiting begin events may arrive before full metadata is cached, so the TUI needs a deterministic first-sight display name rather than waiting for later metadata.
- Current core spawn handling applies requested model/reasoning first, then applies role config:
  - `codex-rs/core/src/tools/handlers/multi_agents/spawn.rs`
  - `codex-rs/core/src/tools/handlers/multi_agents_v2/spawn.rs`
  - `codex-rs/core/src/tools/handlers/multi_agents_common.rs`
- Existing test `spawn_agent_role_overrides_requested_model_and_reasoning_settings` asserts role model/reasoning override explicit request values.
- Existing tool-description test says role model/reasoning settings "cannot be changed".
- Full-history fork currently rejects `agent_type`, `model`, and `reasoning_effort` overrides through `reject_full_fork_spawn_overrides(...)`; this may remain a special inherited-context case unless C7 explicitly changes it.

Desired outcome:

- User-facing lifecycle rows never show raw Subagent IDs when a configured, role-specific, channel-specific, or built-in fallback name can be assigned.
- `Spawned`, `Waiting`, `Sent input`, `Finished waiting`, and `Closed` rows should consistently use the same name for the same active Subagent.
- Names should be leased while active to avoid duplicate simultaneous names when enough candidates are available.
- A name should be released after a terminal lifecycle state such as closed, shutdown, completed, or not found, with a clear policy for completed-but-still-inspectable Subagents.
- Role and reasoning effort should be composable:
  - role supplies defaults for instructions/model/effort/nickname candidates
  - explicit spawn request may override model/effort when the role does not mark them locked
  - tool descriptions should explain defaults and lock policy accurately
- The UI should keep internal thread IDs available only in details/search/debug contexts, not as primary lifecycle labels.

Open design points:

- Decide whether role-provided model/reasoning should be default-overridable by default, or locked only when the role config explicitly marks them locked.
- Decide whether existing role config needs new lock fields, for example `lock_model = true` or `lock_reasoning_effort = true`, or whether some built-in roles remain locked by convention.
- Decide whether full-history fork should continue rejecting role/model/effort overrides because it inherits the parent context, or whether a limited role/effort override should be allowed there too.
- Decide exact name lease release timing: on close only, on completed wait result, on shutdown/not_found, or after detail/report retention ends.
- Decide whether lifecycle rows should be localized directly (`等待李清照`) or rendered with existing English event labels and translated by a proper UI-language layer. Issue 4 argues against blind append-translation for these rows.

Validation ideas:

- Add TUI snapshot tests for `Spawned` and `Waiting` rows where metadata is missing, where role metadata exists, and where configured nickname candidates exist.
- Add tests that the same Subagent keeps the same display name across spawn, wait, completion, and close rows.
- Add tests for simultaneous active Subagents proving name leases avoid duplicate names when the candidate pool has enough names.
- Update core tests so explicit reasoning effort can coexist with `agent_type` under the chosen role-lock policy.
- Update tool-description tests to say role model/reasoning are defaults or locked settings according to actual config.
- Manual acceptance: spawn several Subagents with roles and explicit reasoning efforts, observe readable names in lifecycle rows, inspect details for IDs only when needed, and confirm names are released/reused after close.

### Issue 7: Update Plan Is Not Proactive Enough

Status: discussing
Priority: medium-high
Category: planning UX / collaboration discipline / runtime guidance
Related: Issue 1, Issue 2, Workflow state persistence

User observation:

- The visible `Update Plan` behavior seems insufficiently proactive.
- In a multi-step Workflow discussion or execution, the plan panel may lag behind the real work state unless explicitly nudged.
- This weakens the usefulness of the persistent Plan/SubPlan panel because the user cannot always trust it as the current short-term work state.
- The Plan should not be a flat checklist for a Workflow Cycle. It should show the Cycle as the root, Milestones as root items, and each current Milestone's work as a SubPlan.
- Example desired structure:
  - `Cycle 7 Plan`
  - Cycle summary/description
  - `M1: ...`
    - concrete discovery/design items
    - write tests
    - implement
    - audit
    - update relevant Workflow files/state
  - `M2: ...`
  - Future Milestones that have not started should remain collapsed rather than expanding all details at once.

Initial technical notes:

- The base model instruction currently says to skip plans for simple tasks, avoid single-step plans, and update the plan after completing a shared sub-task.
- That instruction is reasonable for ordinary Codex tasks but may be too weak for Hypo-Workflow, where the visible plan is part of the workflow contract.
- Existing C3 Plan freshness work appears to cover resume, compaction, and stale active-plan reminders. Tests such as `codex-rs/core/tests/suite/plan_update_gate.rs` focus on injecting an `update_plan` reminder after resume/compact context changes.
- This does not necessarily cover ordinary long discussions, issue queue transitions, Subagent lifecycle changes, code-edit boundaries, test/audit transitions, or final response closeout.
- Current Workflow instructions mention using `update_plan` for visible planning discipline, but they may not define concrete trigger points aggressively enough.
- `codex-rs/protocol/src/plan_tool.rs` already supports structured `PlanState` with `title`, `summary`, root `items`, and one-level `subplan`, so the protocol can represent a Cycle/Milestone/SubPlan shape.
- TUI code stores `latest_plan_state` for the persistent panel, but `on_plan_update(...)` also computes terminal/status progress from the normalized legacy flat `plan` field. If only root Milestones are counted, SubPlan progress can be undercounted or visually inconsistent.
- `codex-rs/tui/src/chatwidget/plan_panel.rs` already has expanded/collapsed rendering paths for structured plans and subplans, but C7 should verify the behavior matches Workflow-specific expectations: current Milestone expanded, future Milestones collapsed, and clear Cycle title/summary.
- There is a distinction between:
  - `.pipeline/state.yaml` / `PROGRESS.md` as durable Workflow state
  - `update_plan` as the user-visible live collaboration state
  Both need synchronization, but they should not duplicate each other noisily.

Desired outcome:

- For Workflow planning/execution, `update_plan` should be treated as a live status surface, not an optional courtesy.
- Hypo-Workflow should publish a structured Cycle plan:
  - `state.title`: current Cycle name, for example `Cycle 7 Plan`
  - `state.summary`: short Cycle objective
  - root items: Milestones such as `M1`, `M2`, `M3`, `M4`
  - current root item: expanded SubPlan with actionable steps
  - future root items: collapsed or summary-only until active
  - completed root items: compact completion state with useful summary
- The plan should be refreshed at clear state transitions:
  - starting or resuming a non-trivial task
  - entering a new P1/P2/P3/P4 planning phase
  - starting discussion of a new issue in the issue queue
  - recording an issue as completed/discussed
  - before substantial file edits
  - after meaningful code/config changes
  - before and after Subagent/test/audit phases
  - after resume/compact/context recovery
  - before final response when an active plan still shows stale `in_progress` work
- For ordinary small chat, the plan should not spam updates or create fake progress.
- The TUI should make stale-vs-current plan state understandable if the runtime can detect staleness.
- Progress counts should clearly distinguish current SubPlan progress from overall Plan progress instead of mixing root Milestones and SubPlan items into one total.
- Preferred semantic shape: `(current SubPlan completed / current SubPlan total | overall Plan completed / overall Plan total)`. Exact separators and wording can be adjusted by the TUI design, but the two counts must remain separate.
- Overall Plan progress should count root Milestone items such as `M1/M2/M3/M4`.
- Current SubPlan progress should count only the currently active Milestone's SubPlan items such as discovery/test/implement/audit/update.

Open design points:

- Decide whether this should be mostly prompt/policy guidance, a runtime reminder gate, a Workflow-file-to-Plan bridge, or a combination.
- Decide what "aggressive enough" means in practice: update every phase boundary, every issue boundary, every worker lifecycle boundary, or every meaningful user-visible state change.
- Decide whether Hypo-Workflow mode should have stricter thresholds than ordinary Default Codex mode.
- Decide whether the Plan panel should show a subtle stale marker when `.pipeline/state.yaml` or recent tool activity has advanced after the last `update_plan`.
- Decide whether final-answer guard should require the model to complete or refresh the plan if the active item is stale.
- Decide the exact expanded/collapsed policy:
  - current Milestone expanded
  - future Milestones collapsed
  - completed Milestones collapsed with status/summary
  - optionally allow user toggle to inspect any Milestone's SubPlan
- Decide whether `.pipeline/prompts/*.md`, `.pipeline/state.yaml`, and `.pipeline/cycle.yaml` should be converted into structured PlanState automatically by a Workflow bridge, or whether the agent must always call `update_plan` manually with the full structure.

Validation ideas:

- Add tests proving active Plan freshness reminders fire after more than resume/compact, for example after a long tool burst or after Workflow state advances without a plan update.
- Add tests or snapshots that all-completed and stale-active Plan states render clearly.
- Add Workflow prompt/routing tests that ensure Hypo-Workflow mode includes concrete `update_plan` trigger guidance.
- Add protocol/TUI tests for Cycle/Milestone/SubPlan `PlanState`, including current Milestone expanded and future Milestones collapsed.
- Add tests for progress counting with SubPlan items so counts are not misleading, specifically covering separate current-SubPlan and overall-root-Plan counts.
- Manual acceptance: run a multi-issue planning discussion and a short execution pass; the Plan panel should update at each issue/phase boundary without the user reminding the assistant.
- Manual acceptance: create a C7 plan with four Milestones; the persistent Plan panel should show `Cycle 7 Plan`, a Cycle summary, expanded current Milestone SubPlan, and collapsed not-yet-started Milestones.
- Manual acceptance: when M1 has 2/5 SubPlan steps completed and the overall Cycle has 0/4 Milestones completed, the Plan surface should present those as two distinct progress values rather than a combined 2/9 or similar mixed count.

### Issue 8: VSP Release Update Prompt And Auto-Update Path

Status: discussing
Priority: medium-high
Category: release experience / updater / distribution channel
Related: Issue 3

User request:

- When users open VSP Codex, it should be possible to detect that a newer remote release exists and show an update prompt.
- Ideally the user can update automatically rather than manually copying a curl command or reinstalling by hand.
- This should apply to the VSP remote release flow, not the official OpenAI Codex update channel.

Initial technical notes:

- Existing TUI update code already has an official startup update prompt path:
  - `codex-rs/tui/src/updates.rs`
  - `codex-rs/tui/src/update_prompt.rs`
  - `codex-rs/tui/src/update_action.rs`
  - `codex-rs/tui/src/history_cell/notices.rs`
- That official path currently checks `openai/codex` GitHub releases, npm/Homebrew readiness, and runs official update commands.
- VSP currently disables official services through `vsp_provider::official_services_disabled()`, so the official update check/popup is intentionally skipped.
- `vsp-codex update` currently rejects VSP official-service usage and debug builds; debug tests assert this behavior.
- Current VSP install docs use a preview archive URL and install into:
  - `~/.vsp-codex/packages/standalone/releases/`
  - `~/.vsp-codex/packages/standalone/current`
  - `~/.local/bin/vsp-codex`
- This standalone release layout is compatible with safe staged updates: download a new archive, verify it, unpack into a new release directory, atomically switch `current`, and require restart or relaunch.
- Current docs note that GitLab raw download may redirect to sign-in for unauthenticated users. A reliable updater needs GitLab Release assets, Package Registry assets, or a token/configured credential path rather than relying on private raw URLs.

Desired outcome:

- VSP Codex should have a VSP-specific update source, separate from official OpenAI update checks.
- On startup, release builds should check a cached or asynchronous update manifest and show a user-facing prompt when a newer compatible version exists.
- The prompt should offer clear actions:
  - update now
  - skip this time
  - skip until next version
  - optionally view release notes
- For standalone installs, `Update now` should run a VSP updater that:
  - selects the correct channel and target
  - downloads the archive
  - verifies SHA-256 and preferably signature
  - unpacks into a new release directory
  - atomically updates the `current` pointer
  - preserves user config and VSCode integration
  - tells the user to restart, or relaunches when safe
- Fully silent auto-update should be opt-in, not the default, because it has higher security and surprise risk.
- Default recommended behavior: prompt on open, one-key update, then restart/relaunch.

Open design points:

- Decide release channels:
  - internal GitLab/local release channel
  - external GitHub release channel
  - preview/stable channel if needed
- Decide how the binary knows its update channel: build-time env, embedded release metadata, config, or installer-written metadata.
- Define a VSP release manifest schema, likely including:
  - latest version
  - channel
  - target triple
  - archive URL
  - SHA-256
  - optional signature
  - release notes URL
  - minimum supported updater version
  - whether the update is required or optional
- Decide auth strategy for GitLab private releases:
  - public/internal network-accessible release assets
  - configured GitLab token in user config or credential store
  - no token support in C7, only public/internal accessible URLs
- Decide update timing:
  - check in background after first frame
  - use cached result immediately and refresh for next run
  - block startup only when explicitly requested by `vsp-codex update --check`
- Decide whether C7 supports only "prompt + one-key update" or also opt-in silent auto-apply.
- Decide rollback behavior: keep previous release directory and allow `vsp-codex update rollback` or installer rollback.

Validation ideas:

- Unit tests for VSP release manifest parsing, target selection, semver/VSP version comparison, and checksum mismatch handling.
- CLI tests for `vsp-codex update` in VSP standalone mode using a local mock manifest/archive.
- TUI tests/snapshots for the VSP update prompt, including skip and skip-until-next-version behavior.
- Install-context tests proving standalone release layout maps to a VSP update action instead of official OpenAI commands.
- Manual acceptance: install an older standalone VSP build, publish or mock a newer manifest, start `vsp-codex`, see update prompt, run update, restart, and verify `vsp-codex --version` reports the newer version and VSCode status still points to the working CLI.

### Issue 9: Clarify And Possibly Unify `/theme` And `/ui-theme`

Status: discussing
Priority: medium
Category: TUI appearance / command clarity / theme UX
Related: Issue 4, Issue 5

User question:

- The user noticed that VSP Codex already has a `/theme` command and asked how the current VSP Theme system works.
- This indicates possible confusion between upstream syntax highlighting themes and the VSP UI/Markdown palette system.

Current technical behavior:

- `/theme` is the existing syntax-highlighting theme picker.
  - It lists bundled syntax themes plus custom `.tmTheme` files under `$CODEX_HOME/themes/`.
  - It live-previews code/diff highlighting while navigating.
  - It persists to `[tui].theme`.
  - It primarily affects syntax-highlighted code blocks and diff/code preview rendering.
- `/ui-theme` is the VSP-added UI palette picker from C5.
  - It currently exposes `teal`, `violet`, and `amber`.
  - It persists to `[tui].ui_theme`.
  - It affects Markdown semantic roles, message/history accents, ask/tool/reasoning/user/assistant colors, and composer focus styling.
- The two systems are intentionally separate in config:
  - `[tui].theme`: syntax highlighting theme name
  - `[tui].ui_theme`: VSP UI color group
- Startup applies both:
  - syntax theme through `render::highlight::set_theme_override(...)`
  - UI palette through `style::set_ui_theme(...)`

Potential UX problem:

- Users can reasonably expect `/theme` to control the whole TUI appearance, but today it only controls syntax highlighting.
- `/ui-theme` is more relevant to the visible VSP Markdown/message colors, but its name may be less discoverable.
- Command palette exposes `/ui-theme`, while `/theme` also remains available, so the split needs clearer labeling or a unified appearance entry point.
- Because Issue 5 wants Markdown-rendered Subagent Prompt/report views, theme behavior should remain predictable across Markdown panels, code blocks, tool cards, and Subagent views.

Desired outcome:

- Merge the user-facing entry point while keeping the underlying configuration model separate.
- Preferred direction:
  - `/theme` becomes or opens a unified Appearance/Theme panel because users naturally look for theme settings there.
  - The unified panel has separate sections for `UI palette` and `Syntax theme`.
  - `/ui-theme` can remain as a direct shortcut to the UI palette section for compatibility and discoverability.
- Make the distinction explicit and user-friendly:
  - "Syntax theme" for code/diff highlighting
  - "UI theme" or "Appearance palette" for Markdown/messages/composer
- The unified surface should expose two sections:
  - UI palette
  - syntax highlighting theme
- Preserve existing config keys for compatibility unless there is a strong reason to migrate.
- Ensure docs, command descriptions, and command palette labels make the split obvious.
- Do not collapse `[tui].theme` and `[tui].ui_theme` into one config key; syntax themes can be numerous/custom `.tmTheme` files, while UI palettes are curated product color groups.

Validation ideas:

- Snapshot or command-palette tests that labels/descriptions clearly distinguish syntax theme and UI palette.
- Tests that selecting `/ui-theme` changes Markdown/message colors but not syntax theme.
- Tests that selecting `/theme` changes syntax highlighting but does not unexpectedly change UI palette.
- Manual acceptance: user can open the appearance command and understand which setting controls Markdown/message colors versus code highlighting.

### Issue 10: Structured Request Input And Confirmation/Authorization Cards

Status: recorded during P2 revision
Priority: high
Category: Workflow interaction / confirmation and authorization UX
Related: Issue 7, Issue 8

User request:

- Add a dedicated Milestone for enforcing structured Workflow interactions through `request_user_input` instead of plain assistant prose.
- Design a corresponding user-facing card for confirmation and authorization interactions.
- This should cover Workflow decisions such as confirming plans, approving phase transitions, choosing scopes, and handling authorization-like prompts.

Initial technical notes:

- The protocol already includes `request_user_input` with summary, description, questions, question kinds, options, and secret free-text support.
- The TUI already has a queued Request User Input overlay with card-like rendering and zh-CN snapshots.
- The protocol also includes `request_permissions`, and the TUI has an approval overlay for exec, patch, permissions, and MCP elicitation.
- `workflow_routing.rs` already says required user decisions should prefer Request Questions via `request_user_input`, but this is guidance rather than a strict, designed interaction contract.
- The missing layer is a Workflow interaction-gate policy plus reusable confirmation/authorization card templates, so the assistant and TUI do not fall back to plain text for required decisions.

Desired outcome:

- Required Workflow decisions should use structured `request_user_input` when the tool is available.
- Confirmation and authorization interactions should render as clear cards rather than ordinary chat paragraphs.
- Cards should distinguish:
  - confirmation of a Workflow decision
  - permission or authorization grant
  - ordinary optional clarification
  - informational status that does not need a user answer
- The card design should support zh-CN copy, concise summaries, explicit choices, cancellation/rejection paths, and safe defaults.
- Permission-sensitive flows must not weaken existing sandbox, network, destructive-action, or external-side-effect approvals.
- Ordinary chat should not be over-questioned; the stricter behavior is for required Workflow or high-impact decisions.

Validation ideas:

- Add tests proving Workflow guidance requires `request_user_input` for required user decisions when available.
- Add Request User Input snapshot tests for confirmation and authorization cards in zh-CN.
- Add tests or fixtures that plan confirmation, release update confirmation, and permission-like choices are represented as structured cards rather than only assistant prose.
- Audit the relationship between `request_user_input`, `request_permissions`, and the existing approval overlay so authorization wording is clear and does not bypass policy checks.
