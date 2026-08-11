# Desktop/VSCode Native Ask Compatibility

## Conclusion

VSP Codex now has a durable local routing path for official VSCode/ChatGPT extension and Codex Desktop/remote clients:

- VSCode integration continues to route the official extension through the VSP wrapper and bundled shim.
- Desktop/remote-control daemon now prefers the VSP standalone package entrypoint at `packages/standalone/current/bin/vsp-codex`.
- Question Tool requests map to native Ask UI popups for known popup-capable official clients, even when they do not explicitly declare the VSP `nativeAskUi` capability.
- Unknown or unsupported clients still receive the Markdown fallback and do not receive `item/tool/requestUserInput`.

## Technical Approach

- Added app-server compatibility detection for official popup-capable client names, including `codex_vscode`, `openai.chatgpt`, and Codex Desktop/remote names.
- Kept the compatibility narrow: `item/tool/requestUserInput` is delivered when native Ask UI is effective, but other experimental server requests and client requests still require the existing `experimentalApi` capability.
- Updated managed daemon binary discovery to follow package metadata layout first:
  - `packages/standalone/current/bin/vsp-codex`
  - `packages/standalone/current/bin/codex`
  - legacy flat `vsp-codex`
  - legacy flat `codex`
- Updated install/user/daemon docs to describe VSCode Remote bundled shim, Desktop daemon managed path, native Ask UI mapping, and the current release SHA.

## Modified Files

- `codex-rs/app-server/src/native_ask_ui.rs`
- `codex-rs/app-server/src/request_processors/initialize_processor.rs`
- `codex-rs/app-server/src/transport.rs`
- `codex-rs/app-server/src/transport_tests.rs`
- `codex-rs/app-server/tests/suite/v2/request_user_input.rs`
- `codex-rs/app-server-daemon/src/managed_install.rs`
- `codex-rs/app-server-daemon/src/lib.rs`
- `codex-rs/app-server-daemon/src/managed_install_tests.rs`
- `codex-rs/app-server-daemon/README.md`
- `docs/install.md`
- `docs/vsp-codex.md`

## Validation

Passed:

- `cargo fmt --manifest-path codex-rs/Cargo.toml --all`
- `cargo fmt --manifest-path codex-rs/Cargo.toml --all --check`
- `cargo test --manifest-path codex-rs/Cargo.toml -p codex-app-server --lib native_ask_ui -- --nocapture`
- `cargo test --manifest-path codex-rs/Cargo.toml -p codex-app-server --lib request_user_input_requests_are -- --nocapture`
- `cargo test --manifest-path codex-rs/Cargo.toml -p codex-app-server-daemon managed_codex_bin -- --nocapture`
- `cargo test --manifest-path codex-rs/Cargo.toml -p codex-app-server --test all request_user_input_official_ -- --nocapture`
- `cargo test --manifest-path codex-rs/Cargo.toml -p codex-app-server --test all request_user_input_desktop_remote_client_uses_native_ask_ui_without_declared_capabilities -- --nocapture`
- `cargo test --manifest-path codex-rs/Cargo.toml -p codex-app-server --test all request_user_input_native_ask_ui_false_uses_readable_fallback_and_completes -- --nocapture`
- `cargo test --manifest-path codex-rs/Cargo.toml -p codex-app-server --test all request_user_input_round_trip -- --nocapture`
- `cargo test --manifest-path codex-rs/Cargo.toml -p codex-cli --test vscode vscode_install_remote_scope_repoints_bundled_extension_codex -- --nocapture`
- `cargo test --manifest-path codex-rs/Cargo.toml -p codex-cli --test vscode vscode_install_app_server_probe_invokes_initialize_with_official_extension_contract -- --nocapture`
- `git diff --check`
- `~/.vsp-codex/packages/standalone/current/bin/vsp-codex --version`

Observed warnings were existing stable rustfmt `imports_granularity` warnings and existing `codex-core` dead-code warnings.

## Local Deployment

Completed after the source validation:

- Built `codex-rs/target/release/vsp-codex`.
- New build SHA256: `0c8ccc30acf83167e7e9069bb4d3c02710ac06247484d853852206c4ea6c06bb`.
- Atomically replaced `~/.vsp-codex/packages/standalone/current/bin/vsp-codex`.
- Previous installed binary SHA256: `5a065716207e32bc42aca8a1475bd72e393c114717c31e5cc335d6c0f4e493a7`.
- Backup path: `~/.vsp-codex/packages/standalone/backups/vsp-codex-desktop-native-ask-20260619T151637+0800`.
- Installed runtime smoke: `vsp-codex 0.140.0-vsp.0`.

## Runtime Smoke

Completed during the completion audit:

- Started the installed runtime directly with `~/.vsp-codex/packages/standalone/current/bin/vsp-codex app-server --stdio`.
- Used temporary `CODEX_HOME` directories and a local mock Responses HTTP server.
- Initialized as `openai.chatgpt` with `experimentalApi=false` and `nativeAskUi=false`.
- Initialized as `codex_chatgpt_desktop_remote` with `experimentalApi=false` and `nativeAskUi=false`.
- In both cases, a real mock Responses `request_user_input` tool call was delivered to the client as native `item/tool/requestUserInput` with `itemId=call1` and one question.
- The current VSCode Remote bundled extension executable `/home/heyx/.vscode-server/extensions/openai.chatgpt-26.609.30741-linux-x64/bin/linux-x86_64/codex` is a shim to `/home/heyx/.vsp-codex/vscode/vsp-codex`; the wrapper execs `~/.vsp-codex/packages/standalone/current/bin/vsp-codex`.
- Running the bundled extension path with `--version` returns `vsp-codex 0.140.0-vsp.0`.
- `vsp-codex app-server daemon version` could not connect because no daemon control socket was running. I did not start or restart the daemon for this audit; daemon binary selection is covered by focused `managed_codex_bin` tests and the installed managed path/hash evidence.

## Completion Audit

Requirement-by-requirement result:

- VSCode Plugin / Remote SSH connects to local VSP-Codex instead of official Codex: proven by source tests for `vscode_install_remote_scope_repoints_bundled_extension_codex`, current installed bundled shim inspection, and bundled `codex --version` returning `vsp-codex 0.140.0-vsp.0`.
- Codex Desktop SSH / remote-control daemon uses local VSP-Codex instead of official Codex when VSP is installed: proven by `managed_codex_bin` tests, installed managed path/hash, and source inspection showing `Daemon::from_environment()` stores `managed_codex_bin(codex_home)`, `backend_paths_with_bin()` passes it into `BackendPaths.codex_bin`, and `PidBackend::start()` invokes `Command::new(&self.codex_bin)`.
- Question Tool maps to native client popups for official VSCode/ChatGPT and Desktop remote clients: proven by app-server official-client and Desktop-remote integration tests plus installed-runtime stdio smoke using `openai.chatgpt` and `codex_chatgpt_desktop_remote` with `experimentalApi=false` and `nativeAskUi=false`.
- Unknown clients still receive readable fallback instead of unsupported native requests: proven by existing fallback tests and transport tests.
- Broad experimental API behavior remains gated: proven by transport tests and by using non-experimental official-client smoke while `item/tool/requestUserInput` still delivered.

Additional focused tests re-run at completion:

- `cargo test --manifest-path codex-rs/Cargo.toml -p codex-app-server-daemon managed_codex_bin -- --nocapture` (4 passed)
- `cargo test --manifest-path codex-rs/Cargo.toml -p codex-app-server --test all request_user_input_official_ -- --nocapture` (2 passed)
- `cargo test --manifest-path codex-rs/Cargo.toml -p codex-app-server --test all request_user_input_desktop_remote_client_uses_native_ask_ui_without_declared_capabilities -- --nocapture` (1 passed)
- `cargo test --manifest-path codex-rs/Cargo.toml -p codex-cli --test vscode vscode_install_remote_scope_repoints_bundled_extension_codex -- --nocapture` (1 passed)
- YAML parse for `.pipeline/state.yaml` and `.pipeline/log.yaml`
- `git diff --check`
- Process-leak check for cargo/rustc/temp app-server smoke processes

## Expected Result

- VSCode Plugin Remote SSH sessions that launch through the official extension continue to use local VSP Codex.
- Codex Desktop SSH / remote-control paths that rely on the managed standalone daemon use local VSP Codex when installed under `~/.vsp-codex`.
- Question Tool requests render in native VSCode/Desktop Ask UI popups for known official popup-capable clients.
- Unsupported clients keep the readable Markdown fallback.

## Risks And Follow-Up

- Client-name compatibility depends on current official client identifiers. Additional future client names should be added to `native_ask_ui.rs`.
- This change is local, verified, rebuilt, and deployed to this machine, but not yet pushed or published as a refreshed same-version release asset.
- Existing running VSCode/Desktop app-server processes may need reload/restart to pick up a newly built binary.
