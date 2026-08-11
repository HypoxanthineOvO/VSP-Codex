# v0.145.0-vsp.9 Release Notes

Status: Linux x86_64 GNU lab release. Local installation is not part of this publication.

## Upstream Baseline

- Official release baseline: `rust-v0.145.0-alpha.4`.
- Baseline merge commit: `820f9b14b8`.
- Official `main` reviewed through `5c19155cbd` after the final fetch.
- Ten post-alpha.4 commits were Cherry Picked in upstream order. Seven have exact
  patch IDs; pagination, materialized-plugin-hook, and rollout-ordinal conflicts retain
  documented VSP provider/cache, remote-plugin, discovery, and test context.
- The VSP sequence advances from `vsp.8.x` to `vsp.9`. An official baseline change
  does not reset the VSP suffix.

## Highlights

- Pending Question Tool cards follow the most recently resumed client. Old terminals
  cannot answer after ownership moves; disconnect and unsubscribe restore the prior
  live owner.
- If the thread listener disappears during disconnect, unsubscribe, or answer resolution,
  the fallback path still dismisses the stale card, replays to the restored owner, and
  clears the accepted-answer resolution target.
- Running resume reuses the live pending request and does not revive an answered
  Question from rollout history. Cold cross-process resume creates a continuation.
- Native Ask clients receive the native card without synthesized `Question Context`
  output. Non-native clients keep a readable fallback that completes the turn.
- Plugin thread starts default to `approval_policy=never` and
  `danger-full-access`; explicit request permission settings remain authoritative.
- Local auto-compaction hides its internal summary, clears visibility state on every
  terminal turn path, and continues rendering the real final response.
- VSCode integrated terminals leave local file paths to VSCode's native Ctrl+click
  handling instead of emitting file OSC8 links that open externally.
- VSP update discovery uses the GitLab stable manifest. The displayed version and
  install action come from the same release, while the exact installed release
  supplies the official upstream baseline for Cherry Pick reminders.
- Startup and background refreshes can surface VSP updates or official upstream drift
  in the current TUI session without duplicate notices.
- GPT-5.6 Sol remains available with Ultra reasoning, and the final official changes
  include model announcement priority, paginated thread persistence, reasoning-summary
  capability handling, plugin-hook trust, CODEX_HOME IDE socket preference, upload
  diagnostics, response-item ID prefixes, and canonical configuration links.
- Paginated rollout records now carry durable ordinals, and app-server protocol exposes
  deterministic thread-history projection while legacy rollouts remain unchanged.

## Prepared Asset

- Archive: `vsp-codex-preview-linux-x64-gnu.tar.gz`
- Target: `x86_64-unknown-linux-gnu`
- Channel: `lab`
- Size: `352415153` bytes
- SHA256: `bbc6ecd5abbee8ea8273e57db80086bb05d3ba4dde6804937f1f5559db121a72`
- Upstream provenance: `rust-v0.145.0-alpha.4`

Install command:

```shell
curl -fsSL https://gitlab.vsplab.cn/heyx/Codex/-/raw/main/scripts/install/install.sh | sh -s -- \
  --archive-url https://gitlab.vsplab.cn/heyx/Codex/-/releases/v0.145.0-vsp.9/downloads/vsp-codex-preview-linux-x64-gnu.tar.gz \
  --sha256 bbc6ecd5abbee8ea8273e57db80086bb05d3ba4dde6804937f1f5559db121a72 \
  --target x86_64-unknown-linux-gnu \
  --version 0.145.0-vsp.9-preview \
  --manifest-url https://gitlab.vsplab.cn/heyx/Codex/-/raw/main/.pipeline/release/vsp-release-manifest.json \
  --vscode \
  --no-launch \
  --yes
```

## Validation

- Release and archived binaries report `vsp-codex 0.145.0-vsp.9`.
- Archive checksum, members, package metadata, target, channel, source, and GitLab
  manifest URL were independently verified.
- Stable manifest contains one `0.145.0-vsp.9` Linux lab entry and preserves all
  historical releases.
- Ask focused unit tests passed 5/5; Ask integration passed 24/24; app-server library
  passed 282/282.
- Release-only updater passed 18/18; cache and update-action suites passed 2/2 each.
- Memories passed 39/39; extension API passed 11/11; exec-server passed all package
  targets; core workspace-roots, reasoning, and review suites passed.
- IDE IPC passed 10/10; Full Access defaults/overrides, compaction visibility, VSCode
  file links, and model announcement behavior all passed focused regression.
- Release builder passed 4/4; rustfmt, CI docs, diff, JSON, YAML, and snapshot checks passed.
- Final rollout and protocol regression suites passed 99/99 and 271/271 respectively.

## Distribution

This release declares Linux x86_64 GNU only. Windows and macOS archives are not included.
