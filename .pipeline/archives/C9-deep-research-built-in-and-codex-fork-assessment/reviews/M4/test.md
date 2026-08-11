# C9 M4 Test Report

Generated: 2026-06-17T17:34:11+08:00

## Scope

Milestone: C9 M4 `Version Metadata And Release Surface Update`.

Validated version metadata, release package generation, manifest structure, checksum, package metadata, and documentation references for `0.140.0-vsp.0`.

No push, tag, or GitLab release was performed.

## Results

Verdict: pass_with_warnings

Passed:

- `cargo metadata --manifest-path codex-rs/Cargo.toml --no-deps --format-version=1`
  - All workspace package versions resolve to `0.140.0-vsp.0`.
- `cargo build --manifest-path codex-rs/Cargo.toml --release --bin vsp-codex`
  - Built release binary successfully.
  - `codex-rs/target/release/vsp-codex --version` prints `vsp-codex 0.140.0-vsp.0`.
- `python3 scripts/build_vsp_preview_release.py --entrypoint-bin codex-rs/target/release/vsp-codex --bwrap-bin /usr/bin/bwrap --cargo-profile release --force`
  - Built `.pipeline/release/v0.140.0-vsp.0/vsp-codex-preview-linux-x64-gnu.tar.gz`.
  - Wrote `.pipeline/release/v0.140.0-vsp.0/vsp-codex-preview-linux-x64-gnu.tar.gz.sha256`.
  - Wrote `.pipeline/release/v0.140.0-vsp.0/vsp-release-manifest.json`.
- `sha256sum -c vsp-codex-preview-linux-x64-gnu.tar.gz.sha256`
  - Archive checksum passed.
- Package smoke:
  - Extracted archive.
  - Extracted `bin/vsp-codex --version` prints `vsp-codex 0.140.0-vsp.0`.
  - Extracted `codex-package.json` has version `0.140.0-vsp.0`, target `x86_64-unknown-linux-gnu`, and VSP release metadata:
    - `manifestUrl`: `https://gitlab.vsplab.cn/heyx/Codex/-/raw/main/.pipeline/release/vsp-release-manifest.json`
    - `channel`: `lab`
    - `target`: `x86_64-unknown-linux-gnu`
- Manifest JSON checks:
  - `.pipeline/release/vsp-release-manifest.json` latest entry is `0.140.0-vsp.0` and retains 4 releases.
  - `.pipeline/release/v0.140.0-vsp.0/vsp-release-manifest.json` latest entry is `0.140.0-vsp.0` and contains 1 release.
- `git diff --check`
  - Passed.
- Stale install-doc scan:
  - `rg -n "0\\.134\\.0-vsp\\.7\\.2|v0\\.134\\.0-vsp\\.7\\.2" README.md docs/install.md docs/vsp-codex.md`
  - No matches.

Warnings:

- First packaging attempt without `--bwrap-bin` failed because this host lacks `libcap.pc` for compiling bundled `bwrap`.
- Re-running packaging with existing `/usr/bin/bwrap` succeeded.
- Release build emitted existing warnings in `codex-core`, `codex-tui`, and `codex-cli`.
- Release assets were built locally only; no remote GitLab release asset exists until M5 release gates.
