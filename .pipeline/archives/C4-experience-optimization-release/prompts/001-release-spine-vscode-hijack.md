# M1 Release Spine + VSCode Hijack

## Objective

让预览版二进制能自动接入官方 VSCode 插件，并提供可验证的安装、回滚和 smoke 证据。

## Source Of Truth

- `.plan-state/decompose.yaml` M1
- `.plan-state/p1-discover.yaml` VSCode/release decisions
- P1 已验证官方插件可通过 `chatgpt.cliExecutable` 启动 `vsp-codex app-server --analytics-default-enabled`

## Subworker Assignment Plan

Status: authorized, strict worker separation.

- `test`
  - Owns CLI/settings tests and smoke probe design before implementation.
  - Expected artifacts:
    - `.pipeline/reviews/C4/M1/test.md`
  - Must cover settings write, backup, rollback, locale non-overwrite, workspace fallback, dry-run/status, initialize probe contract.
- `implement`
  - Owns `codex-rs/cli` changes, VSCode settings integration command, and release quickstart/rollback artifacts.
  - Expected artifacts:
    - `.pipeline/reviews/C4/M1/implement.md`
  - Must not modify or vendor the official VSCode extension.
- `audit`
  - Checks settings safety, rollback correctness, release blocker evidence, and scope boundaries.
  - Expected artifacts:
    - `.pipeline/reviews/C4/M1/audit.md`

Main agent orchestrates only; it must not satisfy both test and implement roles locally.

## Technical Solution

Add a first-party `vsp-codex vscode` CLI command group in `codex-rs/cli`, implemented in a new module such as `vscode_cmd.rs`. The integration points official VSCode extension setting `chatgpt.cliExecutable` at the released/current `vsp-codex` binary. Settings edits must be conservative, reversible, and verified.

## Technical Route

1. Add `Vscode(VscodeCommand)` to `codex-rs/cli/src/main.rs`.
2. Implement `install`, `status`, and `rollback`/`uninstall` in a new CLI module.
3. Discover VSCode user settings paths for Linux and VSCode Server first; macOS arm64 support remains should-have if build/test path is not ready.
4. Write settings with backup under `~/.vsp-codex/vscode-settings-backups/`.
5. Preserve unknown settings and existing `chatgpt.cliExecutable`.
6. Set `chatgpt.localeOverride=zh-CN` only when absent/empty.
7. Probe `code --list-extensions` and optionally `code --install-extension openai.chatgpt`; failures must show manual instructions.
8. Add an app-server initialize probe using `app-server --analytics-default-enabled`.
9. Add quickstart and rollback instructions for preview release.

## Research Required

- Resolved: official extension path, `chatgpt.cliExecutable`, spawn args, and VSP app-server initialize were verified during P1.
- Pending implementation check: final VSCode user settings path in remote/server environments. If uncertain, use workspace fallback and explicit diagnostics.

## Risks And Alternatives

- Risk: VSCode restricted setting may not behave as a normal setting. Mitigate with status/probe and manual fallback.
- Risk: JSONC comments may be lost. Prefer a conservative JSONC-preserving strategy or explicitly document the limitation.
- Rejected: patching or redistributing official extension UI.

## Validation Path

Commands/scenarios:

- `just test -p codex-cli vscode`
- `just test -p codex-app-server`
- `vsp-codex app-server --analytics-default-enabled` initialize probe
- settings write/rollback smoke in a temp HOME/settings directory

Pass signal: settings write/backup/rollback pass; initialize probe returns VSP codexHome/userAgent; release evidence records install/status/rollback.

Fail signal: settings cannot be restored, user locale is overwritten, app-server cannot initialize with official plugin args, or extension patching is required.

## Audit Focus

- No official extension code modification.
- Rollback is reliable and tested.
- Linux x64 release path is blocker; macOS arm64 is should-have unless build chain is ready.
