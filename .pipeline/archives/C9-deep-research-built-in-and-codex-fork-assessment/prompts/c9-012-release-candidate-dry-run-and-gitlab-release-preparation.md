# C9 M12 - Release Candidate Dry Run And GitLab Release Preparation

## Objective

Prepare a local release candidate for the VSP vsp.8 series after alpha.25 sync, run broad regression and SQLite I/O evidence, and stop before any GitLab remote publication.

## Requirements

- Confirm final public version before modifying release metadata: strict `v0.140.0-vsp.8` or `v0.140.0-vsp.8.1`.
- Run broad regression, release build, package builder, manifest/sha verification, docs/release notes checks, and SQLite I/O smoke.
- Prepare GitLab release notes and asset list locally.
- Do not tag, push, upload, or mutate GitLab release state without a separate release confirmation gate.

## Boundaries

- In scope: version metadata after confirmation, release notes, `.pipeline/release/v<confirmed-version>/`, manifests, package artifacts, docs consistency, validation reports.
- Out of scope: GitLab tag creation, branch push, package registry upload, release asset link creation, or any other remote write.

## Technical Solution

Use the existing VSP preview release builder and manifest format. Keep remote GitLab writes out of scope until a later release/update confirmation gate.

## Technical Route

1. Ask the user to confirm final release version: `v0.140.0-vsp.8` or `v0.140.0-vsp.8.1`.
2. Run hygiene checks and broad Rust regression.
3. Run the SQLite runtime I/O smoke and include evidence in the release report.
4. Build release `vsp-codex`, run `scripts/build_vsp_preview_release.py`, verify sha256 and manifest JSON.
5. Prepare release notes that explicitly record forked-from `rust-v0.143.0-alpha.25`.
6. Stop at local release readiness and present the remote release gate.

## Research Required

Deferred by user to the M12 pre-release confirmation gate: final release tag/package string. The user accepted the vsp.8 series with possible vsp.8.1 patch.

## Risks And Alternatives

Publishing under unchanged `v0.140.0-vsp.8` can confuse users if artifacts differ from the prior release. `v0.140.0-vsp.8.1` improves traceability but changes the visible version. Remote publishing remains explicitly gated.

## Validation Commands

```bash
cargo fmt --manifest-path codex-rs/Cargo.toml --all -- --check
pnpm format
python3 scripts/check_ci_docs.py
cargo nextest run --manifest-path codex-rs/Cargo.toml --no-fail-fast -p codex-state -p codex-app-server -p codex-app-server-protocol -p codex-core -p codex-cli -p codex-tui
cargo build --manifest-path codex-rs/Cargo.toml --release --bin vsp-codex
python3 scripts/build_vsp_preview_release.py --entrypoint-bin codex-rs/target/release/vsp-codex --force
sha256sum -c .pipeline/release/v<confirmed-version>/vsp-codex-preview-linux-x64-gnu.tar.gz.sha256
python3 -m json.tool .pipeline/release/v<confirmed-version>/vsp-release-manifest.json
git diff --check
```

## Evidence

Pass signals: all required checks pass or have documented environment-only limitations; release artifacts and notes are locally prepared; no remote write occurs.

Fail signals: regression failures are unexplained, SQLite smoke fails, release artifacts are missing/invalid, or remote GitLab state changes without confirmation.

## Human QA

Human confirmation is required before final version metadata changes and again before any GitLab release publication.

## Subworker Assignment Plan

- `test`: owns broad regression command execution, SQLite smoke evidence review, release artifact verification, and pseudo-test rejection. Writes `.pipeline/reviews/C9/M12/test.md`.
- `implement`: owns release metadata, release notes, local package build, and manifest preparation after version confirmation.
- `audit`: read-only review of release evidence, manifest/version consistency, upstream-base disclosure, and remote-write boundary. Writes `.pipeline/reviews/C9/M12/audit.md`.
- Main agent: coordinates confirmation gates, lifecycle evidence, and final readiness summary.
- Non-overlap: `test`, `implement`, and `audit` must remain distinct.
- Spawned worker write scope: spawned workers may edit `.pipeline/` artifacts and explicitly scoped root docs only unless explicitly granted named release/docs paths by the orchestrator. `audit` is read-only.

## Expected Artifacts

- `.pipeline/release/v<confirmed-version>/vsp-codex-preview-linux-x64-gnu.tar.gz`
- `.pipeline/release/v<confirmed-version>/vsp-codex-preview-linux-x64-gnu.tar.gz.sha256`
- `.pipeline/release/v<confirmed-version>/vsp-release-manifest.json`
- `.pipeline/release/v<confirmed-version>/release-notes.md`
- `.pipeline/reports/C9/M12-release-candidate.md`
- `.pipeline/reviews/C9/M12/test.md`
- `.pipeline/reviews/C9/M12/audit.md`
