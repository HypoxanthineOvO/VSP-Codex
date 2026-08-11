# Authentication

VSP Codex does not use the official ChatGPT/OAuth login flow in normal operation.

Use the VSP Provider initialization page or `/provider` to configure an OpenAI Responses compatible Base URL + API Key. The config is stored under `~/.vsp-codex/config.toml`.

Official login, logout, Cloud, update, and feedback entrypoints are hidden or disabled in VSP Codex.
