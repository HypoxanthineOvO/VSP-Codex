---
name: skill-installer
description: List or install Codex Skills from the curated OpenAI catalog or a user-provided GitHub repository path, including private repositories. Use only when the user asks to discover or install Skills.
---

# Skill Installer

Use the bundled scripts rather than reimplementing catalog or installation logic. Listing is read-only apart from network/cache access. Installation changes the user's Skill directory and requires an explicit user request.

## Choose the operation

- If the user asks what can be installed, list the requested catalog. Default to `openai/skills` at `skills/.curated`; use `skills/.experimental` only when requested and label it experimental.
- If the user identifies one or more catalog Skills, install those names.
- If the user provides a GitHub repo/path or URL, install only the specified paths.
- If the user invokes this Skill without selecting an operation or item, list first. When a choice is required and `request_user_input` is available, use it after showing the relevant list.

## Commands

Run the scripts with the active Python 3 launcher for the platform:

- List: `scripts/list-skills.py [--repo owner/repo] [--path repo/path] [--ref ref] [--format text|json]`
- Install by repo/path: `scripts/install-skill-from-github.py --repo owner/repo --path path/to/skill [path/to/skill ...] [--ref ref] [--dest dir] [--method auto|download|git]`
- Install by URL: `scripts/install-skill-from-github.py --url https://github.com/owner/repo/tree/ref/path [--dest dir] [--method auto|download|git]`

The helper resolves the active VSP user-Skills destination. Do not hardcode `$HOME`, assume `$CODEX_HOME`, or target a managed `.system` directory. Use `--dest` only when the user explicitly requests another valid user-owned destination.

## Installation behavior

- Public GitHub repositories use direct download first. `auto` falls back to Git sparse checkout only for authentication or permission failures; Git tries HTTPS and then SSH.
- Private repositories use existing Git credentials or `GITHUB_TOKEN`/`GH_TOKEN`. Never print, persist, or place token values in command arguments or reports.
- Validate every selected directory contains `SKILL.md`, reject paths outside the repository, and abort if the destination already exists. Do not overwrite an installed Skill.
- Packaged System Skills are managed by VSP-Codex. Explain that they are already installed and refuse to overwrite them through this Skill.
- Follow current runtime network and filesystem permissions. Do not hardcode sandbox escalation or claim that access is available when the tool reports otherwise.
- On partial multi-Skill failure, report exactly which destinations were created and which failed. Do not roll back unrelated existing Skills.

After success, report installed names, source, and destination without secrets. Tell the user the new Skill is available on the next turn. For listing failures or unsupported non-GitHub download URLs, explain the exact limitation and stop.
