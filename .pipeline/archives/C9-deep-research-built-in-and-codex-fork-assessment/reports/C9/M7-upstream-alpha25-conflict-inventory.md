# C9 M7 Upstream Alpha.25 Conflict Inventory

## Scope And Non-Goals

This inventory is pinned to upstream tag `rust-v0.143.0-alpha.25`
(`d634f56262`). It is a read-only planning artifact for the later grouped
sync from the current VSP branch to alpha.25.

M7 does not merge product source, does not change package or public version
metadata, does not update release artifacts, and does not push, tag, publish,
or create a release.

## Evidence Commands

- `git diff --name-status rust-v0.143.0-alpha.25...HEAD -- . ':!.pipeline' ':!.plan-state'`
- `git diff --stat rust-v0.143.0-alpha.25...HEAD -- codex-rs/state codex-rs/app-server codex-rs/app-server-protocol codex-rs/core codex-rs/tui codex-rs/cli codex-cli .opencode docs scripts .gitlab-ci.yml`
- `git log --oneline --decorate --max-count=30 rust-v0.143.0-alpha.25`
- `git show --stat --oneline d81fcdf8ef e98d43ac37 91e6da943b 63f8f547c0`
- `git merge-tree $(git merge-base HEAD rust-v0.143.0-alpha.25) HEAD rust-v0.143.0-alpha.25`

Inputs also reviewed:

- `.pipeline/prompts/c9-007-conflict-inventory-and-upstream-fix-baseline.md`
- `.plan-state/c9-upstream-alpha25-architecture.yaml`
- `.pipeline/debug/20260626T134037+0800-sqlite-log-write-amplification.md`

## Baseline Summary

- Merge base: `f297b9f07de10c7d8b9ed284b674d06cc5ff7723`.
- Upstream target: `rust-v0.143.0-alpha.25` at `d634f56262`.
- Current VSP branch contains local VSP release and hotfix commits through
  `268689610e workflow(C9): record vsp8 hotfix release`.
- Three-dot non-pipeline diff count from the architecture checkpoint is about
  523 files; a focused domain subset counted 451 files.
- Path count by requested domain from the full non-pipeline diff:
  - SQLite logging: 4
  - protocol/app-server: 69
  - core workflow/runtime: 79
  - TUI: 250
  - CLI/VSCode: 36
  - plugins/adapters: 35
  - docs/CI/release tooling: 25
- `git merge-tree` reports many `changed in both` conflicts, especially under
  app-server protocol schemas, app-server processors/tests, core runtime,
  core-plugins/core-skills, CLI, and TUI. It also reports upstream removals in
  core-skills while VSP has local surfaces depending on skill/adaptor behavior.

## Conflict Domains

### SQLite Logging

Paths in scope:

- `codex-rs/state/src/log_db.rs`
- `codex-rs/state/src/log_db_filter_tests.rs`
- `codex-rs/app-server/src/lib.rs`
- `codex-rs/tui/src/lib.rs`

Observed state:

- Official alpha.25 contains all four relevant upstream fixes listed below.
- Local VSP hotfix has already applied equivalent persistent-log filtering to
  app-server and TUI initialization, plus an in-layer `target == "log"` guard in
  `LogDbLayer::on_event`.
- The local debug report identified large existing `logs_2.sqlite` and WAL
  files and found retained `target='log'` plus `codex_otel.*` events as the
  dominant write-amplification source.

Conflict/action:

- Treat as the first replacement candidate in M8: converge to alpha.25 official
  implementation where identical or stronger.
- Preserve the local in-layer guard if alpha.25 filtering does not cover a
  bypass path in the VSP integration.
- Validate with targeted `codex-state` filter tests and a runtime SQLite I/O
  smoke using isolated `CODEX_HOME`/`CODEX_SQLITE_HOME`.

### Protocol / App-Server

Paths in scope:

- `codex-rs/app-server-protocol/`
- `codex-rs/app-server-client/`
- `codex-rs/app-server-test-client/`
- `codex-rs/app-server/src/`
- `codex-rs/app-server/tests/suite/v2/`

Observed state:

- VSP carries Question Tool / `request_user_input`, workflow markdown surface,
  plan update metadata, and generated JSON/TypeScript schema changes.
- Alpha.25 adds or changes protocol surfaces such as rate-limit reset credits,
  external agent config import, workspace messages, dynamic tools, path/string
  migration types, model safety buffering notifications, and current-time
  protocol handling.
- `git merge-tree` shows `changed in both` for generated protocol JSON,
  generated TypeScript, `export.rs`, `protocol/common.rs`, v1/v2 protocol
  files, app-server processors, thread state, and v2 integration tests.

Conflict/action:

- Preserve VSP Question Tool and workflow markdown behavior unless alpha.25
  proves exact feature parity.
- Defer generated schema overwrite until M9 can regenerate schemas from the
  reconciled Rust protocol, then test app-server clients and v2 suites.
- Upstream request processor structure can be adopted when it does not remove
  VSP request_user_input semantics.

### Core Workflow / Runtime

Paths in scope:

- `codex-rs/core/src/workflow_context.rs`
- `codex-rs/core/src/workflow_files.rs`
- `codex-rs/core/src/workflow_routing.rs`
- `codex-rs/core/src/workflow_conversation_capture.rs`
- `codex-rs/core/src/state/plan_freshness.rs`
- `codex-rs/core/src/goals.rs`
- `codex-rs/core/src/session/`
- `codex-rs/core/src/tools/handlers/`
- `codex-rs/rollout/`
- `codex-rs/thread-store/`, `codex-rs/thread-manager-sample/`

Observed state:

- VSP owns Hypo-Workflow routing, plan freshness, goals, conversation capture,
  worker separation, and request_user_input tool semantics.
- Alpha.25 contains broad runtime changes: current-time reminders, persisted
  world state, remote executor lifecycle, capability-root availability,
  provider-aware fallback, token budget message changes, and MCP/cache changes.
- `git merge-tree` shows `changed in both` across core session, context,
  config, tool handlers, tests, and core-skills/core-plugins surfaces.

Conflict/action:

- Preserve VSP workflow semantics in M10.
- Integrate upstream runtime improvements case-by-case after protocol contracts
  are stable.
- Do not replace goals, plan freshness, workflow files, or routing without a
  direct official equivalent and matching tests.

### TUI

Paths in scope:

- `codex-rs/tui/src/`
- `codex-rs/tui/tests/`
- TUI snapshots under `codex-rs/tui/src/**/snapshots/`

Observed state:

- This is the largest changed domain: 250 files in the non-pipeline diff.
- VSP owns Question Tool cards, workflow authorization/plan/release cards,
  plan panel, active subagent surfaces, provider popups, VSCode link behavior,
  status surfaces, slash routing, and snapshot expectations.
- Alpha.25 has broad TUI churn including status, model/provider UX, active
  agent/subagent display, command palette, feedback, voice, resize/reflow, and
  remote/app-server behavior.

Conflict/action:

- Preserve VSP visible workflow surfaces until M11.
- Merge after protocol/core/app-server semantics are stable; otherwise snapshot
  churn will hide contract regressions.
- Replace only isolated upstream UI fixes that have no VSP behavioral overlap.

### CLI / VSCode

Paths in scope:

- `codex-rs/cli/`
- `codex-cli/`
- `codex-rs/app-server-daemon/`
- VSCode-oriented VSP files such as `codex-rs/cli/src/vscode_cmd.rs`

Observed state:

- VSP owns `vsp-codex` entrypoint behavior, VSCode Remote file-link handling,
  doctor/update surfaces, state DB recovery, and package entrypoint changes.
- Alpha.25 introduces CLI changes such as sandbox permission flag rename,
  exec-server telemetry/lifecycle integration, code-mode host IPC transport,
  zsh artifact consumption, and app-server daemon changes.
- `git merge-tree` shows `changed in both` for `codex-rs/cli/Cargo.toml`,
  `main.rs`, login, doctor, marketplace/plugin/MCP commands, and exec-server
  tests.

Conflict/action:

- Preserve VSP entrypoint and VSCode Remote behavior in M11/M12.
- Defer upstream CLI flag/telemetry integration until user-facing command names
  and VSP package scripts are checked.

### Plugins / Adapters

Paths in scope:

- `.opencode/`
- `opencode.json`
- `codex-rs/core-plugins/`
- `codex-rs/core-skills/`
- plugin/marketplace CLI surfaces
- app-server plugin tests

Observed state:

- VSP owns Hypo-Workflow OpenCode adapter files, `.opencode` agents/commands,
  hooks, status runtime, and plugin hotfix behavior.
- Alpha.25 adds connector/plugin policy and selected executor plugin support,
  HTTP MCP OAuth support, cached `codex_apps` tools, skill loader changes, and
  marketplace policy changes.
- `git merge-tree` reports `changed in both` across core-plugins and
  core-skills, including upstream removals in core-skills service/root loader
  paths.

Conflict/action:

- Preserve `.opencode` and Hypo-Workflow adapters.
- Defer core-plugins/core-skills convergence until M10/M11 can map VSP startup
  remote sync, permission overrides, marketplace behavior, and upstream
  connector policy together.

### Docs / CI / Release Tooling

Paths in scope:

- `README.md`, `CHANGELOG.md`, `AGENTS.md`
- `docs/`
- `.gitlab-ci.yml`
- `.github/`
- `scripts/`
- `justfile`
- packaging/install scripts

Observed state:

- VSP owns GitLab release staging, VSP docs, vsp package builder, preview
  release scripts, CI docs checks, installer behavior, and release asset
  refresh commits.
- Alpha.25 adds or changes GitHub Actions clean-worktree checks, Bazel setup,
  standalone zsh artifact consumption, remote-test guidance, path-type skill
  guidance, install scripts, and package layout.
- `git merge-tree` reports `changed in both` for `AGENTS.md`, `.gitignore`,
  `Cargo.toml`, `Cargo.lock`, Bazel/wine testing files, and many script/docs
  surfaces.

Conflict/action:

- Preserve VSP GitLab and release tooling until M12 release-candidate dry run.
- Defer docs and CI reconciliation until the product merge groups are stable.
- Do not change public version strings in M7.

## Official SQLite Fix Coverage

| Commit | Upstream title | Files | Coverage in `rust-v0.143.0-alpha.25` | Relation to VSP hotfix |
| --- | --- | --- | --- | --- |
| `d81fcdf8ef` | Reduce SQLite contention from OpenTelemetry SDK debug logs (#26396) | `codex-rs/state/src/log_db.rs`, `codex-rs/state/src/log_db_filter_tests.rs` | Present in `rust-v0.142.2` and `rust-v0.143.0-alpha.25`. | VSP hotfix includes a low-level `opentelemetry_sdk` drop test/guard and should keep equivalent behavior. |
| `e98d43ac37` | Filter noisy targets from persistent logs (#29457) | `codex-rs/app-server/src/lib.rs`, `codex-rs/state/src/log_db.rs`, `codex-rs/tui/src/lib.rs` | Present in `rust-v0.142.2` and `rust-v0.143.0-alpha.25`. | VSP hotfix adds `log_db::default_filter()` and applies it to app-server and TUI SQLite log layers. |
| `91e6da943b` | Stop logging every Responses WebSocket event (#29432) | `codex-rs/codex-api/src/endpoint/responses_websocket.rs`, `codex-rs/otel/src/events/session_telemetry.rs` | Present in `rust-v0.142.2` and `rust-v0.143.0-alpha.25`. | VSP branch does not carry this exact path-level hotfix in the local diff; alpha.25 should be treated as the source for this coverage during upstream sync. |
| `63f8f547c0` | Stop persisting bridged log events (#29599) | `codex-rs/state/src/log_db.rs` | Present in `rust-v0.143.0-alpha.25`; not present in `rust-v0.142.2`. | This directly matches the VSP debug finding that `target='log'` dominated retained rows. Local VSP hotfix includes an in-layer `metadata.target() == "log"` guard and a regression test. |

Conclusion: SQLite is the only domain with a strong official replacement
candidate for immediate convergence. The VSP hotfix already mirrors the
important alpha.25 behavior, so M8 should replace local ad hoc code with the
official implementation only where exact equivalence is confirmed, while
retaining any VSP-only guard proven necessary by runtime smoke.

## Next-Milestone Implications

- M8: converge SQLite logging first and perform runtime SQLite I/O smoke.
- M9: reconcile protocol/app-server contracts and regenerated schemas.
- M10: preserve and merge core workflow/runtime behavior after protocol shape is
  settled.
- M11: converge TUI, CLI, VSCode, and user-facing workflow surfaces after
  runtime contracts are stable.
- M12: run release-candidate dry run, then reconcile docs, CI, package, install,
  version, and release artifacts under an explicit release confirmation gate.
