# C9 M4 Audit Report

Generated: 2026-06-17T17:34:11+08:00

## Verdict

pass_with_warnings

## Findings

- Version surface is internally consistent:
  - Cargo workspace version is `0.140.0-vsp.0`.
  - Release binary reports `vsp-codex 0.140.0-vsp.0`.
  - README, install guide, user guide, changelog, stable manifest, and versioned manifest point to `v0.140.0-vsp.0`.
- Upstream provenance is documented:
  - README, install guide, user guide, and changelog identify upstream `rust-v0.140.0-alpha.18`.
- Manifest/updater surface is compatible with the existing VSP standalone updater contract:
  - Stable manifest remains at `.pipeline/release/vsp-release-manifest.json`.
  - Latest stable manifest entry points at GitLab Release downloads for `v0.140.0-vsp.0`.
  - Versioned release manifest contains the single new release entry.
  - Package metadata embeds the stable manifest URL, `lab` channel, and target.
- Checksum and archive smoke passed.

## Risks

- The archive is 285 MB and still depends on GitLab Release downloads being created in M5. Until then, the documented install command will 404 if used against remote GitLab.
- Packaging requires either system `bwrap` or a host with `libcap.pc` available to compile bundled `bwrap`; this run used `/usr/bin/bwrap`.
- `voice/realtime` and user-facing `remote-control` remain non-retained by prior user decision and are not release blockers.

## Remote Side Effects

No push, tag, GitLab release, or remote asset upload was performed in M4.
