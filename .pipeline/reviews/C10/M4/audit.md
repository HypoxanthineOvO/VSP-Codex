# C10/M4 Audit - Client Connectivity Adaptation And Drift Regression

Verdict: `needs_changes`

## Findings

1. P2 - Official shim overwrite scenario is not actually replayed by the GREEN test.
   - `codex-rs/cli/src/main.rs:3469` defines `release_update_writes_executable_codex_compat_wrapper`, but setup only creates `visible_bin/vsp-codex` at `codex-rs/cli/src/main.rs:3485-3487`, then calls `ensure_codex_compat_entrypoint` at `codex-rs/cli/src/main.rs:3492`.
   - The test never pre-creates `visible_bin/codex` as an official shim/symlink/file and then verifies it is replaced. This misses the prompt's required "official installer re-taking Volta/`~/.local/bin` shims" drift replay, called out in `.pipeline/reviews/C10/M4/test.md:73-75`.
   - Production code likely handles replacement via `std::fs::rename(&tmp, &link)` at `codex-rs/cli/src/main.rs:1139-1150`, but M4 asked for real-path regression evidence, not just likely Unix rename semantics.
   - Recommendation: add a release-only Unix regression that writes an existing fake official `codex` entry in the visible bin dir before repair/update, then asserts it is replaced by the executable VSP wrapper and that execution pins `CODEX_HOME`.

## Validation Reviewed

- Release compat wrapper production path: `codex-rs/cli/src/main.rs:1123`, `codex-rs/cli/src/main.rs:1129-1154`.
- VSCode guarded repair wrapper: `codex-rs/cli/src/vscode_cmd.rs:449-460`; execution coverage in `codex-rs/cli/tests/vscode.rs:618-681`.
- Socket namespace remap production path: `codex-rs/cli/src/main.rs:777-807`, applied at `codex-rs/cli/src/main.rs:1688` and `codex-rs/cli/src/main.rs:1752-1759`; command-path tests at `codex-rs/cli/src/main.rs:3397-3464`.
- `currentTime/read` timeout increased to 60s: `codex-rs/app-server/src/current_time.rs:25`, behavior test at `codex-rs/app-server/tests/suite/v2/current_time.rs:109-183`.
- Provider filtering and capability matrix evidence exists: provider normalization at `thread_processor.rs:204-244` and `thread_processor.rs:3423-3440`; provider capability tests at `model_provider_capabilities_read.rs:17-69`.
- Worker separation is acceptable: test additions in `codex-rs/cli/src/main.rs` are under `#[cfg(test)] mod tests` starting at `codex-rs/cli/src/main.rs:3061`; test report explicitly states this at `.pipeline/reviews/C10/M4/test.md:141`.
- Report quality is mostly acceptable: M4 report states UI acceptance is not claimed at `.pipeline/reports/C10/M4-connectivity.md:43` and `.pipeline/reports/C10/M4-connectivity.md:67`; timer retirement remains gated at `.pipeline/reports/C10/M4-connectivity.md:47-49` and `.pipeline/reports/C10/M4-connectivity.md:73`.

## Residual Risks

- Real VSCode Remote Plugin and Desktop/Remote APP connectivity still require user confirmation.
- `vsp-codex-repair.timer` must remain until explicit user-confirmed retirement.
- M5 release/push/tag remains a separate explicit gate.
- App-server lib-test compile drift noted in `.pipeline/reviews/C10/M4/test.md:172-173` remains outside M4 but should not be forgotten.

## Recommendation

Do not accept M4 as fully green until the official `codex` shim overwrite regression is added or the report downgrades that scenario from proven to pending. All other reviewed drift vectors have credible production and test evidence.

---

# C10/M4 Re-Audit - Official Shim Replay Closure

Verdict: `pass`

## Findings

None.

## Validation Reviewed

- P2 finding is closed. `release_update_writes_executable_codex_compat_wrapper` now pre-creates fake official `visible_bin/codex`, calls `ensure_codex_compat_entrypoint`, then verifies the result is a non-symlink executable VSP wrapper and executes both default `$HOME/.vsp-codex` plus `VSP_CODEX_HOME` override paths.
- Production release behavior remains the intended wrapper path: `ensure_codex_compat_entrypoint` writes `CODEX_HOME=${VSP_CODEX_HOME:-$HOME/.vsp-codex}` and renames it over `codex`.
- Test-worker changes to `codex-rs/cli/src/main.rs` are inside `#[cfg(test)] mod tests`; the audit-fix evidence accurately states no production/runtime code changed in `b62a07b59e`.
- M4 automation scope is passable from reviewed evidence: VSCode wrapper repair, release compat wrapper, socket remap, `currentTime/read` timeout, provider filtering, capability matrix, report/timer recommendation, and manual UI gap are all represented in code/tests/reports.
- Re-audit did not rerun cargo tests because it was strict read-only. Validation is source/evidence review only.

## Residual Risks

- Real VSCode Remote Plugin and Desktop/Remote APP connectivity still require user confirmation in a real UI environment.
- `vsp-codex-repair.timer` should not be retired automatically; the report correctly keeps retirement behind GREEN evidence plus user confirmation.
- M5 release/push/tag remains a separate explicit gate; no release side effect is authorized by this M4 pass.

## Recommendation

Accept M4 automated scope as `pass` and proceed to the remaining manual UI confirmation / M5 gated release workflow.
