# C9 M4 - Version Metadata And Release Surface Update

## Goal

Update VSP version, docs, install commands, release manifest, and release packaging metadata for the new upstream base.

## Technical Solution

Move the workspace version from `0.134.0-vsp.7.2` to a new VSP version based on upstream `rust-v0.140.0-alpha.18`. If Cargo rejects an alpha-derived VSP suffix, use a Cargo-compatible version and document the upstream alpha tag in release notes.

## Technical Route

1. Test candidate version string with Cargo metadata.
2. Update `codex-rs/Cargo.toml` workspace version.
3. Update README/docs install and version text.
4. Rebuild release package and manifest using `scripts/build_vsp_preview_release.py`.
5. Confirm installer/updater compatibility for VSP archive-url/manifest mode.

## Research Required

Pending execution: Cargo/package semver acceptance for the final VSP version string.

## Risks And Alternatives

Risks: semver rejection; docs pointing to assets before release exists; manifest incompatibility.

Alternative: use `0.140.0-vsp.1` while documenting upstream source tag `rust-v0.140.0-alpha.18`.

## Validation Path

Commands:

```bash
cargo metadata --manifest-path codex-rs/Cargo.toml --no-deps --format-version=1
cargo build --manifest-path codex-rs/Cargo.toml --release --bin vsp-codex
python3 scripts/build_vsp_preview_release.py --entrypoint-bin codex-rs/target/release/vsp-codex --cargo-profile release --force
sha256sum -c .pipeline/release/<version>/vsp-codex-preview-linux-x64-gnu.tar.gz.sha256
git diff --check
```

Pass signals: metadata parses, release binary reports expected version, archive checksum validates, manifest points to intended GitLab URL.

## Subworker Assignment Plan

- `test`: Owns version-string validation, package smoke commands, and release metadata checks. Writes `.pipeline/reviews/C9/M4/test.md`.
- `implement`: Owns metadata/docs/package updates. Must not edit test-owned evidence.
- `audit`: Checks version consistency, manifest/updater compatibility, and install docs honesty. Writes `.pipeline/reviews/C9/M4/audit.md`.

Lifecycle evidence required for all roles.
