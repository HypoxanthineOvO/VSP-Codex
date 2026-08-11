# C9 M5 - Broad Regression And GitLab Release

## Goal

Run final regression, create release commit/tag, push to GitLab, and publish the GitLab release assets after explicit execution gates.

## Technical Solution

Use focused tests from earlier milestones plus broad workspace checks. Prepare release commit and tag only after validation passes. Publish GitLab release with archive/checksum assets or release links and verify remote refs and URLs.

## Technical Route

1. Run final focused regression suite.
2. Run broad format/build/test commands or document accepted environment gaps.
3. Ask for explicit user confirmation before release commit/tag/push/release.
4. Push `origin main` and tag after confirmation.
5. Create GitLab release and attach/download-link package assets.
6. Verify remote refs, release URL, raw manifest, archive URL, and local install smoke if applicable.

## Research Required

Pending execution: GitLab credentials and release tooling availability.

## Risks And Alternatives

Risks: GitLab auth failure, asset upload size limits, broad tests taking too long or flaking after alpha merge.

Alternative: if upload size fails, use release asset links to repository raw files as in previous releases; if full tests are impractical, use risk-based focused suite and record a user-accepted gap.

## Validation Path

Commands:

```bash
cargo fmt --manifest-path codex-rs/Cargo.toml --check
cargo test --manifest-path codex-rs/Cargo.toml --workspace
cargo build --manifest-path codex-rs/Cargo.toml --release --bin vsp-codex
git diff --check
git ls-remote origin refs/heads/main refs/tags/<new-tag> refs/tags/<new-tag>^{}
glab release view <new-tag> --repo heyx/Codex
curl -fsSI <archive-url>
```

Pass signals: local validation passes or gaps are explicitly accepted; remote main/tag point to release commit; GitLab release and asset URLs resolve.

## Subworker Assignment Plan

- `test`: Owns final regression evidence, remote ref checks, and release URL validation. Writes `.pipeline/reviews/C9/M5/test.md`.
- `implement`: Owns release commit/tag/push/release only after explicit main-agent gates. Must not edit test evidence.
- `audit`: Checks no upstream push occurred, release metadata matches version/manifest, and notes disclose alpha base and validation gaps. Writes `.pipeline/reviews/C9/M5/audit.md`.

Lifecycle evidence required for all roles.
