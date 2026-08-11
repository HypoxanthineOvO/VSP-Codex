# C8 Prompt Audit

Verdict: pass

## Blocking Findings

None.

## Non-Blocking Suggestions

- `.pipeline/prompts/c8-002-vscode-plugin-protocol-capability-map.md`: M2 allows local VSIX/manifest/log inspection and says not to fetch external artifacts without confirmation. During execution, if no installed extension artifacts are available, record that limitation early and avoid delaying M3-M6 on unavailable black-box evidence.
- `.pipeline/prompts/c8-004-session-discovery-strategy-official-codex.md`: M4 correctly requires field-level privacy classification. During execution, make the chosen default strategy explicit before implementation so sanitizer tests do not drift across export/import, mirror, and gated sharing designs.
- `.pipeline/prompts/c8-006-markdown-workflow-surface-remote-ssh-acceptance.md`: M6 correctly requires real Remote SSH acceptance. During execution, record the exact remote host context, VSCode server settings path, VSP binary path, and session id used for acceptance evidence.

## Evidence Checklist

- P2 technical fields are preserved: pass. Each milestone in `.pipeline/plan-state/c8-p2-decompose.md` has goal, implementation scope, technical solution, route, validation path, and clear non-goals where needed; each generated prompt expands those into objective, route, research, risks, validation, and audit focus.
- Remote SSH is primary: pass. P2, architecture, M1, M2, M3, M5, and M6 consistently treat VSCode Remote SSH as the primary plugin environment, with remote `.vscode-server/data/Machine/settings.json` prioritized in M1.
- Official VSCode plugin black-box boundary: pass. Architecture and M2 explicitly keep the plugin as a black-box/VSIX client by default; M5/M6 avoid plugin source modification unless evidence and confirmation justify it.
- Strict worker separation: pass. `.pipeline/plan-state/c8-p3-generate.md` defines strict separation, and every prompt assigns `test`, `implement`, and `audit` roles with no test/implementation overlap.
- Required Workflow decisions use `request_user_input`: pass. Architecture, P3, and prompts require `request_user_input` when available.
- Authorization intent versus permission grant: pass. Architecture and M5 explicitly keep authorization-intent as intent capture only, with real permission grants handled by existing approval/permission mechanisms.
- Safety gates: pass. Destructive operations, system-level dependency installation, remote side effects, tag/release/publish actions, broad sharing, and secret-adjacent exposure retain confirmation/authorization gates across the prompt set.
- M6 Remote SSH acceptance: pass. M6 requires real Remote SSH VSCode plugin resume/interaction, request input handling, and readable Workflow/Subagent Markdown surface.
- Prompt order and dependencies: pass. M1/M2 establish Remote SSH configuration and protocol capability before M3-M6; M5 depends on M2, and M6 depends on M1-M5 and is the final acceptance milestone.

