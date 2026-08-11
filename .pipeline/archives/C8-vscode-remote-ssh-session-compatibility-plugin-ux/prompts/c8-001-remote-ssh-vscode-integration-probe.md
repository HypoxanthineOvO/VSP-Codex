# M1 Remote SSH VSCode Integration Probe

## Objective

Make `vsp-codex vscode install/status` treat VSCode Remote SSH as the primary integration environment. The command should prefer remote VSCode server Machine settings, produce useful diagnostics for the remote binary/session home, and preserve local VSCode fallback behavior.

## Source Of Truth

- User environment: VSCode plugin is used inside Remote SSH windows. VSP-Codex and sessions live on the remote host.
- P2 technical solution: prefer `.vscode-server/data/Machine/settings.json` for VSP CLI integration when remote server settings are present or when running in a remote server environment.
- Known code: `codex-rs/cli/src/vscode_cmd.rs` currently resolves local user settings before server settings; tests live in `codex-rs/cli/tests/vscode.rs`.

## Subworker Assignment Plan

Status: authorized, strict worker separation.

- `test`
  - Scope: owns failing tests, fixtures, assertions, and command-output expectations for Remote SSH settings priority and status diagnostics.
  - Write boundary: tests/fixtures and `.pipeline/reviews/C8/M1/test.md` only.
  - Expected artifact: `.pipeline/reviews/C8/M1/test.md`.
- `implement`
  - Scope: owns production CLI settings resolution, status/install diagnostics, and minimal docs/help text if needed.
  - Must not create, edit, rewrite, or satisfy tests owned by `test`.
  - Expected artifact: `.pipeline/reviews/C8/M1/implement.md`.
- `audit`
  - Scope: read-only review of diff, settings priority, local fallback, Remote SSH assumptions, and evidence quality.
  - Must reject tests that only assert helper strings without exercising scope selection.
  - Expected artifact: `.pipeline/reviews/C8/M1/audit.md`.

Workers are not alone in the codebase. Do not revert user edits or another worker's edits; adapt around them and stop on scope conflicts. Lifecycle evidence must record requested, started, completed/failed/blocked, and closed/close_failed for all roles.

Required user decisions must use `request_user_input` when available. Destructive operations, system-level dependency installation, network/remote side effects, remote publication, tag creation, and release upload require explicit confirmation/authorization.

## Technical Solution

Change VSCode integration scope resolution so the Remote SSH server Machine settings file has first priority for VSP-Codex. When both local user settings and remote server Machine settings are visible, `vsp-codex vscode install/status` should choose the remote server Machine settings unless the user explicitly requests another scope. Status output should show the chosen settings path, the CLI executable path, and enough CODEX_HOME/session-home context to diagnose plugin resume behavior.

## Technical Route

1. Inspect `codex-rs/cli/src/vscode_cmd.rs`, existing settings path helpers, and tests in `codex-rs/cli/tests/vscode.rs`.
2. Add red tests where both local user settings and `.vscode-server/data/Machine/settings.json` exist; expected chosen scope is remote server Machine settings.
3. Add status/diagnostic tests for the selected settings path and VSP executable path.
4. Implement Remote SSH priority in the smallest shared resolver rather than duplicating path logic.
5. Preserve local VSCode fallback when remote server settings are absent.
6. Update help/docs only if existing user-facing text would mislead Remote SSH users.

Non-goals: modifying the VSCode plugin, publishing a VSIX, changing session schema, or touching official Codex discovery.

## Research Required

- Local code evidence has already identified the resolver and test file.
- Implementation-time research may be needed for exact VSCode remote env variables. Prefer filesystem-based priority if reliable; use env detection only as a secondary signal.

## Risks And Alternatives

- Risk: local desktop users with a stale `.vscode-server` directory could receive remote priority unexpectedly. Mitigation: pair server-settings existence with command context or explicit VSP integration status, and keep visible diagnostics.
- Risk: forcing remote settings could hide a valid local plugin setup. Mitigation: preserve explicit local scope fallback if such a CLI option already exists, or document the fallback behavior.
- Rejected alternative: leave local-first resolution and rely on manual edits. The user's primary path is Remote SSH.

## Validation Path

Required validation:

- `cargo test -p codex-cli vscode -- --nocapture`
- `cargo fmt --check --package codex-cli`
- `git diff --check`

Manual acceptance:

- In a Remote SSH workspace, run `vsp-codex vscode status` and confirm it reports the remote `.vscode-server/data/Machine/settings.json` path and the remote VSP executable.
- Run install/status without changing local desktop user settings.

Pass signal: Remote SSH settings are preferred and local fallback still works.

Fail signal: status/install still chooses local user settings when remote server Machine settings exist.

## Audit Focus

- Verify the resolver has a single authoritative priority order.
- Verify status output is diagnostic enough for Remote SSH resume issues.
- Verify no external update, release, or remote publication side effect was introduced.

