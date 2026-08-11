# C9 M7 - Conflict Inventory And Upstream Fix Baseline

## Objective

Create an evidence-backed conflict inventory for syncing Codex-VSP toward `rust-v0.143.0-alpha.25`, and decide which VSP surfaces are candidates for official replacement before broad product code is changed.

## Requirements

- Inspect upstream alpha.25 changes by conflict domain: SQLite logging, protocol, app-server, core workflow, TUI, CLI/VSCode, plugins, adapters, docs, CI, and release tooling.
- Record official SQLite fix coverage and VSP equivalence candidates.
- Produce a replacement / preserve / defer matrix for VSP surfaces.
- Pin all analysis to `rust-v0.143.0-alpha.25`; do not chase newer tags in this milestone.

## Boundaries

- In scope: read-only git analysis, reports, evidence files, and planning records under `.pipeline/`.
- Out of scope: broad merge resolution, product source edits, release version edits, remote pushes, GitLab release changes.

## Technical Solution

Use read-only git diff/log/merge-tree style analysis to classify conflicts before code migration. Treat VSP features as preserve-by-default until official equivalence is demonstrated with path-level evidence.

## Technical Route

1. Compare the current branch against `rust-v0.143.0-alpha.25` for all confirmed conflict domains.
2. Map upstream commits that address SQLite persistent logging to local code paths.
3. Create `.pipeline/reports/C9/M7-vsp-equivalence-matrix.yaml` with `replace`, `preserve`, `defer`, and evidence fields.
4. Create `.pipeline/reports/C9/M7-upstream-alpha25-conflict-inventory.md` with conflict-domain summaries and next-milestone implications.
5. Record non-goals and known risks for M8-M12.

## Research Required

Resolved by local evidence from Plan:

- Remote tag scan found `rust-v0.143.0-alpha.25` as latest alpha and `rust-v0.142.2` as latest stable.
- Architecture diff counts and conflict domains are recorded in `.plan-state/c9-upstream-alpha25-architecture.yaml`.

## Risks And Alternatives

Rejected alternative: fresh upstream baseline plus reapply VSP, because the VSP change surface is large and the user selected grouped incremental merge.

Primary risk: analysis can become stale if upstream moves. Mitigation: this milestone is pinned to the exact alpha.25 tag.

## Validation Commands

```bash
git diff --name-status rust-v0.143.0-alpha.25...HEAD -- . ':!.pipeline' ':!.plan-state'
git log --oneline --decorate --max-count=20 rust-v0.143.0-alpha.25
python3 - <<'PY'
from pathlib import Path
for path in [
    Path(".pipeline/reports/C9/M7-upstream-alpha25-conflict-inventory.md"),
    Path(".pipeline/reports/C9/M7-vsp-equivalence-matrix.yaml"),
]:
    assert path.is_file(), path
    assert path.read_text(encoding="utf-8").strip(), path
PY
```

## Evidence

Pass signals: reports exist, classify all major conflict domains, and contain no unassigned high-risk VSP surface. Official replacement claims cite concrete upstream files or commits.

Fail signals: any major domain lacks owner/action, or official replacement is claimed without evidence.

## Subworker Assignment Plan

- `test`: owns evidence checklist validation and writes `.pipeline/reviews/C9/M7/test.md`. It must reject reports that leave high-risk domains unassigned.
- `implement`: owns `.pipeline/` report and matrix creation only. It must not edit product source, tests, package manifests, generated schemas, or release artifacts in this milestone.
- `audit`: read-only review of conflict classification, equivalence claims, and append-only prompt numbering. Writes `.pipeline/reviews/C9/M7/audit.md`.
- Main agent: orchestrates workers and lifecycle evidence. It must not perform product implementation during this inventory milestone.
- Non-overlap: the same worker identity must not satisfy `test`, `implement`, and `audit`.
- Spawned worker write scope: spawned workers may edit only `.pipeline/` files and explicitly scoped root docs when requested. `audit` is read-only.

## Expected Artifacts

- `.pipeline/reports/C9/M7-upstream-alpha25-conflict-inventory.md`
- `.pipeline/reports/C9/M7-vsp-equivalence-matrix.yaml`
- `.pipeline/reviews/C9/M7/test.md`
- `.pipeline/reviews/C9/M7/audit.md`
