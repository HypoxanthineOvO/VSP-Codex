# M6 VSP Branding and Resume Compatibility Review

Timestamp: 2026-05-25T13:17:26+08:00

## Scope

- Fixed local `vsp-codex resume` session compatibility after the shared `~/.codex` sessions patch.
- Completed M6 visible VSP branding and official prompt cleanup.
- Kept MCP, Skills, Plugins, OpenAI-compatible providers, and `exec-server --remote` available.

## Changes

- Resume picker no longer filters local sessions by the currently selected provider. This preserves compatibility with official Codex sessions whose metadata uses a legacy provider name.
- First-run welcome, session header, `/status`, and non-interactive human output now use `VSP Codex` branding.
- Startup model availability NUX, official model migration prompt, announcement prewarm, and random official tooltip display are disabled under VSP.
- Plugin and app-link UI keeps the feature surface but removes official ChatGPT/OpenAI labels from user-facing controls:
  - `OpenAI Curated` displays as `VSP Curated`.
  - app install/manage actions use generic app wording.
  - plugin detail pages use marketplace display labels rather than raw marketplace identifiers.
- Login device-code prompt title was rebranded for consistency even though official login entrypoints remain disabled.

## Validation

- `just fmt`: passed
- `just test -p codex-tui status_snapshot`: passed
- `just test -p codex-tui session_header`: passed
- `just test -p codex-tui clear_ui`: passed
- `just test -p codex-tui plugins_popup`: passed
- `just test -p codex-tui vsp_local_picker_does_not_filter_by_current_provider`: passed
- `just test -p codex-tui prepare_startup_tooltip_override_is_disabled_for_vsp`: passed
- `just test -p codex-tui model_migration_prompt_shows_for_hidden_model`: passed
- `just test -p codex-tui app_link_view`: passed
- `just test -p codex-tui session_info_availability_nux_tooltip_snapshot`: passed
- `just test -p codex-tui marketplace_add_success_refreshes_to_new_marketplace_tab`: passed
- `just test -p codex-tui plugin_detail_popup`: passed
- `just test -p codex-tui tooltips`: passed
- `cargo check -p codex-exec -p codex-login -p codex-tui`: passed
- `cargo insta pending-snapshots --manifest-path tui/Cargo.toml`: no pending snapshots
- `just fix -p codex-tui -p codex-exec -p codex-login`: passed
- `cargo build --bin vsp-codex`: passed
- `git diff --check`: passed
- `/home/heyx/.local/bin/vsp-codex --version`: `vsp-codex 0.0.0`

## Residual Notes

- Raw `ChatGPT`/`OpenAI` strings still exist in tests, comments, disabled official paths, protocol-compatible wording, and upstream model migration fixture data. VSP runtime paths now gate the official prompts that would show those messages.
- Plain `vsp-codex resume` remains scoped to the current working directory unless the user asks for all sessions. Use `vsp-codex resume --all` to browse all shared sessions.
