# M3 Implement Record: VSP Entrypoint And Home

## Scope

- Added the `vsp-codex` Rust binary target and aligned top-level CLI help/version output.
- Changed the default configuration home from `~/.codex` to `~/.vsp-codex`.
- Added a TUI startup prompt that offers to import an existing sibling `~/.codex` home when `~/.vsp-codex/config.toml` is missing and `CODEX_HOME` is not set.
- Updated package wrapper, helper scripts, test binary lookup, and Bazel extra binary references from the old CLI target to `vsp-codex`.
- Fixed post-audit gaps in standalone installers, shell completion generation, container helper invocation, and visible command examples.

## Verification

- `just fmt`
- `just test -p codex-utils-home-dir`
- `just test -p codex-cli`
- `just test -p codex-tui vsp_home_import`
- `just test -p codex-tui session_summary`
- `just test -p codex-tui thread_goal_ephemeral_error_message_renders_snapshot`
- `just test -p codex-tui live_app_server_thread_name_update_shows_resume_hint`
- `just test -p codex-utils-cli`
- `just test -p codex-core cli_stream` (first run timed out during compile at 300s; rerun with 600s passed)
- `just test -p codex-state --bin logs_client` (a first attempt used the Clap command name `codex-state-logs` and failed because the Cargo bin is `logs_client`; rerun passed)
- `cargo build --bin vsp-codex`
- `./target/debug/vsp-codex --version`
- `./target/debug/vsp-codex --help`
- `./target/debug/vsp-codex login --help`
- `./target/debug/vsp-codex completion bash > /tmp/vsp-codex-completion.bash`
- `just vsp-codex --version`
- `cargo insta pending-snapshots --manifest-path tui/Cargo.toml`
- `git diff --check`
- `sh -n scripts/install/install.sh`
- `bash -n codex-cli/scripts/run_in_container.sh scripts/debug-codex.sh scripts/start-codex-exec.sh scripts/run_tui_with_exec_server.sh scripts/test-remote-env.sh`
- `python3 -m unittest discover -s scripts/codex_package -p 'test_*.py'`
- `python3` YAML parse of `.gitlab-ci.yml`
- `glab ci lint .gitlab-ci.yml`
- `just fix -p codex-cli`
- `just fix -p codex-tui`
- `just fix -p codex-utils-cli`
- `just fix -p codex-utils-home-dir`

## Known Local Verification Gaps

- Bazel target analysis could not be run locally because neither `bazel` nor `bazelisk` is installed in this environment. The known BUILD references to `//codex-rs/cli:codex` were updated to `//codex-rs/cli:vsp-codex`.
- PowerShell installer syntax could not be executed locally because neither `pwsh` nor Windows PowerShell is installed in this environment.

## Notes

- Earlier implementation workers timed out without producing usable code patches, so the main agent completed the implementation directly. The red-test worker output was retained and used as the implementation guide.
- `.pipeline/` and `.plan-state/` are workflow records and are intentionally not part of the code commit unless explicitly requested.
