# C10 M3 Audit

## Verdict

`pass`: no blocking findings.

`rust-v0.143.0-alpha.33` is an ancestor of current `HEAD`, the final M3 formatting blocker has been repaired in `9e23218d10`, and the key VSP surface anchors remain present.

## Findings

Blocking findings: none.

Non-blocking residual:

- `codex-tui` still emits non-blocking unreachable-pattern warnings for `SlashCommand::{Model, Personality, Permissions}` in `tui/src/slash_command.rs`. This does not block M3 but should stay visible for follow-up cleanup.
- During audit, only Workflow state files and `.pipeline/.lock` were dirty. Those are expected lifecycle artifacts and are being persisted by the main agent.

## Evidence Reviewed

- `git log --oneline --max-count=12`: current `HEAD` is `9e23218d10 fix: format M3 final validation drift`, with `05aed0f7fb Merge rust-v0.143.0-alpha.33` in history.
- `git merge-base --is-ancestor rust-v0.143.0-alpha.33 HEAD`: exit `0`.
- `.pipeline/reports/C10/M3-merge-conflicts.md`: 13 conflicts resolved and staged during M3; protected Workflow file handling and conflict marker cleanup are documented.
- `.pipeline/reports/C10/M3-vsp-surface-recheck.md`: VSP CLI/package identity, remote plugins disabled policy, provider disablement, Workflow routing, Ask/plan/markdown surfaces, VSCode Remote, Subagent UX, and Marketplace labels are recorded as preserved.
- `.pipeline/reviews/C10/M3/test.md`: final matrix passed after formatting follow-up, including workspace compile, focused routing/plugin/app-server/TUI/CLI suites, diff check, marker scan, and final formatting tail check.
- Additional read-only audit checks: `cargo fmt --check` passed with only stable rustfmt warnings about nightly-only `imports_granularity`; `git diff --check` passed; conflict marker scan found no line-start open/close markers.

## Surface Summary

Remote plugin policy remains gated by `Feature::RemotePlugin` through `PluginsConfigInput` and the `core-plugins` remote catalog/sync/fetch paths. VSP provider filtering still hides built-in official providers and preserves VSP/custom compatible provider behavior. Hypo-Workflow `/hw:*` and `$hypo-workflow:*` routing, `request_user_input` metadata, plan freshness, VSP Curated/Marketplace labels, and VSCode Remote wrapper/status/repair test anchors remain present.

Upstream multi-agent v2 / remote-control / WebSocket content is carried internally. Audit did not find VSP-facing README/docs commitments that productize Rendezvous or multi-agent v2 as VSP public capabilities in M3.

## Residual Risk

The audit did not rerun the full Cargo matrix. It relies on M3 test-worker evidence for the full matrix and only rechecked the final known blocker and repository hygiene tail checks.
