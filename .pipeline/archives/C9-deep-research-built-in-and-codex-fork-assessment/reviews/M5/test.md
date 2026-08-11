# C9 M5 Test Report

## Verdict

`local_release_ready_with_broad_suite_risk`

Local focused regression and release-readiness checks passed for `0.140.0-vsp.0`. The broad app-server integration suite is not clean in this environment and is recorded as a release risk, not as a remote-release authorization.

No push, tag, GitLab release, or asset upload was performed.

## Passed Checks

- `cargo fmt --manifest-path codex-rs/Cargo.toml --all --check`
  - Passed. Stable rustfmt still warns that `imports_granularity = Item` is nightly-only.
- `cargo test --manifest-path codex-rs/Cargo.toml -p codex-core workflow_routing -- --nocapture`
  - Passed: 10 tests.
- `cargo test --manifest-path codex-rs/Cargo.toml -p codex-app-server-protocol tool_request_user_input -- --nocapture`
  - Passed: 4 filtered tests.
- `RUST_MIN_STACK=16777216 cargo test --manifest-path codex-rs/Cargo.toml -p codex-app-server --lib in_process_start_clamps_zero_channel_capacity -- --nocapture`
  - Passed: 1 filtered test.
- `cargo test --manifest-path codex-rs/Cargo.toml -p codex-app-server-protocol`
  - Passed: 240 lib tests and 2 schema fixture tests.
- `cargo test --manifest-path codex-rs/Cargo.toml -p codex-arg0`
  - Passed: 7 tests.
- `cargo test --manifest-path codex-rs/Cargo.toml -p codex-rollout`
  - Passed: 72 lib tests and 8 `official_codex_discovery` tests.
- `cargo test --manifest-path codex-rs/Cargo.toml -p codex-core-plugins`
  - Passed: 232 tests.
- `cargo test --manifest-path codex-rs/Cargo.toml -p codex-cli --test vscode -- --nocapture`
  - Passed: 14 tests.
- `cargo test --manifest-path codex-rs/Cargo.toml -p codex-tui workflow_slash_routing -- --nocapture`
  - Passed: 9 tests.
- `cargo build --manifest-path codex-rs/Cargo.toml --release --bin vsp-codex`
  - Passed.
- `codex-rs/target/release/vsp-codex --version`
  - Passed: `vsp-codex 0.140.0-vsp.0`.
- `sha256sum -c .pipeline/release/v0.140.0-vsp.0/vsp-codex-preview-linux-x64-gnu.tar.gz.sha256`
  - Passed.
- `python3 -m json.tool` on both release manifests
  - Passed.
- package extraction smoke for `.pipeline/release/v0.140.0-vsp.0/vsp-codex-preview-linux-x64-gnu.tar.gz`
  - Passed; package contains `bin/vsp-codex`, `codex-package.json`, `codex-path/rg`, and `codex-resources/bwrap`.
- `git diff --check`
  - Passed.

## Broad Suite Findings

- `cargo test --manifest-path codex-rs/Cargo.toml --workspace`
  - First run failed before tests because this host lacks `libcap.pc` for bundled `codex-bwrap`.
- `CODEX_SKIP_BWRAP_BUILD=1 cargo test --manifest-path codex-rs/Cargo.toml --workspace`
  - Exposed several stale test fixtures after upstream API drift. These were fixed and verified with package-level tests.
- `RUST_MIN_STACK=16777216 cargo test --manifest-path codex-rs/Cargo.toml -p codex-app-server`
  - Lib tests passed: 254 lib tests and 1 main test.
  - `codex-app-server --test all` failed broadly: 245 passed, 376 failed, 2 ignored. Failures are dominated by app-server initialization/websocket deadline timeouts and cascading integration failures in this environment.

## Fixes Applied During M5

- Updated Ask protocol tests for Workflow decision metadata defaults.
- Made `autoResolutionMs` omit `None` during serialization.
- Updated `CodexPackageLayout`, rollout recorder, core-plugin, app-server bespoke handler, and thread processor test fixtures for upstream API shape changes.
- Removed two `vsp-codex` release-build warnings from stale CLI code.
- Changed in-process app-server `channel_capacity = 0` handling to use the default capacity instead of constructing a degenerate queue.

## Release Gate Status

Local release artifact checks passed. Remote publication is still gated:

- No release commit created.
- No tag created.
- No push performed.
- No GitLab release created.
- No assets uploaded.
