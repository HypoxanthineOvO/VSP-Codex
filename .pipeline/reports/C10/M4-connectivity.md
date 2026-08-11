# C10 M4 Connectivity Report Draft

## Scope

Implement worker scope covered production/runtime connectivity fixes and this report draft only. Test files, fixtures, snapshots, and assertion assets were left untouched for the M4 test worker.

## Change Summary

- Release compat `codex` entrypoint is now written as an executable shell wrapper instead of a symlink. The wrapper pins `CODEX_HOME=${VSP_CODEX_HOME:-$HOME/.vsp-codex}` before execing the release `vsp-codex` binary.
- VSCode wrapper generation now uses the same runtime `VSP_CODEX_HOME` fallback instead of baking in the current user's home path. The guarded quiet startup repair remains enabled by default, disabled by `VSP_CODEX_VSCODE_AUTO_REPAIR=0`, and recursion is guarded by exporting `VSP_CODEX_VSCODE_AUTO_REPAIR=0` during repair.
- `currentTime/read` client round-trip budget increased from 10s to 60s. Existing behavior is preserved: timeout cancels the outbound request, client cancellation remains a distinct error, and JSON-RPC errors still surface their code/message.
- Socket namespace remap was verified in production command flow: `app-server --listen unix://...` remaps Unix socket transports before server startup, and `app-server proxy --sock ...` remaps explicit socket paths before connecting.

## Extension Capability Matrix

| Capability expected by current VSCode extension | VSP production status | Notes / follow-up |
| --- | --- | --- |
| Launch `codex` compat binary from official/client-owned shims | Fixed in release update path | `codex` is an executable wrapper, not a symlink, so official callers inherit VSP `CODEX_HOME` before launching `vsp-codex`. Test worker should cover installer/update output. |
| Launch configured VSCode CLI executable | Present and adjusted | Wrapper lives under `~/.vsp-codex/vscode/vsp-codex`, points to the selected `vsp-codex`, and now resolves `CODEX_HOME` at runtime from `VSP_CODEX_HOME` or default VSP home. |
| Startup self-heal after extension bundled binary drift | Present | Wrapper runs `vsp-codex vscode repair --yes --quiet` by default and suppresses output. `VSP_CODEX_VSCODE_AUTO_REPAIR=0` disables it; nested repair is guarded. |
| App-server Unix socket namespace | Present | Official `~/.codex/app-server-control/*` socket paths are remapped to `~/.vsp-codex/app-server-control/*`; custom non-official paths are preserved. |
| `currentTime/read` under slow client/remote startup | Fixed | Timeout budget is now 60s. The implementation still cancels timed-out requests and keeps cancellation/error messages distinct. |
| Environment info RPC / environment selections | Present in app-server runtime | Production request routing includes environment processor paths. Test worker should verify the specific extension call surface. |
| Rollout-backed thread read/turn surfaces | Present in app-server runtime | Thread read/list paths build API turns from rollout items and support persisted rollout metadata. Test worker should validate the extension-visible cases. |
| Structured shutdown logs / graceful drain | Present in app-server runtime | Shutdown path emits structured connection/running-turn counts during graceful restart drain. Test worker/audit should decide whether log assertions are sufficient. |
| Blank/unknown provider output normalization | Present in app-server runtime | Production processors normalize blank provider overrides and replace unknown API thread providers with the configured fallback. |

## Hotfix Survival Mapping

| 2026-06-27 hotfix | Current productized path | Status |
| --- | --- | --- |
| `vsp-codex vscode repair --yes [--quiet]` | `codex-rs/cli/src/vscode_cmd.rs` repair command rewrites wrapper and bundled remote extension shims. | Survived. |
| Guarded quiet repair on wrapper startup | `write_vscode_wrapper` emits startup repair guarded by `VSP_CODEX_VSCODE_AUTO_REPAIR`. | Survived; runtime `CODEX_HOME` fallback adjusted in M4. |
| Official app-server socket remap | `remap_vsp_app_server_socket_path` maps official home control sockets to VSP home. | Survived; command flow verified for `--listen` and `proxy --sock`. |
| Compat `codex` entrypoint | `ensure_codex_compat_entrypoint` in release update path. | Fixed in M4: wrapper replaces symlink. |
| Provider filtering / fallback | App-server thread processors normalize blank/unknown providers. | Survived; no M4 code change needed. |

## Real-Path Drift Self-Heal Status

- Extension auto-update drift: repaired by wrapper startup self-heal plus `vscode repair`, which rewrites fresh official bundled `bin/linux-x86_64/codex` files to exec the VSP wrapper.
- Official shim overwrite drift: release/update compat entrypoint now writes an executable `codex` wrapper in the visible VSP bin directory, preserving VSP `CODEX_HOME` even when official callers invoke `codex`.
- Socket drift: explicit official control socket paths are remapped at CLI runtime; default `unix://` resolves through the active `CODEX_HOME`, which the VSP wrappers pin to VSP home.
- User-visible UI acceptance is not claimed here. Real VSCode/Desktop/Remote UI connection acceptance still needs user-side confirmation after GREEN tests.

## Timer Recommendation

Recommendation: keep `vsp-codex-repair.timer` for this cycle as defense-in-depth, then retire only after the test worker proves the real-path drift scenarios GREEN and the user confirms VSCode/Desktop/Remote UI connectivity.

Rationale: M4 product code now covers the primary drift vectors without relying on the timer, but the timer is still useful until independent regression evidence confirms wrapper startup repair and official shim overwrite recovery on real paths. Retirement is a separate user-confirmed action and should not be bundled into this implement step.

## Validation Performed By Implement Worker

- `cargo fmt --manifest-path codex-rs/Cargo.toml -p codex-cli -p codex-app-server`
- `cargo check --manifest-path codex-rs/Cargo.toml -p codex-cli -p codex-app-server`
- `cargo check --manifest-path codex-rs/Cargo.toml --release -p codex-cli`

Known warnings observed were pre-existing unused/dead-code or unreachable-pattern warnings outside the M4 changes.

## Remaining Test Worker Verification

- `cargo test --manifest-path codex-rs/Cargo.toml -p codex-cli --test vscode -- --nocapture`
- `cargo test --manifest-path codex-rs/Cargo.toml -p codex-app-server --test all request_user_input -- --nocapture`
- Real-path drift simulation: fresh official VSCode bundled `codex` overwritten, wrapper startup invoked, bundled shim repaired.
- Official compat overwrite simulation: release/update path writes executable `codex` wrapper and preserves `CODEX_HOME=${VSP_CODEX_HOME:-$HOME/.vsp-codex}`.
- Socket namespace scenarios: `app-server --listen unix://$HOME/.codex/app-server-control/<name>.sock`, `app-server proxy --sock $HOME/.codex/app-server-control/<name>.sock`, and custom non-official sockets.
- Read-only local status: `vsp-codex vscode status --json`.
- User-confirmed acceptance: VSCode Remote Plugin and Desktop/Remote APP connect to VSP Codex, without claiming that this report already verified the UI.

## Risks / Follow-Up

- Release compat wrapper is compiled only in non-debug builds; release `cargo check` passed, but test worker should exercise the actual generated file contents.
- The 60s `currentTime/read` budget is conservative for remote startup latency. If clients routinely exceed it, the next change should split subscriber wait and response wait into separately named budgets.
- Timer retirement remains pending until GREEN regression evidence plus user UI acceptance.
