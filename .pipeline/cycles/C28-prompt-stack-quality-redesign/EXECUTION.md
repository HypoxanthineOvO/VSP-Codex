---
kind: execution
cycle: C28-prompt-stack-quality-redesign
status: active
updated: 2026-08-11T18:52:23+08:00
---

# C28 Execution

## M1 Checkpoint

- 用户确认创建一个讨论优先的 C28；初始确认只授权 Workflow 记录、只读基线和进入 S1。
- 当前 session 使用 `gpt-5.6-sol`、`pragmatic`、Default collaboration mode、explicit-request-only multi-agent mode。
- 真实 rollout 证明首轮请求为 27,536 input tokens，其中 3,840 cached input tokens。
- 用户消息前的客户端可见指令为 77,567 字符，不含工具 schema；基础提示为 16,270 字符。
- 当前 prompt 由 `BaseInstructions`、多个 developer 消息、AGENTS/environment user context 和工具 schema 共同组成。
- 没有修改产品源码，没有运行构建或测试。

## S1 Checkpoint

- 用户拒绝脱离完整来源上下文的单句裁决方式；原 B1 请求已撤回，未形成决定。
- S1 先把当前 live `BaseInstructions` 作为一个完整 Source 解析，再按 BI-00 至 BI-12 的原始顺序逐段推进。
- Source 1 的请求位置、解析链、目录、完整中文内容、比例、内部重复、跨来源关系和稳定性问题已经完成。
- 当前项为 BI-00“身份与使命”，后续讨论必须同时显示目录位置、前后依赖和累计候选稿。
- 用户进一步明确目录和统计仍不能替代具体措辞。撤回目录粒度确认请求，Source 1 改为 A/B/C 三个连续大文本块。
- 每块必须在对话中直接展示完整英文原文、完整中文翻译、整体与具体措辞点评以及大段候选改写。
- 当前为 Block A：原始第 1-22 行，覆盖身份、人格、写作风格和技术沟通。
- 用户已确认 Block A 的品牌、语言、术语、Markdown 和 CommonMark 要求；候选稿 v1 已写入 S1 artifact。
- 用户确认“报告文件不能只报路径”的跨 Block 要求；内容锁定，待 Block B 确定唯一权威落点。
- 用户确认 Block A 候选稿和删除主观人格表演的方向；Block A 完成。
- 当前进入 Block B，范围为 live `BaseInstructions` 第 23-75 行。
- 用户确认 steer 默认追加、明确否定才替换；冲突必须询问，不得静默丢弃旧任务。
- 用户确认 compaction 按最新执行记录恢复，历史最后一条用户消息不得自动复活为命令。
- 用户要求过程与结果分开：过程更新详细说明检查、发现、判断依据、取舍和下一步；final 重新汇总全部重要内容。
- 工程和实验 final 的详细内容合同已写入 Block B 候选稿 v1；孤立的计划夸赞禁句删除。
- Compaction 段落改用正向算法：从最新时间线恢复当前目标，用后续行动和决定解释早期消息。
- 新增探索性任务合同：区分事实、推断和开放问题，比较候选解释与取舍，不制造证据不支持的确定结论，并给出降低不确定性的下一步。
- 用户确认 Block B 当前版本；Block B 完成。
- 当前进入 Block C，范围为 live `BaseInstructions` 第 76-148 行，覆盖执行、编辑、自主性、授权和 Skills。
- Block C 仓库调查确认正式 package 已强制携带 rg 并有 PATH prepend；缺失应区分正式安装缺陷与非 package fallback。
- PowerShell 的 shell identity 与专用执行路径已经存在，具体转义规则适合按环境动态注入。
- 当前 Skills 跨 turn 失效规则不适合多轮任务；详细 Skills 协议与 developer 动态层重复，建议从 BaseInstructions 删除。
- 同 cwd 会话查询和 recent work 数据已有底层能力，但缺少模型专用的结构化改动归属查询。
- 用户确认 rg、shell、多会话和 Skills 迁移方向，并把技能持续期修正为项目内完整工作链，不默认 Workflow。
- Block C 候选稿 v1 已形成；BaseInstructions 的完整 Using skills 章节删除，动态 Skills 来源留待 S3 统一定义。

## S1 Result

- Block A、B、C 合并为 `S1-BASE-INSTRUCTIONS-CANDIDATE.md`，可直接用于后续 override 或实现验证。
- Candidate 为 11,696 字符、99 行，相对 16,270 字符、148 行 live source 减少 28.1%。
- `Using skills`、旧人格表演和旧 compaction/steer 规则均不存在。
- 品牌、中文、Markdown/CommonMark、详细过程/final、多类型报告、rg/shell、多会话、安全与授权合同存在。
- Markdown spacing、Workflow YAML/front matter 和 diff hygiene 通过。
- S1 进入 waiting-review；接受不授权产品源码修改。

## S1 Acceptance

- 用户明确接受 S1 BaseInstructions 候选稿和结果报告。
- 接受锁定 Source 1 的身份、语言、steer、compaction、过程/final、报告、工具、安全、授权和 Skills 迁移决定。
- 验证证据保持有效：11,696 字符、99 行、28.1% 降幅；旧人格和 Skills 章节移除；spacing/YAML/diff checks 通过。
- 接受不授权产品源码、配置、远端服务、发布或安装修改。
- C28 继续 S2，解析通用 developer 运行规则的完整来源和注入边界。

## S2 Start

- 从当前 live rollout 提取 developer 运行规则，按 D1 Permissions/Default、D2 root/shared usage、D3 multi-agent mode、D4 Cycle state 分组。
- D1 为聚合 developer 消息开头的两个 contributor，共 1,330 字符；完整英文、中文和点评已写入 S2 review。
- Developer 聚合消息中的 Skills、Workflow Host、Apps 和 Plugins 明确留到 S3。
- 检索含反引号的源码字面量时，zsh 发生一次无副作用的命令替换并报告 `/root` permission denied；没有文件或进程状态变化，证明 shell-specific escaping 合同有实际必要性。
- S1 全部裁决完成前保持 `in_progress`，不提前生成最终基础提示定稿。
- S5 接受前，M2 不得开始。

## S2 D1a Decision

- 用户确认 D1 不再把 Permissions 与 Default collaboration mode 作为一个裁决单元，后续分别记为 D1a 与 D1b。
- VSP 交互入口在没有显式权限配置时默认使用 `danger-full-access` 与 `approval_policy=never`；默认 bypass 场景不需要解释通用沙箱模型。
- D1a 默认候选只保留三项运行事实：文件系统和网络不受限、命令无需沙箱审批且不得传 `sandbox_permissions`、运行能力不扩大用户请求授予的范围或授权。
- 权限 renderer 仍需保留动态分支；用户显式配置 `read-only`、`workspace-write` 或其他 approval policy 时，必须反映实际生效权限。
- 产品源码仍锁定；当前进入 D1b Default collaboration mode 精简讨论。

## S2 D1b Decision

- 用户接受 D1b 推荐短版：只保留当前 Default 状态、developer mode 指令的覆盖权威，以及对通用执行与授权合同的引用。
- 删除已知模式枚举、重复的提问哲学和禁止文本多选题的绝对规则。
- 用户补充确认 VSP 已增强的 `request_user_input` 应受到鼓励：当确实需要用户输入时，只要工具可用就优先使用；工具不可用时用清晰文本提问。
- “何时需要提问”继续由 S1 的统一执行与授权合同决定，避免 mode block 再建立一套标准。
- D1a 与 D1b 均已完成措辞裁决；产品源码继续锁定，S2 进入 D2。

## S2 D2 Scope Expansion

- 用户确认 D2 不能只做提示压缩；C28 必须让 root 能根据子任务能力需求和成本配置不同子智能体，避免所有 worker 默认使用强模型。
- 用户给出的必要分类维度包括：机械但繁复的任务使用简单/低成本智能体；上下文很长但推理较机械的任务使用更适合长上下文吞吐的模型；高难度任务再使用更强模型。
- 只读核对发现底层 V1/V2 `spawn_agent` 已支持 `agent_type`、`model`、`reasoning_effort` 和 `service_tier`，但 Multi-Agent V2 默认 `hide_spawn_agent_metadata=true`，当前模型可见 schema 因此只暴露 `task_name`、`message` 和 `fork_turns`。
- 本机已有 `worker-routing.toml`，定义 `mechanical / standard / explore / critical / escalation` 候选，但当前仓库没有把该配置接入 `spawn_agent` 的语义路由参数；`long_context` 也不是现有独立档位。
- 推荐方向是给 `spawn_agent` 提供语义 routing class，由可信运行时配置解析 provider/model/reasoning，而不是要求 root 在提示中硬编码具体模型名。
- 先前“首次 spawn 后再注入 D2”的建议尚未接受：模型选择规则必须在首次 spawn 前可见。后续需裁决工具可见时的短 preflight 提示与团队激活后的 runtime 状态是否拆分。
- 该扩展进入 S2/S5 设计与 M2 实现范围；S5 接受前仍不修改产品源码。

## S2 D2-D3 Proactive Routing Decision

- 用户选择“语义 routing class + 受控显式覆盖”：正常路径由 root 选择稳定语义档位，运行时解析具体 provider/model/reasoning；只有用户明确指定或语义档位无法表达需求时才使用显式 model/reasoning override。
- 用户明确拒绝当前 D3 `explicit-request-only` 边界。目标行为改为 proactive：root 可在重复机械工作、多路独立工作、并行调研以及其他确有收益的场景自主创建子智能体。
- root 必须始终拥有项目进度和方向：负责总体计划、关键路径、任务拆分、路由成本、状态监督、冲突处理、结果整合、最终验证和用户沟通，不能把这些职责委派出去。
- D2 与 D3 应合并为一个紧凑的 multi-agent mode 块，在 collaboration 工具对当前轮次可见时、首次 spawn 之前注入；工具不可见时不注入。
- 为落实 root 控制，推荐默认禁止子智能体自行嵌套派生，除非 root 在具体任务中明确授权；该嵌套边界等待用户裁决。
- 产品源码仍锁定，当前只形成 S2/S5 设计。

## S2 D2-D3 Acceptance

- 用户接受 D2-D3 合并候选：在 collaboration 工具可见时注入一个 multi-agent mode 块，替代原 1,844 字符 D2 与 186 字符 D3。
- 主动委派适用于独立并行、重复机械、调研和写入范围互斥的实现/验证；小型、紧耦合、顺序依赖或即时关键路径工作保留在 root。
- root 持续拥有总体计划、项目方向、关键路径、路由成本、进度、整合、验证、用户更新和最终结果。
- routing class 为 `mechanical / long_context / standard / explore / critical / escalation`；选择原则是可靠完成任务的最低成本档位，`escalation` 仅在低成本档位不足后使用。
- 显式 model/reasoning override 只在用户要求或语义档位不足时使用。
- 受控嵌套已确认：子智能体默认不得派生下一层，只有 root 可为具体有边界的需要显式授权。
- D2-D3 完成；产品源码锁保持，S2 进入 D4。

## S2 Scope Correction

- 用户指出 D4 Hypo-Workflow Cycle 状态不是 VSP-Codex 系统/通用操作提示。此前 D4 内容审阅提问已被中止，没有形成决定。
- 用户明确新的两层边界：所有模型可见注入都要知道具体内容、角色、来源和注入条件；确认属于外部 Workflow 插件后，可跳过其业务提示内容改写。
- C28 主要修复 VSP-Codex 基础提示、系统内置 Skills/Apps/Plugins 提示，以及权限、mode、多智能体、工具等各种操作运行提示。
- 外部插件仍接受注入隔离审计：错误常驻、重复、角色错误和普通会话污染属于 VSP Host 集成问题，可以进入修复范围。
- D4 从 S2 内容改写移出，留到 S4 作为 provenance/injection-boundary 证据。S2 由已接受的 D1 与 D2-D3 组成。

## S2 Result

- D1a Permissions、D1b Default mode 与 D2-D3 proactive multi-agent 已合并为纯英文连续候选 `S2-DEVELOPER-RUNTIME-CANDIDATE.md`。
- 原 D1+D2+D3 共 3,360 字符；候选为 2,104 字符、21 行，减少 1,256 字符（37.4%）。外部 Hypo-Workflow D4 不计入改写候选或该分母。
- 候选不存在 `explicit-request-only`、智能体能力相同宣言、消息 envelope、已知 mode 枚举、文本多选绝对禁令和 `all commands are permitted` 授权歧义。
- 候选保留 `sandbox_permissions` 禁用、运行能力不扩大授权、`request_user_input` 优先、developer-only mode 覆盖、共享工作树和 collaboration direct-call 约束。
- 候选新增 proactive delegation、root 项目控制、互斥写范围、受控嵌套、最低成本可靠 routing class、`long_context` 档位、受控显式 model/reasoning override 和 escalation 次序。
- S5/M2 必须实现当前提示尚未具备的产品能力：可见 `routing_class`、可信 worker-routing resolver、显式 override 约束、有效选择返回与调试/telemetry。
- Markdown diff、Workflow YAML 和否定/必要短语静态检查通过；未修改产品源码，因此未运行 Rust test group。
- S2 进入 waiting-review；接受只锁定设计并进入 S3，不授权产品源码修改。

## S2 Acceptance

- 用户明确接受 S2 完整通用 developer 运行提示候选和结果报告。
- 接受锁定 2,104 字符、21 行候选，相对原 D1+D2+D3 的 3,360 字符减少 37.4%。
- 接受范围包括：默认 bypass 权限短版、Default mode 与 `request_user_input` 优先、proactive multi-agent、root 持续控制、受控嵌套、成本 routing class 与受控显式 override。
- 接受同时锁定来源边界：外部 Hypo-Workflow D4 只做 provenance/injection audit，不进入 VSP 通用提示内容改写。
- S5/M2 的 worker routing 产品要求继续有效；当前工具尚不能执行该设计，不能把 prompt 候选误报为已实现行为。
- 验证证据保持有效：字符/行数、必要与禁止短语、Workflow YAML 和 diff hygiene 均通过；无产品源码改动或 Rust 测试。
- 接受不授权产品源码、配置、远端服务、发布或安装修改。C28 进入 S3。

## S3 Source Map

- 从 M1 同一真实 rollout 的首个聚合 developer message 重新提取 S3 各 content item，而不是按当前文件系统目录猜测模型可见内容。
- Skills block 为 29,179 字符、77 个目录条目、76 个唯一名称；`skill-creator` 同名重复两次。来源分布为 VSP system 5、仓库 14、个人 22、已安装插件 36。
- Apps block 为 646 字符，Plugins block 为 1,014 字符；两者均由 VSP-Codex 内置 context contributor 生成。
- Skills+Apps+Plugins 合计 30,839 字符，占 developer #1 32,515 字符的 94.8%，是 S3 的主要改写面。
- Hypo-Workflow Host block 为 343 字符，由 VSP Host contributor 只声明外部插件所有权；按用户边界只做 provenance/injection audit，不改写其业务语义。
- M1 的“80 项 Skills”是人工统计错误，已依据 rollout 修正为 77；M1 字符和 token 基线不变。
- 当前 dynamic Skills block 没有附带 `How to use skills`，因为模型目录关闭了该 flag；旧 BaseInstructions 中的 Skills 手册已在 S1 候选删除。S3 必须建立一个短且唯一的动态使用协议，否则实现 S1 后会缺少触发/读取合同。
- 现有 renderer 使用 context window 的 2% 作为默认 Skills metadata budget；在 353,400-token 模型上预算约 7,068 tokens，导致目录膨胀到约 29K 字符。需要绝对上限、去重和按需发现设计。

## S3 P1-A Decision

- 用户接受 Skills 公共协议与 5 个 system skill 目录描述的精简方向。
- 动态 Skills 层成为使用协议唯一所有者；BaseInstructions 不再保留重复 Skills 手册。
- Skill 在当前项目完整工作链内跨 turn、compaction 和相关后续持续，直到工作链结束或用户转向实质无关任务。
- 首次使用前完整读取 `SKILL.md`，当前工作链入口只读一次；引用资源按需渐进加载。
- 常驻目录描述只负责触发，不提前复制 MCP/manual/fallback/cachebuster/reinstall 等执行细节。
- 文件来源使用 root alias 缩短重复绝对路径；纯名称必须唯一解析。
- system `skill-creator` 与个人同名条目的最终 bare-name 归属延后到 P1-B2 一起裁决。

## S3 System-Skills Scope Correction

- 用户确认 S3 深度内容审阅仅覆盖随 VSP 分发的 System Skills；可以在真实能力缺口成立时新增一到两个。
- Repo Skills 是当前源码仓库的开发辅助资产，只因 cwd 位于 VSP-Codex 仓库而被自动发现；release/installer 没有把 `.codex/skills` 复制进正式 VSP 包。
- User Skills 与外部 Plugin Skills 按现有机制正常注入，C28 不改写其业务正文；最多按来源和实际用途筛选是否继续启用。
- 刚才 P1-B1 的 support-only/目录改写请求被用户中止，没有形成决定；相关候选撤回。
- token 与质量报告必须分开显示 VSP 核心拥有成本和当前用户/项目环境附加成本，不能通过删除 Repo/User/Plugin Skills 声称产品核心缩减。
- system `skill-creator` 已按触发规则完整读取。后续评估遵循：metadata只负责触发、正文保留非显然可复用流程、详细变体移到references、避免重复、按真实任务验证。
- 下一步逐项审阅现有5个System Skill正文、资源和触发描述，再判断新增项。

## S3 skill-installer Decision

- 用户接受 `skill-installer` 精简候选。
- 候选删除硬编码 sandbox escalation，改为服从当前动态权限；在 VSP `approval_policy=never` 时不得申请不可用 escalation。
- 安装位置由脚本解析当前有效用户 Skills root，不在提示中硬编码 `$CODEX_HOME`/`~/.codex`。
- 列表选择优先使用 `request_user_input`；安装必须来自用户明确请求。
- 目标已存在时中止，禁止覆盖受管理 System Skills；私有仓库凭据不得输出。
- 用户授权按相同标准直接审阅 `openai-docs / plugin-creator / skill-creator`，只汇总不符合 VSP 的规则和精简候选。
- `imagegen` 保留独立审阅，重点讨论官方 URL/token 与 VSP 自定义 endpoint/token 注入。

## S3 Ordinary System Skills Review

- 完整候选与中译记录于 `S3-SYSTEM-SKILLS-CANDIDATES.md`，未修改产品源。
- `openai-docs`：区分upstream OpenAI、VSP fork与第三方provider权威来源；VSP问题先读本地source/config/catalog；删除不存在的`openai-platform-api-key` gate和自动MCP安装/escalation/restart；保留Docs MCP、manual helper、resolver、references与citation规则。候选4,758字符，原22,804字符。
- `plugin-creator`：源码确认个人marketplace路径为`~/.agents/plugins/marketplace.json`且`~/plugins`默认成立；CLI改为`vsp-codex plugin`；App deeplink不再强制；字段表下沉到references。发现canonical manifest示例包含validator拒绝的`hooks`字段，M2需同步修正。候选4,116字符，原11,036字符。
- `skill-creator`：正文关于frontmatter只允许两项的说法与validator和loader不一致；候选改为name/description必填、其他受支持字段按运行时需要。个人目标root不得假设`$CODEX_HOME/skills`，受管理`.system` cache不得作为个人目标。forward tests服从S2 root routing/cost/nesting。候选5,156字符，原21,897字符。
- 三项均保留原有核心能力、scripts/references/assets边界与验证要求；缩减主要来自重复说明、陈旧snapshot和重复字段规范。

## S3 imagegen Decision

- 源码纠正了早期假设：built-in `image_gen` 不是固定官方宿主服务，而是仓库内Rust extension，backend从active `model_provider`解析API provider和auth。
- 请求路径为`{base_url}/images/generations`和`{base_url}/images/edits`，默认`gpt-image-2 / quality=auto / size=auto / background=auto`。
- 当前VSPLab provider具备兼容endpoint和bearer credential，但工具未显示，因为visibility gate按provider name/OpenAI auth mode推断能力；普通custom bearer provider被提前过滤。
- 用户接受改为显式provider capability：VSPLab启用image generation，custom provider默认不启用；active model仍需支持image input，feature和namespace tools仍需开启。
- Built-in模式复用VSPLab Base URL和API Key。Base URL是否含`/v1`按服务实际root配置，client不会自动补齐。
- credential优先使用`env_key = "OPENAI_API_KEY"`，不把key放入Skill、源码、日志或argv；现有inline bearer方式应迁移。
- System Skill继续built-in first，但文案改为“使用active provider endpoint/auth”，不再声称built-in永远不需要API key。
- Bundled CLI继续支持`OPENAI_BASE_URL/OPENAI_API_KEY`，只作为显式CLI、批处理、特殊控制或兼容fallback，不作为VSPLab主集成。
- M2验证覆盖capability false/true、generate/edit URL、bearer header不泄漏、gpt-image-2和official OpenAI不回归。

## S3 New System Skill Evaluation

- 推荐新增且只新增一个`vsp-provider` System Skill。
- 真实缺口：`openai-docs`不能拥有VSP fork事实；现有`/provider`、config、addon catalog和redacted `vsp-codex doctor`缺少模型可调用的组合workflow；C28已重复遇到provider/model、API root、credential与capability误判。
- 候选只拥有VSP-specific provider配置与诊断，不复制完整docs：先运行redacted doctor，区分provider/model，优先env_key，精确处理Base URL root，按声明验证Responses/Image等capability。
- 初版只需`SKILL.md`与`agents/openai.yaml`；doctor已经提供确定性诊断，不新增脚本。
- 第二个新增System Skill没有同等级使用证据，不添加。
- 完整英中候选见`S3-VSP-PROVIDER-SKILL-CANDIDATE.md`。

## S3 External Skills and Common Prompts

- 精确按active rollout版本验证5 System/14 Repo/22 User/36 Plugin，共77项，全部frontmatter有效，无超长description。
- plugin cache旧版本不属于模型可见目录；不得把磁盘cache重复当成注入重复。
- 唯一active plain collision为User/System `skill-creator`。候选沿用现有scope precedence并只显示有效winner；当前User覆盖System，structured path仍可精确选择shadowed source。
- 其他Repo/User/Plugin Skills无明确失效证据，全部保留正常注入，不改写正文。
- Apps prompt修正`tools_search`笔误并压缩；Plugins prompt压缩重复规则；world-state dedupe保持。
- 删除最多50项未安装插件的`<recommended_plugins>` user-role fragment；显式缺失插件请求继续使用list/install tools。
- 当前核心拥有成本约5,067字符，环境附加metadata 25,772字符；成本在S3结果中分开报告。

## S3 Waiting Review

- 完整结果、实现面、测试设计、风险与预期行为见`S3-RESULT.md`。
- S5前产品源继续锁定；S3接受只进入S4，不授权实现。

## S3 Acceptance Checkpoint

- 用户接受S3完整结果，无修订意见。
- 接受范围：5项现有System Skills修订；唯一新增`vsp-provider`；Repo/User/Plugin Skills正常注入且只轻筛；User `skill-creator`按scope precedence覆盖System同名项；Apps/Plugins精简并删除recommended user fragment；imagegen使用VSPLab active provider与显式capability。
- 验证证据：77项active frontmatter有效，候选和Workflow YAML/Markdown检查通过，来源与Rust实现已交叉核对。
- 剩余风险保持：interactive `/provider` credential持久化需要S5安全设计；generation/editing分开验证；环境metadata仍是首轮主要成本。
- S4现在开始，讨论AGENTS、环境与外部注入边界；产品源码在S5接受前继续锁定。

## S4 Source and Boundary Audit

- M1 user context 26,472字符中，root `AGENTS.md`正文25,981字符，约占98%；环境事实不是主要成本。
- 根规则包含至少三类平台/authority错误：OpenCode-only工具名进入Codex；要求写仍停在C10的protected legacy `state.yaml`；引用不存在的`.opencode/hypo-workflow.json.analysis`而真实配置在父JSON内。
- 当前AGENTS loader只按project root到session cwd发现规则；root-cwd编辑nested path不会自动获得leaf AGENTS。32KiB root-first budget也可能饿死更具体规则。
- 候选将root控制在4K以内；Rust/Test、TUI、app-server/protocol、Python按域拆分，目标编辑前必须加载target path规则。是否增加runtime edit preflight留给S5集成裁决。
- Environment约0.4K，保留结构化事实和diff更新；候选从user role移到developer role，需S5真实cache/order验证。
- Workflow Host保留短conditional developer边界；外部插件业务提示不改写；active Cycle状态只接受trusted projection。
- 完整来源、体量、候选、测试与风险见`S4-CONTEXT-INJECTION-REVIEW.md`。

## S4 Waiting Review

- S4选择compact root + domain files；规则按作用域移动而不因token目标删除。
- 硬合同为修改前加载target-path所有适用AGENTS；是否增加runtime mutation preflight由S5集成架构决定。
- Environment候选developer role，保留compact structured facts与world-state diff。
- 外部Workflow/plugin保持conditional、owner-isolated；unknown focus不阻塞普通任务。
- 完整验收结果见`S4-RESULT.md`；接受只进入S5，产品源码仍不解锁。

## S4 Acceptance Checkpoint

- 用户接受S4完整结果，无修订。
- 锁定范围：root AGENTS <=4K拆域、修改前target-path规则加载、developer-role environment、conditional owner-isolated external host与trusted focus projection。
- 验证证据：来源/体量/loader/budget/legacy state/analysis config均已交叉核对，Workflow YAML/Markdown/diff检查通过。
- 风险继续带入S5：upstream结构差异、mutation preflight机制、environment role的真实cache/order影响、plugin adapter外部ownership。
- S5开始整合最终角色图、完整提示、注入矩阵、冲突表和预计diff；源码继续锁定。

## S5 Integrated Design

- 形成单一角色顺序：instructions -> aggregated developer policy -> developer world state -> user AGENTS/triggered Skills -> actual user。
- 9项候选artifact构成完整VSP-owned prompt manifest；运行时metadata/facts为数据，不存在未展示的自由文本。
- root AGENTS英文目标3,734字符；domain relocation与apply-patch target preflight确保规则在mutation前可见。
- worker routing正式消费built-in + user-home policy，新增`long_context`，offline验证，冲突拒绝，继承fallback和effective selection observability。
- image capability使用`Disabled/Generate/GenerateAndEdit`，VSPLab先启用Generate，Edit等待真实验证。
- 新session使用新bundled prompt，existing resume保留session meta；Responses/Lite要求语义role parity。
- 6个M2 stage预计总diff 2.8K-4.4K，非机械logic stage约500行上限；对应core/tui/build-contracts/hypo-external验证矩阵已形成。
- 设计估算44.4K non-tool chars（-42.8%）和约19.2K first input tokens（-30.1%）；S6真实rollout才可确认。

## S5 Authorization Gate

- `S5-RESULT.md`为完整授权报告；静态验证、预期行为、问题和风险已列明。
- 接受S5只授权六个M2产品源码/测试stage，不授权commit、push、PR、publish、install、deploy、restart、remote mutation或destructive action。

## M2 Stage A Checkpoint

- 用户接受S5后，Stage A在主线程完成；本阶段未使用subagent，未发生commit、push、publish、install、deploy、service restart或remote mutation。
- `gpt-5.6-sol`的`base_instructions`与`instructions_template`均替换为S1接受稿，11,696 bytes且SHA-256为`33202ab56daa59f39644a36f82935532b910a78f313fbc11147b5e450f39e5b6`。
- 默认Disabled/Never permissions与Default collaboration mode精简；显式权限和命名mode动态分支保留。
- Multi-Agent V2默认改为proactive；root/shared/mode在initial context中并入同一developer policy message，runtime mode update仍可覆盖。
- Apps/Plugins提示精简；`<recommended_plugins>` user fragment、endpoint recommendation分支与第二种install wire shape删除；显式list/install路径保留。
- 目标crate test-target编译通过；core focused 5/5、permissions 20/20、models-manager 3/3、fmt与diff hygiene通过。
- 本地构建`vsp-codex`通过隔离custom provider向本机mock发出真实Responses请求：base prompt逐字匹配，dynamic policy为单developer message的5个有序parts，`recommended_plugins`计数为0。
- 完整实现、验证、问题、风险与后续测试见`M2-STAGE-A-REPORT.md`；raw prompt证据见`M3-PROMPT-DEBUG-EVIDENCE.md`。
- M2仍为`in_progress`，Stage A进入`waiting-review`；用户接受前不进入Stage B。

## M2 Stage A Acceptance

- 用户明确选择“接受并进入 Stage B”，因此Stage A实现、28/28 focused tests与真实prompt evidence被接受并锁定为completed。
- 用户同时表示不记得为何还有多个后续任务，并认为当前系统提示词修复可能已经足够发布。
- 这条反馈会实质改变M2范围：继续原计划将进入System Skills、AGENTS/runtime、image capability和worker routing实现；缩小范围则把这些能力延期，只为当前Stage A增加最终回归与发布准备。
- 在向用户解释两种路径并获得范围决定前，不自动开始Stage B产品改动；commit/push/release/install仍未授权。

## M2 Scope Decision After Stage A

- 向用户解释原Stage B-F中只有最终验证是发版硬要求；System Skills、AGENTS/runtime、image capability与worker routing是不同能力面。
- 明确指出当前prompt已命名routing classes但工具schema尚未实现，因此原样发布存在能力承诺不一致；若追求全栈提示质量，还需完成Skills、AGENTS与worker routing。
- 用户选择“完成提示词全栈再发”：继续Stage B、Stage C、Stage E和Stage F。
- Stage D image-provider capability延期到后续Cycle，不计入本次发布阻塞项。
- Stage B现在开始；每个实现stage继续保留独立review gate。发布与外部动作仍未授权。

## S5 Acceptance / M2 Authorization

- 用户接受S5完整集成设计，授权六个M2产品源码与测试stage立即开始。
- 授权绑定9项prompt artifact、单一角色/注入矩阵、AGENTS preflight、worker routing、image capability、session migration、observability和M3/S6验证。
- 保持禁止：commit、push、PR、publish、install、deploy、service restart、remote mutation、destructive operation、protected Workflow file edits。
- M2从Stage A prompt sources/assembly/debug开始，每个stage完成后记录实现、focused test和剩余风险。

## M2 Stage B Implementation

- 5项接受的System Skill正文已替换，原83,263 bytes降至22,573 bytes；新增唯一`vsp-provider`，6项validator全部通过。
- plugin-creator references统一为`vsp-codex plugin`并移除validator拒绝的manifest `hooks`字段；skill-installer使用canonical `$HOME/.agents/skills`，兼容列举legacy/System root并拒绝覆盖managed System names。
- model-visible catalog在渲染与plain mention前按`Repo > User > System > Admin`选择唯一winner；structured path仍可精确选择enabled shadowed source。
- 默认Skill metadata预算保留2%规则但增加2,000-token绝对上限；已有description截断、alias和omission行为保留。
- 对关闭完整usage guide的模型注入5行compact Skill protocol，明确selected Skill在同一thread跨turn、compaction和resume持续，无需重新mention。
- compaction replacement保留每个Skill path的最新正文，并在initial context插入后重新计算replacement occupancy，避免隐藏超预算。
- clean core-skills 121/121、core available protocol 2/2、core compact 19/19、smoke 7 probes、fmt/diff和Python checks通过。
- 真实CLI capture在640-token metadata预算下输出2个root aliases、20条/20个唯一Skill、6项System Skill和scope collision Repo winner；请求完成并返回`ok`。
- 完整实现、测试、capture、问题与风险见`M2-STAGE-B-REPORT.md`。
- Stage B状态为`waiting-review`；人工接受前禁止开始Stage C。

## M2 Stage B Acceptance

- 用户在看到完整结论、技术实现、modified modules、test design/results、真实prompt evidence、问题与风险后明确选择“接受 Stage B”。
- 接受范围锁定：5项System Skill精简、新增`vsp-provider`、effective-name scope precedence、root aliases、2,000-token metadata cap、compact Skill protocol和跨turn/compaction/resume生命周期。
- 接受证据：6/6 Skill validator、clean core-skills 121/121、core protocol 2/2、core compact 19/19、smoke 7 probes、fmt/diff/Python checks和真实CLI alias/collision capture全部通过。
- 保留风险：live-model质量与完整token对比留Stage F/S6；升级前已经丢失的旧Skill body无法追溯；active Skill body会占用且现已正确计入compact replacement budget。
- Stage C现在被允许作为下一阶段，但本次接受持久化不启动Stage C；commit、push、release、install、restart及其他外部动作仍未授权。

## M2 Remaining-Scope Decision

- 用户确认Worker Routing用于给Subagent选择语义模型、reasoning与成本档位，并授权立即实现。
- 剩余产品实现只保留Stage E Worker Routing；Stage C AGENTS/runtime preflight与Stage D image capability延期，不阻塞本次发布。
- Worker Routing采用独立`agent/worker_routing`边界：从built-in defaults叠加`$VSP_CODEX_HOME/worker-routing.toml`，只用offline provider/model catalog解析，不读取repo/project routing policy。
- V2 `spawn_agent`增加`routing_class`，默认`standard`；semantic routing与显式model/reasoning/service-tier override互斥，role-pinned model设置与routing冲突，全部候选不可用时继承parent并报告skip reason。
- Worker Routing完成后保留独立review gate；接受后直接执行omissions audit、真实prompt/token capture、focused regression与smoke，只补确实遗漏的发布阻塞项。
- 最终验证通过后准备并发布Linux、Windows、macOS产物；发布前展示确切版本、目标矩阵、产物、校验和与manifest/Release一致性。install、deploy和service restart未授权。

## M2 Stage E Worker Routing

- `spawn_agent`公开六类semantic routing，默认standard；explicit model/reasoning/service-tier override与routing class互斥。
- built-in VSP defaults叠加`CODEX_HOME/worker-routing.toml`的per-class override；不读取repo/project policy。
- resolver按provider/model/reasoning/context/service-tier顺序使用offline catalog；unsupported候选记录reason并继续，耗尽后继承parent。
- role-pinned provider/model/reasoning/tier与semantic routing冲突；compatible role instructions仍在routing后应用。
- output与trace报告requested/effective/fallback/skipped决策；无credentials/raw auth。
- cross-provider tier validation初版问题已修复；一个等待不存在legacy events的旧测试已改为真实V2 output/child snapshot合同。
- focused core验证：worker routing 6/6、fallback 1/1、explicit conflict 1/1、spawn schema 6/6、V2 spawn 19/19、full-history tier 1/1、tool-family 1/1；fmt/diff/YAML均通过。
- 完整实现、测试、问题、风险与后续见`M2-STAGE-E-REPORT.md`。Stage E进入`waiting-review`，接受后直接开始omissions audit与最终验证。

## M2 Stage E Acceptance

- 用户在看到结论、技术实现、modified modules、10项行为测试清单、35项focused执行、问题与风险后明确选择“接受并继续”。
- Stage E锁定为completed；M2的实际范围为已接受Stage A、Stage B、Stage E，Stage C/D按最新范围延期。
- 当前进入M3 omissions audit与最终验证；只补审计确认的发布阻塞遗漏，不重开延期能力。
- M3覆盖真实prompt/token capture、Stage A/B/E focused regression、required smoke和release multi-target聚合审计；通过后进入S6效果验收与Linux/macOS/Windows发布。

## M3 Final Validation and Omissions Audit

- accepted Stage A/B/E行为没有漏接；core-skills 121、assembly 2、Skill protocol 2、compact 19、routing 6、V2 spawn 19、explicit plugin 1、smoke 7全部通过。
- 真实VSPLab首轮21,321 input / 3,840 cached tokens，相对M1下降22.6%；约55,565 non-tool chars下降28.4%。未达到原25%/40%目标，原因是已延期Stage C的26K根AGENTS仍注入。
- live Subagent smoke在root进入spawn前被VSPLab连续502阻断；记录为外部残余风险，未伪报通过。
- release audit发现单target builder会覆盖同版本其他target且默认Linux archive名；已新增target-specific naming、strict aggregate script、三平台原生GitHub Actions workflow与tests。
- release checks从48扩展为52并全部通过；`cargo check`、required smoke、format、diff、YAML syntax通过。完整证据见`M3-FINAL-VALIDATION.md`。
- M4采用`0.145.0-vsp.11.1-alpha.6`，targets为Linux x64 GNU、macOS ARM64、Windows x64 MSVC。GitLab无macOS/Windows runner，需GitHub mirror运行native matrix。

## M4 Native Build Diagnostics

- 已创建公开source-build mirror `HypoxanthineOvO/VSP-Codex`；GitHub description/homepage明确GitLab为canonical history与release，mirror `main`为无历史、无LFS payload的alpha.6 source snapshot。
- GitHub Actions run `31471403149`确认三平台jobs均被调度。Linux/macOS因snapshot无tag而无法自动解析upstream baseline；workflow现增加显式`upstream_tag`输入，默认`rust-v0.145.0-alpha.4`。
- Run `31471584439`确认Windows checkout受两条`.opencode/commands/hw:plan:*` colon路径阻塞。它们不参与source build，已仅从GitHub snapshot移除；GitLab canonical源码保持完整。
- macOS ARM64原生编译推进到`codex-tui`后发现merge残留：upstream已删除`cpal`、audio-device模块与realtime app command，但`voice.rs`仍在非Linux cfg下编译。修复不恢复已删除transport，而是停止编译孤立voice模块，并让realtime capability/device picker在所有平台统一为不可用。
- `just test-group run tui --filter user_typing_provider_slash_command_opens_provider_picker`完成完整TUI test-target编译并通过1/1；3195项未选择。`cargo fmt --check`与`git diff --check`通过，无snapshot更新。
- Run `31474763600`确认Windows checkout修复有效，随后setup action因请求不存在的`librusty_v8_release_x86_64-pc-windows-msvc.a.gz`返回curl 22。OpenAI release对MSVC只发布`rusty_v8_ptrcomp_sandbox_release_*.lib.gz` pair；action现按target选择正确archive/binding/checksum。三个matrix target的asset名称全部存在，三个checksum manifest均为严格两行。
- Run `31475827303`进一步确认MSVC下载成功，但官方checksum文本为CRLF；Git Bash `sha256sum`将尾随`\r`解释为filename一部分。action现校验前移除checksum文本中的CR，本地重新下载官方archive/binding并执行相同归一化后，两项SHA256均通过。
- Run `31476110232`在build-only SHA `dc0a24f8c1`完成Linux x64 GNU、Windows x64 MSVC、macOS ARM64和aggregate四个成功job。path-excluded diff及关键blob哈希证明该build tree与GitLab canonical `f61aacca62`的产品/构建源码一致，仅省略`.pipeline`和`.opencode`历史资料。
- 下载的aggregate artifact ID `9097925225`为611,396,708 bytes，外层SHA256为`3c568ab26eec17ff16fb96e734547b9c10091c66e9acbfe4f45069085ca3f32d`且全ZIP CRC通过。三份archive SHA与sidecar/manifest一致，布局、target metadata、ELF/Mach-O/PE格式和native package version均通过。
- M4 release candidate进入`waiting-review`；tag、GitLab Release和stable manifest均保持未修改，等待用户显式批准。完整证据、问题和风险见`M4-RELEASE-CANDIDATE.md`。
- 发布批准后首次上传的manifest在公开下载复验中被发现只有一个target（错误地对已聚合manifest再次执行单输入merge）；三份archive和SHA均正确。已恢复原始三target aggregate、准备替换Generic Package Registry中的manifest，并保留immutable tag不移动。
