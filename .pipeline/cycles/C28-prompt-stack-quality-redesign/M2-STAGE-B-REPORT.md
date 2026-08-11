---
kind: stage-report
cycle: C28-prompt-stack-quality-redesign
milestone: M2
stage: B
status: accepted
updated: 2026-08-11T01:12:10+08:00
accepted_at: 2026-08-11T01:12:10+08:00
---

# M2 Stage B Review: System Skills, Effective Catalog, and Lifecycle

## Conclusion

Stage B is ready for independent human review. The five accepted System Skill rewrites now ship with a new `vsp-provider` System Skill; the existing five bodies fell from 83,263 to 22,573 bytes, a 72.9% reduction, while retaining their deterministic scripts and routed references. The model-visible catalog now exposes one effective winner per unqualified name using `Repo > User > System > Admin`, uses root aliases under budget pressure, and caps the default metadata budget at 2,000 tokens instead of allowing 2% of a 353,400-token context to grow to 7,068 tokens.

Skill activation now has one explicit runtime contract: a selected Skill remains active for the thread, including compaction and resume, and does not require repeated mentions. Compaction preserves the latest rendered body for each active Skill path and includes that body in replacement-budget accounting.

This gate covers Stage B only. Stage C AGENTS/runtime injection, Stage E worker routing, Stage F broad regression/token comparison, release, install, restart, and publication have not started.

## Implemented Behavior

### System Skills

- Replaced `imagegen`, `openai-docs`, `plugin-creator`, `skill-creator`, and `skill-installer` with the accepted concise candidates.
- Added `vsp-provider` with `SKILL.md` and `agents/openai.yaml`; it owns VSP provider/model diagnosis and configuration without duplicating upstream OpenAI documentation.
- Preserved progressive disclosure: core workflow stays in `SKILL.md`; detailed CLI, API, schema, prompt, and install material remains in existing references and scripts.
- Corrected plugin references to `vsp-codex plugin ...` and removed the validator-rejected `hooks` field from the canonical manifest example.
- Changed Skill Installer's default destination to the canonical `$HOME/.agents/skills`, retained legacy-root visibility for listing, and reject names owned by the managed `$CODEX_HOME/skills/.system` cache.

Body-size comparison:

| Skill | Before | After |
| --- | ---: | ---: |
| `imagegen` | 24,003 bytes / 356 lines | 5,486 bytes / 96 lines |
| `openai-docs` | 22,806 bytes / 183 lines | 4,759 bytes / 67 lines |
| `plugin-creator` | 11,040 bytes / 243 lines | 4,117 bytes / 74 lines |
| `skill-creator` | 22,047 bytes / 416 lines | 5,187 bytes / 102 lines |
| `skill-installer` | 3,367 bytes / 58 lines | 3,024 bytes / 36 lines |
| new `vsp-provider` | n/a | 3,773 bytes / 70 lines |

### Effective catalog and budget

- Effective unqualified names are resolved before catalog rendering and plain `$skill-name` selection.
- Scope precedence is deterministic: Repo, then User, System, and Admin. A structured path can still select an enabled shadowed source explicitly.
- Same-scope ties use name and source path ordering for deterministic output.
- Plugin-qualified names remain distinct because namespacing occurs before effective-name selection.
- The catalog budget remains 2% for smaller contexts but is now capped at 2,000 tokens. Existing fair description truncation, minimum lines, omission warnings, and alias selection remain intact.
- Models that opt out of the long upstream Skills guide now receive a fixed five-line Skill protocol instead of no protocol.

### Thread, compaction, and resume lifecycle

- The runtime protocol states that selected Skills stay active for the same thread across compaction and resume.
- Normal turns retain injected Skill bodies through conversation history.
- Local and remote compaction paths now collect active `<skill>` fragments from history and insert one latest body per Skill path into the compacted replacement.
- Because the preserved body is written into the rollout checkpoint, resumed sessions recover it from compacted history without a repeated mention.
- Local compaction now estimates replacement occupancy after initial context and active Skill bodies are inserted, preventing an underestimated replacement budget.

## Modified Modules

- Effective selection, mention resolution, catalog rendering, budget, and protocol: `codex-rs/core-skills/src/{model,injection,render,skill_instructions}.rs` plus focused tests.
- Compact lifecycle and replacement accounting: `codex-rs/core/src/compact.rs` and `compact_tests.rs`.
- Compact/full protocol assembly: `codex-rs/core/src/context/available_skills_instructions.rs` and its new sibling test file.
- Bundled System Skills and supporting references/scripts: `codex-rs/skills/src/assets/samples/{imagegen,openai-docs,plugin-creator,skill-creator,skill-installer,vsp-provider}`.

No Rust dependency, Cargo lockfile, config schema, app-server API, or Bazel compile-data change was required. The large deletion count is the accepted mechanical replacement of five prose-heavy Skill bodies; new runtime logic remains below the repository's complex-change limit.

## Test Design and Results

| Validation | Result | Contract established |
| --- | --- | --- |
| Six `quick_validate.py` runs | 6/6 pass | All bundled Skill frontmatter and folder names are valid. |
| Python compile and isolated helper checks | pass | Installer defaults to `$HOME/.agents/skills`; listing sees canonical, legacy, and System roots; managed System names are detected. |
| Clean `TMPDIR=/dev/shm cargo test -p codex-core-skills` | 121/121 pass | Loader, effective winner, mentions, aliases, budget truncation, services, and environment-backed loading all pass together. |
| Core group `available_skills_instructions::tests` | 2/2 pass | Compact protocol appears when the model omits the full guide; full-guide models do not receive both forms. |
| Core group `compact::tests::` | 19/19 pass | Active Skill preservation, latest-body dedupe, initial-context reinjection, compact placement, and existing compaction behavior pass. |
| Required `smoke` group / `just vsp-ci` | pass, 7 probes | Current binary builds; version/help, exec/resume help, visible TUI frame, app-server initialize, and thread/list work. |
| `cargo fmt --check --package codex-core-skills --package codex-core` | pass | Modified Rust files are formatted; only the repository's stable-rustfmt warning was emitted. |
| `git diff --check` | pass | No whitespace errors. |
| Isolated real CLI Responses capture | pass | Current local binary emitted the effective, aliased, budgeted catalog and completed normally against a localhost mock. |

The first full `codex-core-skills` run used the host default temp root and reported 115 passes plus two loader failures caused by an extra discovered Repo root. The repository already records these tests as temp-root sensitive. Re-running the same entire crate with clean `TMPDIR=/dev/shm` passed all 117 unit and 4 integration tests; no product change was needed.

## Real Prompt Evidence

The final capture used the current locally built `vsp-codex`, an ephemeral session, isolated `HOME` and `CODEX_HOME`, a localhost Responses mock with WebSockets disabled, and `model_context_window = 32000` to exercise the 640-token metadata budget and alias path. It contacted no live inference endpoint.

- Raw request: 68,136 bytes; SHA-256 `feddc46a7cd2e3d31397d76af08bbf9a5f81decd68f8b92cbd3d30285b2d8bc3`.
- Skills block: 3,618 characters.
- Root aliases: `r0` for repository Skills and `r1` for bundled System Skills.
- Catalog: 20 entries and 20 unique names.
- System entries: `imagegen`, `openai-docs`, `plugin-creator`, `skill-creator`, `skill-installer`, and `vsp-provider` under `r1`.
- A temporary User `remote-tests` collision was seeded in the isolated HOME. The request contained exactly one `remote-tests`, pointing to the Repo `r0/remote-tests/SKILL.md`; the shadowed User root was absent.
- The CLI emitted the expected description-truncation warning, completed the mocked turn, and printed `ok`.
- The request contains `### Skill protocol` and the sentence: `Once selected, a skill remains active in the same thread, including across compaction and resume; it does not need to be mentioned again.`
- The obsolete `Do not carry skills across turns unless re-mentioned` rule is absent.

The raw request remains under `/tmp` and is not committed because it contains the complete prompt and tool schemas.

## Problems Encountered

- The initial direct `cargo fmt` command was run from the repository root, where no `Cargo.toml` exists. It changed nothing; targeted `rustfmt` and package-level `cargo fmt --check` then passed.
- The first Python installer import check omitted the sibling scripts directory from `sys.path`. The corrected isolated import passed; product code was unchanged.
- The first localhost SSE mock encoded newline separators incorrectly, so the client saw a disconnected stream after the request was captured. The mock was corrected, and both subsequent real CLI captures completed normally.
- A first session-level lifecycle test compared `Option<Arc<WorldState>>` directly even though `WorldState` has no `PartialEq`. The assertion was corrected to `is_none()` and the full compact unit module passed.
- Prompt inspection showed that `gpt-5.6-sol` disables the long dynamic Skills guide. A compact fixed protocol was therefore added to the false branch; otherwise the new lifecycle rule would not have reached the actual model request.
- Global `.pipeline/state.yaml` and `.pipeline/cycle.yaml` still project legacy C10 state. They are protected and were not overwritten; C28's semantic index and cycle-local Plan/Progress/Execution records remain authoritative.

## Expected User-Facing Result

- System Skills trigger with shorter, VSP-correct descriptions and load concise task workflows rather than large duplicated manuals.
- VSP provider questions select a dedicated `vsp-provider` workflow; upstream OpenAI documentation remains owned by `openai-docs`.
- Plain duplicate Skill names behave predictably by scope and appear only once in the prompt.
- Large-context models no longer spend an unbounded 2% slice on Skill metadata; descriptions shorten and paths alias before entries are omitted.
- Users mention a Skill once for a continuous thread workflow. Follow-up turns, compaction, and resume keep the selected Skill active.
- Personal installations land in the discoverable user root and cannot overwrite managed bundled Skills.

## Risks and Follow-up

- Stage B validates request construction and deterministic runtime behavior, not live-model task quality. Stage F/S6 must compare representative outcomes and tokens.
- Preserving active Skill bodies intentionally consumes compacted replacement space. The corrected accounting can shorten summaries or active tails sooner when many large Skills are active; this is preferable to silently overflowing the context window.
- Effective winner selection is by exact unqualified name. Plugin namespaces remain the supported way to make cross-plugin names independently addressable.
- Existing sessions compacted before this change cannot retroactively recover a Skill body that was already discarded; new compactions preserve it.
- Stage C must still split and dynamically load AGENTS rules. Stage E must still implement worker routing. Stage D image-provider capability remains deferred by user decision.

## Review Gate

Stage B was accepted by the user at `2026-08-11T01:12:10+08:00`. The System Skill rewrites, effective catalog, 2,000-token cap, compact protocol, and Skill lifecycle behavior are locked for this Cycle. Stage C is now permitted but has not started in this handoff.
