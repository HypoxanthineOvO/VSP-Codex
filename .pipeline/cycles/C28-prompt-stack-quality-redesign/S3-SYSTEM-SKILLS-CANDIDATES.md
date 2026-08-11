---
cycle: C28-prompt-stack-quality-redesign
stone: S3
status: draft
updated: 2026-08-10T21:42:07+08:00
scope: openai-docs, plugin-creator, skill-creator
---

# S3 System Skills Candidates

本文件只记录候选提示，不修改 `codex-rs/skills/src/assets/samples/` 产品源。`imagegen` 因自定义 endpoint/token 问题单独审阅；`skill-installer` 已在前一决策中接受。

## 1. openai-docs

### Review conclusion

- 现有正文把 upstream OpenAI Codex 文档当作所有“Codex自身”问题的第一权威，未区分 VSP-Codex fork 行为。
- latest-model resolver 甚至要求在查看repo/config之前运行，会覆盖 VSP 自定义provider和本地model catalog语义。
- `openai-platform-api-key` 在当前System Skills中不存在，不应作为通用前置gate。
- 缺少MCP时自动写配置、自动升级权限并要求重启，与VSP动态权限和服务重启边界冲突。
- product snapshots、source route和workflow重复较多；应保留官方检索顺序、精确URL、显式目标模型、migration scope与citation质量规则。

### English candidate

````markdown
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
````

### 中文候选

````markdown
---
名称：openai-docs
描述：用于获取关于 OpenAI API、模型、产品或 upstream OpenAI Codex 的当前、带引用指导。对于 VSP-Codex 行为、本地配置或第三方 provider 模型，先使用已验证的 VSP 来源；官方 OpenAI 文档只用于说明 upstream 行为。
---

# OpenAI 文档

使用当前官方 OpenAI 来源，并引用真正支撑回答的页面。不得把官方 OpenAI 行为或模型建议当作 VSP-Codex fork 行为的证据。

## 选择权威来源

- OpenAI API、模型、SDK 和公开产品：使用 OpenAI 开发者文档。
- upstream OpenAI Codex 行为：使用下面的当前 Codex manual 路径。
- VSP-Codex 行为：先检查本地 VSP 源码、配置、模型目录和当前会话可调用能力。官方文档只作为 upstream 基线，并明确标注差异。
- 自定义或第三方 provider：保留已配置的模型名、endpoint、能力和价格假设，除非 provider 证据确认发生变化。
- 保留用户明确指定的目标模型。除非用户要求比较或迁移，否则不得替换成官方最新模型。

## 获取官方来源

对于非 Codex 的 OpenAI 文档：

1. 使用 `mcp__openaiDeveloperDocs__search_openai_docs` 搜索。
2. 使用 `mcp__openaiDeveloperDocs__fetch_openai_doc` 获取相关页面。
3. 对 API schema、参数或必填字段问题，在可用时同时使用 `mcp__openaiDeveloperDocs__get_openapi_spec`。
4. 只有聚焦搜索找不到页面时才使用 `list_openai_docs`。

Docs MCP 不可用或无帮助时，fallback 到 OpenAI 官方网页，并把搜索限制在 `developers.openai.com` 和 `platform.openai.com`。不得自动安装 MCP 配置或重启客户端；确实需要修改配置或重启时先询问。

## upstream Codex manual

对于 upstream Codex 设置、配置、Skills、Plugins、MCP、hooks、`AGENTS.md`、automations 或产品 surface 的广泛问题：

1. 有可用且新鲜的同线程 manual cache 时复用。
2. 否则在允许 shell 和临时缓存时运行 `node <skill-dir>/scripts/fetch-codex-manual.mjs`。
3. 使用输出的 outline，只读取相关 manual 小节。
4. helper 无法运行时，使用窄范围 Docs MCP 搜索/获取，再 fallback 到官方域名网页。
5. 官方来源无法证明某项主张时，给出有边界的不确定性，不扩展到非官方来源。

当前会话行为对当前 VSP 环境具有权威性。它与 upstream 文档冲突时，说明冲突并分别标注来源。

## 当前模型与提示指导

先判断问题针对官方 OpenAI 模型，还是当前 VSP/provider catalog。

- 当前 VSP/provider 模型：检查本地配置和 provider metadata；默认不运行官方 latest-model resolver。
- 纯官方 OpenAI 最新模型选择：获取 `https://developers.openai.com/api/docs/guides/latest-model.md`。
- 官方 latest/current/default 模型迁移或提示修改：运行：
  - POSIX：`sh <skill-dir>/scripts/resolve-latest-model-info`
  - Windows：`node <skill-dir>\scripts\resolve-latest-model-info.cjs`
- resolver 成功必须输出 `model`、`migrationGuideUrl`、`promptingGuideUrl` 三个 JSON 字段，并精确获取返回的 URL。
- 用户明确指定模型时，获取该模型的指导，不得替换为最新模型指导。
- 远程指导不可用时，使用匹配的 bundled reference，并披露它可能过时。

只读取请求实际需要的 reference：

- `references/latest-model.md`：模型选择 fallback。
- `references/upgrade-guide.md`：升级路由 fallback。
- `references/upgrading-to-gpt-5p6-sol.md`：GPT-5.6 Sol/系列迁移检查。
- `references/prompting-guide.md`：提示 fallback 与提取规则。

## 实现边界

- 查文档不需要 API key。运行 API-backed artifact 可能需要凭据；使用当前 credential 机制，绝不暴露 secret 值。
- 模型与提示迁移应保持行为，并限制在用户请求范围。
- 保留多模型的成本、延迟与质量角色。除非明确要求，不改历史记录、fixtures、eval baselines、固定 fallback 或无关 provider integration。
- 外部事实使用当前文档而不是记忆。不得编造价格、可用性、参数或 breaking changes。
- 优先使用带引用的简洁释义。官方页面不一致时说明差异并同时引用。
````

## 2. plugin-creator

### Review conclusion

- VSP源码确认个人marketplace确实从用户home下的 `.agents/plugins/marketplace.json` 自动发现，默认插件目录 `~/plugins` 与其相对source路径一致，应保留。
- CLI命令应使用本仓库的 `vsp-codex plugin ...`，而不是硬编码 upstream `codex plugin ...`。
- 固定要求每次输出Codex App deeplink不适合VSP TUI；只在当前surface支持且用户需要时提供。
- 正文重复了reference中的manifest与marketplace字段表；正文应只保留流程和不变量。
- `references/plugin-json-spec.md` 的canonical sample包含validator明确拒绝的 `hooks` 字段，必须修正；hooks目录可以存在，但不能写入该manifest字段。
- marketplace写入服从动态权限和用户请求，不在Skill里硬编码approval步骤。

### English candidate

````markdown
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
````

### 中文候选

````markdown
---
名称：plugin-creator
描述：创建、验证或更新 VSP-Codex Plugins，包括 `.codex-plugin/plugin.json`、可选组件目录、个人或仓库 marketplace，以及本地 cachebuster/reinstall 流程。
---

# Plugin 创建器

使用 bundled scripts 搭建并验证 Plugins。编辑 manifest 或 marketplace 字段时读取 `references/plugin-json-spec.md`；只有更新已安装的本地 Plugin 时才读取 `references/installing-and-updating.md`。

## 创建 Plugin

1. 判断 Plugin 属于个人范围还是仓库/团队范围。
2. 从本 Skill 目录运行 scaffold：

```bash
python3 scripts/create_basic_plugin.py <plugin-name> [options]
```

脚本把名称标准化为小写 hyphen-case，创建 `<plugin-name>/.codex-plugin/plugin.json`，个人 Plugin 默认位于 `~/plugins/<plugin-name>`。

只创建请求实际需要的组件：

```bash
python3 scripts/create_basic_plugin.py <plugin-name> \
  --with-skills --with-hooks --with-scripts --with-assets --with-mcp --with-apps
```

需要通过 marketplace 发现 Plugin 时添加 `--with-marketplace`。个人默认是 `~/.agents/plugins/marketplace.json`。只有用户明确要求仓库/团队 marketplace 时才同时使用 `--path` 与 `--marketplace-path`。

3. 用用户提供的 metadata 替换 scaffold 默认值，并删除所有 placeholders。
4. 交付前验证：

```bash
python3 scripts/validate_plugin.py <plugin-path>
```

写入操作遵循当前运行时权限策略。只有用户要求有意替换且已检查现有内容时才使用 `--force`。

## 必须保持的不变量

- 外层目录名和 manifest `name` 必须等于标准化后的 Plugin 名称。
- 保留 `.codex-plugin/plugin.json`。
- 只有对应组件真实存在时才添加 `apps` 或 `mcpServers`。
- 不得在 `plugin.json` 中添加 `hooks` 等不支持字段；自动发现的 hooks 目录与 manifest schema 分离。
- manifest 路径相对于 Plugin root，并验证所引用 assets。
- 不得保留 `[TODO: ...]` placeholders。

## Marketplace 规则

- 个人 Plugins 默认位于 `~/plugins/<name>`，marketplace 为 `~/.agents/plugins/marketplace.json`；该 marketplace 会自动发现。
- 仓库/团队 marketplace 必须显式选择，默认使用 `<repo-root>/.agents/plugins/marketplace.json`，除非用户提供另一支持路径。
- source path 保持为相对 marketplace root 的 `./plugins/<name>`。
- 默认追加条目；只有用户要求时才重排。只有有意使用 `--force` 时才替换已有条目。
- 保留已有 marketplace `name` 和 `interface.displayName`。
- 包含 `policy.installation`、`policy.authentication` 和 `category`。只有明确要求时才添加 `policy.products`。
- 非默认 marketplace 必须先配置；需要时使用 `vsp-codex plugin marketplace add <marketplace-root>`。默认个人 marketplace 不需要显式添加。

精确的支持字段与值读取 `references/plugin-json-spec.md`，不在正文重复字段表。

## 更新已安装的本地 Plugin

不得仅为使 Plugin cache 失效而手改 marketplace state。

1. 确认 marketplace entry 指向正在编辑的本地 source。
2. 运行 `python3 scripts/update_plugin_cachebuster.py <plugin-path>`。
3. 使用 `python3 scripts/read_marketplace_name.py` 读取 marketplace 名称，或传入明确 marketplace 路径。
4. 使用 `vsp-codex plugin add <plugin-name>@<marketplace-name>` 重新安装。
5. 当运行时需要新的 Plugin snapshot 时，请用户在新线程测试。

marketplace 配置有歧义时使用 `vsp-codex plugin list`。如果 source 非本地或不匹配，应停止，而不是套用本地更新流程。

## 交付

报告创建或修改的路径、验证结果、marketplace 变化，以及是否需要新线程。只有当前 surface 支持且对用户有帮助时才提供 Codex App deeplink；VSP TUI 输出不强制提供。
````

## 3. skill-creator

### Review conclusion

- 416行/21,897字符虽未超过500行上限，但大量理念、比喻、目录例子和forward-testing规则重复。
- 正文称frontmatter只允许两项，与validator和VSP loader实际支持 `metadata.short-description` 不一致。
- 默认 `$CODEX_HOME/skills` 会在VSP中指向受产品管理的home，而当前共享个人Skills来自用户目录；应解析当前可发现的user root，并保护 `.system` cache。
- subagent验证应服从S2的root routing、成本与nesting约束，而不是独立拥有并行策略。
- 必须保留trigger-first description、渐进披露、scripts/references/assets分工、init/validate流程和真实forward test。

### English candidate

````markdown
---
name: skill-creator
description: Create, update, validate, or evaluate Codex skills that package reusable workflows, references, scripts, or assets. Use when defining a new skill, improving an existing skill, fixing triggering, or testing whether a skill generalizes to realistic requests.
metadata:
  short-description: Create or improve a skill
---

# Skill Creator

Build skills as small, self-contained workflow packages for another Codex instance. Include only knowledge or procedures that are non-obvious, reusable, and worth their context cost.

## Design rules

- Put all trigger conditions in the frontmatter `description`; the body is loaded only after triggering.
- Match specificity to failure risk: use flexible guidance for judgment-heavy work and scripts for fragile or repetitive operations.
- Keep `SKILL.md` under 500 lines and materially shorter when possible.
- Avoid duplicate guidance between `SKILL.md` and references.
- Keep references one level from `SKILL.md`; give long references a table of contents and tell the agent when to read them.
- Do not add process-history or auxiliary documents such as README, changelog, or installation guides unless the skill itself needs them at runtime.

## Package structure

```text
skill-name/
├── SKILL.md
├── agents/openai.yaml        # optional UI and invocation metadata
├── scripts/                  # deterministic or repeated operations
├── references/               # detailed knowledge loaded on demand
└── assets/                   # templates or files used in outputs
```

Create only directories the skill needs.

- Use `scripts/` when reliability or repeated implementation justifies executable logic. Run representative scripts during validation.
- Use `references/` for schemas, policies, API details, variants, and long examples.
- Use `assets/` for templates, images, fonts, boilerplate, or other output inputs not meant for model context.

## Creation workflow

1. Gather realistic trigger and non-trigger examples. Confirm the intended scope and output.
2. Identify reusable scripts, references, and assets.
3. Resolve the target skill root from the user's request and the active runtime. For VSP personal skills, use the discoverable user Skills root; do not assume `$CODEX_HOME/skills`. Never edit the installed `.system` cache as a personal-skill destination.
4. For a new skill, initialize it from this skill directory:

```bash
scripts/init_skill.py <skill-name> --path <target-root> [--resources scripts,references,assets]
```

Use `--examples` only when placeholders will be replaced or removed. Supply `--interface key=value` values for `agents/openai.yaml`, or regenerate them with:

```bash
scripts/generate_openai_yaml.py <skill-dir> --interface key=value
```

5. Implement resources first, then write the smallest body that routes to them correctly.
6. Validate:

```bash
scripts/quick_validate.py <skill-dir>
```

7. Exercise the skill on realistic tasks and revise observed failures.

## Naming and frontmatter

- Use a lower-case hyphen-case folder and `name`, at most 64 characters.
- Prefer a short action-oriented name; namespace by tool only when it improves triggering.
- `name` and `description` are required. The description must state both capability and concrete trigger conditions and stay under the validator limit.
- Use optional supported frontmatter fields such as `metadata` only when the active runtime consumes them. Put UI metadata in `agents/openai.yaml` when appropriate.
- Keep the folder name equal to the skill name.

Write body instructions in imperative form. Do not add a body section that merely repeats when the skill should trigger.

## Progressive disclosure

The catalog always carries metadata; `SKILL.md` loads after selection; referenced resources load only when needed. Optimize each layer separately:

- Metadata: enough detail for correct triggering, no workflow prose.
- `SKILL.md`: core workflow, decisions, safety rules, commands, and reference routing.
- References: detailed variants, specifications, and examples.
- Scripts: deterministic behavior that can run without entering model context.

## Update workflow

For an existing skill:

1. Read `SKILL.md` completely once.
2. Inspect only referenced resources relevant to the requested change.
3. Preserve working behavior and remove contradiction or duplication.
4. Update `agents/openai.yaml` if UI metadata no longer matches.
5. Run the validator and relevant scripts.
6. Compare behavior on representative trigger, non-trigger, and execution examples.

## Forward testing

Use independent forward tests for complex or high-risk skills when the active runtime allows it. Follow the root agent's routing, cost, approval, and nesting policy.

- Give the evaluator the skill and a realistic request, not the intended answer or suspected bug.
- Use fresh context and raw outputs, diffs, logs, or traces.
- Test triggering and non-triggering as well as successful execution.
- Do not modify live systems for an evaluation without the required user authority.
- If success depends on leaked conclusions, tighten the skill or test setup before trusting the result.
````

### 中文候选

````markdown
---
名称：skill-creator
描述：创建、更新、验证或评估用于封装可复用 workflows、references、scripts 或 assets 的 Codex Skills。定义新 Skill、改进现有 Skill、修复触发或测试 Skill 能否泛化到真实请求时使用。
metadata：
  简短描述：创建或改进 Skill
---

# Skill 创建器

把 Skill 构建为供另一个 Codex 实例使用的小型、自包含 workflow package。只加入不显然、可复用且值得占用上下文的知识或流程。

## 设计规则

- 把所有触发条件放入 frontmatter `description`；正文只有触发后才加载。
- 让具体程度匹配失败风险：依赖判断的工作使用灵活指导，脆弱或重复操作使用 scripts。
- `SKILL.md` 保持在500行以内，并尽可能明显更短。
- 避免 `SKILL.md` 与 references 重复。
- references 与 `SKILL.md` 只隔一层；长 reference 应有目录，并说明何时读取。
- 除非 Skill 运行时确实需要，不要添加 README、changelog、installation guide 等过程或辅助文档。

## Package 结构

```text
skill-name/
├── SKILL.md
├── agents/openai.yaml        # 可选UI与invocation metadata
├── scripts/                  # 确定性或重复操作
├── references/               # 按需加载的详细知识
└── assets/                   # 输出使用的模板或文件
```

只创建 Skill 实际需要的目录。

- 需要可靠性或重复实现时才用 `scripts/`。验证时实际运行有代表性的 scripts。
- schema、policy、API细节、变体和长示例放入 `references/`。
- 模板、图片、字体、boilerplate 或其他不进入模型上下文的输出输入放入 `assets/`。

## 创建流程

1. 收集真实触发和不触发示例，确认范围与输出。
2. 识别可复用的 scripts、references 和 assets。
3. 根据用户请求和当前运行时解析目标 Skill root。VSP个人Skills使用可发现的user Skills root，不假设 `$CODEX_HOME/skills`。绝不把已安装 `.system` cache 当作个人Skill目的地。
4. 新 Skill 从本 Skill 目录初始化：

```bash
scripts/init_skill.py <skill-name> --path <target-root> [--resources scripts,references,assets]
```

只有会替换或删除 placeholders 时才使用 `--examples`。通过 `--interface key=value` 为 `agents/openai.yaml` 提供值，或使用下面命令重新生成：

```bash
scripts/generate_openai_yaml.py <skill-dir> --interface key=value
```

5. 先实现 resources，再编写能正确路由到它们的最小正文。
6. 验证：

```bash
scripts/quick_validate.py <skill-dir>
```

7. 在真实任务上使用 Skill，并根据观察到的失败修订。

## 命名与 frontmatter

- 文件夹和 `name` 使用小写 hyphen-case，最多64个字符。
- 优先选择简短、动作导向的名称；只有能改善触发时才按工具加 namespace。
- `name` 和 `description` 必填。description 同时说明能力与具体触发条件，并保持在validator限制内。
- 只有当前运行时会读取时才使用 `metadata` 等可选且受支持的 frontmatter 字段。适用时把UI metadata放入 `agents/openai.yaml`。
- 文件夹名与 Skill 名一致。

正文使用祈使形式。不要添加只重复触发条件的正文小节。

## 渐进披露

目录始终携带metadata；选择后加载 `SKILL.md`；只有需要时才加载被引用资源。分别优化每层：

- Metadata：足够正确触发，不包含workflow正文。
- `SKILL.md`：核心workflow、决策、安全规则、命令和reference路由。
- References：详细变体、规范和示例。
- Scripts：无需进入模型上下文即可运行的确定性行为。

## 更新流程

对于现有 Skill：

1. 完整读取 `SKILL.md` 一次。
2. 只检查与请求变化相关的 referenced resources。
3. 保留有效行为，消除矛盾和重复。
4. UI metadata 不再匹配时更新 `agents/openai.yaml`。
5. 运行 validator 和相关 scripts。
6. 用有代表性的触发、不触发和执行示例比较行为。

## Forward testing

对于复杂或高风险 Skills，在当前运行时允许时使用独立 forward tests。遵循root agent的routing、cost、approval和nesting策略。

- 给 evaluator 提供 Skill 和真实请求，不提供预期答案或怀疑的bug。
- 使用新鲜上下文与原始outputs、diffs、logs或traces。
- 同时测试触发、不触发和成功执行。
- 没有必要用户授权时，不得为evaluation修改live systems。
- 如果只有泄露结论才能成功，应先收紧Skill或测试设置再相信结果。
````

## Cross-file changes required in M2

- `openai-docs/SKILL.md`: replace the current source routing and remove automatic MCP install/restart behavior.
- `plugin-creator/SKILL.md`: adopt the concise candidate and VSP CLI naming.
- `plugin-creator/references/installing-and-updating.md`: replace `codex plugin` commands with `vsp-codex plugin` and remove duplicated rules where possible.
- `plugin-creator/references/plugin-json-spec.md`: remove the unsupported `hooks` field from the canonical `plugin.json` sample and field guide while keeping hook-directory discovery distinct.
- `skill-creator/SKILL.md`: adopt the concise candidate and correct frontmatter/path/subagent semantics.
- Corresponding `agents/openai.yaml` files: verify descriptions still match after edits; regenerate only when stale.
