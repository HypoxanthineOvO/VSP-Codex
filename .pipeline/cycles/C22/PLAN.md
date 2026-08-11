# C22 Semantic Interface Adaptation

## Goal

Make VSP-Codex consume the current Hypo-Workflow command, semantic workspace, and Host status v1 contracts while preserving strict read-only Host boundaries.

## Work Items

1. Accept bounded manifest-declared command sets and expose `/hw:experiment`.
2. Route semantic workspaces independently of legacy Host status projections.
3. Accept current and legacy Host status v1 shapes with strict validation.
4. Remove retired Workflow ownership claims from VSP documentation.
5. Run contract, routing, palette, formatting, and focused crate tests.

## Acceptance Criteria

- The local Hypo-Workflow contract test passes with `HYPO_WORKFLOW_ROOT=/home/heyx/Hypo-Workflow`.
- Ten commands, including `/hw:experiment`, parse and appear in the palette.
- Invalid, duplicate, unsafe, excessive, and undeclared commands remain rejected.
- Semantic workspaces ignore missing or invalidated legacy projections without weakening legacy fail-closed behavior.
- Host status v1 accepts `plan`, `work_items`, and `session` while rejecting unknown or sensitive fields.
- Focused tests prove VSP does not write Workflow files.
