# C7/M7 Implementation Worker Report

## Scope

Implemented the VSP release-channel metadata and manifest-driven standalone updater path for M7.

The original implement subagent `019e75fd-a596-7491-8c0b-1a31c2945d21` did not converge and was closed while still running. Per the recorded continuation, the main agent took over the bounded production implementation and used focused green validation plus an independent audit/recheck to compensate for the worker-separation gap.

## Changed Files

- `codex-rs/tui/src/update_versions.rs`
- `codex-rs/tui/src/update_action.rs`
- `codex-rs/tui/src/updates.rs`
- `codex-rs/tui/src/update_prompt.rs`
- `codex-rs/tui/src/history_cell/notices.rs`
- `codex-rs/tui/src/lib.rs`
- `codex-rs/tui/src/app.rs`
- `codex-rs/tui/src/chatwidget/active_subagents.rs`
- `codex-rs/cli/src/main.rs`
- `codex-rs/cli/Cargo.toml`
- `codex-rs/Cargo.lock`

## Implemented Behavior

- Added VSP release manifest parsing and selection:
  - channel-aware `VspReleaseChannel`
  - target/current-version query selection
  - VSP semver comparison including `-vsp.N`
  - manifest source configured by `VSP_CODEX_RELEASE_MANIFEST_URL`
  - channel override by `VSP_CODEX_RELEASE_CHANNEL`
- Added release-source safety boundaries:
  - VSP manifest URL cannot point at official OpenAI/GitHub/chatgpt update endpoints.
  - Manifest-selected archive and notes URLs cannot point at official OpenAI/GitHub/chatgpt update endpoints.
  - CLI `--from-vsp-manifest` validates the same manifest/archive/notes URL boundaries before downloading.
- Added checksum-safe standalone update planning:
  - SHA-256 mismatch returns `ChecksumMismatch`.
  - mismatch explicitly reports `would_switch_current=false`.
  - CLI verifies downloaded archive bytes before extract/staging/current switch.
- Added VSP manifest update action:
  - `UpdateAction::VspStandaloneManifest`
  - command surface: `vsp-codex update --from-vsp-manifest ...`
  - release notes URL and latest version metadata flow through the TUI.
  - `UpdateAction` is `Clone` instead of `Copy` to carry manifest metadata.
- Added release-only CLI updater path:
  - downloads selected archive
  - verifies SHA-256
  - extracts `tar.gz` into a staging directory
  - accepts package layout and legacy standalone layout
  - moves staged release under `packages/standalone/releases`
  - updates `packages/standalone/current`
  - uses async/await in the async CLI path instead of nesting `Handle::block_on`.
- Added startup update semantics:
  - VSP mode only checks for updates when a VSP manifest URL is configured.
  - startup prompt remains explicit and non-silent.
  - VSP history update notice does not fall back to official npm/brew/chatgpt updater actions when manifest selection is unavailable.
- Added release-channel Subagent name metadata:
  - user explicit nickname remains highest priority.
  - configured name pool remains above release-channel defaults.
  - internal/local/GitLab channels get VSP Lab name defaults.
  - external GitHub channel metadata can use ancient-name defaults.

## Audit Fixes

First audit result: `fail`.

Fixed findings:

- Critical: manifest-selected archive URL could point at official OpenAI/GitHub/chatgpt endpoints.
  - Added `reject_official_update_endpoint`.
  - Called it from `VspSelectedRelease::new` for archive and notes URLs.
  - Called it from CLI manifest-argument validation before download.
  - Added test `c7_m7_manifest_selected_urls_reject_official_update_endpoints`.
- Warning: VSP update history notice could surface official updater commands via fallback.
  - Added `update_history_action_for_vsp_notice`.
  - VSP mode now returns only the selected VSP manifest action; no official fallback.
  - Added test `c7_m7_vsp_update_notice_does_not_fallback_to_official_update_actions`.

## Validation

- `cargo test -p codex-tui c7_m7 -- --nocapture`
  - pass: 8 tests.
- `cargo test -p codex-cli c7_m7 -- --nocapture`
  - pass: 1 test.
- `cargo check -p codex-cli --release --bin vsp-codex`
  - pass.
- `cargo fmt --check --package codex-cli --package codex-tui`
  - pass; rustfmt prints the repository's existing stable-toolchain warning for nightly-only `imports_granularity`.
- `git diff --check`
  - pass.

Known non-M7 warning:

- `core/src/session/mod.rs:1753` still warns about deprecated `turn_context.cwd`; this is pre-existing C7 Workflow routing debt, not introduced by M7.

## Residual Risks

- Windows `current` switching currently uses `std::os::windows::fs::symlink_dir` after deleting the existing path. This may require Developer Mode/admin privileges and is not atomic on failure.
- Manual standalone validation is still required against a real packaged release:
  - `vsp-codex --version`
  - `vsp-codex vscode status --json`
  - installed launcher path resolving through `packages/standalone/current`
- Private GitLab authentication is not implemented in C7. The manifest and assets must be reachable by the installed client.
