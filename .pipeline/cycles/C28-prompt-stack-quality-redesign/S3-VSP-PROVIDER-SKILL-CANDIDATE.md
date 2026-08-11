---
cycle: C28-prompt-stack-quality-redesign
stone: S3
status: candidate
updated: 2026-08-10T22:00:47+08:00
---

# New System Skill Candidate: vsp-provider

## Why this is a real gap

- Existing `openai-docs` owns upstream/public OpenAI facts and must not claim authority over VSP fork behavior.
- VSP already has `/provider`, provider config, addon catalogs and redacted `vsp-codex doctor`, but no Skill combines them into a safe task workflow.
- C28 found repeated VSP-only failure modes: exact Base URL root, provider/model separation, inline vs env credential, capability gating, and official-service disablement.
- No Repo/User/Plugin/System Skill currently covers VSP provider configuration or diagnosis.
- A second new System Skill has no comparable evidence and is not recommended.

## English candidate

````markdown
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
````

## 中文候选

````markdown
---
名称：vsp-provider
描述：配置或诊断 VSP-Codex 和 OpenAI-compatible model providers，包括 VSPLab Base URL、API-key环境变量、provider/model选择、model catalogs、endpoint兼容性，以及图像生成等provider-backed能力。用于VSP专有provider设置或故障，不用于一般OpenAI API文档。
metadata：
  简短描述：配置或诊断VSP providers
---

# VSP Provider

以VSP运行时状态、配置和诊断为权威。OpenAI官方文档只用于建立upstream API contract。

## 分离关注点

- Provider选择Base URL、认证、协议、headers、retry policy和capabilities。
- Model选择该provider暴露的模型。
- `/provider`切换provider；`/model`切换model。
- VSP配置与认证位于当前VSP home。不得把官方Codex auth文件当作替代。

## 先诊断

对于提问或诊断，不修改配置。先运行有边界且已脱敏的报告：

```bash
vsp-codex doctor --json
```

需要紧凑的人类可读视图时使用`vsp-codex doctor --summary`。只检查相关provider、credential存在性、model catalog、network和capability结果。绝不输出secret值。

判断失败属于：

1. provider选择或配置缺失；
2. Base URL格式错误或API root错误；
3. credential环境变量缺失；
4. Responses API不兼容；
5. 所选provider catalog不存在该model；
6. provider未声明图像生成等能力；
7. network、TLS、proxy或server失败。

## 安全配置

只有用户明确要求时才修改provider配置。

- 普通VSPLab设置优先使用交互式`/provider`流程。
- 持久provider credential优先使用`env_key`。配置文件只保存环境变量名，secret不进入repo文件、prompts、logs、命令参数或生成artifact。
- 存在environment或secret-store路径时，不要求用户把token粘贴进chat。
- 支持的OpenAI-compatible provider使用`wire_api = "responses"`。
- Base URL配置为精确API root。VSP会追加endpoint path，不会自动添加`/v1`。
- endpoint没有真实支持某项optional capability时不得启用它。
- 保留无关providers、models、catalogs和project settings。

需要用户选择provider或credential方式时，若`request_user_input`可用则优先使用。

## 验证

获得授权并修改后：

1. 通过正常VSP路径重新加载配置；只有当前surface无法重新加载时才要求重启。
2. 运行`vsp-codex doctor --json`并确认报告仍然脱敏。
3. 分别确认selected provider和model。
4. 只验证用户workflow需要的endpoints。
5. 对图像生成，确认provider声明image支持，并验证generation使用provider auth请求`{base_url}/images/generations`。承诺editing前单独验证。

报告有效provider ID、masked endpoint、credential source名称、selected model、declared capabilities、已执行检查和精确剩余失败。绝不报告credential值。

## 边界

- 不得用upstream `openai-docs`证明当前VSP行为。
- 不得切换到VSP有意禁用的官方provider。
- 不得在无关model change中迁移provider credential、重写catalog或扩大capabilities。
- 不得仅因provider兼容OpenAI Responses API就声称它支持其他endpoint。
````

## Packaging

- Add one System Skill directory: `codex-rs/skills/src/assets/samples/vsp-provider/`.
- Initial version needs only `SKILL.md` and matching `agents/openai.yaml`; no script or reference is justified yet because `vsp-codex doctor` already owns deterministic diagnostics.
- Add it to embedded System Skills validation/snapshots through the existing asset build path.
