# v0.145.0-vsp.11.1-alpha.1

VSP 11.1 alpha.1 introduces a transactional local context-compaction state machine, bounded long-context summaries, and an explicit execution-continuity contract while retaining the focused TUI and workflow improvements prepared on the 11.0 alpha.3 baseline. The default `lab` channel remains on 9.5; this release is available only through the explicit `alpha` channel.

## Changes

- C22 compact hotfix: when a mid-turn replacement reaches the structural summary floor, older completed tool interactions move into the checkpoint summary instead of making the running turn uncompactionable; the active user request and newest context remain live.
- Local compaction installs history transactionally from the current attempt and fails explicitly instead of reusing a stale assistant response.
- Mid-turn compaction preserves the complete active user, reasoning, function-call, and function-output tail without reviving completed historical requests.
- Active checkpoints carry a bounded continuation contract so the successor resumes executable work instead of stopping at a plan/status recap or treating optional recovery metadata as a gate.
- Summary output is limited to 10% of the model context with a 16,384-token ceiling; total replacement history is limited to 30% with a 64,000-token ceiling and must contract long source histories.
- Plan freshness receives a once-only delegated-work reminder, while Alt+Up and thread navigation return predictably to the root session.
- Multi-Agent V2 defaults overridden child configurations to the latest three turns and improves the active-agent panel, counts, aligned columns, context details, and root-return affordance.
- Markdown blockquotes fill their subtle background through the render edge while retaining the rail and outer margin.

## Install Or Update

```shell
VSP_CODEX_RELEASE_CHANNEL=alpha vsp-codex update
```

For a fresh alpha installation:

```shell
curl -fsSL https://gitlab.vsplab.cn/heyx/Codex/-/raw/main/scripts/install/install.sh | VSP_CODEX_RELEASE_CHANNEL=alpha sh
```

## Verification

- The C22 hotfix binary differs from the preserved pre-fix binary, matches the locally installed alpha.1 binary, and its rebuilt release archive passes SHA-256 verification.
- Local compaction covers manual, pre-turn, true mid-turn, repeated, resume, fork, rollback, model-switch, context-window failure, and snapshot layouts.
- The live post-compaction execution-stop incident is covered by a bounded active-continuation regression.
- Blockquote-focused tests: 42 passed.
- Active-subagent tests: 17 passed.
- Plan freshness, recent-turn multi-agent defaults, path-backed navigation, and Alt+Up root navigation passed their focused suites.
- Scoped Clippy, Rustfmt, diff hygiene, snapshot hygiene, release-builder tests, package provenance, installation, upgrade, and rollback are release gates.
