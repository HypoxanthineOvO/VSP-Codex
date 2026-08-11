---
kind: stone-result
cycle: C28-prompt-stack-quality-redesign
stone: S5
status: accepted
updated: 2026-08-10T22:37:45+08:00
source_edits: authorized_for_M2
acceptance_effect: authorizes_M2_product_source_edits_only
---

# S5 Result: Integrated Design and Implementation Authorization

Accepted by the user on 2026-08-10. This acceptance authorizes the six M2 product-source/test stages defined below and no external, publication, installation, restart, destructive, or protected-Workflow action.

## Conclusion

S1-S4 now compose into one complete target prompt stack with deterministic roles, ordering, ownership, dynamic conditions, conflict resolution, session migration, cost-aware worker routing, provider image capabilities, AGENTS path enforcement, prompt observability, staged implementation, and focused tests.

The estimated ordinary first turn moves from 77,567 to about 44,400 non-tool instruction characters (-42.8%) and from 27,536 to about 19.2K input tokens (-30.1%). Both design estimates exceed C28 targets; only S6 real rollout evidence can prove them.

## Technical approach

- Stable S1 VSP identity/work behavior remains in Responses `instructions` and first cache prefix.
- One deterministic developer policy aggregate owns permissions, mode, multi-agent, Skills/catalog, Apps, Plugins, and Workflow Host.
- Developer world state owns environment and trusted Workflow projection with stable diff/replace section IDs.
- User contextual fragments own repository/global AGENTS and triggered Skill bodies; actual user input remains last.
- Root AGENTS is 3,734 characters and routes to scoped Rust/Test, TUI, app-server/protocol, leaf, and Python rules.
- `apply_patch` defers mutation and injects missing target-path AGENTS before retry; arbitrary shell scripts are not parsed as a path oracle.
- `spawn_agent.routing_class` uses trusted built-in + user-home policy, offline validation, deterministic candidates, inherited fallback, conflict rejection, and effective-selection output/telemetry.
- Image generation uses `Disabled | Generate | GenerateAndEdit`; VSPLab begins at Generate and editing stays gated until proven.

## Complete prompt artifacts

- Accepted S1 BaseInstructions and S2 developer runtime candidates.
- Five revised packaged System Skills plus exact `skill-installer`, `imagegen`, and new `vsp-provider` candidates.
- Exact Skills protocol, Apps, Plugins, Workflow Host, Workflow projection, and environment templates.
- Exact compact root AGENTS candidate and deterministic domain-relocation source map.
- Nine prompt artifact hashes passed the manifest check; runtime metadata/environment values are data rather than unspecified prose.

The detailed role graph, matrix, conflict table, routing algorithm, provider capability, migration, observability, stage plan, tests, budget, and risks are in `S5-INTEGRATED-DESIGN.md`.

## M2 implementation stages

1. Prompt sources/assembly/debug.
2. Skill catalog/runtime alias, winner, lifecycle, and budget behavior.
3. Packaged Skill bodies/resources and `vsp-provider`.
4. AGENTS relocation, path loading, world state, and apply-patch preflight.
5. Provider image capability, endpoints/auth, schema, and UI if affected.
6. Worker-routing config, resolver, tool schema/handler, result, and telemetry.

Expected total review size is 2.8K-4.4K changed lines, dominated by mechanical prompt/AGENTS/Skill movement and tests. Each logic stage stays below about 500 lines or is split again. No single review mixes all domains.

## Test design

- `core` focused filters: prompt roles/order/cache/debug, new/resumed/Lite behavior, Skills, AGENTS/preflight, conditional host state, images, routing/forks/results.
- `tui` focused snapshots only for provider/image UI text changes.
- `build-contracts` when config schema or Bazel compile data changes.
- `hypo-external` only when a compatible external checkout is configured.
- Real prompt dump and same-task before/after rollout are mandatory for S6.

## Validation completed before authorization

- All C28 Markdown frontmatters and `.pipeline/log.yaml` parse.
- `git diff --check` passes for C28 artifacts.
- Nine candidate hashes match the S5 manifest.
- `skill-installer`, `imagegen`, and `vsp-provider` extracted English frontmatters parse with required names/descriptions.
- Root AGENTS target is 3,734 characters, below the accepted 4K ceiling.
- No old `tools_search`, OpenCode tool names, legacy state path, missing analysis path, explicit-request-only rule, recommended-plugin fragment, TODO, or TBD remains in S5 candidates.
- Product `AGENTS.md` and `codex-rs` have no tracked source diff from S5 work. No product tests were run while source was locked.

## Expected user-visible result

- Normal questions receive substantially less irrelevant code/test/workflow context.
- Relevant file changes still receive every scoped repository rule before mutation.
- VSP communication, compaction, steer, reporting, authorization, and dirty-worktree behavior remain explicit.
- User/Repo/Plugin Skills remain available while duplicate names, path cost, and loaded body size improve.
- VSPLab image generation works through the active provider without a second default endpoint/token path.
- Subagents use semantic cost/capability classes and report the actual resolved selection.
- Prompt debug can explain every visible component, role, source, size, condition, cache position, and replacement.

## Problems and risks carried into implementation

- Bundled/remote model merge must be validated through the effective new-session prompt, not only JSON edits.
- Existing sessions retain recorded BaseInstructions; migration is intentionally new-session-first.
- Apply-patch preflight cannot police arbitrary shell mutations; manual edits remain restricted to apply_patch.
- Environment developer-role migration may change provider cache boundaries.
- VSPLab editing is disabled until its real edit endpoint passes.
- Proactive delegation may increase total calls even when latency or quality improves.
- AGENTS splitting creates upstream merge structure differences; stages isolate that mechanical change.
- Inline bearer remains supported until a secure interactive credential store is designed.

## Acceptance scope

Accepting S5 authorizes M2 to modify the product source and tests only within the six stages and accepted target artifacts above. It does not authorize commit, push, PR, publication, installation, deployment, service restart, remote mutation, destructive operations, or protected Workflow file changes.
