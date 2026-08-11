# VSP Codex 0.145.0-vsp.11.1-alpha.4

This alpha closes the C26 TUI heading and CI review after exhaustive test-quality repair and real
terminal acceptance. It also includes the accepted C27 behavior-oriented local test portfolio.

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
  thread/list.
- A real 100x34 PTY verified heading hierarchy, long-token wrapping, code alignment, final tail,
  composer separation, clean trace, and exit status 0.

Cargo-native bwrap tests require `libcap.pc`, which is not installed on the release host. The
equivalent Bazel native sandbox targets passed and the package includes the validated bwrap asset.
