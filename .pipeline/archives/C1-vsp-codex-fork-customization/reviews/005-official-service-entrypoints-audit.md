# M5 Record: Official Service Entrypoints

## Scope

M5 hides or disables official OpenAI/Codex service entrypoints in VSP Codex while preserving the internals that are still useful for merge compatibility.

Preserved by design:

- MCP client/server functionality.
- Skills and Plugins systems.
- OpenAI-compatible BaseURL + API Key providers.
- `vsp-codex exec-server --remote`, per explicit user decision.

## Subagent Audit

Audit worker:

- Archimedes: `019e5d3c-71cb-7da1-a3f6-ea6530c79210`

Read-only audit findings used for implementation:

- Hide or disable CLI `login`, `logout`, `cloud`, `remote-control`, and `update`.
- Hide or disable app-server remote-control entrypoints, except preserve `exec-server --remote`.
- Hide TUI `/feedback` and `/logout`, and reject direct dispatch if invoked internally.
- Hide OpenAI-auth providers from `/provider` while preserving custom non-OpenAI-auth providers.
- Remove ChatGPT usage/account guidance from `/status` in VSP mode and stop rate-limit prefetch.
- Stop TUI update checks and announcement prewarm requests against official OpenAI/GitHub release channels.

## Implementation Summary

- Added a VSP official-service policy gate in `codex-rs/tui/src/vsp_provider.rs`.
- CLI:
  - Hidden `login`, `logout`, `cloud`, `remote-control`, and `update` from top-level help.
  - Runtime-rejected those commands with a VSP-specific disabled-service message.
  - Rejected `app-server daemon enable-remote-control` and `app-server daemon bootstrap --remote-control`.
  - Disabled direct `app-server --remote-control` behavior by warning and starting without remote control.
- TUI:
  - Hid `/feedback` and `/logout` from slash command resolution.
  - Direct slash dispatch for `/feedback` and `/logout` now shows the VSP disabled-service message instead of emitting upload/logout events.
  - `/provider` filters out providers that require OpenAI auth, and direct provider selection rejects them.
  - `/status` no longer shows ChatGPT usage links or ChatGPT login guidance under the VSP policy.
  - Rate-limit prefetch, update checks, update popups, and remote announcement prewarm are disabled.
- Doctor:
  - `doctor updates` now reports that official update checks are disabled and does not probe GitHub/Homebrew/npm release channels.

## Validation

- `just fmt`: passed.
- `cargo check -p codex-cli -p codex-tui`: passed.
- `just test -p codex-cli official_service_cli_commands_are_disabled`: passed.
- `just test -p codex-cli vsp_codex_help_uses_vsp_codex_usage`: passed.
- `just test -p codex-cli login_with_api_key_is_disabled_in_vsp`: passed.
- `just test -p codex-cli login_with_access_token_is_disabled_in_vsp`: passed.
- `just test -p codex-cli update_check_does_not_probe_official_release_channels`: passed.
- `just test -p codex-tui official_service_commands_are_hidden`: passed.
- `just test -p codex-tui official_service_commands_do_not_resolve_for_dispatch`: passed.
- `just test -p codex-tui slash_logout_is_disabled_in_vsp`: passed.
- `just test -p codex-tui slash_feedback_is_disabled_in_vsp`: passed.
- `just test -p codex-tui user_typing_provider_slash_command_opens_provider_picker`: passed.
- `just test -p codex-tui prefetch_rate_limits_is_gated_on_chatgpt_auth_provider`: passed.
- `just test -p codex-tui official_openai_auth_providers_are_hidden_from_vsp_selection`: passed.
- `just test -p codex-tui vsp_provider_is_allowed_even_when_complete`: passed.
- `just test -p codex-tui status_model_provider_uses_bedrock_runtime_base_url_and_gates_usage_link`: passed.
- `cargo insta pending-snapshots --manifest-path tui/Cargo.toml`: no pending snapshots.
- `just fix -p codex-cli -p codex-tui`: passed.
- `git diff --check`: passed.

## Residual Risk

- Official-service code is intentionally gated rather than deleted. This lowers merge cost but means unreachable official code remains in the tree.
- Plugin/app-link UI still contains ChatGPT Marketplace wording because Plugins are explicitly preserved; this belongs to M6 branding/content cleanup rather than M5 entrypoint disabling.
- `exec-server --remote` remains available as requested. Upstream remote authentication details may still reference ChatGPT/Agent Identity; this was not disabled in M5.
- Full workspace tests were not run after M5. Validation focused on changed crates and official-service regression tests.
