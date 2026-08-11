---
kind: execution
cycle: C25-vsp-11-1-alpha-3-release
status: completed
updated: 2026-08-08T21:59:50+08:00
---

# C25 Execution

## M1

- Workspace `main` equals `origin/main` before release work; current version/tag is alpha.2.
- `glab auth status` confirms GitLab write access as `heyx`.
- Nod current is alpha.2 with managed app-server/proxy processes.
- Eden current is vsp.9.5 and still has vsp.9.2/vsp.9.5 processes; deployment must retire stale paths.
- Stable promotion will use a new formal version/tag after soak, not move alpha.3.
- User confirmed the next CI validation Cycle and soak feedback will be the Stable release gate.

## M2-M3 Candidate

- Workspace/package/Cargo.lock versions are alpha.3; Bazel lock refresh passed after deterministic
  workspace package version regeneration.
- Release builder tests passed 7/7; CLI check, release build, rustfmt, CI docs and diff hygiene passed.
- Candidate archive SHA256 is `682d8004c098420d78efd6879f58265e0ec856991cf49f6a11489696c95beb6b`.
- Extracted package reports alpha.3 and its entrypoint is byte-identical to the release build
  (`7881eaebf1efa9eb8cf6e2dda3908653d6452840a0a91b28484c4d5fb73251b5`).

## M4-M6 Publication And Deployment

- Pushed main and annotated tag to `cd7921547e`; created GitLab Release and package-registry assets.
- Remote SHA file and streamed archive digest match the local candidate; pipeline #346 succeeded.
- Local/Nod/Eden current binary hashes are identical and stdio initialize reports alpha.3.
- Nod/Eden control socket servers run alpha.3. Eden legacy thin proxies remain until this pre-release
  TUI session exits; repaired shims/current are ready for reconnect.
- Session compatibility auto-migration was skipped on migration-set mismatch and recorded for the CI
  validation Cycle.
