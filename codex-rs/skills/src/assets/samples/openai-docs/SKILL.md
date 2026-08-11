---
name: openai-docs
description: Use for current, cited guidance about OpenAI APIs, models, products, or upstream OpenAI Codex. For VSP-Codex behavior, local configuration, or third-party provider models, use verified VSP sources first and use official OpenAI docs only to describe upstream behavior.
---

# OpenAI Docs

Use current official OpenAI sources and cite the pages that support the answer. Do not treat official OpenAI behavior or model recommendations as proof of VSP-Codex fork behavior.

## Choose the authority

- For OpenAI APIs, models, SDKs, and public products, use OpenAI developer documentation.
- For upstream OpenAI Codex behavior, use the current Codex manual route below.
- For VSP-Codex behavior, inspect local VSP source, configuration, model catalog, and callable session capabilities first. Use official docs only for the upstream baseline and label any divergence.
- For a custom or third-party provider, preserve its configured model names, endpoint, capabilities, and pricing assumptions unless provider evidence establishes a change.
- Preserve an explicitly requested target model. Do not replace it with the latest official model unless the user asks for that comparison or migration.

## Retrieve official sources

For non-Codex OpenAI documentation:

1. Search with `mcp__openaiDeveloperDocs__search_openai_docs`.
2. Fetch the relevant page with `mcp__openaiDeveloperDocs__fetch_openai_doc`.
3. For API schemas, parameters, or required fields, also use `mcp__openaiDeveloperDocs__get_openapi_spec` when available.
4. Use `list_openai_docs` only when a focused search cannot identify a page.

If Docs MCP is unavailable or unhelpful, use official OpenAI web pages as fallback and restrict the search to `developers.openai.com` and `platform.openai.com`. Do not automatically install MCP configuration or restart the client. Ask first when configuration mutation or restart is actually necessary.

## Upstream Codex manual

For broad questions about upstream Codex setup, configuration, skills, plugins, MCP, hooks, `AGENTS.md`, automations, or product surfaces:

1. Reuse a fresh same-thread manual cache when available.
2. Otherwise run `node <skill-dir>/scripts/fetch-codex-manual.mjs` when shell and temporary-cache access are allowed.
3. Use the emitted outline to read only relevant manual sections.
4. If the helper cannot run, use a narrow Docs MCP search/fetch, then official-domain web fallback.
5. If the official sources do not establish a claim, state bounded uncertainty instead of widening to unofficial sources.

Current-session behavior is authoritative for the active VSP environment. When it conflicts with upstream documentation, state the conflict and identify each source.

## Current model and prompting guidance

First decide whether the question concerns official OpenAI models or the active VSP/provider catalog.

- For the active VSP/provider model, inspect local configuration and provider metadata; do not run the official latest-model resolver by default.
- For a pure official OpenAI latest-model selection question, fetch `https://developers.openai.com/api/docs/guides/latest-model.md`.
- For an official latest/current/default model migration or prompting change, run:
  - POSIX: `sh <skill-dir>/scripts/resolve-latest-model-info`
  - Windows: `node <skill-dir>\scripts\resolve-latest-model-info.cjs`
- Resolver success requires JSON fields `model`, `migrationGuideUrl`, and `promptingGuideUrl`. Fetch the returned URLs exactly.
- For an explicitly named model, fetch guidance for that model and do not substitute latest-model guidance.
- If remote guidance is unavailable, use the matching bundled reference and disclose that it may be stale.

Read only the reference needed for the request:

- `references/latest-model.md`: model-selection fallback.
- `references/upgrade-guide.md`: upgrade-routing fallback.
- `references/upgrading-to-gpt-5p6-sol.md`: GPT-5.6 Sol/family migration checks.
- `references/prompting-guide.md`: prompting fallback and extraction rules.

## Implementation boundaries

- Documentation lookup does not require an API key. Running an API-backed artifact may require credentials; use the active credential mechanism and never expose secret values.
- Keep model and prompt migrations behavior-preserving and within the user's requested scope.
- Preserve multi-model cost, latency, and quality roles. Leave history, fixtures, eval baselines, pinned fallbacks, and unrelated provider integrations unchanged unless requested.
- Use current docs rather than memory for external facts. Do not invent pricing, availability, parameters, or breaking changes.
- Prefer concise paraphrases with citations. If official pages disagree, describe the difference and cite both.
