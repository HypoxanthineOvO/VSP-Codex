---
kind: discussion-summary
cycle: C24-responses-multi-provider-config-model-switching
status: closed
updated: 2026-08-08T21:22:00+08:00
---

# C24 Discussion Summary

## Confirmed Decisions

- C24 uses a standalone model-providers.toml addon layer and does not add third-party provider definitions or defaults to official config.toml.
- Runtime identity is provider_id + model_id; provider-local model metadata must not leak across identical model IDs.
- The unified /model surface uses provider tabs and keeps the official base provider reachable when an addon is the startup default.
- Missing credentials remain discoverable but disabled, and only the environment-variable name may be shown.
- At S1 the user accepted the four rendered UI cases and authorized the Agent to complete M4 using automated evidence without requiring user-operated testing.

## Scope Boundary

- M4 includes compact inheritance, remote app-server ownership analysis/fix where required, final automated validation, review and Cycle reporting.
- Publishing, pushing, installing and restarting services remain outside C24 and require a separate explicit action scope.

## Final Outcome

- M4 completed server-owned remote persistence, provider-specific compact regression and the final
  automated validation matrix. C24 is release-ready and closed, but not published.

## Incidental Regressions

- The same working set includes fixes for resize replay anchoring and streamed/canonical whitespace reconciliation. Both have focused regression tests and are recorded separately from the provider feature.
