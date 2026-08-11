# C22 Discussion Summary

## Outcome

Adapt VSP-Codex to consume the current Hypo-Workflow release contract without taking ownership of Workflow semantics, lifecycle, or persistence.

## Confirmed Contract Changes

- Command manifests contain ten public commands, including `/hw:experiment`.
- Host status v1 accepts `delivery.kind = plan` and optional `work_items` and `session` fields.
- Semantic workspaces are identified by `.pipeline/INDEX.md` and `.pipeline/cycles/`.
- Semantic SessionStart guidance does not require a legacy Host status refresh.

## Boundaries

- Work only in the C22 VSP-Codex worktree.
- Treat `/home/heyx/Hypo-Workflow` as read-only contract input.
- Do not restore compact fallback, conversation capture, or Workflow writers.
- Do not push, publish, release, or perform a system-level install.
