---
cycle: C28-prompt-stack-quality-redesign
stone: S3
status: accepted
updated: 2026-08-10T22:00:47+08:00
---

# S3 imagegen Endpoint and Token Review

## Evidence

- System `imagegen/SKILL.md` is 356 lines / 23,963 chars and currently mandates built-in `image_gen` first.
- The built-in tool is host-managed, does not require `OPENAI_API_KEY`, and exposes no user-controlled base URL in the Skill contract.
- The bundled `scripts/image_gen.py` constructs `OpenAI()` / `AsyncOpenAI()` without explicit arguments. The SDK therefore already consumes `OPENAI_BASE_URL` and `OPENAI_API_KEY`; no official URL is hardcoded in the script.
- The script requires `OPENAI_API_KEY` for live calls but does not print its value. Dry-run does not require it.
- The current session exposes no callable built-in `image_gen` tool, so the existing built-in-first contract cannot complete image generation here without moving to CLI.
- User `hypo-image` proves a working isolation pattern: `~/.config/gpt_image_cli/env` is mode 600; wrapper maps `OPENAI_BASE_URL` and `OPENAI_API_KEY` only into the child process; backend remains the bundled System Skill CLI; model is locked to `gpt-image-2`.
- User-level `hypo-image` remains external evidence only and is not rewritten or made a required System dependency.

## Security boundary

- Never place a real token in System Skill text, repository source, command-line arguments, generated docs, logs, or ordinary checked-in config.
- Prefer image-specific secret environment variables over reusing chat-provider credentials.
- Map custom variables to the OpenAI SDK variables only inside the CLI child process.
- Do not read an arbitrary personal config path from a packaged System Skill unless VSP formally owns that config contract.

## Accepted route

```text
active provider explicitly supports image generation
  -> built-in Rust image_gen using that provider's base URL and auth
else explicit CLI request or compatible fallback credentials
  -> bundled CLI
else
  -> report unsupported provider or missing setup without exposing a token
```

The built-in tool sends:

- `POST {provider.base_url}/images/generations` for generation.
- `POST {provider.base_url}/images/edits` for editing.
- `Authorization: Bearer <provider credential>` through `provider.api_auth()`.
- model `gpt-image-2` with `quality=auto`, `size=auto`, and `background=auto`.

For VSPLab, configure the base URL exactly at the OpenAI-compatible root expected by the service, including `/v1` when required. Prefer `env_key = "OPENAI_API_KEY"` over an inline bearer token. The actual key must never enter Skill text, repository source, logs, or command arguments.

The current visibility gate incorrectly infers image support from provider name/auth mode. M2 should replace that inference with an explicit provider image-generation capability. Custom providers default to unsupported unless enabled; built-in OpenAI and VSPLab enable it. The active model must also support image input, namespace tools and the image-generation feature must remain enabled.

An image request already authorizes normal use of the selected provider; no second confirmation is needed merely because the provider is VSPLab. Confirmation remains necessary before switching to another provider, credential set, or downgraded model.

## Alternatives

The earlier custom-CLI-first alternatives are superseded. Source inspection proved that the built-in Rust extension already uses the active model provider; its missing tool was caused by capability gating, not a fixed official endpoint.

## Prompt simplification independent of route

- Remove repeated model/transparency rules from `SKILL.md`; keep exact flags and model matrices in `references/cli.md` and `references/image-api.md`.
- Keep built-in vs CLI capability boundaries, generate/edit decision, non-destructive output, inspection, and project asset placement.
- Resolve the installed System Skill path rather than assuming `$CODEX_HOME` falls back to `~/.codex`; VSP system cache currently lives under the VSP home.
- Replace the absolute prohibition on modifying `scripts/image_gen.py` with a user-task boundary: do not patch the installed managed copy during image generation; product maintainers edit the source asset and validate it normally.
- Replace stale sandbox examples in `references/codex-network.md` with active-runtime permission guidance.
- State that built-in mode uses the active provider's endpoint and auth; do not claim it never needs an API key.
- Keep CLI environment support (`OPENAI_BASE_URL` / `OPENAI_API_KEY`) as fallback behavior, not the primary VSPLab integration.

## Required validation in M2

- Provider capability false: image namespace is absent.
- VSPLab capability true with an image-input model: image namespace is present.
- Generation request reaches `{base_url}/images/generations`, carries bearer auth, and uses `gpt-image-2`.
- Edit request reaches `{base_url}/images/edits` with referenced images when the provider supports that endpoint.
- A missing or invalid provider credential fails without logging the credential.
- Official OpenAI behavior remains unchanged.
