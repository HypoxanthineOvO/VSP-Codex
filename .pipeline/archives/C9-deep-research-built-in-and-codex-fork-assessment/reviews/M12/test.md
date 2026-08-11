# C9 M12 Test Review

## Verdict

PASS with post-release risks.

M12 produced and published `v0.140.0-vsp.8.1`, then verified the core release surfaces and local distribution.

## Reviewed Refs

- `codex-rs/Cargo.toml`
- `codex-rs/Cargo.lock`
- `codex-cli/package.json`
- `README.md`
- `docs/install.md`
- `docs/vsp-codex.md`
- `CHANGELOG.md`
- `scripts/install/install.sh`
- `codex-rs/cli/src/main.rs`
- `codex-rs/cli/src/vscode_cmd.rs`
- `codex-rs/cli/tests/vscode.rs`
- `.pipeline/release/v0.140.0-vsp.8.1/`
- `.pipeline/release/vsp-release-manifest.json`
- `.pipeline/evidence/C9/M12-sqlite-io-smoke.json`

## Validation Results

Passed:

- `cargo metadata --manifest-path codex-rs/Cargo.toml --no-deps --format-version=1`
  - `codex-cli` resolved as `0.140.0-vsp.8.1`.
- `sh -n scripts/install/install.sh`
- `cargo check --manifest-path codex-rs/Cargo.toml -p codex-cli --bin vsp-codex`
- `cargo build --manifest-path codex-rs/Cargo.toml --release --bin vsp-codex`
  - `codex-rs/target/release/vsp-codex --version` returned `vsp-codex 0.140.0-vsp.8.1`.
- `python3 scripts/build_vsp_preview_release.py --entrypoint-bin codex-rs/target/release/vsp-codex --bwrap-bin /usr/bin/bwrap --force`
  - Generated `.pipeline/release/v0.140.0-vsp.8.1/vsp-codex-preview-linux-x64-gnu.tar.gz`.
  - Generated SHA256 `0ff6e0d34daf6265e23cb99d21b69d9acbbcce4df2067c168f72f0e7a82e567e`.
- `cd .pipeline/release/v0.140.0-vsp.8.1 && sha256sum -c vsp-codex-preview-linux-x64-gnu.tar.gz.sha256`
- `python3 -m json.tool .pipeline/release/v0.140.0-vsp.8.1/vsp-release-manifest.json`
- `python3 -m json.tool .pipeline/release/vsp-release-manifest.json`
- Package metadata extraction:
  - `codex-package.json.version == 0.140.0-vsp.8.1`
  - `entrypoint == bin/vsp-codex`
  - `vspRelease.manifestUrl` points to the VSP stable manifest raw URL.
- Isolated installer smoke using `file://` archive:
  - Installed `0.140.0-vsp.8.1-preview`.
  - Both temporary `~/.local/bin/vsp-codex` and `~/.local/bin/codex` returned `vsp-codex 0.140.0-vsp.8.1`.
  - Both links resolved to the same managed release binary.
- `cargo test --manifest-path codex-rs/Cargo.toml -p codex-cli --test vscode -- --test-threads=1`
  - 17 passed, 0 failed.
- `cargo test --manifest-path codex-rs/Cargo.toml -p codex-state filter -- --nocapture`
  - 8 passed, 0 failed.
- Runtime SQLite smoke with isolated `CODEX_HOME` and `CODEX_SQLITE_HOME`:
  - `logs_2.sqlite` existed and was 49152 bytes.
  - WAL size was 0 bytes.
  - `logs` table total rows: 0.
  - noisy target rows for `log`, `codex_otel.log_only`, and `codex_otel.trace_safe`: 0.
- `codex-rs/target/release/vsp-codex vscode status --json`
  - `configured: true`
  - no warnings.
- `cargo fmt --manifest-path codex-rs/Cargo.toml --all -- --check`
- `python3 scripts/check_ci_docs.py`
- `python3 -m json.tool codex-cli/package.json`
- YAML parse for `.pipeline/state.yaml` and `.pipeline/log.yaml`
- `git diff --check`
- GitLab refs for `main`, `c9-upstream-0.143.0-alpha.25-vsp8`, and `v0.140.0-vsp.8.1` dereference to release commit `26ca6333792c0b6ea0f2504658617447389ba27f`.
- GitLab Release assets exist for the archive and `.sha256`.
- Remote `.sha256` downloads to `0ff6e0d34daf6265e23cb99d21b69d9acbbcce4df2067c168f72f0e7a82e567e  vsp-codex-preview-linux-x64-gnu.tar.gz`.
- Raw `main` stable manifest first entry is `0.140.0-vsp.8.1`.
- Public-download local distribution installed `0.140.0-vsp.8.1-preview`.
- `~/.vsp-codex/packages/standalone/current/bin/vsp-codex --version`, `PATH=/home/heyx/.local/bin:$PATH vsp-codex --version`, and `PATH=/home/heyx/.local/bin:$PATH codex --version` all returned `vsp-codex 0.140.0-vsp.8.1`.
- `PATH=/home/heyx/.local/bin:$PATH vsp-codex vscode status --json` returned `configured: true`.

Observed non-product issues:

- First package-builder run without `--bwrap-bin /usr/bin/bwrap` failed because this host lacks `libcap.pc`; rerunning with the system `bwrap` succeeded. This matches prior release-builder evidence.
- First repository-root `sha256sum -c` failed because the generated `.sha256` file records the archive basename; running from the release directory passed.
- First installer smoke shell snippet used zsh's read-only `status` variable; rerun with `rc` passed.
- Runtime `app-server --listen off` exits with `no transport configured`; the smoke still initializes the SQLite log DB and provides the intended I/O evidence.
- VSCode warns that `chatgpt.cliExecutable` and `chatgpt.localeOverride` are application-scoped settings when written to remote Machine settings. The installer also repoints bundled server shims; the local VSCode UI User settings should keep the same values.

## Coverage

The validation covers version metadata, lockfile sync, release build, archive generation, manifest/sha correctness, install/update compatible entrypoints, VSCode bundled shim drift regression, real VSCode status, and SQLite persistent-log write behavior.

Broad full-workspace nextest remains outside M12 evidence. M11 already recorded broad TUI nextest historical failures and stack-overflow risk; this M12 evidence should not be represented as a full workspace clean regression.
