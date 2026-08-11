---
kind: progress
cycle: C28-prompt-stack-quality-redesign
plan: PLAN.md
status: active
updated: 2026-08-11T14:40:46+08:00
current: M4 cross-platform release preparation
next: commit and run GitHub-native Linux/macOS/Windows build matrix
---

# C28 Progress

## 当前状态

C28 已创建并聚焦。M1与S1-S5讨论均已完成；M2 Stage A和Stage B均已接受。用户将剩余实现收敛为 Worker Routing，完成后直接执行遗漏审计与最终验证，再准备 Linux、Windows、macOS 构建发布；AGENTS/runtime preflight 与 image capability 延期。

| ID | 阶段 | 状态 | 当前证据 | 下一步 |
| --- | --- | --- | --- | --- |
| M1 | 真实基线 | completed | `M1-BASELINE.md`；rollout 首轮 27,536 input tokens | 已完成 |
| S1 | 基础提示与人格讨论 | completed | 用户接受完整候选稿和 S1 报告 | 已完成 |
| S2 | 通用运行规则讨论 | completed | 用户接受 2,104 字符候选、外部 D4 边界与 worker routing 设计 | 已完成 |
| S3 | Skills / Apps / Plugins 讨论 | completed | 用户接受5项System Skills、1项新增vsp-provider、外部Skills轻筛和Apps/Plugins候选 | 已完成 |
| S4 | AGENTS / 环境 / Workflow 讨论 | completed | 用户接受root<=4K拆域、target-path加载、developer环境role与external边界 | 已完成 |
| S5 | 集成设计审阅 | completed | 用户接受完整集成设计并授权六个M2产品源码/测试stage | 已完成 |
| M2 | 实现获批设计 | completed | Stage A、Stage B、Stage E均已接受；C/D延期 | 已完成 |
| M3 | 回归与质量验证 | completed | accepted scope全绿；token目标因Stage C延期未达；live Subagent被VSPLab 502阻断；release遗漏已补 | 见`M3-FINAL-VALIDATION.md` |
| S6 | 真实效果验收 | completed-with-residual-risk | 真实prompt/token已测；功能验证通过，原量化目标未达 | 风险带入release notes |
| M4 | 跨平台构建发布与 Cycle 收尾 | in_progress | alpha.6版本与三平台workflow已准备；等待native builds | commit、mirror/Actions build、RC evidence、GitLab publish |

## 时间线

| Time (Asia/Shanghai) | State | Detail |
| --- | --- | --- |
| 2026-08-10 16:39 | started | 用户确认同一 Cycle、讨论优先；创建并聚焦 C28。S5 接受前禁止修改产品源码。 |
| 2026-08-10 16:39 | completed | M1 固化本次真实 rollout、提示层、角色、字符数、token、缓存与源码入口。 |
| 2026-08-10 16:39 | in_progress | S1 开始；按 B1-B13 逐项讨论基础提示与人格，当前为 B1。 |
| 2026-08-10 17:15 | revised | 用户指出孤立句子缺少来源位置、完整目录和内容关系，无法作出可靠裁决。撤回原 B1 请求；S1 改为先完整解析一个 Source，再按原始目录逐段讨论。 |
| 2026-08-10 17:15 | completed | Source 1 `BaseInstructions` 的请求位置、选择优先级、13段目录、完整中文内容、比例、内部重复、跨来源重叠和稳定性问题已整理。 |
| 2026-08-10 17:20 | revised | 用户明确拒绝以目录、统计或孤立句子代替具体措辞。S1 改为三个连续大文本块，每块直接展示完整英文、完整中文、整体点评和候选改写；撤回目录粒度确认请求。 |
| 2026-08-10 17:20 | decided | 用户指定 VSP-Codex 品牌身份、中文主体沟通、术语熟悉度适配、Markdown 不过度分段、CommonMark 空行要求，以及报告重要内容必须在对话中说明。Block A 候选稿 v1 已形成。 |
| 2026-08-10 17:42 | completed | 用户确认 Block A 无问题，并接受删除 old friend、个人品味和主体性表演的方向。进入 Block B。 |
| 2026-08-10 17:55 | decided | 用户纠正 Block B：steer 默认追加；compaction 按最新执行时间线恢复；复杂工程和实验 final 默认详细；过程输出与结果分开且过程需要具体判断依据；删除孤立的计划夸赞禁句。候选稿 v1 已形成。 |
| 2026-08-10 18:16 | revised | 用户要求 compaction 使用正向恢复算法而不是绝对否定句，并补充探索性任务回答合同。候选稿现按时间线恢复，并区分事实、推断、开放问题和降低不确定性的下一步。 |
| 2026-08-10 18:25 | completed | 用户确认 Block B 当前版本。过程与结果、steer、compaction、工程/实验/探索报告、文件链接和可视化合同暂定完成；进入 Block C。 |
| 2026-08-10 18:34 | discovered | 正式 VSP packages 已全平台捆绑 rg 并提供 PATH prepend；PowerShell 有环境标识和专用执行路径；thread/list 与 rollout 支持 cwd 会话查询，但缺少模型专用改动归属工具。形成 Skills 跨任务生命周期和按需引用建议。 |
| 2026-08-10 18:38 | decided | 用户确认 Block C 其他方向，并修正 Skills 生命周期：绑定当前项目的一轮完整工作链，不默认存在 Workflow；另起无关任务或功能时释放。Block C 候选稿 v1 已形成，完整 Using skills 从基础提示删除。 |
| 2026-08-10 19:02 | waiting-review | Block A/B/C 已合并为完整候选稿：11,696字符、99行，较live基础提示减少28.1%；旧人格与Skills章节移除，核心新合同和CommonMark检查通过。提交S1验收。 |
| 2026-08-10 19:28 | accepted | 用户接受 S1 完整 BaseInstructions 候选、验证和风险；接受不授权产品源码修改。C28 进入 S2 通用 developer 运行规则讨论。 |
| 2026-08-10 19:35 | in_progress | S2 将通用 developer 运行规则拆为 D1-D4；D1 Permissions + Default mode 共1,330字符，完整原文、翻译和初步点评已形成。Skills/Apps/Plugins/Workflow Host 留到S3。 |
| 2026-08-10 20:39 | decided | 用户确认将 D1 拆成 D1a Permissions 与 D1b Default mode。VSP 默认 bypass 场景使用短权限提示；显式权限覆盖仍动态生成对应说明；运行能力不得扩大用户授权。进入 D1b。 |
| 2026-08-10 20:43 | decided | 用户接受 D1b 推荐短版，并明确 VSP 增强后的 `request_user_input` 应在确实需要用户输入时受到鼓励和优先使用；文本提问保留为工具不可用时的清晰 fallback。进入 D2。 |
| 2026-08-10 20:53 | expanded | 用户要求 C28 实现成本感知的子智能体模型配置，不能只精简 D2 文案。当前 V2 隐藏 model/reasoning 参数，已有 worker-routing 配置未接入可见 spawn 接口；S2 需补齐语义档位、长上下文机械任务、fallback 与注入时机设计。 |
| 2026-08-10 20:56 | decided | 用户选择语义 routing class 加受控显式 model/reasoning 覆盖，并将 D3 从 explicit-request-only 改为 proactive：root 可在重复、多路和调研任务中自主委派，但必须持续控制方向、进度、成本、整合与验收。 |
| 2026-08-10 21:00 | decided | 用户接受 D2-D3 完整合并候选及受控嵌套：子智能体默认不得派生，root 可对具体有边界任务显式授权。D2-D3 完成，进入 D4。 |
| 2026-08-10 21:04 | revised | 用户纠正范围：全量模型注入必须盘点，但外部 Workflow 插件提示确认归因后不做内容改写；重点修复 VSP 基础、内置插件和操作运行提示。D4 移出 S2。 |
| 2026-08-10 21:08 | waiting-review | S2 默认运行候选完成：D1+D2+D3 从3,360字符降至2,104字符，减少37.4%；权限、Default、主动多智能体、成本路由和root控制契约均存在。提交验收。 |
| 2026-08-10 21:14 | accepted | 用户接受 S2 完整候选、37.4% 缩减、D4 外部归因边界及 worker routing 产品要求。接受不授权源码修改；C28 进入 S3 内置 Skills/Apps/Plugins 提示审阅。 |
| 2026-08-10 21:17 | discovered | S3 来源地图完成：Skills 29,179字符/77条目/76唯一名，Apps 646，Plugins 1,014，外部 Workflow Host 343。Skills+Apps+Plugins 占developer #1的94.8%；纠正M1“80项”为77项。 |
| 2026-08-10 21:30 | decided | 用户接受 P1-A：Skills 动态层唯一拥有短协议；Skill 在项目工作链内持续，入口完整读取一次，引用按需加载；目录使用路径别名和短触发描述，纯名称必须唯一。进入仓库 Skills。 |
| 2026-08-10 21:39 | revised | 用户明确 S3 只需深审 System Skills，并可基于缺口新增1-2个；Repo/User/外部 Plugin Skills 正常注入，不改写正文，最多轻筛 usefulness。撤回未决 P1-B1 候选。 |
| 2026-08-10 21:42 | decided | 用户接受 skill-installer 候选，并授权同规格直接审阅 openai-docs、plugin-creator、skill-creator；imagegen 需单独讨论自定义URL和token设计。 |
| 2026-08-10 21:50 | completed | 完成 openai-docs、plugin-creator、skill-creator 的VSP兼容审阅与双语候选；确认upstream/VSP authority、vsp-codex CLI、manifest hooks矛盾、frontmatter validator和user root问题。进入imagegen专项。 |
| 2026-08-10 22:00 | decided | 用户接受imagegen复用active provider：VSPLab显式声明图像能力后，内置Rust image_gen直接使用其Base URL/Bearer API Key/gpt-image-2；修复provider capability gate，CLI保留为备用。 |
| 2026-08-10 22:04 | proposed | 基于openai-docs权威边界、imagegen capability gate和现有/provider/doctor能力，形成唯一新增System Skill候选vsp-provider；第二新增项缺少证据，不建议添加。 |
| 2026-08-10 22:10 | waiting-review | S3完成：77项active metadata全部有效；唯一plain collision采用scope winner；5项System Skill修订、唯一新增vsp-provider、Apps/Plugins精简及删除recommended_plugins user fragment提交整体验收。 |
| 2026-08-10 22:15 | accepted | 用户接受S3整体结果：保留并修订5项System Skills，新增唯一vsp-provider，外部Skills正常注入，采用同名scope winner，接受imagegen active-provider与Apps/Plugins候选。进入S4，源码仍锁定。 |
| 2026-08-10 22:15 | discovered | S4来源地图完成：26,472字符user context约98%来自25,981字符根AGENTS；发现OpenCode工具名、C10 legacy state写入、缺失analysis派生文件与Codex/插件authority冲突。形成根<=4K、Rust/Test/TUI/app-server按域拆分及环境/外部role候选。 |
| 2026-08-10 22:22 | waiting-review | S4完成：提交compact root+领域规则、修改前target-path加载、developer-role environment、conditional external host和trusted focus projection整体验收；实现机制留S5审阅，源码仍锁定。 |
| 2026-08-10 22:24 | accepted | 用户接受S4整体结果，无修订。C28进入S5完整集成设计审阅；S5接受前产品源码继续锁定。 |
| 2026-08-10 22:35 | waiting-review | S5完成：整合单一角色栈、完整prompt manifest、AGENTS preflight、worker routing、image capability、session migration、6-stage实现和M3/S6验证；预计非工具字符-42.8%、首轮tokens-30.1%，提交产品源码授权gate。 |
| 2026-08-10 22:37 | accepted | 用户接受S5并明确授权M2六个产品源码/测试stage。源码锁定解除；commit/push/PR/publish/install/deploy/restart/remote/destructive/protected Workflow仍未授权。Stage A开始。 |
| 2026-08-10 23:19 | waiting-review | M2 Stage A完成：Sol base prompt、compact permissions/default mode、proactive Multi-Agent单developer policy、Apps/Plugins精简、删除unsolicited recommendations均已实现。目标crate编译通过；core 5/5、permissions 20/20、models 3/3通过；本地CLI真实request与接受稿逐字一致。等待用户接受或拒绝Stage A。 |
| 2026-08-11 00:08 | accepted | 用户接受M2 Stage A实现与验证，同时询问剩余任务并提出当前系统提示词改动可能已足够发布。Stage A锁定为completed；在解释原计划剩余能力与最小发布路径并获得范围决定前，不自动开始Stage B。 |
| 2026-08-11 00:28 | decided | 用户选择“完成提示词全栈再发”：继续Stage B System Skills、Stage C AGENTS/runtime与Stage E worker routing，跳过无关Stage D image capability，最后执行Stage F回归/token/smoke；发布动作仍需单独授权。Stage B开始。 |
| 2026-08-11 01:07 | waiting-review | M2 Stage B完成：5项System Skill精简并新增vsp-provider；目录采用scope winner与alias；metadata预算封顶2000 tokens；Skill跨turn/compaction/resume持续。clean core-skills 121/121、core 21/21、smoke 7项及真实CLI capture通过。等待独立人工审阅。 |
| 2026-08-11 01:12 | accepted | 用户接受M2 Stage B实现、验证与剩余风险。System Skills、effective catalog、2000-token cap、compact protocol和Skill lifecycle锁定；Stage C允许后续开始，本次未启动。 |
| 2026-08-11 13:50 | decided | 用户确认Worker Routing用于为Subagent分配语义模型/推理/成本档位，并要求完成该项后直接做遗漏审计与最终验证，再准备Linux、Windows、macOS构建发布。Stage C AGENTS/runtime与Stage D image capability延期。 |
| 2026-08-11 14:08 | waiting-review | Stage E完成：trusted policy overlay、六类semantic resolver、offline candidate validation、fallback/skip reasons、role/explicit冲突、fork与routing output/telemetry已接通；focused core 35项执行全部通过，等待审阅。 |
| 2026-08-11 14:14 | accepted | 用户接受Stage E完整实现、验证、已解决问题与剩余风险。M2完成；立即进入M3 omissions audit与最终验证，不启动延期的AGENTS/runtime或image capability。 |
| 2026-08-11 14:40 | completed | M3/S6完成：accepted scope regressions/smoke通过；真实input tokens下降22.6%、non-tool chars下降28.4%，因Stage C延期未达原目标；live Subagent被VSPLab 502阻断。补齐multi-target archive/manifest/workflow遗漏，进入M4 alpha.6三平台构建准备。 |
