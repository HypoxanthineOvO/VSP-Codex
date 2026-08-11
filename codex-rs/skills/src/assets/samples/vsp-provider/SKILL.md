---
name: vsp-provider
description: Configure or diagnose VSP-Codex and OpenAI-compatible model providers, including VSPLab Base URLs, API-key environment variables, provider/model selection, model catalogs, endpoint compatibility, and provider-backed capabilities such as image generation. Use for VSP-specific provider setup or failures, not for general OpenAI API documentation.
metadata:
  short-description: Configure or diagnose VSP providers
---

# VSP Provider

Use VSP runtime state, configuration, and diagnostics as the authority. Use official OpenAI documentation only to establish the upstream API contract.

## Separate the concerns

- Provider selects the Base URL, authentication, protocol, headers, retry policy, and capabilities.
- Model selects a model exposed by that provider.
- `/provider` changes the provider; `/model` changes the model.
- VSP config and authentication live under the active VSP home. Do not read or modify official Codex auth files as a substitute.

## Diagnose first

For a question or diagnosis, do not mutate configuration. Start with the redacted bounded report:

```bash
vsp-codex doctor --json
```

Use `vsp-codex doctor --summary` for a compact human view. Inspect only the relevant provider, credential-presence, model-catalog, network, and capability checks. Never print secret values.

Determine whether the failure is:

1. provider selection or missing configuration;
2. malformed Base URL or wrong API root;
3. missing credential environment variable;
4. Responses API incompatibility;
5. model absent from the selected provider catalog;
6. a capability not declared by the provider, such as image generation;
7. network, TLS, proxy, or server failure.

## Configure safely

Only change provider configuration when the user clearly requests it.

- Prefer the interactive `/provider` flow for ordinary VSPLab setup.
- Prefer `env_key` for durable provider credentials. Store the environment-variable name in configuration and keep the secret outside repository files, prompts, logs, command arguments, and generated artifacts.
- Do not ask the user to paste a token into chat when an environment or secret-store path is available.
- Use `wire_api = "responses"` for supported OpenAI-compatible providers.
- Configure the Base URL at the exact API root. VSP appends endpoint paths and does not automatically add `/v1`.
- Do not enable an optional provider capability until the endpoint actually supports it.
- Preserve unrelated providers, models, catalogs, and project settings.

When user input is required to choose a provider or credential method, prefer `request_user_input` if available.

## Validate

After an authorized change:

1. Reload configuration through the normal VSP path; request a restart only when the active surface cannot reload it.
2. Run `vsp-codex doctor --json` and verify the report remains redacted.
3. Confirm the selected provider and model separately.
4. Validate only the endpoints required by the user's workflow.
5. For image generation, verify the provider declares image support and that generation reaches `{base_url}/images/generations` with provider auth. Validate edits separately before promising them.

Report the effective provider ID, masked endpoint, credential source name, selected model, declared capabilities, checks performed, and exact remaining failure. Never report the credential value.

## Boundaries

- Do not use upstream `openai-docs` as proof of current VSP behavior.
- Do not switch to an official provider that VSP intentionally disables.
- Do not migrate provider credentials, rewrite catalogs, or broaden capabilities during an unrelated model change.
- Do not claim a provider supports an endpoint merely because it uses an OpenAI-compatible Responses API.
