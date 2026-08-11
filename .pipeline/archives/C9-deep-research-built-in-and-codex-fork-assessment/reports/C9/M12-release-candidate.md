# C9 M12 Release And Local Distribution Report

## Conclusion

M12 published `v0.140.0-vsp.8.1` and completed local distribution.

The release commit, annotated tag, GitLab Release, release assets, stable manifest, and local standalone installation are now in place.

## Change Summary

- Version metadata now targets `0.140.0-vsp.8.1`:
  - `codex-rs/Cargo.toml`
  - `codex-rs/Cargo.lock`
  - `codex-cli/package.json`
- User-facing installation and release docs now point at published tag `v0.140.0-vsp.8.1`:
  - `README.md`
  - `docs/install.md`
  - `docs/vsp-codex.md`
  - `CHANGELOG.md`
  - `.pipeline/release/v0.140.0-vsp.8.1/release-notes.md`
- Local release artifacts were generated:
  - `.pipeline/release/v0.140.0-vsp.8.1/vsp-codex-preview-linux-x64-gnu.tar.gz`
  - `.pipeline/release/v0.140.0-vsp.8.1/vsp-codex-preview-linux-x64-gnu.tar.gz.sha256`
  - `.pipeline/release/v0.140.0-vsp.8.1/vsp-release-manifest.json`
- Stable manifest candidate `.pipeline/release/vsp-release-manifest.json` now lists `0.140.0-vsp.8.1` first and preserves historical `0.140.0-vsp.8`, `0.134.0-vsp.7.2`, `0.134.0-vsp.7`, and `0.134.0-vsp.6`.
- Persistent startup-path fixes from the plugin/Remote APP debug are included in the release surface:
  - installer keeps both `vsp-codex` and `codex` visible entrypoints pointed at managed standalone current;
  - updater recreates a sibling `codex` compatibility entrypoint when the managed `vsp-codex` path is updated;
  - VSCode status diagnoses remote bundled `codex` shim drift.
- GitLab release notes were updated from local-RC wording to published-release wording.
- Local distribution installed `0.140.0-vsp.8.1-preview` into `~/.vsp-codex/packages/standalone/current`, updated `~/.local/bin/vsp-codex` and compatibility `~/.local/bin/codex`, and repointed current VSCode server bundled `codex` shims to the VSP wrapper.

## Release Artifact

- Version: `0.140.0-vsp.8.1`
- Tag: `v0.140.0-vsp.8.1`
- Release commit: `26ca6333792c0b6ea0f2504658617447389ba27f`
- Tag object: `153e1b4bd7be7bfe921f4d0511cf2547a58a8301`
- Target: `x86_64-unknown-linux-gnu`
- Archive: `.pipeline/release/v0.140.0-vsp.8.1/vsp-codex-preview-linux-x64-gnu.tar.gz`
- Archive size: about 286 MiB
- SHA256: `0ff6e0d34daf6265e23cb99d21b69d9acbbcce4df2067c168f72f0e7a82e567e`
- GitLab Release: `https://gitlab.vsplab.cn/heyx/Codex/-/releases/v0.140.0-vsp.8.1`
- Downloads URL: `https://gitlab.vsplab.cn/heyx/Codex/-/releases/v0.140.0-vsp.8.1/downloads/vsp-codex-preview-linux-x64-gnu.tar.gz`

Package metadata contains:

- `version: 0.140.0-vsp.8.1`
- `target: x86_64-unknown-linux-gnu`
- `variant: codex`
- `entrypoint: bin/vsp-codex`
- `vspRelease.manifestUrl: https://gitlab.vsplab.cn/heyx/Codex/-/raw/main/.pipeline/release/vsp-release-manifest.json`

## Upstream And Scope Statement

This RC records convergence work against upstream `rust-v0.143.0-alpha.25`, but it is not a full alpha.25 fork reset and must not be described as a full upstream merge.

The release notes emphasize:

- SQLite logging convergence and the retained VSP in-layer `target=log` guard.
- Plugin/Remote APP session visibility startup-path fix.
- VSCode bundled `codex` shim and drift diagnostic.
- Installer/updater maintenance of both `vsp-codex` and compatibility `codex` entrypoints.

## Validation Summary

Passed:

- metadata and lockfile version sync;
- release build and `vsp-codex --version`;
- package builder with `/usr/bin/bwrap`;
- sha256 check;
- versioned and stable manifest JSON parsing;
- package metadata extraction;
- isolated installer smoke for both `vsp-codex` and `codex` entrypoints;
- VSCode regression test suite, 17 passed;
- real `vscode status --json`, configured true with no warnings;
- SQLite log filter tests, 8 passed;
- runtime SQLite I/O smoke, 0 persisted rows and 0 noisy target rows;
- `cargo fmt --check`, `check_ci_docs.py`, `git diff --check`, package JSON parse, and Workflow YAML parse.
- GitLab refs: `main`, `c9-upstream-0.143.0-alpha.25-vsp8`, and tag dereference point at release commit `26ca6333792c0b6ea0f2504658617447389ba27f`.
- GitLab Release assets exist for `vsp-codex-preview-linux-x64-gnu.tar.gz` and `.sha256`.
- Remote `.sha256` downloads to `0ff6e0d34daf6265e23cb99d21b69d9acbbcce4df2067c168f72f0e7a82e567e  vsp-codex-preview-linux-x64-gnu.tar.gz`.
- Raw `main` stable manifest first entry is `0.140.0-vsp.8.1`.
- Local install smoke after public-download installation:
  - `~/.vsp-codex/packages/standalone/current/bin/vsp-codex --version` returned `vsp-codex 0.140.0-vsp.8.1`.
  - `PATH=/home/heyx/.local/bin:$PATH vsp-codex --version` returned `vsp-codex 0.140.0-vsp.8.1`.
  - `PATH=/home/heyx/.local/bin:$PATH codex --version` returned `vsp-codex 0.140.0-vsp.8.1`.
  - `vsp-codex vscode status --json` returned `configured: true`.
- GitLab `main` pipeline `#216` completed successfully after the release push.

## Problems Encountered

- Host lacks `libcap.pc`, so package builder cannot compile `codex-bwrap` from source. Using `/usr/bin/bwrap` produced the release archive successfully.
- The generated `.sha256` uses the archive basename, so `sha256sum -c` must be run from the release directory or with an adjusted path.
- The VSCode Remote Machine settings file records `chatgpt.cliExecutable`, but VSCode treats that key as application-scoped. The installer also repointed bundled server shims; for local UI-side settings, keep the same `chatgpt.cliExecutable` value in the local VSCode User settings.
- Existing VSCode/Remote APP processes were not killed. Users may still need to reload VSCode or restart Remote APP sessions to pick up the new runtime path.

## Release Result

M12 is complete. `v0.140.0-vsp.8.1` is published and installed locally.

Residual risks are limited to process reload and broad-regression scope: already-running VSCode/Remote APP processes may still hold old binaries until restarted, and this release should not be described as a full-workspace clean nextest or full upstream alpha.25 merge.
