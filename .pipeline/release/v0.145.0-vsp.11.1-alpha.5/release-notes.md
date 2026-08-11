# VSP Codex 0.145.0-vsp.11.1-alpha.5

This immutable recovery alpha contains the complete C26/C27 delivery from alpha.4 and fixes the
session compatibility check exposed by installing alpha.4 on a real Official/VSP home.

## Highlights

- H1, H2, and H3 use distinct width-safe TUI surfaces across narrow terminals, CJK text, long
  tokens, quotes, lists, Setext streaming, and OSC 8 hyperlinks.
- Completed app-server turns carry canonical final text. The TUI restores a complete final reply
  when item completion is missing or a stream is still active, without duplicating content.
- Thread events use one FIFO relay, preserving order under channel pressure.
- Classic shell only advertises parallel execution when its backend actually supports it; ZshFork
  remains serial.
- Local tests are organized into 17 behavior groups with explicit purpose, prerequisites, failure
  ownership, current-checkout SDK contracts, and Cargo/Bazel binary-root build coverage.
- Session compatibility now requires an exact shared migration prefix and the concrete
  `threads`/`backfill_state` columns it uses, while allowing either database to have compatible
  trailing migrations.

The package targets `x86_64-unknown-linux-gnu` and remains on the `alpha` channel. The default
`lab` channel remains on VSP 9.5.

## Install Or Update

```shell
VSP_CODEX_RELEASE_CHANNEL=alpha vsp-codex update
```

For a fresh alpha installation:

```shell
curl -fsSL https://gitlab.vsplab.cn/heyx/Codex/-/raw/main/scripts/install/install.sh | VSP_CODEX_RELEASE_CHANNEL=alpha sh
```

## Validation

- App-server protocol: 273/273; rollout persistence: 107/107.
- Focused core, TUI, app-server, shell-parallelism, permission, Linux denial, and remote-notification
  regressions passed with reviewed snapshots.
- Cargo workspace test binaries built and listed; four Bazel binary-root suites passed 337/337.
- Cargo/Bazel build contracts, strict Clippy, rustfmt, snapshot hygiene, and three Bazel native
  sandbox targets passed.
- Python default SDK: 128 passed with 38 explicit real-provider skips; current-checkout Python:
  44/44; current-checkout TypeScript: 5/5.
- Local and GitLab `vsp-smoke` validated build, CLI, real PTY TUI startup, app-server initialize, and
  thread/list. GitLab pipelines 354-357 passed through the alpha.4/alpha.5 publication chain.
- Session compatibility tests passed 9/9; release checks passed 48/48. A live Official migration
  46 / VSP migration 40 home backed up both databases, neutralized 63 historical rollouts, and
  rechecked at 0 pending / 1925 neutral.
- A real 100x34 PTY verified heading hierarchy, long-token wrapping, code alignment, final tail,
  composer separation, clean trace, and exit status 0.

Cargo-native bwrap tests require `libcap.pc`, which is not installed on the release host. The
equivalent Bazel native sandbox targets passed and the package includes the validated bwrap asset.
