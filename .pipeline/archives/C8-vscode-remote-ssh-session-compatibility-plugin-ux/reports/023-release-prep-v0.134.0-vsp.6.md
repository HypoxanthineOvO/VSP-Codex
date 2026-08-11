# v0.134.0-vsp.6 Release Prep

Status: prepared

## Scope

- Version: `0.134.0-vsp.6`
- Local package: Linux x64 GNU/glibc debug preview
- Archive: `.pipeline/release/C8-preview/vsp-codex-preview-linux-x64-gnu-debug.tar.gz`
- SHA-256: `666ee5841238914d200a496339d5d103b220fb933484defd3b4299fe959fa3bc`
- GitLab scope: commit, tag, push, and GitLab Release/assets after final validation.
- GitHub scope: draft notes only; no external GitHub publication in this prep step.

## Notes

- `codex-rs/Cargo.toml`, `codex-rs/Cargo.lock`, and `codex-cli/package.json` now use `0.134.0-vsp.6`.
- README, install docs, and VSP guide point to the C8 preview archive and SHA.
- The package was assembled from the freshly built debug `vsp-codex` binary plus packaged `rg`, `bwrap`, and `zsh` resources.
- The earlier `dev-small` build path did not produce a binary in time and was terminated; the release asset is explicitly the debug preview artifact.
- The target-specific `bwrap` build remains blocked by missing system `libcap.pc`; no system dependency was installed.

## Package Verification

- `codex-rs/target/debug/vsp-codex --version`: `vsp-codex 0.134.0-vsp.6`
- Packaged `bin/vsp-codex --version`: `vsp-codex 0.134.0-vsp.6`
- `sha256sum -c .pipeline/release/C8-preview/vsp-codex-preview-linux-x64-gnu-debug.tar.gz.sha256`: passed.
- Archive contains `bin/vsp-codex`, `codex-package.json`, `codex-path/rg`, `codex-resources/bwrap`, and `codex-resources/zsh/bin/zsh`.
- Extracted smoke path `/tmp/vsp-codex-c8-preview-smoke/bin/vsp-codex --version`: `vsp-codex 0.134.0-vsp.6`
- Extracted resources smoke:
  - `ripgrep 15.1.0`
  - `bubblewrap 0.9.0`
  - `zsh 5.9.0.3-test`

## Release Validation

- `cargo test --manifest-path codex-rs/Cargo.toml -p codex-app-server --test all request_user_input -- --nocapture`: 11 passed.
- `cargo test --manifest-path codex-rs/Cargo.toml -p codex-app-server --test all workflow_markdown_surface -- --nocapture`: 2 passed.
- `cargo test --manifest-path codex-rs/Cargo.toml -p codex-app-server --test all thread_resume -- --nocapture`: 37 passed.
- `cargo test --manifest-path codex-rs/Cargo.toml -p codex-rollout --test official_codex_discovery`: 8 passed.
- `cargo test --manifest-path codex-rs/Cargo.toml -p codex-cli --test vscode`: 10 passed.
- `cargo fmt --manifest-path codex-rs/Cargo.toml --check --package codex-app-server --package codex-app-server-protocol --package codex-protocol --package codex-core --package codex-rollout --package codex-cli --package codex-tui`: passed with the existing stable rustfmt warning about nightly-only `imports_granularity = Item`.
- `git diff --check`: passed.
- Read-only release prep audit worker: pass; follow-up warnings about `.pipeline/PROGRESS.md` and `PROJECT-SUMMARY.md` stale top-level status were fixed before commit.

## Next Actions

- Local commit is prepared as `release: v0.134.0-vsp.6`.
- Local tag is prepared as `v0.134.0-vsp.6`.
- Push `main` and tag to GitLab `origin` after Git write authentication is fixed.
- Create the GitLab Release and attach the archive plus `.sha256` after the remote tag exists.

## Remote Publication Blocker

GitLab remote publication is currently blocked by Git authentication on this machine:

- `git push origin main`: failed with `Permission denied (publickey,password)`.
- `git push origin v0.134.0-vsp.6`: failed with `Permission denied (publickey,password)`.
- HTTPS push with the `glab auth git-credential` helper: failed with GitLab HTTP Basic access denied.
- `glab` API token is present for API calls, but remote commit lookup for the local release commit returned `404 Commit Not Found`, confirming the commit is not on GitLab.

Do not create the GitLab Release until `main` and `v0.134.0-vsp.6` are pushed; otherwise the Release could point at an old remote ref.
