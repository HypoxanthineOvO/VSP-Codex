# Configuration

VSP Codex reads its default configuration from `~/.vsp-codex/config.toml`.

The default Provider shape is:

```toml
model_provider = "vsp"

[model_providers.vsp]
name = "VSP Provider"
base_url = "https://your-provider.example/v1"
experimental_bearer_token = "your-api-key"
wire_api = "responses"
requires_openai_auth = false
supports_websockets = false
```

Additional user-defined OpenAI Responses compatible providers may be added under `[model_providers.<id>]` and selected from TUI with `/provider`.

VSP Codex intentionally does not share official Codex `auth.json`, `.credentials.json`, or complete `config.toml`.

## Lifecycle hooks

Admins can set top-level `allow_managed_hooks_only = true` in
`requirements.toml` to ignore user, project, and session hook configs while
still allowing managed hooks from requirements and managed config layers. This
setting is only supported in `requirements.toml`; putting it in `config.toml`
does not enable managed-hooks-only mode.
