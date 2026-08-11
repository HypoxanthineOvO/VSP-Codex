# M6.1 Provider List and Version Patch

## Summary

This follow-up patch tightens the VSP Provider picker and assigns the fork a visible version.

## Versioning Decision

Use `official-base-vsp.N`.

- Current version: `0.134.0-vsp.1`
- `0.134.0` tracks the official Codex Rust release line this fork is based on.
- `vsp.1` identifies the first VSP internal release on that baseline.
- VSP-only patches should increment the suffix, for example `0.134.0-vsp.2`.
- Rebase or merge to a newer official line should reset the suffix, for example `0.135.0-vsp.1`.

## Code Changes

- `codex-rs/Cargo.toml`
  - Workspace version changed from `0.0.0` to `0.134.0-vsp.1`.
- `codex-rs/Cargo.lock`
  - Workspace package versions refreshed by Cargo.
- `codex-rs/tui/src/vsp_provider.rs`
  - The Provider picker now hides built-in provider IDs:
    - `openai`
    - `amazon-bedrock`
    - `ollama`
    - `lmstudio`
  - Custom OpenAI-compatible providers remain allowed when they do not require OpenAI auth.
- `codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__user_simulation_provider_picker_from_slash.snap`
  - Snapshot updated to show only `VSP Provider` in the test fixture.

## Validation

- `just test -p codex-tui built_in_providers_are_hidden_from_vsp_selection`: passed
- `just test -p codex-tui custom_provider_is_allowed`: passed
- `just test -p codex-tui user_typing_provider_slash_command_opens_provider_picker`: passed
- `cargo build --bin vsp-codex`: passed
- `just fmt`: passed
- `just fix -p codex-tui -p codex-cli`: passed
- `cargo insta pending-snapshots --manifest-path tui/Cargo.toml`: no pending snapshots
- `git diff --check`: passed
- `/home/heyx/.local/bin/vsp-codex --version`: `vsp-codex 0.134.0-vsp.1`

## Residual Risk

- Built-in provider crates are still present in the codebase to keep merge cost low; this patch hides them from VSP selection rather than deleting their implementation.
- User-defined custom providers still appear intentionally, because the VSP fork keeps OpenAI-compatible BaseURL/API Key provider support.
