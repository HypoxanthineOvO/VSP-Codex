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
