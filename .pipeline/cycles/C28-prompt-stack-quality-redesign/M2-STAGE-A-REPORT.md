---
kind: stage-report
cycle: C28-prompt-stack-quality-redesign
milestone: M2
stage: A
status: accepted
updated: 2026-08-11T00:08:17+08:00
accepted_at: 2026-08-11T00:08:17+08:00
---

# M2 Stage A Review: Prompt Sources, Initial Assembly, and Debug Evidence

## Conclusion

Stage A is ready for human review. The accepted VSP base instructions now ship for `gpt-5.6-sol`; default permissions and collaboration prose are compact; Multi-Agent V2 is proactive by default and its root/shared/mode sections are assembled into one initial developer policy message; Apps/Plugins prompts are shorter; unsolicited `<recommended_plugins>` user fragments and their alternate install-tool wire shape are removed; and a locally built CLI emitted a real request matching these choices.

This gate covers Stage A only. It does not authorize or claim completion of Stage B System Skills, Stage C AGENTS/runtime preflight, Stage D image-provider capability, Stage E worker routing, or Stage F broad integration/regression work.

## Implemented Behavior

### Base instructions

- Replaced both `gpt-5.6-sol` prompt sources in `models.json`: `base_instructions` and `model_messages.instructions_template`.
- Both fields are byte-identical to the accepted S1 candidate: 11,696 bytes, SHA-256 `33202ab56daa59f39644a36f82935532b910a78f313fbc11147b5e450f39e5b6`.
- Removed the obsolete static test that required the old Consultation-first/Mini-contract phrases across every bundled model.
- Preserved bundled JSON parsing and round-trip validation.

### Permissions and default collaboration mode

- Added a compact branch for the VSP default `PermissionProfile::Disabled` plus `AskForApproval::Never` combination.
- The compact text states actual unrestricted filesystem/network runtime, omits unavailable `sandbox_permissions`, and preserves the authorization boundary.
- Non-default permission profiles continue through the existing dynamic renderer unchanged.
- Default collaboration mode now contributes no redundant prose; explicit plan/code/personality mode instructions remain supported.

### Multi-Agent V2

- Root guidance now defines proactive delegation criteria, root ownership, bounded/disjoint tasks, controlled nesting, and cost-aware routing-class selection.
- Subagent guidance is bounded to the assigned task and prohibits descendants unless the parent explicitly authorizes them.
- Shared guidance is reduced to direct collaboration-tool calls plus shared-filesystem behavior.
- Default V2 mode is `Proactive` instead of reasoning-effort-derived `ExplicitRequestOnly`; a configured custom mode still overrides it, and later runtime mode updates retain their existing override path.
- Initial root/shared/mode guidance is appended to the same developer-policy message as permissions, skills, and host context. It is no longer emitted as separate initial developer messages.

### Apps and plugins

- Corrected `tools_search` to `tool_search` in Apps guidance and removed repeated generic tool-search instructions.
- Shortened installed-plugin guidance to activation, manifest metadata, and namespace-name facts.
- Removed the endpoint-recommendation prompt path, `<recommended_plugins>` contextual user fragment, recommendation-specific router enum, simplified `plugin_id` request shape, and tests owned only by that removed behavior.
- Preserved the explicit workflow: discoverable connectors and configured remote plugins are merged by `list_tool_suggest_discoverable_tools_with_auth`, exposed through `list_available_plugins_to_install`, and installed through the single `request_plugin_install` wire shape.

### Workflow host text

- Replaced the long unconditional host paragraph with the accepted short authority/projection boundary.
- Conditional injection and trusted-focus work remain deferred to Stage C.

## Modified Modules

- Model prompt source: `codex-rs/models-manager/models.json`, `models-manager/src/manager_tests.rs`.
- Runtime policy sources: `codex-rs/prompts/src/permissions_instructions.rs`, collaboration-mode templates/presets, `core/src/config/mod.rs`, `core/src/context/multi_agent_mode_instructions.rs`, and `core/src/workflow_routing.rs`.
- Initial assembly: `core/src/session/mod.rs`, `core/src/session/multi_agents.rs`, and focused session tests.
- Apps/plugins text and control flow: `core/src/context/apps_instructions.rs`, `available_plugins_instructions.rs`, contextual-user registration, `core/src/session/turn.rs`, tool router/spec/handler modules, and their unit/integration tests.
- Removed source: `core/src/context/recommended_plugins_instructions.rs`.
- Prompt evidence: `M3-PROMPT-DEBUG-EVIDENCE.md`.

No Rust dependency, Cargo lockfile, config schema, app-server API, compile-time asset, or Bazel compile-data change was needed.

## Test Design and Results

The repository's named `core` behavior group was used for core changes; `just test` was used only for focused crates not represented by a narrower named group.

| Validation | Result | Contract established |
| --- | --- | --- |
| `cargo check -p codex-prompts -p codex-models-manager -p codex-core --tests` | pass | Modified crates and all their test targets compile together. |
| Core group: `build_initial_context_merges_multi_agent_v2` | 2/2 pass | Root and subagent policy each share one initial developer message with proactive mode and do not cross-contaminate roles. |
| Core group: `explicit_list_install_workflow` | 1/1 pass | Explicit discoverable-tool listing still reaches the model and returns connector metadata. |
| Core group: `request_plugin_install_requires_all_discovery_features` | 1/1 pass | List/install tools remain protected by Apps, Plugins, and ToolSuggest gates. |
| Core group: explicit discovery wire-shape test | 1/1 pass | The sole request tool requires `tool_type`, `action_type`, `tool_id`, and `suggest_reason`. |
| `codex-prompts` permissions filter | 20/20 pass | Compact VSP default and all existing non-default permission/approval variants remain correct. |
| `codex-models-manager` collaboration preset filter | 2/2 pass | Default mode is compact while named modes retain their instructions. |
| Bundled-model round trip | 1/1 pass | Updated `models.json` remains valid and losslessly parseable. |
| `cargo fmt --all -- --check` | pass | Rust formatting is clean; only stable-toolchain warnings about an unavailable nightly formatting option were printed. |
| `git diff --check -- codex-rs` | pass | No whitespace errors in the Stage A product diff. |
| `cargo build -p codex-cli` | pass | A current local `vsp-codex` binary was produced for prompt capture. |
| Isolated real CLI prompt capture | pass | Actual outbound request matches the accepted base prompt and dynamic-policy hierarchy. |

The changed product/test diff is deletion-heavy: 186 inserted and 1,155 deleted lines across 27 files. The size exceeds the usual line-count guideline because it removes large static prompt bodies, the unsolicited recommendation control-flow branch, and tests specific to that branch. New complex logic is small; the behavior change remains isolated to prompt assembly and plugin suggestion presentation.

## Real Prompt Evidence

The successful capture used the locally built binary, an ephemeral session, an isolated temporary `CODEX_HOME`, and a local Responses API mock with WebSockets disabled. It did not load user config or contact a live inference endpoint.

- Base prompt equals the accepted candidate byte-for-byte.
- The request contains two developer messages: one base-instruction message and one runtime-policy message.
- The runtime-policy message contains five ordered parts: compact permissions, root/shared Multi-Agent policy, one proactive mode fragment, skills metadata/instructions, and Hypo-Workflow host projection.
- The request contains zero `recommended_plugins` occurrences and zero Apps feature-catalog blocks.
- The isolated tool surface contains `exec`, `wait`, `request_user_input`, and `collaboration`.

Full evidence, command shape, limitations, and interpretation are in `M3-PROMPT-DEBUG-EVIDENCE.md`.

## Problems Encountered

- One initial large `apply_patch` failed atomically because its context did not exactly match the current source. It left no partial edits; the change was reapplied as smaller exact patches.
- A combined models-manager nextest filter matched zero tests. The two intended filters were rerun separately and passed 2/2 and 1/1.
- The first CLI capture attempt showed that setting `OPENAI_BASE_URL` did not redirect the built-in OpenAI WebSocket provider. It received 401 responses and produced no local capture. The successful run used an explicit local provider with WebSockets disabled and zero retries.
- Three untracked `codex-rs/core/.tmp*` directories were investigated after testing. Their timestamps predate Stage A by one day, so they were treated as existing user worktree state and left untouched.
- Global `.pipeline/state.yaml` and `.pipeline/cycle.yaml` still project legacy C10 state. C28's semantic index and cycle-local Plan/Progress/Execution records are authoritative, so Stage A does not overwrite those protected legacy projections.

## Expected User-Facing Result

- New `gpt-5.6-sol` sessions receive the accepted VSP identity, Chinese-first communication, continuity, reporting, workspace-safety, and autonomy contract.
- Default full-access sessions see a short accurate permission policy instead of a general sandbox tutorial.
- Multi-Agent V2 can delegate proactively while root remains responsible for direction, cost, integration, and validation.
- Initial dynamic policies are easier for the model to interpret because they share one developer message and have one active mode declaration.
- Ordinary requests no longer receive unsolicited plugin recommendations as a user-role fragment. Explicit missing-plugin requests can still use list/install discovery.

## Risks and Follow-up

- Live-model inference quality is not evaluated in Stage A; M3/S6 must compare representative task outputs and token usage.
- Existing resumed sessions may retain prior prompt history; the accepted session migration behavior remains a later stage.
- Stage A names routing classes in policy but does not yet expose or resolve `routing_class`; that is Stage E.
- Skills deduplication/system-skill rewrites, AGENTS split/preflight, conditional host injection, environment role, and image-provider capability remain intentionally unimplemented.
- Removing endpoint-recommendation tests also removes their cache-refresh and analytics assertions. The underlying explicit install handler remains covered at unit level, but Stage F should add positive end-to-end coverage for remote-plugin install metadata/cache refresh through the surviving explicit list/install path where practical.

## Major Behaviors Still Requiring Later Tests

1. System Skill name deduplication, alias rendering, trigger/loading lifecycle, and candidate content.
2. Target-path AGENTS preflight, domain rule loading, environment developer role, resume parity, and conditional Workflow host injection.
3. VSPLab image generation capability, endpoint construction, bearer redaction, and generate/edit distinction.
4. Worker-routing class schema, config precedence, deterministic resolution, fallback, controlled overrides, and effective-selection observability.
5. Same-config before/after prompt characters, input tokens, cache behavior, representative task quality, and final smoke coverage.
