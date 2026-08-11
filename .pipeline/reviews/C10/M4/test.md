# C10/M4 Test Worker Baseline - Client Connectivity Adaptation And Drift Regression

Verdict: `baseline_red`

Scope observed: test worker only. I did not edit production code, test source, schemas, Cargo files, lockfiles, `.pipeline/state.yaml`, `.pipeline/log.yaml`, or `.pipeline/PROGRESS.md`.

## Commands Executed

| Command | Result | Notes |
| --- | --- | --- |
| `git status --short` | completed | Pre-existing external changes: `.pipeline/PROGRESS.md`, `.pipeline/log.yaml`, `.pipeline/state.yaml`, `.pipeline/.lock`. Left untouched. |
| `CODEX_SKIP_BWRAP_BUILD=1 cargo test --manifest-path codex-rs/Cargo.toml -p codex-cli --test vscode -- --nocapture` | **failed** | 18 passed, 1 failed: `vscode_install_remote_scope_repoints_running_bundled_elf` returned `Text file busy (os error 26)`. This is a M4-relevant real-path drift/self-heal scenario. |
| `CODEX_SKIP_BWRAP_BUILD=1 cargo test --manifest-path codex-rs/Cargo.toml -p codex-app-server --test all request_user_input -- --nocapture` | passed | 18 passed, 0 failed, 685 filtered. Output included temp-HOME trust/watcher warnings only. |
| `command -v vsp-codex` | passed | `/home/heyx/.local/bin/vsp-codex` |
| `vsp-codex vscode status --json` | passed, read-only | Returned `configured: true`, settings scope `vscode-server`, cli executable `/home/heyx/.vsp-codex/vscode/vsp-codex`; warning says remote Machine settings ignore application-scoped `chatgpt.*` keys and local VSCode User settings must be set. |
| `CODEX_SKIP_BWRAP_BUILD=1 cargo test --manifest-path codex-rs/Cargo.toml -p codex-cli --test vscode vscode_install_remote_scope_repoints_running_bundled_elf -- --nocapture` | passed | Single-test rerun passed. Treat the full-suite failure as timing/order-sensitive until the full baseline command is green. |

## Hotfix Survival Mapping

1. `vsp-codex vscode repair --yes [--quiet]`
   - Present in `codex-rs/cli/src/vscode_cmd.rs`.
   - Existing coverage:
     - `vscode_repair_repoints_bundled_extension_codex_without_settings_backup`
     - `vscode_repair_quiet_suppresses_output`
   - Wrapper startup self-heal is present in `write_vscode_wrapper`: it runs `vscode repair --yes --quiet` guarded by `VSP_CODEX_VSCODE_AUTO_REPAIR=0`.
   - Existing coverage: `vscode_install_writes_wrapper_that_pins_default_vsp_codex_home`.

2. Socket remap `~/.codex/app-server-control/*` -> `~/.vsp-codex/app-server-control/*`
   - Present in `codex-rs/cli/src/main.rs` via `remap_vsp_app_server_socket_path`.
   - Applied to `app-server --listen unix://...` through `remap_vsp_app_server_transport`.
   - Applied to `app-server proxy` socket path handling.
   - Existing coverage is only direct unit-level remap:
     - `vsp_app_server_socket_remaps_official_codex_home_control_socket`
     - `vsp_app_server_socket_keeps_custom_socket_path`
   - Gap: add command-path tests that assert `--listen unix://` default/official paths and `proxy --sock` official paths resolve to `.vsp-codex`, not `.codex`.

3. Compat `codex` wrapper setting `CODEX_HOME=${VSP_CODEX_HOME:-$HOME/.vsp-codex}`
   - **Not satisfied.**
   - `codex-rs/cli/src/main.rs::ensure_codex_compat_entrypoint` currently creates a symlink named `codex` pointing at `vsp-codex` in release builds.
   - No test covers a compat `codex` wrapper with `VSP_CODEX_HOME` fallback semantics.
   - GREEN requirement: replace the symlink behavior with an executable wrapper and test both default HOME fallback and `VSP_CODEX_HOME` override in a temp release/bin layout.

4. App-server output filtering of blank/unknown providers
   - Present in `codex-rs/app-server/src/request_processors/thread_processor.rs`:
     - `normalize_request_model_provider`
     - `normalize_known_persisted_resume_model_provider`
     - `remove_empty_model_provider_config_overrides`
   - Existing coverage includes:
     - `persisted_resume_model_provider_is_used_only_when_configured`
     - `summary_from_stored_thread_uses_fallback_for_blank_model_provider`
     - `c8_m3_thread_resume_empty_string_model_provider_falls_back_to_configured_provider`
     - `c8_m3_thread_resume_omitted_vsp_provider_falls_back_without_persisting_private_provider`

## New Extension Capability Matrix

| Capability | Baseline Status | Evidence / Gap |
| --- | --- | --- |
| `environment/info` RPC | mostly covered | `codex-rs/app-server/tests/suite/v2/environment_info.rs` covers remote info, missing cwd, unknown env, and connection failure. |
| `currentTime/read` | red/gap | `codex-rs/app-server/src/current_time.rs` still has `CURRENT_TIME_REQUEST_TIMEOUT = 10s`. M4 asks for increased timeout. Existing test covers round trip, not timeout budget/cancellation behavior expected by new extension. |
| Rollout turn items in `thread/read` / resume | covered but should be in M4 matrix | `thread_read_can_include_turns`, loaded-thread include-turns tests, and resume/read rollout tests exist. Need explicit M4 matrix row tying extension consumption to `include_turns=true` and rollout item reconstruction. |
| Structured shutdown logs | basic covered | `codex-rs/app-server/tests/suite/logging.rs` asserts JSON shutdown event fields. Need M4 matrix to include standalone app-server connection shutdown and forced/non-forced fields if implement changes touch lifecycle. |
| Native ask UI compatibility for official clients | covered in baseline command | `request_user_input` filtered suite passed; official `openai.chatgpt`, `VS Code`, and desktop remote client aliases are covered. |

## Real-Path Drift Scenarios

Required M4 scenarios are not fully green yet:

1. Official bundled `codex` in temp extension dir
   - Partial existing coverage through temp `.vscode-server/extensions/openai.chatgpt-*/bin/linux-x86_64/codex`.
   - Full-suite baseline failed once on the running ELF replacement test with `Text file busy`.
   - GREEN: stabilize the all-test command, preserve backup behavior, assert shim target, and assert repair is idempotent after official extension reinstall overwrites the shim.

2. Official shim overwrite in Volta/`~/.local/bin`
   - **No observed coverage.**
   - GREEN: temp HOME + temp bin dir simulation where `codex` is overwritten by an official shim, then VSP repair/update path writes the compat wrapper and `codex --version`/app-server launch uses VSP home.

3. Repair-chain self-heal
   - Partial coverage: VSCode wrapper contains guarded quiet repair.
   - GREEN: execute the generated wrapper in a temp HOME with an official bundled codex reintroduced after install, with a fake real `vsp-codex` recording argv/env. Assert:
     - wrapper invokes `vscode repair --yes --quiet` once;
     - recursion guard suppresses nested repair;
     - final exec preserves user args;
     - bundled codex is shimmed back to the VSP wrapper.

4. Socket namespace assertions
   - Partial unit coverage only.
   - GREEN: temp HOME tests for `app-server --listen unix://`, `app-server --listen unix://$HOME/.codex/app-server-control/name.sock`, and `app-server proxy --sock $HOME/.codex/app-server-control/name.sock`, asserting no effective socket path remains under `.codex/app-server-control`.

## Proposed GREEN Matrix

Implementation can start against this RED. Minimum GREEN before M4 acceptance:

1. `codex-cli --test vscode` all-test command is green repeatedly:
   - full command: `CODEX_SKIP_BWRAP_BUILD=1 cargo test --manifest-path codex-rs/Cargo.toml -p codex-cli --test vscode -- --nocapture`
   - include the currently timing-sensitive running bundled ELF scenario.

2. Add/adjust CLI tests for:
   - `vscode repair --yes` and `--quiet` remain idempotent and no-output quiet mode still mutates temp extension shims;
   - wrapper guarded quiet repair actually runs when a temp official extension overwrites bundled `codex`;
   - release compat `codex` is an executable wrapper, not a symlink, and sets `CODEX_HOME=${VSP_CODEX_HOME:-$HOME/.vsp-codex}`;
   - official shim overwrite in temp `~/.local/bin` or Volta-style bin dir self-heals to the VSP wrapper.

3. Add socket command-path regression tests:
   - `--listen unix://` default path resolves under `.vsp-codex/app-server-control`;
   - explicit official namespace socket paths are remapped for both `--listen unix://...` and `proxy --sock ...`;
   - custom non-official socket paths are preserved.

4. Add app-server capability matrix tests:
   - `environment/info` remains green for official extension initialize shape;
   - `currentTime/read` timeout budget is increased and cancellation/error path is asserted;
   - `thread/read include_turns=true` returns reconstructed rollout turn items consumed by extension clients;
   - structured shutdown logs include stable fields: message, exit_reason, remaining_connection_count, shutdown_forced.

5. Keep provider filtering regressions in the M4 validation set:
   - blank request provider and blank persisted provider fall back to configured provider;
   - unknown persisted provider is ignored;
   - omitted VSP provider does not get written back into external rollout metadata.

6. Real UI acceptance remains post-implementation/user-confirmed:
   - VSCode Remote Plugin connects to VSP-Codex;
   - Desktop/Remote APP connects;
   - sockets exist under `~/.vsp-codex/app-server-control` only;
   - `vsp-codex vscode status --json` reports configured without unresolved local User settings issues.

## Current Blocking Gaps

- The required compat `codex` wrapper behavior is missing in release update code; current behavior is a symlink.
- The required full baseline command for `codex-cli --test vscode` failed once on a real-path running bundled executable scenario.
- Current time RPC has a 10s production timeout and lacks a M4 assertion for the requested increased timeout.
- Drift scenarios for official shim overwrite and repair-chain execution are not yet complete.
- Real UI acceptance needs later user confirmation; this test worker did not interact with VSCode/Desktop UI.

---

# C10/M4 GREEN Test Worker Follow-up - Client Connectivity Adaptation

Verdict: `green_with_manual_ui_gap`

Timestamp: `2026-07-02T21:38:17+08:00`

Scope observed: GREEN test worker only. I did not edit workflow lifecycle files, production behavior intentionally, or `.pipeline/reports/C10/M4-connectivity.md`. Test-owned changes were added under allowed assets; `codex-rs/cli/src/main.rs` was touched only inside the existing `#[cfg(test)] mod tests` plus release-only test coverage for the implement worker's compat wrapper.

## Test Asset Changes

- `codex-rs/cli/tests/vscode.rs`
  - Added exact assertions that the VSCode wrapper sets `CODEX_HOME=${VSP_CODEX_HOME:-$HOME/.vsp-codex}`.
  - Added execution coverage proving the generated wrapper runs guarded `vscode repair --yes --quiet`, exports `VSP_CODEX_VSCODE_AUTO_REPAIR=0`, preserves final user args, and uses `VSP_CODEX_HOME` over inherited official `CODEX_HOME`.
  - Stabilized the existing running bundled ELF regression by retrying transient Unix `ETXTBSY` during test setup.
- `codex-rs/cli/src/main.rs`
  - Added command-path tests for `app-server --listen unix://$HOME/.codex/app-server-control/...` remapping to `$HOME/.vsp-codex/app-server-control/...`.
  - Added command-path tests for `app-server proxy --sock $HOME/.codex/app-server-control/...` remapping to VSP home.
  - Added custom socket preservation coverage.
  - Added release-only Unix test that verifies compat `codex` is an executable wrapper, not a symlink, and that both default `$HOME/.vsp-codex` and `VSP_CODEX_HOME` override execution paths work.
- `codex-rs/app-server/tests/suite/v2/current_time.rs`
  - Added integration behavior coverage delaying a `currentTime/read` response by 11 seconds. This exceeds the old 10s budget and verifies the turn still completes with the current-time reminder under the new 60s timeout.

## Commands Executed

| Command | Result | Notes |
| --- | --- | --- |
| `CODEX_SKIP_BWRAP_BUILD=1 cargo test --manifest-path codex-rs/Cargo.toml -p codex-cli --test vscode -- --nocapture` | failed once, then passed | Initial run reproduced existing `vscode_install_remote_scope_repoints_running_bundled_elf` `Text file busy (os error 26)`. After test setup retry stabilization, rerun passed `20 passed; 0 failed`. |
| `CODEX_SKIP_BWRAP_BUILD=1 cargo test --manifest-path codex-rs/Cargo.toml -p codex-app-server --test all request_user_input -- --nocapture` | passed | `18 passed; 0 failed; 685 filtered out`. Temp HOME trust and watcher warnings only. |
| `CODEX_SKIP_BWRAP_BUILD=1 cargo test --manifest-path codex-rs/Cargo.toml -p codex-cli --bin vsp-codex app_server_ -- --nocapture` | passed | `30 passed; 0 failed; 185 filtered out`; includes new listen/proxy socket remap command-path tests. |
| `CODEX_SKIP_BWRAP_BUILD=1 cargo test --manifest-path codex-rs/Cargo.toml -p codex-app-server --test all current_time_read_allows_extension_to_exceed_legacy_timeout -- --nocapture` | passed | `1 passed; 0 failed; 703 filtered out`; turn duration was about 11.25s, proving response after old 10s budget still succeeds. |
| `CODEX_SKIP_BWRAP_BUILD=1 cargo test --release --manifest-path codex-rs/Cargo.toml -p codex-cli --bin vsp-codex release_update_writes_executable_codex_compat_wrapper -- --nocapture` | passed | `1 passed; 0 failed; 215 filtered out`; covers release-only compat wrapper behavior. |
| `cargo fmt --manifest-path codex-rs/Cargo.toml --check` | blocked by workspace shape | Reported `Failed to find targets`; reran the valid workspace form below. |
| `cargo fmt --manifest-path codex-rs/Cargo.toml --all --check` | passed | rustfmt emitted existing nightly-only `imports_granularity` warnings, but check exited 0. |
| `git diff --check` | passed | No whitespace errors. |

## Non-blocking Failed Attempts

- `CODEX_SKIP_BWRAP_BUILD=1 cargo test --manifest-path codex-rs/Cargo.toml -p codex-cli app_server_ -- --nocapture` was too broad and ran `codex-rs/cli/tests/app_server.rs`, which failed because that legacy integration test expects a debug binary named `codex` (`CARGO_BIN_EXE_codex`) while this package builds `vsp-codex`. The corrected `--bin vsp-codex app_server_` command passed.
- `CODEX_SKIP_BWRAP_BUILD=1 cargo test --manifest-path codex-rs/Cargo.toml -p codex-app-server current_time_request_timeout_budget_is_sixty_seconds -- --nocapture` could not compile the app-server lib-test target because pre-existing `thread_processor_tests.rs` initializers are missing `history_mode`, `recency_at`, and `originator` fields. I avoided this unrelated blocker by replacing the private constant unit assertion with a public integration behavior test in `app-server/tests`.

## Coverage Against M4 GREEN Matrix

- Compat wrapper: covered by release-only test; verifies wrapper file, executable bit, fallback content, default home execution, and `VSP_CODEX_HOME` override.
- VSCode wrapper fallback/repair guard: covered by wrapper content assertions and execution test; repair call and final exec both observe the sanitized VSP home and recursion guard.
- Socket command-path remap: covered for `app-server --listen unix://...`, `app-server proxy --sock ...`, and custom non-official socket preservation.
- `currentTime/read` timeout budget: covered behaviorally by delayed 11s response succeeding.
- Baseline VSCode suite: green after test stabilization, including the running bundled ELF replacement scenario.
- Native ask UI compatibility: request_user_input focused suite remains green.

## Remaining Gaps / Risks

- Real VSCode Remote Plugin and Desktop/Remote APP connection acceptance still need manual user confirmation in a real UI environment.
- The app-server lib-test target has unrelated pre-existing compile drift in `thread_processor_tests.rs`; not fixed by this GREEN worker because it is outside the requested connectivity test scope.
- The release compat wrapper test is Unix-only, matching the implemented release helper gate. Windows release behavior remains no-op by cfg and was not exercised here.

---

# C10/M4 Audit-Fix Test Worker - Official Shim Replay

Verdict: `green_with_manual_ui_gap`

Timestamp: `2026-07-02T22:17:00+08:00`

Scope observed: audit-fix test/evidence only. I did not edit production behavior, runtime code outside the existing `#[cfg(test)] mod tests`, the M4 connectivity report, or workflow lifecycle files.

## Audit Finding Closed

The release-only Unix test `release_update_writes_executable_codex_compat_wrapper` now replays the real drift case where the visible `codex` entrypoint is already occupied by an official installer shim before VSP repair runs:

- precondition creates an executable fake official `visible_bin/codex` file;
- precondition asserts the fake official shim exists and does not already contain the VSP wrapper fallback;
- `ensure_codex_compat_entrypoint` is then invoked with `visible_bin` on `PATH`;
- postcondition asserts `visible_bin/codex` is replaced by a non-symlink executable VSP wrapper;
- postcondition asserts wrapper content pins `CODEX_HOME=${VSP_CODEX_HOME:-$HOME/.vsp-codex}`;
- execution coverage verifies both default `$HOME/.vsp-codex` and `VSP_CODEX_HOME` override paths reach the VSP release binary.

## Commands

| Command | Result | Notes |
| --- | --- | --- |
| `CODEX_SKIP_BWRAP_BUILD=1 cargo test --release --manifest-path codex-rs/Cargo.toml -p codex-cli --bin vsp-codex release_update_writes_executable_codex_compat_wrapper -- --nocapture` | passed | `1 passed; 0 failed; 215 filtered out`. |
| `cargo fmt --manifest-path codex-rs/Cargo.toml --all --check` | passed | rustfmt emitted existing nightly-only `imports_granularity` warnings, check exited 0. |
| `git diff --check` | passed | No whitespace errors. |

## Remaining Manual UI Gap

Real UI acceptance remains manual/post-implementation: VSCode Remote Plugin and Desktop/Remote APP connection checks were not exercised by this audit-fix worker. The current automated closure is limited to release installer shim drift replay and wrapper execution behavior.
