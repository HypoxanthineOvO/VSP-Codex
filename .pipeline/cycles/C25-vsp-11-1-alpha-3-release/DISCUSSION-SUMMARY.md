---
kind: discussion-summary
cycle: C25-vsp-11-1-alpha-3-release
status: closed
updated: 2026-08-08T21:59:50+08:00
---

# C25 Discussion Summary

- Publish full `0.145.0-vsp.11.1-alpha.3` to `origin/main` with tag, GitLab Release and Linux
  x64 GNU preview asset.
- Include the current closed C23+C24 source batch; do not represent the intertwined worktree as
  C24-only.
- Deploy and restart managed standalone on local, Nod and Eden.
- After soak, promote the same source commit through a new stable version such as
  `0.145.0-vsp.11.1`; rebuild so the binary reports the formal version and keep alpha.3 immutable.
- Stable promotion will follow a separate CI validation Cycle: use alpha.3 on local/Nod/Eden,
  collect soak feedback, complete CI validation/fixes, then publish the resulting verified commit as
  `0.145.0-vsp.11.1`.
