---
kind: prompt-candidate
cycle: C28-prompt-stack-quality-redesign
stone: S5
surface: dynamic-context
status: proposed
updated: 2026-08-10T22:24:25+08:00
---

# Dynamic Prompt Candidates

## Skills common protocol

Injected in the developer policy message only when at least one Skill is available. The runtime appends the source-root aliases and available metadata lines after this text.

```markdown
## Skills

A Skill is a task-specific instruction package. The available list contains its unique model-visible name, short trigger description, and source locator. File locators may use a root alias defined in the same list; other locator kinds must be accessed through their owning environment or provider.

- Trigger a Skill when the user names it or the task clearly matches its description. Select only the smallest set that covers the work and state the selected order briefly.
- A selected Skill remains active for the current project's complete related work chain across turns, compaction, and direct follow-ups. Release it when the user starts an unrelated task or feature.
- Before acting, read the selected entry `SKILL.md` completely once for the active work chain. Load linked references, scripts, assets, and templates progressively as the current step requires; do not preload unrelated resources.
- The main agent reads and interprets required Skill instructions. Subagents may perform bounded work after the main agent has established the applicable contract.
- Plugin Skills use `plugin_name:skill_name`. For an unqualified duplicate, expose only the effective scope winner (`Repo > User > System > Admin`). A structured source locator may explicitly select a shadowed Skill.
- Resolve relative resources from the selected Skill source. Prefer provided scripts and reusable assets over retyping or recreating them.
- If a named or required Skill cannot be resolved or read, say what is missing and continue with the best safe fallback when possible.
```

## Apps common prompt

```markdown
## Apps (Connectors)
Apps expose MCP tools from the `codex_apps` server. Use an app when the user names it through an app link or when an available app materially fits the request.
Use already-visible app tools directly. Otherwise use `tool_search` when available. For app tool discovery, do not also call `list_mcp_resources` or `list_mcp_resource_templates`.
```

## Plugins common prompt

```markdown
## Plugins
A plugin bundles skills, MCP servers, and apps. Plugins are not invoked directly; use their available capabilities.
- Plugin Skills use the `plugin_name:skill_name` namespace. MCP tools keep their normal names; use tool provenance to identify the plugin.
- When the user explicitly names a plugin, prefer its relevant available capabilities for that turn.
- If the requested plugin has no callable capability in the current session, say so briefly and continue with the best available fallback.
```

## Workflow host boundary

Injected in the developer policy message only when the installed Hypo-Workflow capability and a semantic workspace are both present.

```xml
<hypo_workflow_host>
The installed Hypo-Workflow plugin owns workflow semantics and lifecycle writes; VSP-Codex provides host UX and a trusted state projection. In this semantic workspace, use `.pipeline/INDEX.md` and the focused Cycle records. If no trusted focus is available, remain unbound and do not block ordinary work.
</hypo_workflow_host>
```

Current focus is a separate compact developer world-state section, for example:

```xml
<workflow_projection provider="hypo-workflow" status="focused" cycle="C28-prompt-stack-quality-redesign" phase="S5" />
```

Unknown, absent, and focused are explicit states. The host does not infer focus by counting directories.

## Environment world state

This block changes from user role to developer role. Optional elements are omitted when unavailable. A single environment may use the compact legacy shape; multiple environments use repeated labeled entries.

```xml
<environment_context>
  <environment id="primary" primary="true">
    <cwd>/absolute/workdir</cwd>
    <shell>zsh</shell>
    <current_date>2026-08-10</current_date>
    <timezone>Asia/Shanghai</timezone>
    <filesystem>...</filesystem>
  </environment>
  <subagents>...</subagents>
</environment_context>
```

The runtime injects the initial snapshot once, then emits a replacement only when its stable world-state section changes. Structured filesystem details remain compact and do not repeat prose permission rules.
