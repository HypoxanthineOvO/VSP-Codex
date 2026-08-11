# M7 VSP Release Channels, Naming Metadata, And Updater

## Objective

Provide a VSP-specific release/update path and release-channel metadata, including internal/external Subagent name defaults and a startup update prompt with one-key standalone update.

## Source Of Truth

- P1 issues: Issue 3 internal/local/GitLab builds and external GitHub builds need different Subagent display-name pools; Issue 8 VSP release update prompt and auto-update path.
- P2 technical solution: build a VSP updater parallel to the official OpenAI updater; use manifest-driven channel metadata and a structured update confirmation card from M5.
- Dependencies: M2 name resolver/channel default hooks; M5 confirmation card semantics.
- Relevant modules: `codex-rs/tui/src/updates.rs`, `codex-rs/tui/src/update_prompt.rs`, `codex-rs/tui/src/update_action.rs`, `codex-rs/tui/src/history_cell/notices.rs`, `codex-rs/cli`, VSP provider official-service gates, installer docs, and standalone release layout.

## Subworker Assignment Plan

Status: authorized, strict worker separation.

- `test`
  - Scope: owns failing tests, fixtures, assertions, and snapshots for VSP manifest parsing, target/channel selection, checksum failure, standalone update action, update prompt cards, skip behavior, and channel name metadata.
  - Write boundary: test/snapshot files, mock manifest/archive fixtures when needed, and `.pipeline/reviews/C7/M7/test.md` only.
  - Expected artifact: `.pipeline/reviews/C7/M7/test.md`.
- `implement`
  - Scope: owns production VSP release manifest structs, update source, standalone update action, prompt integration, channel metadata, installer/release metadata, and docs needed for implemented behavior.
  - Must not create, edit, rewrite, or satisfy tests owned by `test`.
  - Expected artifact: `.pipeline/reviews/C7/M7/implement.md`.
- `audit`
  - Scope: read-only review of updater security, no official OpenAI endpoint usage, checksum behavior, standalone path safety, VSCode integration preservation, update prompt semantics, and channel nickname priority.
  - Must reject silent update behavior by default or checksum-failure paths that switch `current`.
  - Expected artifact: `.pipeline/reviews/C7/M7/audit.md`.

Workers must not revert user or other-worker edits. Required user decisions must use `request_user_input` when available. Destructive operations, external network use, release publication, and system-level dependency installation require explicit confirmation/authorization.

## Technical Solution

Build a VSP updater separate from official OpenAI update checks. The TUI checks a VSP manifest cache/source asynchronously or from cache, then shows a VSP-branded structured update confirmation card. Standalone `Update now` downloads the selected archive, verifies SHA-256, unpacks into the standalone releases directory, atomically updates `current`, and asks the user to restart. Release-channel metadata identifies internal/local/GitLab builds versus external GitHub builds for default Subagent name pools. User-configured nicknames remain higher priority than channel defaults.

## Technical Route

1. Add `vsp_release_manifest` structs and tests for version, target, channel, URL, SHA-256, notes URL, and minimum updater fields.
2. Add config/build metadata for release channel and manifest URL, with safe local/debug defaults.
3. Add a VSP update source that does not call `openai/codex` GitHub release APIs.
4. Add VSP standalone update CLI/action guarded to standalone release installs.
5. Integrate update prompt with M5 structured confirmation semantics: update now, skip this time, skip this version, view notes, cancel.
6. Add installer/release metadata so installed builds know channel, target, version, and manifest URL.
7. Integrate channel default Subagent name pools with M2's resolver, preserving user-configured nicknames above channel defaults.
8. Update docs for manifest requirements, standalone updater behavior, no silent update default, and GitLab/private asset limitations.

Non-goals: silent background auto-update by default; private GitLab token management unless explicitly added later; changing semver/tag semantics for name pools.

## Research Required

- External release hosting is a hard integration point. C7 scope is manifest-driven and testable against local/mock HTTP or file URLs.
- GitLab private-token auth is deferred from C7 unless explicitly added by the user. C7 assumes release assets or manifest URLs are reachable in the target environment.
- GitHub external release naming remains supported through channel metadata, not semver/tag changes.

## Risks And Alternatives

- Risk: auto-update security. Mitigation: prompt by default, checksum verification required, no silent update default.
- Risk: private GitLab assets are inaccessible. Mitigation: document accessible Release/Package Registry requirement and fail clearly.
- Risk: update breaks VSCode integration. Mitigation: standalone `current` switch preserves `~/.local/bin/vsp-codex` and VSCode `cliExecutable`.
- Rejected alternative: re-enable official updater. It targets official OpenAI release channels, not VSP.

## Validation Path

Required validation:

- Unit tests for manifest parse, target selection, channel selection, version comparison, and checksum mismatch.
- CLI integration test using a local mock archive/manifest.
- TUI snapshot/test for VSP update prompt, skip this time, and skip until next version.
- Install-context tests proving standalone release layout maps to VSP update action instead of official OpenAI commands.
- `cargo fmt --check`.
- `git diff --check`.

Manual acceptance:

- Install an older standalone VSP build.
- Point it at a newer mock or published manifest.
- Start `vsp-codex`, see the update prompt, run update, restart, verify `vsp-codex --version` and `vsp-codex vscode status --json`.

Pass signal: VSP release manifests select the correct channel/target, update prompts use structured confirmation cards, checksum-verified standalone updates switch `current` safely, and channel default names do not override user nicknames.

Fail signal: VSP mode calls official OpenAI update endpoints, checksum failure changes `current`, update runs silently by default, or private GitLab assumptions are hidden instead of documented.

## Audit Focus

- Verify updater never calls official OpenAI release endpoints in VSP mode.
- Verify update prompt is a clear confirmation card, not a silent update or plain chat question.
- Verify checksum failure aborts without switching `current`.
- Verify VSCode `cliExecutable` remains valid after update.
- Verify release-channel name defaults do not override explicit user nicknames.
