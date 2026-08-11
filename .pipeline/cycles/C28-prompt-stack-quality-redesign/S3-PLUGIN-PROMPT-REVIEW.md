# S3 内置 Skills / Apps / Plugins 提示审阅

## 来源边界

S3 完整盘点所有插件相关模型注入，但内容深审与改写聚焦 VSP-Codex 随包 System Skills、内置 contributor 和内置 Apps/Plugins 通用提示。Repo/User/外部 Plugin Skills 正常注入，C28 只做来源归因和 usefulness 轻筛，不改写其业务正文。外部插件其他业务提示同样只审计错误常驻、重复、角色和触发隔离。

## 真实首轮来源地图

来源为 M1 session `019feab5-dd36-77b1-a06c-b2fd37dce6f8` 的首个聚合 developer `response_item`。

| ID | 模型可见块 | 字符 | 所有者 | S3处理 |
| --- | --- | ---: | --- | --- |
| P1 | `<skills_instructions>` | 29,179 | VSP-Codex renderer + 各来源 metadata | 改公共协议；只深审System条目 |
| P2 | `<apps_instructions>` | 646 | VSP-Codex内置 | 改写 |
| P3 | `<plugins_instructions>` | 1,014 | VSP-Codex内置 | 改写 |
| X1 | `<hypo_workflow_host>` | 343 | VSP Host 对外部插件的所有权声明 | 只审计注入边界 |

P1-P3 合计30,839字符，占聚合developer #1 32,515字符的94.8%。

## P1 Skills 目录事实

| 来源 | 条目数 | 内容所有权 |
| --- | ---: | --- |
| VSP system skills | 5 | VSP/OpenAI内置 |
| 仓库 `.codex/skills` | 14 | 当前仓库 |
| 个人 `~/.codex/skills` | 22 | 用户安装 |
| 插件 cache skills | 36 | 已安装插件 |
| 合计 | 77 | 76个唯一名称 |

同名冲突为 `skill-creator` 两条。M1 原“80项”已用同一rollout纠正为77项。

## P1 构造与问题

- `core-skills/src/render.rs` 生成目录引言、路径和每项描述。
- `AvailableSkillsInstructions` 根据模型目录 flag 决定是否附带使用协议；当前 live block 没有 `How to use skills`。
- S1 候选已经从 BaseInstructions 删除完整 Skills 手册，因此 S3 必须提供唯一的短动态协议，不能继续依赖模型目录偶然开关。
- 默认 metadata budget 为 context window 的2%；353,400-token模型得到约7,068-token预算，目录实际达到29,179字符。
- 相同名字可从不同来源同时进入目录，当前没有在模型可见层消除歧义。
- 77项完整长描述常驻，让普通任务为大量不相关能力付费，也降低隐式触发的区分度。

## P1 已有跨Stone决定

- Skills协议从BaseInstructions移到动态Skills层唯一拥有。
- 技能在当前项目的一轮完整工作链内跨turn、compaction和相关后续持续；另起无关任务或功能时释放。
- 选中Skill的入口 `SKILL.md` 完整读取一次；引用资源按当前需要渐进加载。
- 用户明确点名或任务与描述清楚匹配时触发；多Skill只选择覆盖任务所需的最小集合。

## P1 待讨论

- 目录是否改为“短常驻索引 + 按需搜索/展开”，避免77项长描述全部注入。
- metadata budget 是否同时使用绝对token上限，而不是只按context window百分比扩张。
- 同名Skill采用来源优先级、命名空间强制还是禁止启动并提示冲突。
- 内置Skill正文何时需要逐项审阅，外部Skill正文何时只记录来源并跳过内容改写。

## P1-A System Skills 已确认方向

用户接受以下设计：

- Skills 动态层唯一拥有短使用协议。
- Skill 在当前项目完整工作链内跨 turn、compaction 和相关后续持续。
- 首次使用前完整读取入口 `SKILL.md`，工作链内入口只读一次；引用资源按需加载。
- 目录描述只承担触发职责，不提前复制 Skill 正文的 MCP、fallback、cachebuster、reinstall 等操作流程。
- 重复绝对路径改用 root alias；纯名称必须唯一解析。
- `imagegen / openai-docs / plugin-creator / skill-creator / skill-installer` 使用已展示的短触发描述。

`skill-creator` 与个人同名条目的 bare-name 归属等待 P1-B2 裁决。

## P1-B1 Repo Skills 归因纠正

14个仓库Skills来自当前源码树 `.codex/skills`，是面向Codex仓库维护工作的项目开发资产。它们只因当前cwd位于VSP-Codex仓库而被自动发现；release与installer没有把该目录复制进正式VSP package。

用户没有使用或见过这些Skill符合运行证据：初始rollout只常驻目录metadata，正文只有触发后才以 `<skill>` user fragment注入；审计开始前未发现这14个正文被加载。

此前提出的10项常驻候选和4项support-only设计没有获得用户接受，现已撤回。P1-B1只保留来源、14项/3,301字符和usefulness证据，不进入System Skills内容改写。

## S3 当前深审范围

- 现有5个System Skills：`imagegen / openai-docs / plugin-creator / skill-creator / skill-installer`。
- 基于真实能力缺口最多新增2个System Skills。
- VSP内置Apps/Plugins通用提示与System Skills公共协议。
- Repo/User/外部Plugin Skills仅归因和usefulness轻筛。

## System Skill: skill-installer 已接受方向

- frontmatter只保留 `name` 与短触发 `description`。
- 使用bundled scripts列出和安装；安装必须由用户明确请求。
- 用户需要从列表选择时优先 `request_user_input`。
- 服从当前动态权限，不硬编码sandbox escalation。
- 让脚本解析当前有效用户Skills目标，不硬编码home路径。
- 保留public download、Git sparse checkout、private credentials和核心CLI参数。
- 目标已存在时中止，禁止覆盖受管理System Skills，绝不输出token值。

用户授权按同规格直接审阅 `openai-docs / plugin-creator / skill-creator`。`imagegen` 需要单独讨论自定义endpoint与token设计。

## Ordinary System Skills Review Result

完整英文候选与完整中文翻译见 `S3-SYSTEM-SKILLS-CANDIDATES.md`。

| Skill | Original | Candidate | Main VSP correction |
|---|---:|---:|---|
| openai-docs | 22,804 chars | 4,758 chars | upstream/VSP/provider authority split; no automatic MCP mutation/restart |
| plugin-creator | 11,036 chars | 4,116 chars | keep verified personal paths; use `vsp-codex`; fix invalid hooks sample; optional deeplink |
| skill-creator | 21,897 chars | 5,156 chars | actual frontmatter support; runtime user root; protect `.system`; S2-governed evals |

这些缩减不删除核心脚本或references；主要删除正文与reference重复、陈旧snapshot和冲突性运行时指令。

## imagegen Accepted Direction

- Built-in Rust `image_gen`复用active provider的base URL与auth；VSPLab不需要另一条默认CLI集成。
- 以显式provider capability替代provider-name/auth-mode推断，VSPLab启用、其他custom providers默认关闭。
- 默认请求`gpt-image-2`，generation/edit路径分别追加到配置的OpenAI-compatible base URL。
- API Key优先来自provider `env_key`，不得写入Skill或日志。
- CLI保留为显式/批处理/特殊参数fallback，并继续尊重`OPENAI_BASE_URL/OPENAI_API_KEY`。
- 完整证据和验证矩阵见`S3-IMAGEGEN-REVIEW.md`。

## New System Skill Candidate

推荐新增唯一一项`vsp-provider`，负责VSP-specific provider配置与诊断。候选不复制全量配置文档，使用现有`/provider`与redacted `vsp-codex doctor`，并拥有provider/model分离、精确API root、env-key secret、catalog和capability验证流程。

第二个新增System Skill缺少真实使用证据，不建议添加。完整英中候选见`S3-VSP-PROVIDER-SKILL-CANDIDATE.md`。
