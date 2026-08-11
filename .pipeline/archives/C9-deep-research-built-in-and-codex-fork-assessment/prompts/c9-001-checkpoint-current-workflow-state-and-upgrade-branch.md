# C9 M1 - Checkpoint Current Workflow State And Upgrade Branch

## Goal

Create a clean, auditable starting point for the upstream upgrade by checkpointing current archive/planning changes and preparing an isolated upgrade branch. Do not fetch, merge, push, tag, or publish in this milestone.

## Technical Solution

Classify the dirty worktree into C9 archive/planning changes, existing user/local changes, and unrelated runtime noise. Checkpoint only the intended Workflow/Cycle planning and archive changes unless the user explicitly expands scope. Create a dedicated upgrade branch for `rust-v0.140.0-alpha.18`.

## Technical Route

1. Inspect `git status --short`, `git diff --stat`, and relevant scoped diffs.
2. Identify files created by C9 planning/archive maintenance.
3. Ask for explicit confirmation before creating a commit/checkpoint if command execution would mutate git history.
4. Create a checkpoint commit or equivalent branch marker.
5. Create/switch to an isolated branch such as `c9-upstream-0.140.0-alpha.18`.

## Research Required

Resolved. User selected commit/checkpoint strategy during Discover. Execution still needs a confirmation gate before creating a commit.

## Risks And Alternatives

Risks: dirty worktree can hide merge conflicts; staging unrelated user changes would violate worktree safety.

Rejected alternatives: merging directly on dirty `main`; destructive reset or checkout of user changes.

## Validation Path

Commands:

```bash
git status --short
git branch --show-current
git log --oneline -n 3
```

Pass signals: intended branch exists, checkpoint is recorded, no unrelated user changes were staged or reverted.

Audit must reject claims without actual git status/branch/log evidence.

## Subworker Assignment Plan

- `test`: Owns git-state evidence collection and checks that unrelated dirty files were not staged. Must write validation notes to `.pipeline/reviews/C9/M1/test.md`.
- `implement`: Owns checkpoint/branch operations after main-agent user confirmation. Must not edit test evidence.
- `audit`: Read-only review of staged files, checkpoint commit, branch state, and worktree safety. Must write `.pipeline/reviews/C9/M1/audit.md`.

Lifecycle evidence must include requested, started, completed/failed/blocked, and closed/close_failed for each role.
