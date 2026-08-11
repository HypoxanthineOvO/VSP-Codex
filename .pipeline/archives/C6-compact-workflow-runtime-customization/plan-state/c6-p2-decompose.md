# C6 P2 Decompose Checkpoint

Timestamp: 2026-05-29T00:00:32+08:00
Status: proposed

## P1 Summary

- Primary outcome: make Hypo-Workflow a native VSP-Codex capability.
- Delivery priority: solve empty Compact output continuity first.
- Empty Compact recovery priority: architecture/project summary, global/current goal, recent N dialogue turns, state/cycle, plan prompts, compact logs.
- Automatic Subagent dispatch: allowed in all useful contexts; still ask before destructive operations and system-level dependency installation.
- Ordinary chat capture: default Journal + Inbox; long ordinary topics automatically upgrade into Mini-Cycles.
- Initial native Workflow tools: structured YAML writer, State/Progress/Log API, Question/Plan bridge, Subagent lifecycle API.
- Deferred from initial scope: lock/concurrency API unless discovered as necessary.

## Evidence

- Local/Responses compaction installs replacement history through `codex-rs/core/src/compact.rs` and `Session::replace_compacted_history`.
- Remote compaction paths process model-returned replacement history in `codex-rs/core/src/compact_remote.rs` and `codex-rs/core/src/compact_remote_v2.rs`.
- Session context recovery is centralized in `Session::build_initial_context`, and post-compact currently queues `SessionStartSource::Compact`.
- Tool registration is centralized in `codex-rs/core/src/tools/spec_plan.rs`; existing native tools already include `update_plan`, `request_user_input`, and `spawn_agent`.
- TUI slash command definitions live in `codex-rs/tui/src/slash_command.rs`.
- Existing hooks expose `SessionStart`, `PreCompact`, `PostCompact`, `SubagentStart`, and `SubagentStop`, but PostCompact output cannot currently inject additional model context.

## Milestone 1: Compact Fallback Context Pack

### technical_solution

Introduce a core-owned `WorkflowContextPack` builder that can synthesize a model-visible fallback context when local or remote compaction produces an empty or semantically unusable summary/history. The pack reads bounded project-local Workflow sources and emits stable `ResponseItem` context in the user-requested priority order:

1. architecture/project summary
2. global/current goal
3. recent N dialogue turns or rollout-derived summaries
4. `.pipeline/state.yaml` and `.pipeline/cycle.yaml`
5. active plan prompt / current milestone prompt
6. compact logs such as `PROGRESS.compact.md`, `log.compact.yaml`, and `knowledge.compact.md`

The first implementation should be local to core/session context management and compaction paths, not a broad new subsystem.

### technical_route

1. Add a small module near core context/session code, for example `codex-rs/core/src/workflow_context.rs`, guarded by project detection for `.pipeline/`.
2. Add a bounded file reader that refuses paths outside the current workspace and tolerates missing files.
3. Add `WorkflowContextPack::for_compact_fallback(...)` with byte/token budgets per source class.
4. In local compaction, detect empty summary suffix or `(no summary available)` replacement and inject the fallback pack before installing compacted history.
5. In remote v1/v2 compaction, detect empty/near-empty replacement history after filtering and inject the fallback pack through `process_compacted_history` or an adjacent helper.
6. Keep `SessionStartSource::Compact` queueing so normal post-compact SessionStart hooks still run after fallback history is installed.

### research_required

- No external research required.
- Confirm during implementation whether "empty" should mean literally no assistant summary, only compaction markers, or no non-context user/assistant content after filtering.

### risks_and_alternatives

- Risk: context pack can become too large and defeat compaction. Mitigation: strict budgets and deterministic truncation.
- Risk: duplicating stale instructions from `.pipeline`. Mitigation: label pack as Workflow recovery context and prioritize factual state over instructions.
- Alternative rejected for first milestone: only implement a hook-based fallback. Current PostCompact output schema cannot inject additional context, so core-owned fallback is more reliable.

### validation_path

- Add unit tests for local compaction empty summary fallback history shape.
- Add unit tests for remote v1/v2 empty filtered replacement history fallback.
- Add a scenario using a temporary `.pipeline/` with architecture/state/plan/recent chat files and assert recovered history contains the prioritized pack.
- Pass signal: replacement history is non-empty, YAML-derived content is rendered without parse/write errors, and normal next turn can build model input.

### audit_focus

- Verify fallback never reads outside project root.
- Verify budgets prevent excessive context.
- Verify empty compact output no longer leaves the model without current Workflow continuity.

## Milestone 2: Structured Workflow File API

### technical_solution

Add native Workflow utility tools or internal APIs for safe structured updates to Workflow files. The initial API should cover:

- structured YAML read/patch/write with parse validation
- state/progress/log event append helpers
- protected file guard for `.pipeline/state.yaml`, `.pipeline/cycle.yaml`, and `.pipeline/rules.yaml`
- Question/Plan bridge policies that encourage `request_user_input` and `update_plan` for Workflow gates
- Subagent lifecycle event writer for requested/started/completed/failed/blocked/closed states

Expose model-facing tools only where they reduce unsafe shell editing; otherwise keep helpers internal and call them from Workflow-aware runtime paths.

### technical_route

1. Create a `workflow_files` or `workflow_runtime` module with typed structs for state/log/progress append operations.
2. Use `serde_yaml` for YAML parse/write validation; avoid hand-written YAML string concatenation for protected files.
3. Add append-only helpers for log/progress where possible; for state/cycle use parse-modify-serialize with field-preserving tolerance where practical.
4. Register direct tools in `codex-rs/core/src/tools/spec_plan.rs` only after handler contracts are small and auditable.
5. Add tool tests mirroring `PlanHandler` and `RequestUserInputHandler` patterns.

### research_required

- Need implementation-time inspection of existing `.pipeline` schemas across archives to decide how strict typed structs can be without breaking older files.

### risks_and_alternatives

- Risk: strict YAML serialization churns formatting. Mitigation: initially target narrow operations and keep append-only logs where possible.
- Risk: too many model-visible tools increase prompt surface. Mitigation: expose a compact grouped namespace or hidden internal handlers first.
- Alternative: continue shell/apply_patch updates. Rejected because user explicitly wants YAML formatting failures prevented.

### validation_path

- Unit tests for malformed YAML rejection and no partial writes.
- Golden tests for state heartbeat update, log append, progress append, and lifecycle event append.
- Run `cargo test -p codex-core workflow` or narrower package tests once implemented.

### audit_focus

- Protected file guard behavior.
- Atomic write or temp-file swap where files are rewritten.
- No silent YAML corruption on invalid inputs.

## Milestone 3: Native Workflow Runtime Routing

### technical_solution

Teach VSP-Codex runtime to treat Hypo-Workflow as native behavior instead of only a user-invoked Skill. This includes:

- automatically loading Workflow guidance when `.pipeline/` exists
- routing hard planning/user decisions through `request_user_input`
- routing visible plan discipline through `update_plan`
- automatically delegating Subagent work in useful contexts, with stop gates only for destructive operations and system-level dependency installation
- making `/hw:*` and `$hypo-workflow:*` feel like first-class command surfaces

### technical_route

1. Add a Workflow project detector around session startup/context construction.
2. Inject concise Workflow runtime instructions into `build_initial_context` through a dedicated contributor rather than bloating base instructions.
3. Extend slash command handling to recognize `/hw:*` as Workflow commands or route them through existing prompt/skill invocation.
4. Adjust skill-selection/injection behavior so Hypo-Workflow Skills can be selected by detected context, not only explicit mention.
5. Add or revise Subagent usage hints so Workflow planning/execution/explore contexts encourage automatic spawn when useful.

### research_required

- Need focused inspection of skill injection and slash dispatch internals during implementation to choose the least invasive routing layer.

### risks_and_alternatives

- Risk: over-injection makes every project feel Workflow-heavy. Mitigation: only activate when `.pipeline/` exists or config enables it.
- Risk: automatic Subagent delegation causes role confusion. Mitigation: lifecycle API from Milestone 2 and explicit role evidence.
- Alternative: only improve AGENTS.md instructions. Rejected because user wants default internal ability, not just text guidance.

### validation_path

- Start a session in a temp project with `.pipeline/` and assert Workflow context is model-visible.
- Simulate `/hw:plan` or `$hypo-workflow:plan` routing and assert Request Questions/update_plan are available/encouraged.
- Subagent-related tests assert no repeated authorization prompt is required for normal Workflow delegation.

### audit_focus

- Activation guard only in Workflow projects unless configured.
- No destructive/system-install actions bypass user confirmation.
- Skill and slash routing remains compatible with non-Workflow usage.

## Milestone 4: Ordinary Conversation Capture

### technical_solution

Add lightweight Workflow capture for ordinary Codex conversations:

- append-only Journal for each meaningful turn summary
- Inbox extraction for actionable or reusable items
- automatic Mini-Cycle upgrade for long-running ordinary topics

This should be a passive recording layer at first, not a new execution runner.

### technical_route

1. Define `.pipeline/chat/journal.yaml`, `.pipeline/inbox/items.yaml`, and `.pipeline/chats/<id>/state.yaml` schemas.
2. Record concise turn summaries from root conversation turns after assistant completion.
3. Extract Inbox candidates when the model or runtime marks follow-up/actionable content.
4. Add a threshold policy for automatic Mini-Cycle upgrade, such as topic continuity across N turns or explicit ongoing-topic markers.
5. Add compaction fallback pack integration so recent Journal/Inbox/Mini-Cycle state can be used after empty compaction.

### research_required

- Need implementation-time choice of the right turn completion hook to avoid recording partial/interrupted assistant outputs.

### risks_and_alternatives

- Risk: noisy Inbox. Mitigation: conservative extraction and status fields.
- Risk: privacy or secret leakage into persistent Workflow files. Mitigation: reuse existing redaction concepts from Hypo-Workflow config and avoid raw full transcript by default.
- Alternative: Mini-Cycle for every chat. Rejected because it is too heavy for casual usage.

### validation_path

- Unit/integration test: ordinary chat appends Journal.
- Test actionable content creates Inbox item.
- Test long topic auto-upgrades to Mini-Cycle.
- Test empty Compact fallback includes recent conversation capture after ordinary chat.

### audit_focus

- No duplicate entries on resume/replay.
- Redaction and bounded summaries.
- Mini-Cycle creation does not mutate active formal Cycle unexpectedly.

## Milestone 5: Workflow Command And UX Polish

### technical_solution

After the durable runtime pieces exist, polish command and UI surfaces:

- first-class descriptions and routing for Workflow commands
- status output for current Workflow mode, Journal/Inbox/Mini-Cycle capture, and Compact fallback health
- docs and release notes for VSP-Codex Workflow-native behavior

### technical_route

1. Extend TUI slash command registry or command palette entries.
2. Add status/debug output for Workflow project detection and context pack source selection.
3. Update `README.md`, `docs/vsp-codex.md`, and relevant architecture notes.
4. Add release validation commands.

### research_required

- No external research; implementation should inspect current TUI command palette patterns before editing.

### risks_and_alternatives

- Risk: UI work before runtime stability creates confusing promises. Mitigation: keep this last.

### validation_path

- TUI command/search snapshot tests.
- Docs lint or existing release prep checks.
- Manual smoke: `/status` or Workflow command visibly reports native mode.

### audit_focus

- Documentation matches actual runtime behavior.
- Non-Workflow projects do not show misleading Workflow state.

## Recommended Order

1. Compact Fallback Context Pack
2. Structured Workflow File API
3. Native Workflow Runtime Routing
4. Ordinary Conversation Capture
5. Workflow Command And UX Polish

This order prioritizes the user's highest-risk failure mode first while building the durable APIs needed for the broader native Workflow behavior.
