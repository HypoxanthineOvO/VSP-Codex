# C10 M5 - Version 0.145.0-vsp.9 Release And Distribution

## Objective

Cut the new public series `0.145.0-vsp.9` on the merged `rust-v0.145.0-alpha.4` base plus the ten selected post-tag production and reliability fixes through `upstream/main@5c19155cbd`, run the release dry run, and — only after an explicit user gate — publish the GitLab release and distribute locally.

## Requirements

- Keep workspace version at `0.145.0-vsp.9`; update release notes and the VSP release manifest (new series entry first in the stable manifest).
- Verify merge ancestry for `rust-v0.145.0-alpha.4` and retain the dependency-closed post-tag Cherry Pick set recorded in `.pipeline/cycle.yaml` before packaging.
- Build release `vsp-codex`, package via `scripts/build_vsp_preview_release.py`, verify sha256.
- Release dry run report before any remote mutation.
- HARD GATE: no push, tag, upload, or GitLab release until explicit user confirmation via Question Tool.
- After confirmation: push branch/main, tag on the release artifact commit, upload via GitLab Generic Package Registry-backed release asset path, publish release, then local distribution and three-entrypoint verification.

## Boundaries

- In scope: version metadata, packaging, release notes/manifest, gated remote release, local install.
- Out of scope: new feature work; moving the published tag after later commits (C9 M12 lesson).

## Technical Solution

Reuse the C9 M12-validated release pipeline. The cross-series bump (0.140.0-vsp.8.x -> 0.145.0-vsp.9) is validated at the manifest level: stable manifest first-entry semantics drive client update detection, so update-path behavior is explicitly tested.

## Technical Route

1. Version metadata edits + `cargo metadata` sanity.
2. Release build + packaging + sha256 verification.
3. Manifest JSON validation incl. first-entry `0.145.0-vsp.9`; verify updater behavior on cross-series bump.
4. Dry-run report -> Question Tool release gate.
5. On approval: push, tag, upload assets, publish release; verify remote download 200 + remote sha256.
6. Local distribution: install public package; `vsp-codex`/`codex`/standalone entrypoints all report `vsp-codex 0.145.0-vsp.9`; GitLab pipeline green.

## Research Required

None; the release path was validated in C9 M12 (incl. the HTTP 413 workaround via Generic Package Registry).

## Risks And Alternatives

- Risk: cross-series version jump confuses the VSP updater; mitigation is an explicit updater test against the new manifest before release.
- Rejected alternative: staying on 0.140.0-vsp.8.x (user chose the new series).
- Known host limitation: full workspace suite is historically red (libcap.pc, app-server integration timeouts); the release gate contract is focused suites + smoke + scenarios, and the release notes must not claim a full-workspace-clean run.

## Validation Commands

```bash
cargo metadata --manifest-path codex-rs/Cargo.toml --no-deps --format-version=1 | python3 -c "import json,sys; assert json.load(sys.stdin)['packages'][0]"
cargo build --manifest-path codex-rs/Cargo.toml --release --bin vsp-codex
codex-rs/target/release/vsp-codex --version
python3 scripts/build_vsp_preview_release.py --entrypoint-bin codex-rs/target/release/vsp-codex --bwrap-bin /usr/bin/bwrap --cargo-profile release --upstream-tag rust-v0.145.0-alpha.4 --force
git merge-base --is-ancestor rust-v0.145.0-alpha.4 HEAD
sha256sum -c .pipeline/release/v0.145.0-vsp.9/vsp-codex-preview-linux-x64-gnu.tar.gz.sha256
python3 -m json.tool .pipeline/release/vsp-release-manifest.json
git diff --check
```

Post-release: remote asset URLs return 200; remote sha256 matches; three local entrypoints report `vsp-codex 0.145.0-vsp.9`; GitLab main pipeline success.

## Evidence

Pass signals: dry run fully green before the gate; after approval, tag sits on the release artifact commit, remote assets verified, local entrypoints on the new version.

Fail signals: any remote mutation before the gate; tag moved by later commits; updater mishandles the cross-series bump; release notes overclaim test coverage.

## Subworker Assignment Plan

- `test`: owns dry-run validation runs, updater cross-series test, and post-release verification evidence; writes `.pipeline/reviews/C10/M5/test.md`.
- `implement`: owns version metadata, release notes, manifest, and packaging invocation edits. No test asset edits.
- `audit`: read-only; verifies gate discipline (no pre-gate remote mutations), tag placement, claim accuracy in release notes; writes `.pipeline/reviews/C10/M5/audit.md`.
- Main agent: orchestration, lifecycle, presents the release gate to the user, executes gated remote steps only after explicit approval.
- Non-overlap: distinct identities for the three roles.

## Expected Artifacts

- Release commit + tag `v0.145.0-vsp.9` (post-gate)
- `.pipeline/release/v0.145.0-vsp.9/` artifacts + release notes
- Updated `.pipeline/release/vsp-release-manifest.json`
- `.pipeline/reports/C10/M5-release.md`
- `.pipeline/reviews/C10/M5/test.md`
- `.pipeline/reviews/C10/M5/audit.md`
