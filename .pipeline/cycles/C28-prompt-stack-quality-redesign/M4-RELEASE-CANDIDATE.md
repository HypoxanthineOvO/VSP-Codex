---
kind: release-candidate-report
cycle: C28-prompt-stack-quality-redesign
milestone: M4
status: completed
updated: 2026-08-11T19:10:44+08:00
version: 0.145.0-vsp.11.1-alpha.6
canonical_commit: f61aacca62593cc2ff2da56cbbaac4f89a4be4c8
build_commit: dc0a24f8c1d1ff53be8470fd9a03034ff0e46a95
build_run: 31476110232
---

# M4 Release Candidate

## Conclusion

The alpha.6 release candidate passed the accepted C28 behavior matrix and native Linux, macOS,
and Windows release builds. The downloaded aggregate artifact has valid ZIP CRCs; all three
release archives have independently recomputed SHA-256 digests matching both their sidecars and
the aggregate manifest. Each native runner executed the packaged entrypoint with `--version` as
part of `validate_built_package`, and the downloaded Linux entrypoint was executed a second time.

No tag or GitLab Release exists yet. Publication remains behind explicit human approval.

## Source Provenance

- Build source on GitLab canonical `main`: `f61aacca62593cc2ff2da56cbbaac4f89a4be4c8`.
- Release metadata commit: `e068ac464d1eb24686c582201fe2f6280cf48304`.
- Post-publication manifest correction on canonical `main`: `c0c9cf6224`.
- GitHub build-only `main`: `dc0a24f8c1d1ff53be8470fd9a03034ff0e46a95`.
- A path-excluded diff proves the two trees are identical outside `.pipeline/**` and
  `.opencode/**`. The setup action, release workflow, and preview builder blobs are byte-identical.
- Immutable GitLab tag `v0.145.0-vsp.11.1-alpha.6` points to release metadata commit
  `e068ac464d1eb24686c582201fe2f6280cf48304`.

## Product Changes

- Stage A installs the accepted Sol base instructions, compact default runtime policy, proactive
  Multi-Agent policy assembly, shorter Apps/Plugins prompts, and removes unsolicited plugin
  recommendations.
- Stage B rewrites the bundled System Skills, adds `vsp-provider`, resolves effective Skill names
  by scope, caps catalog metadata at 2,000 tokens, and preserves active Skill bodies through
  compaction and resume.
- Stage E connects trusted semantic Worker Routing to Multi-Agent V2 with six routing classes,
  offline provider/model validation, ordered fallback reasons, explicit/role conflict handling,
  fork selection, and structured output/telemetry.
- Release preparation adds target-specific archive naming, strict same-version manifest
  aggregation, and native Linux/macOS/Windows build coverage.

## Validation

- Core Skills: 121/121; initial Multi-Agent assembly: 2/2; Skill protocol: 2/2; compact/lifecycle:
  19/19; Worker Routing: 6/6; Multi-Agent V2 spawn: 19/19; explicit plugin flow: 1/1.
- Required smoke: 7/7; release checks: 52/52; focused TUI platform-gating regression: 1/1.
- `cargo check -p codex-cli --bin vsp-codex`, formatting, diff hygiene, and workflow YAML passed.
- GitHub Actions run `31476110232` completed successfully:
  - Linux x86-64 GNU: 46m06s.
  - Windows x86-64 MSVC: 1h23m06s.
  - macOS ARM64: 1h29m58s.
  - Aggregate release bundle: 32s.
- Aggregate GitHub artifact ID `9097925225`: 611,396,708 bytes; outer ZIP SHA-256
  `3c568ab26eec17ff16fb96e734547b9c10091c66e9acbfe4f45069085ca3f32d`; all ZIP CRCs pass.

## Release Assets

| Target | Archive | Bytes | SHA-256 |
| --- | --- | ---: | --- |
| `x86_64-unknown-linux-gnu` | `vsp-codex-preview-linux-x64-gnu.tar.gz` | 343,318,608 | `ced4c72705006186cc9c91a13ad6a1fff47afb91028d21967941e8399938d7dd` |
| `aarch64-apple-darwin` | `vsp-codex-preview-macos-arm64.tar.gz` | 132,640,319 | `5824b199315a154f3fe377ed5b8813533b351a15a632db5adad1f7a77bab1b07` |
| `x86_64-pc-windows-msvc` | `vsp-codex-preview-windows-x64-msvc.tar.gz` | 142,891,041 | `1cefdba90a3f29d29b36c436482aa96985a4e321aec25da2a727453e86c02333` |

The package metadata reports version `0.145.0-vsp.11.1-alpha.6`, channel `alpha`, and the exact
target triple in every archive. Entrypoints are Linux ELF x86-64, macOS Mach-O ARM64, and Windows
PE32+ x86-64. Linux and macOS contain `vsp-codex` and `codex-code-mode-host`; Windows contains the
matching `.exe` files plus the command runner and sandbox setup helpers. Linux also contains
`bwrap`. The downloaded Linux entrypoint prints `vsp-codex 0.145.0-vsp.11.1-alpha.6`.

## Aggregate Manifest

The manifest has schema 1, minimum updater `0.134.0-vsp.6`, upstream tag
`rust-v0.145.0-alpha.4`, and exactly three unique targets. Every release has version
`0.145.0-vsp.11.1-alpha.6`, channel `alpha`, the matching archive digest, and canonical GitLab
Release/download URLs. Structured assertions and all three `sha256sum -c` checks pass.

## Problems Encountered

- Native diagnostics exposed and fixed snapshot tag discovery, Windows-invalid colon paths,
  stale non-Linux voice compilation, MSVC V8 artifact naming, and CRLF checksum parsing.
- The aggregate download encountered Azure EOFs and low single-connection proxy throughput. A
  retrying 12-range download reproduced the exact GitHub byte count; the reconstructed ZIP then
  passed SHA-256 and full CRC validation.
- The first Generic Package Registry upload accidentally reduced the already-aggregated manifest
  to one entry. Public asset verification caught this immediately; the candidate and stable
  manifests were restored from the original aggregate, and the public manifest asset is being
  replaced with the three-target version. Archive bytes and all archive digests were unaffected.

## Residual Risks

- Live Subagent inference was blocked before spawn by repeated VSPLab 502 responses. Offline
  routing, cross-provider child configuration, and handler tests pass, but this RC does not claim
  a successful live child inference.
- The original prompt targets were not fully met: input tokens fell 22.6% rather than 25%, and
  non-tool characters fell 28.4% rather than 40%, because the user deferred Stage C AGENTS/runtime
  splitting. Accepted product behavior is green.
- The macOS and Windows alpha binaries are not code-signed or notarized by this workflow. Users may
  encounter Gatekeeper or SmartScreen warnings.
- Manifest download URLs are intentionally not live until the GitLab Release is published, and the
  stable manifest has not been changed before approval.

## Publication Result

- GitLab Release: `https://gitlab.vsplab.cn/heyx/Codex/-/releases/v0.145.0-vsp.11.1-alpha.6`.
- The Release contains seven assets: three archives, three SHA-256 sidecars, and the aggregate
  manifest. Generic Package Registry was used because the default upload endpoint rejects files
  above its request-size limit.
- Public re-downloads of all three archives match the sidecars; the public manifest contains all
  three targets and exact matching digests. Direct Release download URLs return redirects to the
  public package assets.
- The first manifest upload was one-target due to an accidental second merge; public verification
  caught it, the stale package file/link was replaced, and the corrected three-target manifest is
  now live. Archive bytes and archive digests were unchanged.

## Approval Gate

The approved publication scope was executed: the three candidate entries were merged into the
stable manifest, release metadata was committed, the immutable annotated tag was pushed, the
GitLab Release and seven assets were published, and public downloads were reverified. No local
installation, deployment, or service restart was performed.
