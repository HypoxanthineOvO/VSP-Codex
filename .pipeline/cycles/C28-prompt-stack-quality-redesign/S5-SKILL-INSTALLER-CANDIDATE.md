---
kind: prompt-candidate
cycle: C28-prompt-stack-quality-redesign
stone: S5
surface: skill-installer
status: proposed
updated: 2026-08-10T22:24:25+08:00
---

# skill-installer Candidate

## English target

````markdown
---
name: skill-installer
description: List or install Codex Skills from the curated OpenAI catalog or a user-provided GitHub repository path, including private repositories. Use only when the user asks to discover or install Skills.
---

# Skill Installer

Use the bundled scripts rather than reimplementing catalog or installation logic. Listing is read-only apart from network/cache access. Installation changes the user's Skill directory and requires an explicit user request.

## Choose the operation

- If the user asks what can be installed, list the requested catalog. Default to `openai/skills` at `skills/.curated`; use `skills/.experimental` only when requested and label it experimental.
- If the user identifies one or more catalog Skills, install those names.
- If the user provides a GitHub repo/path or URL, install only the specified paths.
- If the user invokes this Skill without selecting an operation or item, list first. When a choice is required and `request_user_input` is available, use it after showing the relevant list.

## Commands

Run the scripts with the active Python 3 launcher for the platform:

- List: `scripts/list-skills.py [--repo owner/repo] [--path repo/path] [--ref ref] [--format text|json]`
- Install by repo/path: `scripts/install-skill-from-github.py --repo owner/repo --path path/to/skill [path/to/skill ...] [--ref ref] [--dest dir] [--method auto|download|git]`
- Install by URL: `scripts/install-skill-from-github.py --url https://github.com/owner/repo/tree/ref/path [--dest dir] [--method auto|download|git]`

The helper resolves the active VSP user-Skills destination. Do not hardcode `$HOME`, assume `$CODEX_HOME`, or target a managed `.system` directory. Use `--dest` only when the user explicitly requests another valid user-owned destination.

## Installation behavior

- Public GitHub repositories use direct download first. `auto` falls back to Git sparse checkout only for authentication or permission failures; Git tries HTTPS and then SSH.
- Private repositories use existing Git credentials or `GITHUB_TOKEN`/`GH_TOKEN`. Never print, persist, or place token values in command arguments or reports.
- Validate every selected directory contains `SKILL.md`, reject paths outside the repository, and abort if the destination already exists. Do not overwrite an installed Skill.
- Packaged System Skills are managed by VSP-Codex. Explain that they are already installed and refuse to overwrite them through this Skill.
- Follow current runtime network and filesystem permissions. Do not hardcode sandbox escalation or claim that access is available when the tool reports otherwise.
- On partial multi-Skill failure, report exactly which destinations were created and which failed. Do not roll back unrelated existing Skills.

After success, report installed names, source, and destination without secrets. Tell the user the new Skill is available on the next turn. For listing failures or unsupported non-GitHub download URLs, explain the exact limitation and stop.
````

## 中文对照

````markdown
---
name: skill-installer
description: 从OpenAI精选目录或用户提供的GitHub仓库路径列出或安装Codex Skills，包括私有仓库。只有用户要求发现或安装Skills时使用。
---

# Skill Installer

使用随包脚本，不重新实现目录或安装逻辑。列出操作除了网络/cache访问外是只读的；安装会修改用户Skill目录，必须有用户明确请求。

## 选择操作

- 用户询问可安装内容时列出对应目录；默认 `openai/skills` 的 `skills/.curated`，只有明确请求时使用并标注 `.experimental`。
- 用户指定一个或多个目录Skill时安装这些名称。
- 用户提供GitHub repo/path或URL时，只安装指定路径。
- 用户只触发本Skill但未选择操作或条目时先列出；需要选择且 `request_user_input` 可用时，在展示相关列表后使用它。

## 命令

使用当前平台有效的Python 3 launcher运行：

- 列出：`scripts/list-skills.py [--repo owner/repo] [--path repo/path] [--ref ref] [--format text|json]`
- repo/path安装：`scripts/install-skill-from-github.py --repo owner/repo --path path/to/skill [path/to/skill ...] [--ref ref] [--dest dir] [--method auto|download|git]`
- URL安装：`scripts/install-skill-from-github.py --url https://github.com/owner/repo/tree/ref/path [--dest dir] [--method auto|download|git]`

helper解析当前VSP用户Skills目标。不要硬编码 `$HOME`、假设 `$CODEX_HOME`，也不要指向受管理的 `.system` 目录。只有用户明确要求另一个有效的用户自有目标时才使用 `--dest`。

## 安装行为

- 公共GitHub仓库先直接下载；`auto`仅在认证或权限失败时回退Git sparse checkout，Git先HTTPS后SSH。
- 私有仓库使用已有Git凭据或 `GITHUB_TOKEN`/`GH_TOKEN`；绝不输出、持久化或把token值放进命令参数/报告。
- 验证每个目录包含 `SKILL.md`，拒绝仓库外路径；目标已存在时中止，不覆盖已安装Skill。
- 随包System Skills由VSP-Codex管理；说明已经安装，并拒绝通过本Skill覆盖。
- 遵循当前网络与文件权限，不硬编码sandbox escalation，也不在工具拒绝时声称已有访问能力。
- 多Skill部分失败时准确报告已创建和失败目标，不回滚无关既有Skills。

成功后报告安装名称、来源和目标但不包含秘密，并说明新Skill在下一turn可用。目录失败或非GitHub下载URL不支持时，说明准确限制并停止。
````
