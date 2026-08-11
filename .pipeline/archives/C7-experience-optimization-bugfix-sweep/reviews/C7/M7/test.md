# C7/M7 Test Worker Report

## Changed Files

- `codex-rs/tui/src/update_versions.rs`
- `codex-rs/tui/src/update_action.rs`
- `codex-rs/tui/src/bottom_pane/request_user_input/mod.rs`
- `codex-rs/tui/src/chatwidget/active_subagents.rs`

## Tests Added

- `c7_m7_vsp_manifest_parses_required_metadata_and_selects_channel_target`
  - Red test for VSP manifest schema, `minimum_updater`, channel, target, archive URL, SHA-256, notes URL, and current-version selection.
- `c7_m7_vsp_update_source_never_uses_official_openai_endpoints`
  - Red test requiring VSP update source endpoints to derive from the configured manifest URL and reject official OpenAI/GitHub installer endpoints.
- `c7_m7_checksum_mismatch_rejects_archive_without_switching_current`
  - Red test requiring SHA-256 mismatch to fail with `ChecksumMismatch` and report that `current` would not be switched.
- `c7_m7_vsp_standalone_update_action_uses_manifest_archive_not_official_installers`
  - Red test requiring standalone VSP updates to use the selected manifest archive, not `chatgpt.com/codex` installers, `openai/codex`, or `@openai/codex`.
  - Also rejects exposing this standalone action for non-standalone install contexts.
- `c7_m7_vsp_release_update_card_exposes_structured_non_silent_choices`
  - Red/guard test for M5 structured update confirmation choices: `update_now`, `skip_once`, `skip_this_version`, `view_notes`, and `cancel`.
  - Explicitly rejects a silent `auto_update` default.
- `c7_m7_channel_default_name_pool_priority_preserves_explicit_and_configured_names`
  - Red test for release-channel name metadata priority: explicit nickname first, user-configured pool second, then VSP internal/GitLab defaults or external GitHub defaults.

## Commands Run

- `cargo test -p codex-tui c7_m7 --no-run`
  - Expected compile-red failure.

## Expected Failure Points

- `update_versions.rs` has no production VSP updater API yet:
  - `VspReleaseManifest`
  - `VspReleaseQuery`
  - `VspReleaseChannel`
  - `VspReleaseSource`
  - `VspSelectedRelease`
  - `VspStandaloneUpdatePlan`
  - `VspStandaloneUpdateErrorKind`
- `update_action.rs` has no `UpdateAction::from_vsp_manifest_release` yet.
- `active_subagents.rs` has no release-channel metadata API yet:
  - `ReleaseChannelMetadata`
  - `ReleaseBuildSource`
  - `SubagentDisplayNameResolver::with_release_channel_metadata`
- Existing structured card helper currently emits option id `skip_version`; the C7/M7 guard expects stable id `skip_this_version`.

## Implement Worker Next Steps

- Add production VSP release manifest parsing and selection by current version, channel, and target.
- Add a VSP update source that never calls official OpenAI release or installer endpoints in VSP mode.
- Add checksum-verified standalone archive update planning that aborts before switching `current` on digest mismatch.
- Add `UpdateAction::from_vsp_manifest_release` guarded to standalone installs and driven by selected VSP archive metadata.
- Wire VSP structured update confirmation through the M5 request-user-input card semantics, including view notes, skip once, skip this version, cancel, and no silent update default.
- Add release-channel metadata for Subagent name defaults while preserving explicit nicknames and user-configured nickname candidates above channel defaults.
