# C9 M12 Audit Review

## Verdict

PASS with post-release risks.

The `v0.140.0-vsp.8.1` release evidence is sufficient for completion. Publication and local distribution have been performed after explicit user confirmation.

## Reviewed Refs

- `codex-rs/Cargo.toml`
- `codex-rs/Cargo.lock`
- `codex-cli/package.json`
- `.pipeline/release/vsp-release-manifest.json`
- `.pipeline/release/v0.140.0-vsp.8.1/vsp-release-manifest.json`
- `.pipeline/release/v0.140.0-vsp.8.1/vsp-codex-preview-linux-x64-gnu.tar.gz`
- `.pipeline/release/v0.140.0-vsp.8.1/vsp-codex-preview-linux-x64-gnu.tar.gz.sha256`
- `.pipeline/release/v0.140.0-vsp.8.1/release-notes.md`
- `README.md`
- `docs/install.md`
- `docs/vsp-codex.md`
- `CHANGELOG.md`
- `.pipeline/reviews/C9/M12/test.md`
- `.pipeline/reports/C9/M12-release-candidate.md`
- `.pipeline/evidence/C9/M12-sqlite-io-smoke.json`

## Findings

No blocking release-candidate evidence gaps were found.

Confirmed:

- Version consistency: Rust workspace metadata, lockfile, npm metadata, versioned manifest, stable manifest candidate, README/docs/CHANGELOG, and release notes all point to `0.140.0-vsp.8.1` / `v0.140.0-vsp.8.1`.
- RC artifact set exists under `.pipeline/release/v0.140.0-vsp.8.1/`: archive, `.sha256`, versioned manifest, and release notes.
- SHA256 consistency: archive SHA256 is `0ff6e0d34daf6265e23cb99d21b69d9acbbcce4df2067c168f72f0e7a82e567e`, matching the `.sha256`, versioned manifest, stable manifest candidate, docs, and release notes install command.
- Package metadata consistency: package `codex-package.json` records `version=0.140.0-vsp.8.1`, `target=x86_64-unknown-linux-gnu`, `variant=codex`, `entrypoint=bin/vsp-codex`, `channel=lab`, and the stable VSP manifest URL.
- Remote publication evidence: GitLab `main`, `c9-upstream-0.143.0-alpha.25-vsp8`, and tag dereference point at release commit `26ca6333792c0b6ea0f2504658617447389ba27f`; GitLab Release assets exist for the archive and `.sha256`.
- Validation evidence supports release completion: release build/version, package build, checksum, manifest JSON, package metadata, isolated installer smoke, VSCode tests/status, SQLite smoke, fmt/docs/diff/YAML/package JSON checks are recorded.
- Local distribution evidence: public-download install completed successfully, and `~/.vsp-codex/packages/standalone/current/bin/vsp-codex`, `~/.local/bin/vsp-codex`, and `~/.local/bin/codex` all report `vsp-codex 0.140.0-vsp.8.1`.

## Risks

- M12 evidence is not a full-workspace clean regression. Broad full-workspace nextest remains explicitly outside the clean evidence and must not be claimed as passed.
- SQLite runtime smoke exits with `no transport configured` because it intentionally invokes `app-server --listen off`; the useful evidence is that SQLite initialized with `logs` total rows 0 and noisy target rows 0.
- Existing VSCode/Remote APP processes may need reload/restart before they pick up the new standalone binary and bundled shim updates.

## Recommendation

Accept M12 as complete. Keep residual follow-up focused on process reload and any later broad-regression expansion; do not move the already-published release tag for post-release documentation/status commits.
