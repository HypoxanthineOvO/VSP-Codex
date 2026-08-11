# C10 M3 VSP Surface Recheck

## Summary

The alpha.33 merge conflicts are resolved and staged, including the two app-server v2 test conflicts that were handed to the test worker. This report is a pre-GREEN surface recheck: code-level and focused-suite evidence is recorded here, while the final M3 GREEN command matrix still runs after the merge commit is created.

## Surface Results

| Surface | Recheck result | Evidence |
| --- | --- | --- |
| VSP CLI/package identity | Preserved in resolved conflict hunks. | `README.md` still documents `vsp-codex`, `~/.vsp-codex`, VSP installer manifest, and VSCode VSP wrapper behavior. `codex-rs/Cargo.toml` keeps workspace version `0.140.0-vsp.8.2` instead of upstream `0.143.0-alpha.33`. |
| Official service/provider disablement | No M3 conflict resolution re-enabled official provider defaults. | README still states the default VSP Provider and that `/provider` only shows VSP/custom OpenAI-compatible providers. Full executable proof remains with test worker. |
| Remote plugins disabled policy | Code-level gates preserved with executable focused proof. | `codex-rs/core/src/config/mod.rs` passes `Feature::RemotePlugin` into `PluginsConfigInput`; `codex-rs/core-plugins/src/manager.rs` now keeps `features.remote_plugin` gates for remote installed/featured fetches and handles `Npm` as non-local in sync. `CODEX_SKIP_BWRAP_BUILD=1 cargo test --manifest-path codex-rs/Cargo.toml -p codex-core-plugins` passed with 320 tests. |
| VSP Curated / Marketplace labels | Test conflict resolved; final broad UI evidence pending. | `codex-rs/app-server/tests/suite/v2/plugin_list.rs` now preserves both upstream remote-enabled catalog expectations and VSP remote-disabled helper/coverage. TUI snapshot/label validation remains in the final GREEN matrix/audit. |
| VSCode Remote behavior | Surface still present. | `README.md` and `codex-rs/cli/tests/vscode.rs` still reference `vsp-codex vscode` install/status/rollback/repair and Remote SSH wrapper behavior. The dedicated `codex-cli --test vscode` suite remains part of final GREEN validation. |
| Hypo-Workflow `/hw:*` routing | Surface still present. | `README.md` still documents `/hw:*` and `$hypo-workflow:*` routing. `codex-rs/tui/tests/suite/workflow_slash_routing.rs` still contains `/hw:status`, `/hw:plan`, `/hw:audit`, `/hw:quality`, and `/hw:optimize` coverage. |
| request_user_input / Ask metadata | Surface still present. | `codex-rs/core/src/tools/handlers/request_user_input*`, `codex-rs/tui/src/app/pending_interactive_replay.rs`, and `codex-rs/app-server/README.md` still contain request_user_input handling and cleanup semantics. |
| Workflow markdown / plan freshness | Surface still present; schema lockstep validated. | `codex-rs/app-server/README.md` still documents `turn/plan/updated`; `just write-app-server-schema` passed without unstaged drift, and `codex-app-server-protocol` tests passed with 260 lib tests plus 2 schema fixture integration tests. |
| Subagent UX | Surface still present. | README still lists Subagent behavior; TUI modules such as `agent_navigation`, `loaded_threads`, and `session_lifecycle` remain present after merge. |
| multi-agent v2 / Rendezvous / WebSocket | Upstream internal code carried; no extra VSP commitment added by conflict resolution. | Upstream websocket/remote-control and multi-agent v2 code/docs were merged naturally. This worker did not add new VSP CLI/UI copy promising these as VSP product capabilities. App-server upstream README changes remain a documentation risk for later audit/product review. |
| Sandbox and permissions defaults | No forbidden sandbox env edits by this worker. | `rg -n 'CODEX_SANDBOX_NETWORK_DISABLED_ENV_VAR|CODEX_SANDBOX_ENV_VAR' codex-rs` shows existing references; this worker did not edit those files. |

## Important Limitations

- Merge commit is not created yet at this report update point; `rust-v0.143.0-alpha.33` ancestry must be confirmed after commit creation.
- `just write-app-server-schema` passed and produced no unstaged schema drift; staged schema changes are from the alpha.33 merge.
- Focused `codex-core-plugins` and `codex-app-server-protocol` suites passed. The full M3 GREEN matrix still needs to run with `CODEX_SKIP_BWRAP_BUILD=1`.
- `models-manager/models.json` accepted upstream `instructions_template` metadata to match merged parser expectations. This is treated as internal model metadata, not as a new VSP user-facing product promise.

## Recommended GREEN Follow-up

Run the M3 GREEN plan from `.pipeline/reviews/C10/M3/test.md`, including final ancestry check after the merge commit, workspace check, Workflow routing, app-server protocol/schema validation, app-server focused plugin/thread tests, VSCode behavior, VSP entrypoint, `git diff --check`, and conflict-marker scan.
