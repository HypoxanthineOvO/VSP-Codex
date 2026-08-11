---
kind: stone-result
cycle: C28-prompt-stack-quality-redesign
stone: S3
status: accepted
updated: 2026-08-10T22:15:16+08:00
source_edits: locked
---

# S3 Result: System Skills, Apps, and Plugins

Accepted by the user on 2026-08-10. Acceptance advances C28 to S4 and does not authorize product source edits before S5.

## Conclusion

S3 recommends retaining and revising all five packaged System Skills, adding exactly one VSP-specific System Skill (`vsp-provider`), keeping all Repo/User/installed Plugin Skills normally injected, resolving the single plain-name collision deterministically, and shortening VSP-owned Apps/Plugins common prompts. No product source has been edited.

## Source and cost map

The observed first-turn blocks were:

| Block | Characters | Owner |
|---|---:|---|
| Skills | 29,179 | VSP renderer + all Skill metadata sources |
| Apps | 646 | VSP core |
| Plugins | 1,014 | VSP core |
| Total | 30,839 | mixed |

The Skills block separates into:

| Source | Count | Rendered line chars | Treatment |
|---|---:|---:|---|
| System | 5 | 2,409 | deep review and rewrite |
| Repo | 14 | 2,825 | normal injection, provenance/usefulness only |
| User | 22 | 10,633 | normal injection, provenance/usefulness only |
| Installed Plugin | 36 | 12,314 | normal namespaced injection, provenance/usefulness only |
| Skills header/protocol | n/a | 998 | VSP common layer |

Current VSP-owned core cost across Skills header/System metadata/Apps/Plugins is approximately 5,067 characters. Current project/user/plugin environment additions are 25,772 characters. System Skill body reductions affect only turns where those Skills trigger; they are not counted as first-turn baseline savings.

## Active Skill health

- Actual rollout count: 77 entries, 76 unique plain names.
- All 77 active `SKILL.md` files are readable, have valid YAML frontmatter, non-empty names/descriptions, and descriptions under 1,024 characters.
- Historical plugin cache versions exist on disk but are not injected by the active plugin manager and are not counted as duplicates.
- No Repo/User/Plugin Skill has sufficient evidence for disablement; all continue normal injection.
- The sole active collision is `skill-creator` from User and System scopes.

Plain duplicate behavior should use the existing scope precedence (`Repo > User > System > Admin`) to expose one effective bare name. The current User `skill-creator` therefore overrides the packaged one for `$skill-creator`; structured path selection can still target the shadowed source. Plugin Skills remain namespace-qualified as `plugin_name:skill_name`.

## Existing System Skills

### skill-installer

- Accepted: runtime permissions, explicit install intent, structured selection, script-resolved user destination, credential protection, accurate failures, and no managed System Skill overwrite.
- Keep public download, Git sparse-checkout fallback, private repositories, and core CLI options.

### openai-docs

- Split upstream OpenAI authority from VSP/provider behavior.
- VSP source/config/catalog/current capabilities are authoritative for VSP behavior.
- Remove the unavailable API-key Skill gate and automatic MCP config mutation, escalation, and restart.
- Preserve Docs MCP, Codex manual helper, official latest-model resolver, explicit target preservation, references, and citations.
- Body: 22,804 -> 4,758 characters.

### plugin-creator

- Keep verified personal paths: `~/plugins` and `~/.agents/plugins/marketplace.json`.
- Use `vsp-codex plugin ...`, not upstream-only command spelling.
- Make Codex App deeplinks optional by active surface.
- Move field detail to references and fix the canonical manifest sample that includes validator-rejected `hooks`.
- Body: 11,036 -> 4,116 characters.

### skill-creator

- Align frontmatter guidance with the validator/loader: `name` and `description` required; supported optional fields only when consumed.
- Resolve the active VSP user Skill root instead of assuming `$CODEX_HOME/skills`; protect managed `.system` cache.
- Preserve trigger-first descriptions, progressive disclosure, resource types, init/validate workflow, and realistic forward tests.
- Forward tests follow S2 root routing, cost, permission, and nesting policy.
- Body: 21,897 -> 5,156 characters.

### imagegen

- Source inspection proved built-in `image_gen` is a VSP Rust extension using the active model provider, not a fixed official host service.
- VSPLab should explicitly declare image capability; built-in generation then uses its configured Base URL, provider auth, and `gpt-image-2`.
- Requests are `{base_url}/images/generations` and `{base_url}/images/edits`; `/v1` is not added automatically.
- Replace provider-name/auth-mode gating with explicit provider capability gating; custom providers default off unless enabled.
- Keep Python CLI as explicit/batch/special-control fallback. It already respects `OPENAI_BASE_URL` and `OPENAI_API_KEY`.
- Body: 23,963 -> 5,485 characters.

## New System Skill

Add only `vsp-provider`:

- Own VSP-specific provider setup and diagnosis after `openai-docs` exits that authority domain.
- Combine `/provider`, provider/model separation, exact API root, model catalogs, redacted `vsp-codex doctor`, credential-source safety, and provider capabilities.
- Initial package needs only `SKILL.md` and `agents/openai.yaml`; existing doctor logic already owns deterministic diagnostics.
- Do not add a second new Skill without evidence.

## Apps and Plugins common prompts

- Apps remains conditional on an enabled and accessible connector; fix `tools_search` to `tool_search` and reduce the body from 646 to approximately 364 characters.
- Installed Plugins remains developer-role and conditional; reduce six overlapping rules to four compact rules, approximately 511 characters.
- Preserve explicit plugin capability summaries and world-state single-injection/restoration behavior.
- Remove the `<recommended_plugins>` contextual user fragment. It injects system-generated promotion as user text, may list 50 uninstalled plugins, and conflicts with the explicit-request contract of `list_available_plugins_to_install`.
- Missing plugin installation remains available when the user explicitly requests an unavailable plugin/connector.

## Implementation surface for M2

- `codex-rs/skills/src/assets/samples/`: revise five Skills and add `vsp-provider`.
- `plugin-creator/references/`: fix VSP CLI commands and invalid `hooks` manifest guidance.
- `codex-rs/core-skills/src/render.rs` and mention selection: aliases, short protocol, effective plain-name winner, structured-path escape hatch, and observability.
- `codex-rs/ext/image-generation/`, `model-provider-info`, provider config/UI, and schema: explicit image capability and VSPLab enablement.
- `codex-rs/core/src/context/`: Apps/Plugins candidates and removal of recommended-plugin user fragment.

## Test design

- Skills: 77-source fixture health, aliases, scope winner, structured path selection, plugin namespace, compaction persistence, and metadata budget reporting.
- System Skill packages: validator pass, embedded asset fingerprint/install, required resources, and representative script dry-runs.
- Image generation: capability false/true visibility, VSPLab generate/edit URLs, bearer auth without secret logging, `gpt-image-2`, provider/model prerequisites, and official OpenAI regression.
- Apps/Plugins: conditional injection, `tool_search` spelling, developer role, world-state dedupe, no recommended user fragment, and explicit missing-plugin install flow.
- Provider diagnostics: doctor output remains redacted and configuration changes preserve unrelated providers/settings.

## Expected behavior

- Normal tasks retain all user/project/plugin capabilities without loading Skill bodies.
- System Skills trigger more accurately and load much smaller bodies.
- `$skill-creator` resolves instead of silently doing nothing.
- VSPLab image generation uses the active provider directly.
- Ordinary requests no longer receive unsolicited uninstalled-plugin suggestions.
- VSP-specific provider questions have a packaged authority instead of leaking into upstream OpenAI docs guidance.

## Problems and risks

- Current `/provider` persists an inline bearer token. `env_key` is preferable when credentials are supplied through the environment, but interactive setup cannot persist a parent-process environment variable. Do not remove inline support until a secure VSP credential-store design is accepted; image generation does not depend on that migration.
- Provider generation and editing support may differ. M2 must test both and avoid promising editing from generation-only evidence.
- User/Plugin metadata remains the dominant first-turn cost by explicit user decision. Alias and duplicate improvements reduce mechanical overhead but do not remove those entries.
- Full C28 token targets depend on S1, S2, S4 and final integration; S3 alone must not claim the overall target is met.

## Validation performed in S3

- Read and compared all five System Skill entry files; read relevant scripts/references for provider, installer, plugin, and image behavior.
- Cross-checked image generation and plugin paths against VSP Rust source.
- Parsed all 77 active Skill frontmatters successfully.
- Checked candidate Markdown fences, YAML records, and `git diff --check` for Workflow artifacts.
- No product test suite was run because product source remains locked until S5 acceptance.
