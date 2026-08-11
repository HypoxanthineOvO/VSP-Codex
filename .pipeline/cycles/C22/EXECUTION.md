# C22 Execution

## Current Step

Complete. Both C22 release parts are packaged and alpha.2 is installed locally.

## Validation Log

- RED: local Host Contract integration test rejected `plan`, `work_items`, and `session`.
- RED: semantic routing tests used legacy missing/invalidated guidance.
- RED: TUI workflow tests rejected the ten-command release and manifest-declared future command.
- GREEN: local Host Contract integration test, 7/7.
- GREEN: semantic and legacy workflow routing, 7/7.
- GREEN: TUI release routing and palette integration, 11/11.
- GREEN: command palette unit and reviewed snapshot, 1/1.
- GREEN: affected `codex-core` and `codex-tui` cargo check.
- GREEN: rustfmt check and `git diff --check`.
- GREEN: no pending `insta` snapshots.
- GREEN: retired compact/capture/audit/quality/optimize claims absent from the four target documents.
- GREEN: C22 has no diff in the three compact files modified in the main worktree.
- GREEN with known allowances: focused Clippy passes after allowing the pre-existing redundant-closure deny and two previously identified repository lints; no C22 lint was reported.

## Result

- Command routing consumes a bounded, verified manifest-declared command set and exposes all ten current commands, including `/hw:experiment`.
- Semantic workspaces route from `.pipeline/INDEX.md` and `.pipeline/cycles/` without depending on legacy Host status refresh.
- Legacy workspaces retain current, missing, invalidated, and invalid fail-closed behavior.
- Host status v1 accepts `delivery.kind = plan`, `work_items`, and `session`, including nullable optional fields, while preserving strict unknown-field and sensitive-key rejection.
- Documentation assigns Workflow semantics, lifecycle, recovery, Records, Receipts, and writes to the installed plugin; VSP retains Host UX only.

## Residual Risk

The local Hypo-Workflow source currently identifies itself as `14.0.0-alpha.5`, not `15.0.0-alpha.1`. C22 verifies the local ten-command/current-schema contract and uses a `15.0.0-alpha.1` fixture to prove version-agnostic parsing, but the final published 15.x artifact still requires an end-to-end rerun when it is available.

## Side Effects

### Local Packages

- `0.145.0-vsp.11.1-alpha.1`: compact continuation fix, rebuilt from the already installed package layout after confirming its binary matches the main-worktree release binary and differs from the pre-fix backup.
- `0.145.0-vsp.11.1-alpha.2`: compact continuation fix plus Hypo-Workflow semantic Host UX, built once in release mode and packaged from prebuilt binaries.

### Installation

- `~/.vsp-codex/packages/standalone/current` points to `0.145.0-vsp.11.1-alpha.2-x86_64-unknown-linux-gnu`.
- `vsp-codex --version` reports `0.145.0-vsp.11.1-alpha.2`.
- The installer reported that Official and VSP state DB migrations differ, so optional session compatibility migration did not run. It explicitly completed the CLI install successfully.

### Cleanup

- Removed the shared Cargo target: 104,524 files and 83.1 GiB.
- Removed the 1.4 GiB temporary alpha.2 package directory.
- Preserved both release archives and installed alpha.1 for rollback.

### GitLab Publication

- Alpha.1 hotfix: `https://gitlab.vsplab.cn/heyx/Codex/-/releases/v0.145.0-vsp.11.1-alpha.1`
  - Existing tag retained; no tag rewrite.
  - Old generic package retained for provenance.
  - Public fixed download paths now resolve to the C22 hotfix package with archive SHA-256 `cf0702acb2b5ee7e5642be520dc1b429d0f39657531567c74d1a86df2328bff8`.
- Alpha.2: `https://gitlab.vsplab.cn/heyx/Codex/-/releases/v0.145.0-vsp.11.1-alpha.2`
  - Release tag resolves to source commit `1e88aa0008c73396f60d4132540c96ec97e5d9be`.
  - GitLab package registry reports archive SHA-256 `84bd6631f09d1a2ad2824cab1250cfc75d98c2efcaaf15a7ca6771ad01591a32`.
  - Both `main` and `c22-hypo-workflow-semantic-interface` were pushed.
  - The stable alpha manifest on `main` selects alpha.2 and retains alpha.1 hotfix as the previous release.

No source write under `/home/heyx/Hypo-Workflow` was performed.

## Constraints

- No edits to the three compact files in the main worktree.
- No writes to the Hypo-Workflow source repository.
- No remote or installation side effects.
