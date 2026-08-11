---
name: plugin-creator
description: Create, validate, or update VSP-Codex plugins, including `.codex-plugin/plugin.json`, optional component folders, personal or repository marketplaces, and the local cachebuster/reinstall workflow.
---

# Plugin Creator

Use the bundled scripts to scaffold and validate plugins. Read `references/plugin-json-spec.md` when editing manifest or marketplace fields. Read `references/installing-and-updating.md` only when updating an already-installed local plugin.

## Create a plugin

1. Determine whether the plugin is personal or repository/team scoped.
2. Scaffold it from this skill directory:

```bash
python3 scripts/create_basic_plugin.py <plugin-name> [options]
```

The script normalizes names to lower-case hyphen-case, creates `<plugin-name>/.codex-plugin/plugin.json`, and defaults personal plugins to `~/plugins/<plugin-name>`.

Use only the components required by the request:

```bash
python3 scripts/create_basic_plugin.py <plugin-name> \
  --with-skills --with-hooks --with-scripts --with-assets --with-mcp --with-apps
```

Add `--with-marketplace` when the plugin should be discoverable through a marketplace. The personal default is `~/.agents/plugins/marketplace.json`. Use `--path` and `--marketplace-path` together for an explicitly requested repository/team marketplace.

3. Replace scaffold defaults with supplied metadata and remove all placeholders.
4. Validate before handoff:

```bash
python3 scripts/validate_plugin.py <plugin-path>
```

Use the active runtime permission policy for writes. Use `--force` only when the user requested an intentional replacement and the existing content has been inspected.

## Required invariants

- Keep the outer folder name and manifest `name` equal to the normalized plugin name.
- Keep `.codex-plugin/plugin.json` present.
- Add `apps` or `mcpServers` only when the matching component exists.
- Do not put unsupported fields such as `hooks` in `plugin.json`; a discovered hooks directory is separate from the manifest schema.
- Keep manifest paths relative to the plugin root and validate referenced assets.
- Do not leave `[TODO: ...]` placeholders.

## Marketplace rules

- Personal plugins default to `~/plugins/<name>` and `~/.agents/plugins/marketplace.json`; this marketplace is discovered automatically.
- Repository/team marketplaces are opt-in and use `<repo-root>/.agents/plugins/marketplace.json` unless the user provides another supported path.
- Keep each source path relative to the marketplace root as `./plugins/<name>`.
- Append entries unless the user asks to reorder. Replace an existing entry only with intentional `--force` use.
- Preserve the existing marketplace `name` and `interface.displayName`.
- Include `policy.installation`, `policy.authentication`, and `category`. Add `policy.products` only when explicitly requested.
- A non-default marketplace must be configured before reinstall, using `vsp-codex plugin marketplace add <marketplace-root>` when needed. Do not add the default personal marketplace explicitly.

See `references/plugin-json-spec.md` for exact accepted fields and values instead of duplicating its field guide here.

## Update an installed local plugin

Do not hand-edit marketplace state merely to invalidate the plugin cache.

1. Confirm the marketplace entry points to the local source being edited.
2. Run `python3 scripts/update_plugin_cachebuster.py <plugin-path>`.
3. Read the marketplace name with `python3 scripts/read_marketplace_name.py` or pass an explicit marketplace path.
4. Reinstall with `vsp-codex plugin add <plugin-name>@<marketplace-name>`.
5. Ask the user to test in a new thread when the runtime requires a fresh plugin snapshot.

Use `vsp-codex plugin list` to resolve ambiguous configured marketplaces. Stop on a non-local or mismatched source instead of applying the local update flow.

## Handoff

Report created or changed paths, validation results, marketplace changes, and any new-thread requirement. Provide Codex App deeplinks only when the active surface supports them and they help the user; they are not mandatory in VSP TUI output.
