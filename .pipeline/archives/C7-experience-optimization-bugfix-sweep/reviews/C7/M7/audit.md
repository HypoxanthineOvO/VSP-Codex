---
verdict: pass
cycle: C7
milestone: M7
title: VSP Release Channels, Naming Metadata, And Updater
audit_type: read-only
auditor: "庄周 the 20th"
---

# C7/M7 Audit Report

## Verdict

pass

## Findings

No blocking findings.

The first audit found two issues and both were fixed before completion:

- Manifest-selected `archive_url` and `notes_url` now go through `reject_official_update_endpoint` in `VspSelectedRelease::new`, and CLI `--from-vsp-manifest` arguments are revalidated in `VspManifestUpdateSelection::try_from`.
- VSP update history notices now use `update_history_action_for_vsp_notice`; when official services are disabled, it returns only the manifest action and does not fall back to `get_update_action()`.

After the pass recheck, the main agent also tightened the remaining non-blocking VSP notice text so no-action VSP update notices omit official GitHub/OpenAI update links and point at `VSP_CODEX_RELEASE_MANIFEST_URL`.

## Validation Reviewed

Reviewed relevant diff and call paths in:

- `codex-rs/tui/src/update_versions.rs`
- `codex-rs/cli/src/main.rs`
- `codex-rs/tui/src/history_cell/notices.rs`
- `codex-rs/tui/src/app.rs`
- supporting paths in `update_action.rs`, `updates.rs`, and `update_prompt.rs`

Reviewed added/related tests:

- `c7_m7_manifest_selected_urls_reject_official_update_endpoints`
- `c7_m7_vsp_update_notice_does_not_fallback_to_official_update_actions`
- `c7_m7_vsp_update_notice_without_action_omits_official_links`
- existing VSP manifest updater command/action coverage

Final main-agent validation after the audit recheck:

- PASS: `cargo test -p codex-tui c7_m7 -- --nocapture` — 9 tests
- PASS: `cargo test -p codex-cli c7_m7 -- --nocapture` — 1 test
- PASS: `cargo check -p codex-cli --release --bin vsp-codex`
- PASS: `cargo fmt --check --package codex-cli --package codex-tui`
- PASS: `git diff --check`

## Residual Risks

- Windows `current` update uses `std::os::windows::fs::symlink_dir` after deleting the existing path. This can require Developer Mode/admin privileges and is not atomic on failure.
- Standalone path preservation depends on installed launchers and VSCode `cliExecutable` targeting the stable `packages/standalone/current` entry rather than a versioned release directory. Manual validation should still cover `vsp-codex --version` and `vsp-codex vscode status --json` after an update.
- Private GitLab authentication remains out of C7 scope; manifests and assets must be reachable by installed clients.
