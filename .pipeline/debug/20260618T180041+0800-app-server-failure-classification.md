# App Server Failure Classification

## Symptom

M5 broad regression previously reported:

- `cargo test --manifest-path codex-rs/Cargo.toml -p codex-app-server --test all`
- Result: 245 passed / 376 failed / 2 ignored
- Dominant visible failures: initialize/websocket deadline timeouts and cascaded integration failures.

The user asked whether the large failure count represented test problems or real regressions.

## Reproduction And Evidence

Focused checks:

- `initialize_is_not_blocked_by_slow_plugin_startup_warmup` alone: passed.
- `initialize` filter with default parallelism: 13 passed / 1 failed once with `deadline has elapsed`.
- `initialize --test-threads=1`: 14 passed / 0 failed.
- `websocket_transport_routes_per_connection_handshake_and_responses`: passed.
- `thread_list`: 27 passed.
- `request_user_input`: 11 passed.

Discriminating full run:

- `RUST_MIN_STACK=16777216 cargo test --manifest-path codex-rs/Cargo.toml -p codex-app-server --test all -- --test-threads=1`
- Result: 618 passed / 3 failed / 2 ignored, finished in 756.62s.

This shows the original 376 failures are mostly parallel integration-suite pressure and timeout cascade, not broad app-server breakage.

## Stable Failures Found

The serial full run and single-test reruns confirmed three stable failures:

1. `suite::v2::plugin_list::plugin_installed_includes_workspace_directory_without_plugin_sharing`
   - Single-test rerun fails consistently.
   - `plugin/installed` returns `marketplaces: []`.
   - Test expects one `workspace-directory` marketplace with `workspace-linear@workspace-directory`.
   - Code currently always includes `REMOTE_WORKSPACE_MARKETPLACE_NAME` in `remote_installed_plugin_visible_marketplaces`, so this looks like either a remote installed plugin visibility regression or a test/fixture mismatch around workspace directory installed plugins when `remote_plugin=false` and `plugin_sharing=false`.

2. `suite::v2::turn_start::turn_profile_tracks_blocking_tool_and_follow_up_sampling`
   - Single-test rerun fails consistently.
   - Failure is a Wiremock verification panic: expected 2 matching `GET /v1/models?client_version=0.140.0` requests but observed 1.
   - This points to an analytics/test fixture expectation drift around model-list requests, not a broad turn execution failure.

3. `suite::v2::turn_start::turn_start_emits_spawn_agent_item_with_effective_role_model_metadata_v2`
   - Single-test rerun fails consistently.
   - Actual completed `collabAgentToolCall.model` is `gpt-5.2`; test expects role config model `gpt-5.4`.
   - Code path applies role config, then unconditionally applies requested spawn model overrides, so the model argument overrides the role-locked model. This conflicts with existing role tests/tool guidance that role model/reasoning settings are locked.

## Classification

- The original large failure count is primarily a test harness/concurrency/resource issue: parallel app-server integration runs spawn many server instances and hit fixed deadlines, causing initialize/websocket cascades.
- There are still three real, stable failures in the serial suite. They are localized and should be fixed or explicitly accepted before treating broad app-server regression as clean.
- Core VSP surfaces checked during this investigation are not broadly broken: `request_user_input`, `thread_list`, websocket focused routing, provider fallback thread resume tests, and initialize serial paths pass.

## Release Risk

Recommendation: do not push/tag/release until the three stable failures are either fixed or consciously waived.

Suggested next fixes:

- Decide the intended `plugin/installed` behavior for workspace directory installed plugins when `remote_plugin=false`.
- Update or fix the turn-profile analytics test so model catalog request expectations match current behavior.
- Fix spawn-agent role precedence so role-locked model/reasoning metadata wins over requested overrides, or update tests only if the product decision changed.
