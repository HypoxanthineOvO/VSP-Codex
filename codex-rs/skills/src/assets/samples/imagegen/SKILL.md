---
name: imagegen
description: Generate or edit raster images such as photos, illustrations, textures, sprites, mockups, and transparent cutouts. Use when the requested output should be a bitmap asset; prefer native SVG, HTML/CSS, canvas, or existing editable source when the task is inherently code-native or must match an established vector system.
---

# Image Generation

Generate or edit images, inspect the result, and place project-bound assets in the workspace.

## Execution routes

Use the built-in `image_gen` tool by default when it is available. It uses the active model provider's configured Base URL and authentication. For VSP/VSPLab, do not switch to an official endpoint or ask for another API key when the active provider already supplies compatible image generation.

Use `scripts/image_gen.py` only when:

- the user explicitly asks for CLI, API, model, mask, file-path, or batch controls;
- the built-in tool is unavailable and compatible CLI credentials are already configured; or
- the user confirms a different model/provider fallback required by the request.

Do not switch provider, credential set, or model silently. An image request already authorizes normal use of the selected provider; it does not authorize a different billable provider.

The CLI supports `generate`, `edit`, and `generate-batch`. It uses the OpenAI SDK and respects `OPENAI_BASE_URL` and `OPENAI_API_KEY`. Never print, persist, or pass credential values as command arguments.

## Choose the operation

- Generate when the user wants a new image, including generation guided by reference images.
- Edit when the user wants to preserve parts of an existing image while changing others.
- Treat supplied images as references unless the user identifies an edit target.
- Use one request per distinct asset. Use variants only for alternate renderings of the same prompt.

For built-in edits, make local images visible with `view_image` first. Label each input as edit target, style/reference, or compositing input. Repeat edit invariants on every iteration.

## Workflow

1. Determine generate vs edit, preview vs project asset, and single vs multiple outputs.
2. Collect the prompt, exact text, input images and roles, constraints, and avoid list. Ask only when missing information blocks a useful result.
3. Preserve a detailed user prompt. For a generic prompt, add only details that materially improve composition or intended use.
4. Run the selected route.
5. Inspect every output for subject, composition, text accuracy, artifacts, and requested invariants.
6. Iterate with one targeted change at a time.
7. Save non-destructively. Do not overwrite an existing asset unless replacement was requested.
8. Move or copy every selected project asset into the workspace and update consuming references. Preview-only outputs may remain in the runtime generation directory.
9. Report final paths, final prompt(s), and whether built-in or CLI execution was used.

## Prompt shape

Use only useful fields:

```text
Use case: <purpose>
Asset type: <where it will be used>
Primary request: <user request>
Input images: <image and role>
Scene/backdrop: <environment>
Subject: <main subject>
Style/medium: <photo, illustration, 3D, etc.>
Composition/framing: <layout and camera>
Lighting/mood: <lighting and mood>
Text (verbatim): "<exact text>"
Constraints: <must preserve or include>
Avoid: <negative constraints>
```

Do not add unrequested characters, brands, slogans, palettes, or narrative elements. For text in images, quote it exactly and verify the rendered text.

Read `references/prompting.md` for detailed prompting principles and `references/sample-prompts.md` only when an asset-specific template is useful.

## Transparent outputs

`gpt-image-2` does not provide model-native `background=transparent` through the current Images API contract.

For a simple opaque subject, use the built-in tool with a flat removable chroma-key background, then run this Skill's `scripts/remove_chroma_key.py` from the resolved Skill directory. Validate alpha, corners, subject coverage, and color fringe.

For hair, fur, feathers, smoke, glass, liquids, translucent or reflective materials, soft shadows, or failed key removal, explain the limitation before changing models. Use CLI `gpt-image-1.5 --background transparent --output-format png` only when the selected provider supports it and the user confirms that model fallback.

## Built-in provider contract

The built-in Rust tool uses the active provider and sends:

- generation: `POST {base_url}/images/generations`;
- edit: `POST {base_url}/images/edits`;
- model: `gpt-image-2`;
- defaults: `quality=auto`, `size=auto`, `background=auto`;
- authentication: the active provider's resolved auth.

Provider capability controls whether the tool is exposed. Do not infer image support only from OpenAI-compatible Responses support. If generation and editing support differ, validate them separately.

## CLI references

Read fallback resources only when CLI mode is selected:

- `references/cli.md`: commands, output control, batch jobs, and model-specific flags.
- `references/image-api.md`: endpoint and parameter compatibility.
- `references/codex-network.md`: active-runtime network guidance.

Run `--dry-run` when validating a CLI payload without making an API call. Do not patch the installed managed script during an image task. Product maintainers should edit the packaged source asset and validate it through the repository workflow.
