---
kind: discussion-summary
cycle: C28-prompt-stack-quality-redesign
status: active
updated: 2026-08-11T00:28:05+08:00
---

# C28 Discussion Summary

## 用户目标

- 针对当前会给模型的提示词逐项讨论和点评。
- 英文提示需要提供中文翻译。
- 讨论形成明确决定后，再做一轮修改。

## 已确认决定

- 使用同一个 C28 承载讨论、设计、实现和最终验收。
- Cycle 讨论优先，以 Stone 承载真实内容审阅和用户裁决。
- 初始启动不授权产品源码修改；只有接受 S5 集成设计才授权 M2。
- 每项讨论必须在对话中解释原文、翻译、问题和建议，不只提供文件路径。
- “逐项”必须建立在一个完整来源的解析之上：先展示该 Source 在请求中的位置、来源链、完整目录、全部内容和关系，再按原始顺序逐段裁决。
- 孤立句子不能作为可靠决策单元；原 S1/B1 请求已撤回且没有形成裁决。
- 目录和统计也不能代替实际提示措辞；逐段讨论必须直接展示连续的大段英文原文及完整中文翻译。
- Source 1 分成三个大文本块：1-22、23-75、76-148 行。每块整体讨论，不再要求用户先确认目录粒度。

## 当前未决

- Stage B System Skills具体实现与review。
- Stage C AGENTS/runtime injection与Stage E worker routing实现。
- Stage F最终prompt/token/smoke结果与发布授权。

## Block A 已确认内容

- 身份：VSP-Codex，由上海科技大学 VSP Lab 学生基于 OpenAI Codex 开发。
- 语言：任何时候以中文为主体，仅保留必要的计算机英文术语。
- 术语：用户熟悉时可提高英文术语密度，否则提供中文翻译和解释。
- Markdown：允许并鼓励用于提高清晰度，但不要过度拆分短段落。
- CommonMark：列表前和标题后必须保留空行，以保证渲染。
- 报告呈现：不能只报文件路径，必须在对话说明重要内容；最终落点待 Block B 固化。
- 人格：删除 old friend、个人品味、另一个主体和 unique 等主体性表演；保留自然沟通、理解用户和主动预判问题。
- Block A 候选稿已由用户确认。

## Block B 已确认方向

- Steer 默认追加；只有用户明确停止、暂停、否定或替换时才停止旧任务。
- 新旧要求冲突时说明并询问，不得静默丢弃任何一方。
- Compaction 使用正向恢复算法：按最新执行时间线重建当前目标和下一步，用后续行动、结果和决定解释早期用户消息。
- 恢复时读取可用摘要、计划、日志、Workflow 状态、工具结果、工作树和运行进程，并遵守后续拒绝、修订和完成状态。
- 过程与结果明确分开；过程输出更详细，说明检查、发现、判断依据、假设、取舍和下一步。
- Final 重新汇总过程中的重要内容，允许必要重复，并对复杂工程和实验提供详细报告。
- 工程报告对照用户要求说明实现、文件、验证、问题和风险；实验报告用通俗中文解释原理、设置、指标、结果、解释和局限。
- 探索性任务说明问题、范围、搜索路径、事实、推断、候选解释、取舍、不确定性、当前判断和下一步；结论确定程度与证据强度匹配，证据不足时给出暂定判断和补证路径。
- 删除孤立的“不要通过明显更差的方案夸奖计划”规则。
- Block B 当前候选稿已由用户确认。

## Block C 已确认方向

- 正式 VSP-Codex package 保证随包 rg 和 PATH；非 package 环境一次探测后稳定降级。
- 基础提示遵循当前 shell，PowerShell/POSIX 详细规则动态注入。
- Using skills 从 BaseInstructions 删除，由动态 Skills 来源唯一拥有。
- 技能在当前项目的一轮完整工作链内跨 turn、compaction 和相关后续持续，不默认存在 Workflow；另起无关任务或其他功能时释放。
- SKILL 入口读取一次，引用资源按当前需要渐进加载。
- 意外改动可查询同 workspace/cwd 近期相关会话判断归属，但不继承其任务和授权。
- Block C 候选稿 v1 已形成，等待整体措辞审阅。

## S1 提交验收

- 三个 Block 已合并为完整英文候选稿和 S1 结果报告。
- 接受 S1 表示锁定当前 Source 1 候选并进入 Source 2，不授权修改产品源码。
- 拒绝 S1 时保留全部历史决定，按具体反馈恢复 S1 `in_progress` 修订。

## S1 接受

- 用户接受当前完整 BaseInstructions 候选稿、S1 报告、验证结果和剩余风险。
- 接受范围只覆盖 Source 1；不授权产品源码修改，S5 之前的 source edit lock 保持不变。
- C28 进入 S2，开始通用 developer 运行规则的完整来源解析。

## S2 D1a 已确认方向

- 原 D1 拆成 D1a Permissions 与 D1b Default collaboration mode，避免把两个独立动态来源混成一个复杂裁决单元。
- VSP 默认 bypass 场景使用短权限提示，不再展开通用沙箱概念。
- 默认候选说明文件系统和网络不受限、命令无需沙箱审批、不得传 `sandbox_permissions`，并明确运行能力不扩大用户请求授予的范围或授权。
- 用户显式覆盖默认权限时仍按实际配置动态生成提示，不能把 bypass 状态硬编码为所有入口永远成立。
- D1b 尚未裁决；产品源码继续锁定。

## S2 D1b 已确认方向

- Default mode 动态块采用短版，只保留当前模式、developer 指令切换权威及通用执行与授权规则引用。
- 删除已知模式名称清单、重复的提问标准和禁止文本多选题的绝对规则。
- 当确实需要用户输入且 `request_user_input` 可用时，应优先并积极使用 VSP 增强后的结构化 Request；不可用时保留清晰文本 fallback。
- 是否需要提问仍由 S1 的统一合同判断，D1b 不重复定义。
- D1 已完成，下一项为 D2 多智能体运行提示。

## S2 D2 已确认需求

- C28 必须支持根据任务难度、上下文需求和成本选择不同子智能体配置，不能让所有 worker 无差别使用强模型。
- 机械繁复任务应可选择简单、低成本模型；长上下文但机械的任务应可选择适合大上下文吞吐的模型；高难度、高风险或复杂综合任务才使用强模型。
- 推荐采用语义 routing class 到 provider/model/reasoning 的运行时解析，避免提示硬编码具体模型名和价格。
- 当前 V2 虽有底层 model/reasoning override，但默认从 `spawn_agent` schema 隐藏；已有 `worker-routing.toml` 尚未接入当前可见工具接口，因此该需求需要 M2 产品实现，不能靠提示词单独完成。
- 具体档位、fallback 和注入时机仍待 D2 裁决；先前“首次 spawn 后才注入全部 D2”不视为已确认方案。

## S2 D2-D3 主动委派决定

- 默认使用语义 routing class；仅在用户明确要求或语义档位不足时允许受控的显式 model/reasoning override。
- 删除 D3 的 `explicit-request-only` 限制，root 可在重复机械、多路独立、调研和其他有明确收益的场景主动委派。
- root 始终负责总体方向、计划、关键路径、成本路由、进度监督、结果整合、最终验证和用户汇报，不得把项目控制权交给子智能体。
- D2 与 D3 合并成一个短动态块，在当前轮次存在 collaboration 工具时、首次 spawn 前注入；工具不可见时不注入。
- 推荐子智能体默认不得继续派生下一层，除非 root 对具体任务明确授权；此项尚待用户确认。

## S2 D2-D3 接受

- 用户接受完整合并候选和受控嵌套边界。
- root 可主动委派独立并行、重复机械、调研及互斥写范围任务，但小型、紧耦合、顺序依赖和即时关键路径工作留在本地。
- root 始终负责计划、方向、关键路径、路由成本、进度、整合、验证、用户沟通和最终交付。
- routing class 固定为 `mechanical / long_context / standard / explore / critical / escalation`；默认选择可靠的最低成本档位，显式 model/reasoning override 仅作受控例外。
- 子智能体默认不得继续派生；root 可为具体有边界的需要显式授权嵌套。
- D2-D3 完成，进入 D4。

## Prompt 来源与改写边界纠正

- 所有注入给模型的内容都必须盘点具体文本、API 角色、所有者和注入条件。
- 确认属于外部插件（例如 Hypo-Workflow）的业务提示后，不在 C28 中改写其内容；只审计错误常驻、重复、角色错误和普通会话污染等 Host 注入问题。
- VSP-Codex 基础提示、系统内置插件提示及权限、mode、多智能体、工具等操作提示是主要修复对象。
- D4 Hypo-Workflow Cycle 状态已归因到外部插件并移出 S2；此前 D4 提问中止，无任何 D4 决定。
- S2 现只包含已确认的 D1 与 D2-D3，可以汇总提交验收。

## S2 提交验收

- 完整候选为 2,104 字符、21 行，相对原 D1+D2+D3 的 3,360 字符减少 37.4%。
- D4 Hypo-Workflow 状态只保留来源盘点，未混入候选。
- 接受 S2 表示锁定权限、Default mode、主动多智能体、root 控制、成本 routing class、受控 override 与受控嵌套设计，并进入 S3 内置插件提示审阅。
- S5 之前继续禁止产品源码修改。

## S2 接受

- 用户接受 S2 的 2,104 字符、21 行完整候选及结果报告；相对原 D1+D2+D3 减少 37.4%。
- 接受内容包括默认权限、Default mode、Ask 优先、主动委派、root ownership、语义成本路由、受控显式覆盖和受控嵌套。
- 用户接受外部插件来源边界：Hypo-Workflow D4 不进入内容改写，但其注入条件与隔离仍需审计。
- 接受只推进到 S3，不授权产品源码修改；worker routing 仍是 S5/M2 待实现能力。

## S3 来源地图

- 当前真实 Skills block 为 29,179 字符、77 个条目、76 个唯一名；`skill-creator` 重复。M1 原“80项”已纠正为77项。
- Skills 来源：VSP system 5、仓库 14、个人 22、已安装插件 36。
- Apps 为646字符，Plugins为1,014字符；Skills+Apps+Plugins合计30,839字符，占developer #1的94.8%。
- 外部 Hypo-Workflow Host 为343字符，只做来源与注入隔离审计。
- 实际动态 Skills block 当前没有使用协议；S1 又已删除基础提示中的旧手册，因此 S3 要建立短、唯一、项目工作链持续的动态协议。
- 2% context-window skills metadata budget 在大上下文模型上过宽，需要绝对上限、同名去重和按需发现。
- 当前首个讨论块为 Skills 动态目录与使用协议；之后为 Apps、Plugins 和内置 Skill 正文触发边界。

## S3 P1-A 已确认

- Skills 公共协议采用短动态版本，由 Skills 层唯一拥有。
- Skill 在项目完整工作链内持续；入口 `SKILL.md` 首次完整读取且工作链内只读一次，引用资源按需加载。
- 常驻目录采用路径别名和只负责触发的短描述，不提前展开 Skill 正文操作细节。
- 纯名称必须唯一解析；`skill-creator` 冲突的具体归属等待 P1-B2。
- 5 个 system skills 的短描述方向已接受，下一项为14个仓库 Skills。

## S3 范围再次纠正

- 深度审阅和内容改写只覆盖随包 System Skills；可基于真实缺口新增1-2个。
- Repo Skills 只在当前源码仓库自动发现，不随正式 VSP package 分发；User和外部Plugin Skills也不属于系统业务正文。
- Repo/User/外部Plugin Skills继续正常注入，C28只做来源归因和usefulness轻筛，不改写其正文。
- P1-B1仓库Skill候选没有获得接受，已撤回；不产生support-only设计决定。
- S3报告必须拆分核心系统成本与用户/项目环境附加成本。
- 后续直接审阅5个System Skill完整正文与资源，并评估是否存在值得新增的1-2个系统能力。

## S3 skill-installer 已确认

- 接受当前精简候选：动态权限、脚本解析用户Skills目标、Ask优先、明确安装授权、凭据保护、失败准确报告和禁止覆盖System Skills。
- `openai-docs / plugin-creator / skill-creator` 由Agent按同一标准直接审阅并汇总。
- `imagegen` 单独讨论，重点是自定义Base URL和token配置。
- 新增System Skill延后到现有集合审阅完成后按能力缺口判断。

## S3 其他普通 System Skills 候选

- `openai-docs`：官方OpenAI事实与VSP fork/provider事实分流；不自动改MCP配置或重启；候选4,758字符。
- `plugin-creator`：保留已验证的个人marketplace路径，CLI使用`vsp-codex`，修复reference中的非法hooks字段，deeplink仅在兼容surface可选；候选4,116字符。
- `skill-creator`：纠正frontmatter validator、VSP user root和`.system` cache语义，压缩重复方法说明并让forward test服从S2 routing；候选5,156字符。
- 三项完整英文候选与完整中文翻译见 `S3-SYSTEM-SKILLS-CANDIDATES.md`；尚未进入产品源。
- 当前进入imagegen自定义endpoint/token专项。

## S3 imagegen 已确认

- Built-in imagegen是VSP仓库内Rust extension，实际复用active model provider，并非固定官方URL。
- VSPLab provider显式声明图像能力后，内置工具使用其Base URL、Bearer API Key和`gpt-image-2`请求generation/edit endpoint。
- 修复重点是provider capability gate，不是新增默认CLI wrapper。
- API Key优先由`env_key = "OPENAI_API_KEY"`提供，避免inline secret；CLI继续作为备用。
- System Skill文案同步active-provider事实并适度精简；详细证据、路由和M2测试见`S3-IMAGEGEN-REVIEW.md`。

## S3 唯一新增 System Skill 候选

- `vsp-provider`：配置与诊断VSPLab/OpenAI-compatible provider，组合`/provider`、config、catalog、redacted doctor和capability验证。
- 它填补`openai-docs`退出VSP-specific事实后的明确空缺，并直接覆盖本轮发现的Base URL、credential和image capability问题。
- 初版无需脚本；完整英中候选见`S3-VSP-PROVIDER-SKILL-CANDIDATE.md`。
- 不建议第二个新增项，避免为凑数量扩大System Skills。

## S3 整体待验收

- 77项active Skill metadata全部有效；Repo/User/Plugin正文不改写、不禁用。
- User `skill-creator`按既有scope precedence覆盖System同名项，解决bare mention静默失败；structured path保留精确访问。
- 5项现有System Skills均保留并修订，新增且只新增`vsp-provider`。
- Apps/Plugins公共提示精简并修正tool_search；删除主动推广未安装插件的user-role fragment。
- 当前核心拥有成本约5,067字符，环境附加成本25,772字符，后续分别验收。
- 完整结论、实现面、测试与风险见`S3-RESULT.md`；接受后进入S4，产品源仍锁定。

## S3 已接受

- 用户接受S3整体结果，无修订。
- 锁定5项现有System Skills修订、唯一新增`vsp-provider`、外部Skills正常注入、scope winner、imagegen active-provider与Apps/Plugins公共提示方向。
- 77项active metadata验证和核心/环境成本拆分作为后续基线。
- C28进入S4 AGENTS/环境/外部注入边界；S5前仍不修改产品源码。

## S4 初步结论

- 当前26,472字符user context约98%是25,981字符根AGENTS；环境块很小且事实有用。
- 根AGENTS混入OpenCode-only工具、stale legacy state写入和不存在的analysis派生路径，不能只做字数压缩。
- 推荐root <=4K，只保留通用规则与target-path路由；Rust/Test、TUI、app-server/protocol、Python拆到适用域。
- 当前loader只按session cwd加载nested AGENTS，root-cwd任务需明确的target-path读取门；S5决定是否增加runtime edit preflight。
- Environment候选为developer-role structured world state；Workflow Host保持短conditional developer边界；外部业务提示不改写。

## S4 待验收

- 推荐compact root <=4K，领域规则按Rust/Test、TUI、app-server/protocol、leaf和Python拆域。
- 修改前加载target-path规则是硬合同；runtime preflight机制留S5审阅。
- Environment使用developer role和diff-only world state；外部Host/Plugin保持conditional与owner isolation。
- 普通root任务AGENTS baseline预计至少降低84%；规则不因成本目标删除。
- 接受后进入S5完整提示与集成设计，仍不自动授权产品源码修改。

## S4 已接受

- 用户接受S4整体结果，无修订。
- compact root、domain rules、target-path mutation contract、developer environment role和external owner isolation成为S5输入。
- C28进入S5；产品源码仍锁定，S5接受才构成M2实现授权。

## S5 待授权

- S1-S4已整合为唯一目标角色栈与完整9-artifact prompt manifest。
- root AGENTS 3,734字符，target-path apply-patch preflight；environment developer role；external host owner isolation。
- worker routing新增正式trusted config loader、`long_context`、deterministic resolver与effective result；image capability分Generate/Edit。
- M2拆6个reviewable stages，预计2.8K-4.4K总diff，logic stage约500行上限。
- 设计估算non-tool chars -42.8%、first input tokens -30.1%；S6实测验收。
- 接受S5将授权M2产品源码和测试改动，其他外部/发布/破坏性动作仍未授权。

## S5 已接受 / M2 已授权

- 用户接受完整S5，无修订，并授权六个M2产品源码/测试stage。
- M2从prompt sources/assembly/debug开始；全部外部、发布、安装、重启、破坏性和protected Workflow动作仍在授权外。

## M2 Stage A 已接受 / 后续范围待定

- 用户接受Stage A：Sol基础提示、默认permissions/mode、proactive Multi-Agent initial assembly、Apps/Plugins精简、unsolicited recommendations删除及其验证均锁定。
- 用户表示不清楚为何还有多个后续任务，并认为当前系统提示词修复可能已足够发布。
- 后续不应机械进入Stage B；先解释原计划剩余项与最小发布方案，再由用户选择继续全范围或延期附加能力。

## M2 后续范围已决定

- 用户选择“完成提示词全栈再发”，而不是最小Stage A发布或原完整六阶段。
- 本次继续：System Skills与目录去重、AGENTS/runtime injection、真实worker routing、最终回归与token/smoke。
- 本次跳过：VSPLab image-provider capability；它是独立产品能力，延期到后续Cycle。
- 当前进入Stage B；最终发布、安装和远端动作仍需单独明确授权。

## M2 Stage B 已接受

- 用户接受5项System Skill精简、新增`vsp-provider`、effective-name scope precedence、root aliases、2,000-token metadata cap、compact Skill protocol及跨turn/compaction/resume生命周期。
- 接受时已展示实现范围、modified modules、clean core-skills 121/121、core 21/21、smoke 7 probes、真实CLI prompt capture、问题与剩余风险。
- Stage C AGENTS/runtime injection现在允许作为下一阶段开始；本次只持久化接受结果，没有启动Stage C。
- Stage F/S6仍负责live-model质量与完整token对比；Stage D image-provider capability继续延期。

## 未授权

- S5 接受前的任何产品源码修改。
- 远端 provider 服务修改、commit、push、publish、install、deploy 或 service restart。

## M2 Stage E 已接受 / M3 开始

- 用户接受Worker Routing实现：六类semantic class、默认standard、trusted home policy、offline resolver、inherited fallback、skip reasons、role/explicit冲突、fork与output/telemetry均锁定。
- 接受证据为focused core 35项执行全部通过，以及fmt、diff、Workflow YAML检查通过；cross-provider tier catalog问题已在接受前修复。
- 用户要求接受后直接做omissions audit与最终验证，再准备Linux、Windows、macOS构建发布；Stage C AGENTS/runtime与Stage D image capability不再进入本次。
- 发布准备和三平台发布已由最新用户请求授权；install、deploy与service restart仍未授权。
