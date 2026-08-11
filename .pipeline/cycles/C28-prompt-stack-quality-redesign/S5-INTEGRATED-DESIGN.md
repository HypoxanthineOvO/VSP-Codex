---
kind: integration-design
cycle: C28-prompt-stack-quality-redesign
stone: S5
status: accepted
updated: 2026-08-10T22:37:45+08:00
source_edits: authorized_for_M2
---

# S5 Integrated Prompt-Stack Design

## One target stack

```text
Responses instructions
  S1 VSP-Codex BaseInstructions (stable client identity and work contract)
        |
        v
Developer policy message (one deterministic initial aggregate)
  permissions -> collaboration mode -> multi-agent -> Skills/catalog
  -> Apps -> Plugins -> Workflow host boundary
        |
        v
Developer world state (stable section IDs, diff-only replacement)
  environment -> workflow focus projection -> other enabled host state
        |
        v
User contextual instructions
  global/project AGENTS in scope -> triggered Skill bodies
        |
        v
Actual user input
```

Stable, high-reuse content stays first. Dynamic facts and triggered bodies appear after stable policy. VSP-owned initial developer contributors are aggregated once; later world-state changes use section-aware replacement instead of repeating the whole policy.

## Complete target prompt manifest

The following accepted/candidate artifacts collectively contain every VSP-owned target string. Runtime metadata rows and environment values are data, not hidden prose.

| Surface | Authoritative candidate | SHA-256 / extraction |
|---|---|---|
| BaseInstructions | `S1-BASE-INSTRUCTIONS-CANDIDATE.md` | `33202ab56daa59f39644a36f82935532b910a78f313fbc11147b5e450f39e5b6` full file |
| Permissions/Default/Multi-Agent | `S2-DEVELOPER-RUNTIME-CANDIDATE.md` | `da0ae64ade16248f605adf9e0c48848a828c4dcd97ddac8a23b30eeaf3493b44` full file |
| openai-docs/plugin-creator/skill-creator | `S3-SYSTEM-SKILLS-CANDIDATES.md` | English candidate blocks; file hash `fb8177c9695780690701da1a501013763fb3db56dc240215b77146cbca665b77` |
| skill-installer | `S5-SKILL-INSTALLER-CANDIDATE.md` | English target block; file hash `e9d8d70e188ef32d7722ef47ad72c31276692f77673f99ec630bae225a1b3eb3` |
| imagegen | `S3-IMAGEGEN-SKILL-CANDIDATE.md` | English candidate block; file hash `85feaf2d7e88054e166193a9041351121e2deb0c1ffc1fc2598661f8b7bb1197` |
| vsp-provider | `S3-VSP-PROVIDER-SKILL-CANDIDATE.md` | English candidate block; file hash `bab33f36799702a5f35fca25d29de941c2cb98338db3794d3a44f4d9377165d3` |
| Apps/Plugins | `S3-APPS-PLUGINS-CANDIDATES.md` | English candidate blocks; file hash `4ca6be82aa6070fc98d5ccff086bd004a731cf758bae231654ea9c17740a536f` |
| Skills protocol/Workflow/environment | `S5-DYNAMIC-PROMPTS-CANDIDATE.md` | exact fenced blocks; file hash `cb7d9008eb9e56c61bdede5a21fb8660f47a47e967d9f932936299d32caba400` |
| Root AGENTS | `S5-ROOT-AGENTS-CANDIDATE.md` | English target block, 3,734 chars; file hash `0059c888fd8beef2a82fc68213561f4bc853c6631886f62dd445a0453c24bb79` |
| Scoped domain AGENTS | current matching source sections | mechanical relocation with stale Workflow/OpenCode sections excluded |

## Injection matrix

| Component | Role | Condition | Initial order | Update/compaction |
|---|---|---|---:|---|
| S1 BaseInstructions | instructions; Responses Lite maps equivalently to developer | every supported new VSP session unless explicit config override | 1 | recorded in session meta; old resumes keep recorded version |
| Permissions | developer policy | effective permission profile | 2 | replace when effective permissions change |
| Collaboration mode | developer policy | active mode | 3 | replace when later developer mode changes |
| Multi-Agent | developer policy | collaboration tools visible | 4 | remove when tools unavailable; state updates separate |
| Skills protocol/catalog | developer policy | at least one allowed Skill | 5 | deterministic sort/alias/winner; replace on catalog change |
| Apps/Plugins | developer policy | capability enabled and accessible | 6 | section-aware replace/remove |
| Workflow host | developer policy | installed capability + semantic workspace | 7 | remove when capability/workspace disappears |
| Environment | developer world state | config enabled | 8 | replace only changed snapshot |
| Workflow projection | developer world state | trusted projection exists or explicit unbound state | 9 | replace by stable section ID |
| AGENTS | user contextual | global/project rules available for active scope | 10 | ordered scope replacement; target paths may add rules before mutation |
| Triggered Skill body | user contextual | selected Skill | after selection | once per related work chain; restore active set after compaction |
| Actual user input | user | user turn | last | preserved as conversation history |

## Conflict resolution

| Conflict | Resolution |
|---|---|
| Base general rule vs active collaboration/permission mode | later developer mode/profile describes the effective runtime |
| Runtime capability vs user scope | capability never expands user authorization |
| New steer vs active request | additive by default; explicit stop/pause/replace cancels |
| Resume prompt version vs new package prompt | session meta wins for existing session; new sessions use the new package candidate |
| Root vs nested AGENTS | deeper file wins only in its directory tree; direct prompt instructions remain higher authority |
| AGENTS vs Skill procedure | AGENTS owns repository constraints; Skill owns task method; unresolved material conflict is surfaced before mutation |
| Duplicate unqualified Skill | one effective winner by `Repo > User > System > Admin`; structured locator reaches a shadowed source |
| Plugin Skill name | namespace required as `plugin:skill` |
| Workflow repository adapter vs installed plugin | installed plugin owns semantics; VSP Host only projects trusted state |
| Old vs new world state | stable section ID replaces the old section; no duplicate current snapshots |
| Routing class vs explicit model/reasoning | same call is rejected; caller must choose one selection method |
| Routing class vs role-pinned model | reject when the role pins provider/model/reasoning; otherwise apply role instructions after routing selection |
| Full-history fork vs any model change | reject; full history inherits parent model configuration |

## AGENTS implementation decision

M2 uses the accepted compact root and scoped files. To avoid a false promise, path coverage is enforced for `apply_patch`:

1. Parse target paths using the existing patch parser.
2. Resolve missing AGENTS sources from root through each target with existing filesystem/discovery abstractions.
3. If new instructions are required, do not apply the patch. Update the AGENTS world-state section, return a retry result, and let the model reconsider the edit with the rules visible.
4. On retry, apply only when every target has current coverage.

Manual edits remain restricted to `apply_patch`. Formatting and mechanical rewrite commands may run only after applicable target instructions have been loaded. M2 does not attempt to infer mutations from arbitrary shell strings.

Prompt debug reports source path, scope root, content size, truncation, and whether a source was initial or target-triggered. More-specific instructions receive budget before broader repeated prose so the 32 KiB aggregate limit cannot silently starve the leaf rule.

Scoped content relocation is deterministic from current root AGENTS hash `d70670fad7b08efc46e59f87002e96161bb0aae4e57879c2b0a8d25e6ba54c01`:

- `codex-rs/AGENTS.md` receives current Rust-specific `Runtime contract` rules, `Change size guidance`, the complete `Tests` section, and the Python 3 rule for Python embedded under `codex-rs`.
- `codex-rs/tui/AGENTS.md` receives the complete current `TUI style conventions` and `TUI code conventions` sections. The existing bottom-pane file remains unchanged.
- `codex-rs/app-server/AGENTS.md` and `codex-rs/app-server-protocol/AGENTS.md` receive the complete current `App-server API Development Best Practices` section from one synchronized source fragment.
- `scripts/AGENTS.md`, `sdk/python/AGENTS.md`, `sdk/python-runtime/AGENTS.md`, `.github/scripts/AGENTS.md`, and `tools/argument-comment-lint/AGENTS.md` receive the current Python 3/minimum-runtime rule where applicable.
- Rules already restated by the compact root are not duplicated in child files. The OpenCode/Hypo lifecycle blocks, platform-specific tool names, stale state writes, missing analysis path, and empty derived Active Rules/Habits entries are removed rather than relocated.

## Worker routing design

### Public semantic interface

`spawn_agent` adds:

```text
routing_class: mechanical | long_context | standard | explore | critical | escalation
```

The default remains `standard` when the caller delegates without selecting a class. The tool may expose explicit `model` and `reasoning_effort` only when the effective trusted policy allows overrides. A call cannot combine explicit overrides with `routing_class`.

### Trusted policy

- Load built-in VSP defaults, then overlay `$VSP_CODEX_HOME/worker-routing.toml`.
- Do not load worker routing from repository/project config; an untrusted checkout must not select credentials, providers, or expensive models.
- Preserve the current repeated candidate format and add `[[worker_routing.long_context]]`.
- Candidate fields: stable `id`, `model_provider`, `model`, `reasoning_effort`, optional `service_tier`, and optional `min_context_window`.
- Validate with the offline Provider/Model Catalog. Spawn does not perform a remote refresh.

Resolution tries candidates in declared order. Unsupported provider/model/reasoning/tier/context candidates are skipped with a recorded reason. If no candidate is usable, inherit the parent provider/model/reasoning and report `fallback: inherited`; never silently jump to a stronger routing class. `escalation` is requested only after a cheaper class has proven insufficient.

Recommended VSP ordering starts mechanical and long-context Luna candidates at the lowest reliable effort before their stronger fallback; critical and escalation use Sol. S5 accepts the semantic resolver, not a promise that one hardcoded model name will remain forever.

### Spawn precedence and result

1. Parent effective config and runtime permissions.
2. Semantic routing candidate or explicit override, never both.
3. Compatible role instructions; role-pinned model settings conflict rather than silently winning.
4. Service-tier validation and runtime permission/cwd refresh.
5. Spawn with full-history inheritance or latest-three-turn default for changed model configuration.

Successful output and telemetry include `requested_class`, `policy_version`, `candidate_id`, `effective_provider`, `effective_model`, `effective_reasoning`, `effective_service_tier`, `fallback`, and skipped-candidate reason codes. Secrets and raw auth configuration never appear.

## Image-generation capability

Use a self-documenting provider capability enum rather than a boolean:

```text
ImageGenerationCapability = Disabled | Generate | GenerateAndEdit
```

Custom providers default to `Disabled`. VSPLab starts at `Generate`; it moves to `GenerateAndEdit` only after the real edit endpoint passes. Tool visibility requires at least `Generate`; edit input requires `GenerateAndEdit`, otherwise the Skill explains or uses its explicit CLI fallback.

The Rust extension uses the active provider's exact configured API root, auth source, and `gpt-image-2`. It appends `/images/generations` or `/images/edits`, never inserts `/v1`, and never logs credentials. Inline bearer remains supported until an interactive secure credential store is separately accepted.

## Prompt source and session migration

- Update both bundled `gpt-5.6-sol.base_instructions` and `model_messages.instructions_template`; they are currently identical source strings.
- Preserve explicit `config.base_instructions` as highest priority.
- Preserve session-meta BaseInstructions for resume/fork continuity. Do not silently rewrite old rollouts.
- New sessions without an explicit override use the new bundled VSP candidate.
- Responses and Responses Lite must render equivalent semantic roles; M3 compares their actual request bodies.

## Prompt observability

Extend prompt debug output from raw items to a structured report with:

- API role and ordinal;
- component/section ID and owner;
- source locator and injection condition;
- characters and approximate tokens;
- stable/dynamic classification;
- cache-prefix membership;
- replacement/removal status;
- Skill counts, aliases, duplicate winner, truncation;
- worker-routing requested/effective selection for spawned sessions.

Raw full prompt text remains available locally, with credential-bearing config always redacted.

## M2 reviewable stages

| Stage | Main modules | Scope / size control |
|---|---|---|
| A. Prompt sources and assembly | models-manager catalog, context contributors, session assembly, prompt debug | S1/S2/common prompts, deterministic aggregate, new-session migration; <500 logic lines |
| B. Skill catalog/runtime | core-skills renderer/injection/tests | aliases, effective winner, lifecycle state, observability; <500 logic lines |
| C. Packaged Skills | skills assets and references | five rewrites + `vsp-provider`; mostly mechanical content, isolated from Rust logic |
| D. AGENTS scoping | root/domain AGENTS, agents manager/world state, apply-patch preflight | mechanical relocation first; preflight logic/tests <500 lines |
| E. Provider images | provider config/schema/UI, image extension, tool spec | capability enum, VSPLab Generate, endpoints/auth; <500 logic lines |
| F. Worker routing | config/agent/tools/telemetry | resolver/config first, schema/handler/result second if diff approaches 500 logic lines |

No stage commits, pushes, installs, publishes, deploys, or restarts without separate user authorization. M2 may execute these stages after S5 acceptance because they are the explicitly reviewed product-source scope.

Expected review size is approximately 2.8K-4.4K changed lines across the full M2, dominated by mechanical prompt/Skill/AGENTS relocation and focused tests. Estimated stage ranges are: A 450-700, B 350-550, C 500-800, D 900-1,300 including mechanical moves, E 300-500, and F 500-750. Any non-mechanical stage that approaches 500 logic lines is split before implementation; no single review should mix all six domains.

## M3 validation matrix

| Behavior contract | Smallest group / evidence |
|---|---|
| Prompt roles/order/source/count/cache | `core` filtered prompt-debug/context/world-state tests + real prompt dump |
| New vs resumed BaseInstructions and Lite parity | `core` filtered session/client/compact-resume tests |
| Skill alias/winner/lifecycle/budget | `core` filtered Skills tests and core-skills package tests |
| AGENTS path coverage/preflight/remote/multi-root | `core` filtered agents/apply-patch/remote-env tests |
| Apps/Plugins/Workflow conditional sections | `core` filtered context tests; `hypo-external` only when compatible checkout is configured |
| Image capability/provider endpoints/auth | `core` filtered image/provider tests; `tui` snapshot filter if provider UI changes |
| Worker routing/resolver/fork/output/telemetry | `core` filtered multi-agent/config tests |
| Config/BUILD/schema drift | `build-contracts` when config schema or Bazel data changes |

Run `just write-config-schema` for provider/routing config changes and `just bazel-lock-update` only if dependencies change. UI text changes require reviewed `insta` snapshots.

## Quantitative estimate

| Layer | M1 chars | S5 estimate |
|---|---:|---:|
| BaseInstructions | 16,270 | 11,696 |
| Developer policy/world context | 34,825 | about 28,500 |
| AGENTS + environment user context | 26,472 | about 4,200 AGENTS; environment moves to developer and remains about 400 |
| Total non-tool instructions | 77,567 | about 44,400 |

Estimated non-tool character reduction is about 42.8%, above the 40% target. Holding tool schemas approximately constant, the M1 27,536-token first turn is estimated near 19.2K tokens, about 30.1% lower and above the 25% target. The added routing field is small but included only in real S6 measurement.

These are design estimates, not acceptance evidence. S6 must capture the same representative task, tool configuration, provider/model, cold/warm cache state, API roles, raw characters, input tokens, and cached tokens before declaring the targets met.

## Remaining risks

- Remote model-catalog refresh may change the selected bundled entry; M3 must prove the effective new-session prompt, not only edit JSON.
- AGENTS preflight covers deliberate patch edits but cannot safely classify arbitrary shell scripts. The prompt keeps manual edits on `apply_patch`.
- Proactive delegation can reduce latency or improve coverage while increasing total calls. S6 measures cost and quality, not just routing correctness.
- VSPLab edit support remains unproven; it is gated separately from generation.
- User/plugin Skill metadata remains the largest developer component by accepted design.
- Moving environment to developer role may affect cache boundaries and requires a real rollout comparison.
