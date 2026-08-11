# C10 M1 alpha.25 merge conflict report

## Scope

- Worker role: implement.
- Merge target: `rust-v0.143.0-alpha.25`.
- Scope handled here: non-test, non-snapshot conflict paths explicitly assigned to the implement worker.
- Out of scope preserved for other workers: test files, snapshots, fixtures, assertion files, protected pipeline state/log/progress files, commits, tags, pushes, releases, and version bumping.

## Summary

Resolved conflict markers in the 46 implement-owned paths and staged those paths with `git add`.
The merge direction followed the C9 equivalence matrix and C10 architecture boundary:

- Preserve VSP by default for Hypo-Workflow routing/state, goals and plan freshness, Question Tool/native Ask UI, Workflow Markdown bridge, VSP CLI/package identity, VSCode Remote behavior, OpenCode/Hypo adapters, and GitLab/VSP release/install documentation.
- Adopt or reconcile upstream where it does not replace a preserve-boundary surface, especially app-server capability additions, MCP OpenAI form elicitation, current-time/world-state plumbing, token/usage UI plumbing, and generated schema aggregate entries.
- Defer generated schema regeneration and broad protocol/app-server convergence to M2/M9. The JSON/TypeScript schema conflicts were manually marker-cleaned to match the combined Rust contract; no schema regeneration is claimed in M1.

## Conflict domains

### Workflow and repository instructions

Paths:

- `AGENTS.md`

Direction: manual reconcile, preserve VSP plus adopt upstream contributor rules.

Rationale: `AGENTS.md` is Hypo-Workflow managed and must retain `.pipeline/` source-of-truth, consultation-first boundary, plan gate visibility, protected-file rules, and Workflow report-surface rules. Upstream Rust/TUI/testing guidance was also preserved because it is compatible and useful for later C10 work.

Validation risk: the file is a derived adapter surface; source Hypo-Workflow rules may need regeneration later to avoid drift.

### Version, install, and VSP package identity

Paths:

- `codex-rs/Cargo.toml`
- `docs/install.md`
- `codex-rs/models-manager/models.json`

Direction: preserve VSP / defer release-facing upstream changes.

Rationale: M1 explicitly does not change version numbers or release artifacts. `Cargo.toml` keeps `0.140.0-vsp.8.2`. `docs/install.md` keeps VSP preview install and GitLab release guidance. `models.json` keeps the current VSP model base instruction string rather than adopting broad upstream model text during the merge conflict pass.

Validation risk: model metadata and install docs should be reviewed again in M12/release-candidate work. No release artifact or manifest regeneration was performed.

### App-server protocol, schemas, and clients

Paths:

- `codex-rs/app-server-client/src/lib.rs`
- `codex-rs/app-server-client/src/remote.rs`
- `codex-rs/app-server-protocol/schema/json/ClientRequest.json`
- `codex-rs/app-server-protocol/schema/json/codex_app_server_protocol.schemas.json`
- `codex-rs/app-server-protocol/schema/json/codex_app_server_protocol.v2.schemas.json`
- `codex-rs/app-server-protocol/schema/json/v1/InitializeParams.json`
- `codex-rs/app-server-protocol/schema/typescript/InitializeCapabilities.ts`
- `codex-rs/app-server-protocol/schema/typescript/ServerRequest.ts`
- `codex-rs/app-server-protocol/src/export.rs`
- `codex-rs/app-server-protocol/src/protocol/common.rs`
- `codex-rs/app-server-protocol/src/protocol/v1.rs`

Direction: manual reconcile with VSP preserve plus upstream capability adoption; schema regen deferred.

Rationale: `InitializeCapabilities` now carries both VSP `native_ask_ui` and upstream `mcp_server_openai_form_elicitation`. Client start/connect args forward both fields. `ServerRequest.ts` keeps VSP `item/tool/requestUserInput` with its parameter import. Experimental TS filtering uses the upstream generalized function name while preserving VSP filtering assertions for request-user-input methods.

Validation risk: generated JSON/TS files were not regenerated from Rust in M1, so schema drift must be resolved in M2/M9 with `just write-app-server-schema` after protocol/app-server convergence.

### App-server runtime and request processors

Paths:

- `codex-rs/app-server/src/lib.rs`
- `codex-rs/app-server/src/message_processor.rs`
- `codex-rs/app-server/src/request_processors/initialize_processor.rs`
- `codex-rs/app-server/src/request_processors/thread_lifecycle.rs`
- `codex-rs/app-server/src/request_processors/turn_processor.rs`

Direction: manual reconcile.

Rationale: app-server initialization records both native Ask UI support and upstream OpenAI form elicitation support. VSP request_user_input fallback/native-ask imports are preserved, while upstream image URL validation and multi-agent imports are retained where they do not replace VSP Question Tool behavior. Module declarations keep `native_ask_ui` and add `models_refresh_worker`.

Validation risk: request_user_input, MCP elicitation, thread lifecycle, and turn image validation need focused app-server tests after test worker resolves test conflicts.

### Plugins and marketplace policy

Paths:

- `codex-rs/core-plugins/src/manager.rs`

Direction: upstream code with VSP policy.

Rationale: preserved VSP startup remote plugin sync wrapper and kept remote warmup behind `config.remote_plugin_enabled`, while retaining upstream curated marketplace path import and remote cache warmup behavior when remote plugins are explicitly enabled.

Validation risk: remote plugin cache, installed bundle sync, and featured plugin warmup can still conflict semantically. M10/M11 should verify disabled-by-default remote plugin behavior and marketplace cache updates.

### Core runtime, compaction, goals, and plan freshness

Paths:

- `codex-rs/core/BUILD.bazel`
- `codex-rs/core/src/compact.rs`
- `codex-rs/core/src/compact_remote.rs`
- `codex-rs/core/src/session/mod.rs`
- `codex-rs/core/src/session/session.rs`
- `codex-rs/core/src/session/turn.rs`
- `codex-rs/core/src/state/session.rs`
- `codex-rs/core/src/tasks/mod.rs`
- `codex-rs/core/src/tools/spec_plan.rs`

Direction: manual reconcile, preserve VSP workflow/goal/plan behavior while adopting upstream world-state/current-time/window IDs where required.

Rationale: retained VSP workflow context fallback, goals handlers, plan freshness tracking, workflow turn capture, and shell snapshot refresh. Adopted upstream `AutoCompactWindowIds`, world-state baseline persistence, `StepContext`-based context update signature, `CodexResult` turn return path, abort event lifecycle, current-time handler import, and token-budget reminder support where current code already depends on them.

Validation risk: this is the highest-risk domain. Full core compile and focused workflow routing/compaction tests are required after test conflicts are resolved. M10 should re-evaluate broader upstream core runtime changes instead of treating this marker resolution as full convergence.

### Exec, rollout, and protocol shared types

Paths:

- `codex-rs/exec/src/lib.rs`
- `codex-rs/protocol/src/protocol.rs`
- `codex-rs/rollout/src/lib.rs`

Direction: manual reconcile.

Rationale: exec client params forward both native Ask UI and MCP form elicitation defaults. `TurnEnvironmentSelection` preserves VSP serde/schema derives while keeping the upstream TODO. Rollout exports keep VSP `official_codex_discovery` and add upstream `persistence_metrics`.

Validation risk: protocol schema and rollout discovery should be checked with app-server/schema convergence in M2/M9.

### TUI user surfaces and resize/reflow

Paths:

- `codex-rs/tui/BUILD.bazel`
- `codex-rs/tui/src/app/resize_reflow.rs`
- `codex-rs/tui/src/app/thread_routing.rs`
- `codex-rs/tui/src/bottom_pane/request_user_input/render.rs`
- `codex-rs/tui/src/bottom_pane/status_line_setup.rs`
- `codex-rs/tui/src/chatwidget.rs`
- `codex-rs/tui/src/chatwidget/plugins.rs`
- `codex-rs/tui/src/chatwidget/rendering.rs`
- `codex-rs/tui/src/chatwidget/slash_dispatch.rs`
- `codex-rs/tui/src/lib.rs`
- `codex-rs/tui/src/onboarding/auth.rs`
- `codex-rs/tui/src/slash_command.rs`
- `codex-rs/tui/src/transcript_reflow.rs`

Direction: preserve VSP user-facing behavior, manually reconcile compatible upstream additions.

Rationale: kept VSP translated history wrapping, plan panel, active subagent panel, workflow slash/Chinese command descriptions, VSP plugin curated label/loading behavior, native Ask UI, resize-reflow observed-size handling, and startup replay gating. Also retained compatible upstream pieces such as usage/rate-limit pending insertion, rate-limit hint rendering, active-task slash blocking, and MCP form capability defaults.

Validation risk: TUI snapshot tests remain owned by the test worker and are unresolved. UI behavior requires snapshot review after the test worker resolves test/snapshot conflicts.

## Out-of-scope test and snapshot conflicts

The implement worker intentionally did not edit the following test/snapshot paths that were outside this worker's scope. During final validation, `git diff --name-only --diff-filter=U` returned no paths, so these paths were not still reported as unmerged by Git at handoff time. If they were resolved, that resolution belongs to the test worker or main thread, not this implement worker.

- `codex-rs/app-server/tests/suite/v2/request_user_input.rs`
- `codex-rs/app-server/tests/suite/v2/thread_settings_update.rs`
- `codex-rs/core-plugins/src/manager_tests.rs`
- `codex-rs/core/src/agent/control_tests.rs`
- `codex-rs/core/tests/suite/compact_remote.rs`
- `codex-rs/core/tests/suite/request_user_input.rs`
- `codex-rs/tui/src/app/tests.rs`
- `codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__plugin_detail_popup_installable.snap`
- `codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__plugin_detail_popup_installed.snap`
- `codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__plugins_popup_curated_marketplace.snap`
- `codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__plugins_popup_newly_installed_marketplace.snap`
- `codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__plugins_popup_search_filtered.snap`
- `codex-rs/tui/src/chatwidget/tests.rs`
- `codex-rs/tui/src/chatwidget/tests/popups_and_settings.rs`
- `codex-rs/tui/src/chatwidget/tests/status_command_tests.rs`
- `codex-rs/tui/src/history_cell/tests.rs`

## Validation performed

- `rg -n "^(<<<<<<<|=======|>>>>>>>)" <46 implement paths>`: no matches after resolution.
- `jq empty` on touched generated JSON files and `codex-rs/models-manager/models.json`: passed.
- `rustfmt --edition 2024 --check` on touched Rust source files: passed; stable rustfmt emitted warnings that the repo's `imports_granularity = Item` setting is nightly-only.
- `git add` was run only for the 46 implement-owned paths.

## Not performed

- No full `cargo check`, full test suite, snapshot update, schema regeneration, release artifact generation, version bump, commit, push, or tag.
- No edits to protected `.pipeline/state.yaml`, `.pipeline/cycle.yaml`, `.pipeline/log.yaml`, `.pipeline/PROGRESS.md`, or `.pipeline/.lock`.
