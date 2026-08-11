# C10 M4 - Client Connectivity Adaptation And Drift Regression

## Objective

Make VSP-Codex reliably reachable by VSCode Plugin, Codex Desktop, and Remote APP after the upstream merge: productize the 2026-06-27 hotfixes, align app-server capabilities with what the newest extension expects, prove drift self-healing with real-path regression scenarios, and decide the fate of the local systemd repair timer.

## Requirements

- Verify the 2026-06-27 hotfix set survived the merges and is versioned product behavior with regression tests:
  - `vsp-codex vscode repair --yes [--quiet]` + wrapper guarded quiet repair on startup
  - app-server socket remap `~/.codex/app-server-control/*` -> `~/.vsp-codex/app-server-control/*` for `--listen unix://` and `proxy --sock`
  - compat `codex` written as wrapper pinning `CODEX_HOME=${VSP_CODEX_HOME:-$HOME/.vsp-codex}` (installer, not bare symlink)
  - app-server API output-layer filtering of blank/unknown providers
- Align protocol capabilities the newest VSCode extension (openai.chatgpt-26.6xx+, bundled codex-cli 0.142.3+) relies on: environment info RPC, increased currentTime/read timeouts, rollout turn items, structured shutdown logs. Verify VSP app-server serves them; fix gaps.
- Drift regression scenarios against REAL paths: simulate extension auto-update installing a fresh official bundled `codex`, and official installer re-taking Volta/`~/.local/bin` shims; verify the repair chain self-heals.
- Evaluate whether `vsp-codex-repair.timer` (5-min systemd user timer) can be retired once productized self-heal is proven; write an evidence-backed keep/retire recommendation (retirement itself is a user-confirmed action).
- Real client acceptance on this machine: VSCode Remote Plugin and Desktop/Remote APP connect to VSP-Codex; sockets exist only under `~/.vsp-codex/app-server-control`; `vsp-codex vscode status --json` reports configured.

## Boundaries

- In scope: cli/app-server/installer connectivity code, tests, local machine verification.
- Out of scope: version/release edits (M5), deleting the systemd timer without user confirmation, modifying the official `~/.codex` namespace beyond the documented repair semantics.

## Technical Solution

Treat connectivity as a contract: launch path (which binary), home (CODEX_HOME), and socket namespace must all resolve to VSP. Every drift vector identified on 2026-06-27 gets a versioned self-heal in product code plus a real-path regression test, so the machine-local systemd timer becomes defense-in-depth rather than the primary mechanism.

## Technical Route

1. Hotfix survival audit: map each 2026-06-27 fix to current post-merge code paths (`codex-rs/cli/src/vscode_cmd.rs`, app-server socket handling, installer wrapper generation, API provider filtering).
2. Extension capability matrix: enumerate RPCs/behaviors the new extension uses; test VSP app-server against each; fix gaps.
3. Drift regression tests: real shim/wrapper/socket fixtures (temp HOME layouts), no mock-only tests.
4. Local acceptance: rebuild debug/release binary, verify real VSCode Remote Plugin + Desktop/Remote APP connections (user confirms UI side).
5. Timer keep/retire recommendation in `.pipeline/reports/C10/M4-connectivity.md`.

## Research Required

The newest extension's actual call surface must be confirmed by local observation (extension dir inspection + app-server request logs), not assumed. This is testable on this machine; if the extension expects a capability that requires official-service semantics VSP disables, report the conflict for user decision.

## Risks And Alternatives

- Pseudo-test risk: mock-only tests would mask real drift; audit MUST reject tests that never touch real shim/wrapper/socket paths.
- Premature timer retirement loses the safety net; the recommendation must be evidence-backed and execution deferred to user confirmation.
- Alternative rejected: pinning the extension version — user chose adapting VSP to the latest extension expectations.

## Validation Commands

```bash
cargo test --manifest-path codex-rs/Cargo.toml -p codex-cli --test vscode -- --nocapture
cargo test --manifest-path codex-rs/Cargo.toml -p codex-app-server --test all request_user_input -- --nocapture
vsp-codex vscode status --json
ls ~/.vsp-codex/app-server-control/ && ! ls ~/.codex/app-server-control/*.sock 2>/dev/null
```

Scenario validation: scripted drift simulation (fresh official bundled codex in a temp extension dir; official shim overwrite) followed by repair-chain run and post-conditions assert; real client connection check with user confirmation for UI surfaces.

## Evidence

Pass signals: every 2026-06-27 fix mapped to live code + passing regression; extension capability matrix fully green or gaps fixed; drift scenarios self-heal on real paths; local clients connect; timer recommendation written with evidence.

Fail signals: any drift vector lacking versioned self-heal; mock-only scenario tests; client connection unverified on this machine.

## Subworker Assignment Plan

- `test`: owns drift scenario fixtures/tests and capability matrix verification; writes `.pipeline/reviews/C10/M4/test.md`. Enforces fake-test rejection: scenarios must touch real shims/wrappers/sockets.
- `implement`: owns connectivity product code fixes and installer/CLI changes. No test asset edits.
- `audit`: read-only; checks fix-to-drift-vector coverage, test realism, timer recommendation evidence; writes `.pipeline/reviews/C10/M4/audit.md`.
- Main agent: orchestration, lifecycle, and coordinating user-facing UI acceptance confirmation.
- Non-overlap: distinct identities for the three roles.

## Expected Artifacts

- Connectivity fixes + regression tests in product source
- `.pipeline/reports/C10/M4-connectivity.md` (incl. extension capability matrix + timer keep/retire recommendation)
- `.pipeline/reviews/C10/M4/test.md`
- `.pipeline/reviews/C10/M4/audit.md`
