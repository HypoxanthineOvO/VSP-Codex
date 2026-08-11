---
cycle: C28-prompt-stack-quality-redesign
stone: S3
status: candidate
updated: 2026-08-10T22:08:00+08:00
---

# Apps and Plugins Common Prompt Candidates

## Apps

### Findings

- Injected only when at least one connector is enabled and accessible; world-state logic restores it after compaction without duplicate injection.
- Current text says `tools_search`, but the real tool is `tool_search`.
- Discovery guidance is repeated in the `tool_search` tool description.

### English candidate

```markdown
## Apps (Connectors)
Apps expose MCP tools from the `codex_apps` server. Use an app when the user names it through an app link or when an available app materially fits the request.
Use already-visible app tools directly. Otherwise use `tool_search` when available. For app tool discovery, do not also call `list_mcp_resources` or `list_mcp_resource_templates`.
```

### 中文

```markdown
## Apps（Connectors）
Apps通过`codex_apps` server暴露MCP tools。用户通过app link点名，或某个available app确实适合请求时使用。
已可见的app tools直接使用；否则在可用时使用`tool_search`。发现app tools时不要再调用`list_mcp_resources`或`list_mcp_resource_templates`。
```

## Installed Plugins

### Findings

- The developer-role condition and world-state deduplication are correct.
- Six bullets repeat plugin relevance and capability routing.
- Explicit plugin capability summaries are already generated separately and should remain.

### English candidate

```markdown
## Plugins
A plugin bundles skills, MCP servers, and apps. Plugins are not invoked directly; use their available capabilities.
- Plugin Skills use the `plugin_name:skill_name` namespace. MCP tools keep their normal names; use tool provenance to identify the plugin.
- When the user explicitly names a plugin, prefer its relevant available capabilities for that turn.
- If the requested plugin has no callable capability in the current session, say so briefly and continue with the best available fallback.
```

### 中文

```markdown
## Plugins
Plugin把Skills、MCP servers和Apps打包在一起。Plugin本身不能直接调用，应使用它当前可用的底层能力。
- Plugin Skills使用`plugin_name:skill_name` namespace。MCP tools保留普通名称；通过tool provenance识别所属Plugin。
- 用户明确点名Plugin时，本轮优先使用其相关且可用的能力。
- 当前会话没有该Plugin的可调用能力时，简短说明并继续使用最佳fallback。
```

## Uninstalled Plugin Recommendations

Remove the `<recommended_plugins>` contextual user fragment.

Reasons:

- It injects system-generated installation advice with the `user` role.
- “Could possibly be better” encourages irrelevant plugin promotion.
- It can include up to 50 uninstalled entries even when the user did not ask for a plugin.
- The existing `list_available_plugins_to_install` and `request_plugin_install` tools already implement the correct explicit-request path.

Keep the tool contract: only search or suggest installation when the user explicitly requests an unavailable plugin/connector and current tools cannot satisfy that request.

## Tests required in M2

- Apps prompt is absent with no enabled+accessible connector and present otherwise.
- Apps candidate names `tool_search`, never `tools_search`.
- Apps and Plugins world-state restoration remains single-injection across compaction.
- Installed plugin prompt remains developer-role.
- No `<recommended_plugins>` user fragment is injected for ordinary requests.
- Explicit missing-plugin requests can still list candidates and invoke `request_plugin_install`.
