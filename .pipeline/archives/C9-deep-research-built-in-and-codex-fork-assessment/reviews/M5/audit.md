# C9 M5 Audit Report

## Verdict

`release_gate_pending`

M5 local validation is sufficient to present a release decision, but it is not sufficient to silently publish. The broad app-server integration suite remains a material risk and must be disclosed before any release commit/tag/push/GitLab release.

## Evidence

- Release version: `0.140.0-vsp.0`.
- Upstream base: `rust-v0.140.0-alpha.18`.
- Preview archive: `.pipeline/release/v0.140.0-vsp.0/vsp-codex-preview-linux-x64-gnu.tar.gz`.
- SHA256: `90086fcdcb607439178a60680c3e12c80fdacf752de7a3111e2a2c8954c3792b`.
- Stable manifest: `.pipeline/release/vsp-release-manifest.json`.
- Version manifest: `.pipeline/release/v0.140.0-vsp.0/vsp-release-manifest.json`.

## Remote Side Effects

No remote side effects were performed:

- No `git push`.
- No release tag.
- No GitLab release.
- No asset upload or release link mutation.

## Risks

- `codex-app-server --test all` is not clean in this environment. The result was 245 passed, 376 failed, 2 ignored, with many failures caused by initialize/websocket deadline timeouts.
- Full workspace tests need environment adjustments:
  - `libcap.pc` is absent, so bundled `codex-bwrap` debug builds fail unless `CODEX_SKIP_BWRAP_BUILD=1` is set.
  - app-server in-process tests require a larger test stack; `RUST_MIN_STACK=16777216` passed the targeted in-process case.
- `voice/realtime` and `remote-control` are still explicitly non-retained VSP surfaces by user decision.

## Recommendation

Do not publish automatically. Ask for an explicit release gate decision with the broad-suite risk disclosed. If the user accepts the risk, the next implementation step can create a release commit/tag and publish to GitLab.
