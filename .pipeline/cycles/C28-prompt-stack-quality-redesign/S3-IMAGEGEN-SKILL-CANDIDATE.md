---
cycle: C28-prompt-stack-quality-redesign
stone: S3
status: candidate
updated: 2026-08-10T22:08:00+08:00
---

# imagegen System Skill Candidate

## English candidate

````markdown
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
````

## 中文候选

````markdown
---
名称：imagegen
描述：生成或编辑照片、插画、纹理、sprites、mockups和透明cutouts等光栅图片。请求输出应为bitmap asset时使用；任务本质上是code-native或必须匹配已有vector system时，优先使用原生SVG、HTML/CSS、canvas或现有可编辑source。
---

# 图像生成

生成或编辑图片，检查结果，并把项目使用的assets放入workspace。

## 执行路线

内置`image_gen`可用时默认使用。它使用active model provider配置的Base URL和authentication。对于VSP/VSPLab，当active provider已经提供兼容图像生成时，不得切换到官方endpoint或索取另一API key。

只有以下情况使用`scripts/image_gen.py`：

- 用户明确要求CLI、API、model、mask、file-path或batch controls；
- 内置工具不可用，且兼容CLI credentials已经配置；
- 请求需要不同model/provider fallback，并且用户已确认。

不得静默切换provider、credential set或model。图像请求已经授权正常使用selected provider，但没有授权使用另一付费provider。

CLI支持`generate`、`edit`和`generate-batch`。它使用OpenAI SDK，并尊重`OPENAI_BASE_URL`和`OPENAI_API_KEY`。绝不输出、持久化credential值，也不把它作为命令参数传入。

## 选择操作

- 用户需要新图，包括由reference images指导的新图时使用generate。
- 用户希望保留现有图的一部分并修改其他部分时使用edit。
- 除非用户明确指出edit target，否则把提供的图片视为references。
- 每个不同asset使用独立请求；variants只用于同一prompt的不同渲染。

内置edit处理本地图片时，先用`view_image`让图片进入会话可见上下文。把每个输入标为edit target、style/reference或compositing input。每轮迭代都重复edit invariants。

## Workflow

1. 判断generate/edit、preview/project asset、single/multiple outputs。
2. 收集prompt、精确文字、input images及其角色、constraints和avoid list。只有信息缺失会阻止有效结果时才提问。
3. 保留详细用户prompt。对于generic prompt，只添加真正改善构图或用途的细节。
4. 运行所选路线。
5. 检查每个output的subject、composition、text accuracy、artifacts和requested invariants。
6. 每次只做一个有针对性的变化并重新检查。
7. 默认非破坏性保存。除非用户要求替换，否则不覆盖已有asset。
8. 把所有选中的project assets移动或复制到workspace，并更新消费者引用。preview-only outputs可以留在runtime generation目录。
9. 报告最终paths、最终prompt(s)以及使用built-in还是CLI。

## Prompt结构

只使用有帮助的字段：

```text
Use case: <用途>
Asset type: <使用位置>
Primary request: <用户请求>
Input images: <图片及角色>
Scene/backdrop: <环境>
Subject: <主体>
Style/medium: <照片、插画、3D等>
Composition/framing: <布局与镜头>
Lighting/mood: <光线与氛围>
Text (verbatim): "<精确文字>"
Constraints: <必须保留或包含>
Avoid: <负面约束>
```

不得添加未请求的角色、品牌、slogans、palettes或narrative elements。图片包含文字时精确引用，并验证最终文字。

详细prompting原则读取`references/prompting.md`；只有asset-specific template有用时才读取`references/sample-prompts.md`。

## 透明输出

当前Images API contract下，`gpt-image-2`不提供model-native `background=transparent`。

对于简单不透明主体，使用内置工具生成平坦、可移除的chroma-key背景，然后从已解析的Skill目录运行`scripts/remove_chroma_key.py`。验证alpha、corners、subject coverage和color fringe。

对于hair、fur、feathers、smoke、glass、liquids、translucent或reflective materials、soft shadows，或key removal失败，切换model前说明限制。只有selected provider支持且用户确认model fallback时，才使用CLI `gpt-image-1.5 --background transparent --output-format png`。

## Built-in provider contract

内置Rust工具使用active provider并发送：

- generation：`POST {base_url}/images/generations`；
- edit：`POST {base_url}/images/edits`；
- model：`gpt-image-2`；
- defaults：`quality=auto`、`size=auto`、`background=auto`；
- authentication：active provider解析出的auth。

Provider capability决定工具是否暴露。不能仅根据OpenAI-compatible Responses支持推断image支持。generation和editing支持不同时应分别验证。

## CLI references

只有选择CLI mode时才读取fallback resources：

- `references/cli.md`：commands、output control、batch jobs和model-specific flags。
- `references/image-api.md`：endpoint与parameter兼容性。
- `references/codex-network.md`：当前运行时network指导。

无需API调用地验证CLI payload时使用`--dry-run`。图像任务中不得patch已安装的managed script；产品维护者应修改packaged source asset，并通过仓库workflow验证。
````

## Size

- Original: 23,963 chars / 356 lines.
- English candidate: target approximately 6-8K chars and under 130 lines.
- Detailed model matrices, commands, and examples remain in references.
